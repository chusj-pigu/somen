// standard
use std::{
    fs::File,
    sync::Arc,
    io::Error,
    panic::RefUnwindSafe,
    ptr::NonNull,
    ops::{
        Deref,
        ControlFlow,
    },
    sync::mpsc::{
        self,
        Sender,
        Receiver,
        RecvError,
    },
    thread::{
        spawn,
        JoinHandle,

    },
};
// third party
use arrow::buffer::Buffer;
use memmap2::{
    self,
    Advice,
    UncheckedAdvice,
};
// local
use crate::io::reader::{PageOffset, PageSize};

/// todo
#[derive(Debug, Copy, Clone)]
pub struct ByteRange {
    /// The `start` of the `ByteRange`.
    start: usize,
    /// The `length` of the `ByteRange`.
    length: usize,
}

impl ByteRange {
    /// Creates a new `ByteRange`.
    pub fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }

    /// Returns the `start` of the `ByteRange`.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the `length` of the `ByteRange`.
    pub fn length(&self) -> usize {
        self.length
    }
}

/// The operations which can be requested from the worker
enum HintingOrder {
    /// Tells the worker to start prefaulting pages
    Prefault{
        /// The start of the range of pages to prefault.
        start: PageOffset,
        /// The number of pages to prefault
        page_count: usize,
    },
    /// Tells the worker to stop the waiting loop
    Stop,
}


/// The receiver portion of the `Worker`, living on a separate thread.
/// It prefaults pages following the [`Orders`](HintingOrder) of the [`WorkerHandle`].
///
/// # Safety
///
/// The `WorkerHandle` must be joined before the pointed mmap is dropped.
struct Worker {
    /// A raw pointer to a [`Mmap`](memmap2::Mmap).
    ptr: NonNull<u8>,
    /// The maximum offset within the lenght of a [`Mmap`](memmap2::Mmap).
    max_offset: usize,
    /// The size of page.
    page_size: PageSize,
}

unsafe impl Send for Worker {}

impl Worker {
    /// Returns a new `Worker`.
    fn new(mmap: &memmap2::Mmap, page_size: PageSize) -> Self {
        // Safety:
        // mmap.as_mut_ptr() is never null on successful map.
        let ptr = unsafe {
            NonNull::new_unchecked(mmap.as_ptr() as *mut u8)
        };
        let max_offset = mmap.len().saturating_sub(1);
        Self {
            ptr,
            page_size,
            max_offset,
        }
    }

    /// Returns a closure which calls prefaults with the `Worker` until a
    /// [`Stop`](HintingOrder::Stop) [`HintingOrder`] is received, at which point the closure
    /// drops it.
    ///
    /// # Panics
    ///
    /// If the [`sender`](mpsc::Sender) the Worker is listening too is
    /// disconnect or dropped before the Worker receives the `Stop` order.
    fn start(self, receiver: Receiver<HintingOrder>) -> impl FnOnce() + Send + 'static {
        move || {
            loop {
                let order = receiver.recv()
                    .expect("worker sender disconnected out of order");
                match order {
                    HintingOrder::Prefault{start,page_count} => {
                        self.prefault(start, page_count);
                    }
                    HintingOrder::Stop => {
                        break;
                    }
                }
            }
        }
    }

    /// Reads one byte of each page, based of the requested `length` and
    /// known `page_size` to fault them and force the os to load them
    ///
    /// # Panics
    ///
    /// `offset` and `length` must be multiples of the page size the
    /// `Worker` was initialized with and their sum must not overflow and
    /// be smaller than the lenght of the source [`Mmap`](memmap2::Mmap).
    fn prefault(&self, start: PageOffset, page_count: usize) {
        if page_count == 0 {
            return;
        }
        let (page_offsets, end_offset) = self.page_size.iter_pages(start, page_count);
        let end_offset = end_offset.as_inner();
        if end_offset > self.max_offset {
            panic!("prefault offset {} exceeds maximum memory map offset {}", end_offset, self.max_offset)
        }
        for offset in page_offsets {
            // Safety:
            // - offset is within bounds due to the assert above.
            // - ptr is a valid, aligned and readable pointer, since
            //   it is made from a mmap pointer and the offset.
            unsafe {
                std::ptr::read_volatile(
                    self.ptr.as_ptr().add(offset.as_inner())
                );
            }
        }
    }
}

/// The [`Worker`] serves to prefault pages to fully load them in, to help the os load the data.
/// `WorkerHandle` is the sender portion of the `Worker`, it is owned by the [`Mmap`].
///
/// # Safety
///
/// The WorkerHandle must be dropped before the pointed mmap is dropped.
struct WorkerHandle {
    /// The [`channel`](mpsc::channel) through which the `WorkerHandle` sends orders to the `Worker`.
    sender: Sender<HintingOrder>,
    /// The [`handle`](JoinHandle) to the thread the `Worker` lives on.
    handle: Option<JoinHandle<()>>,
}


impl WorkerHandle {
    /// returns a new `WorkerHandle`.
    fn new(mmap: &memmap2::Mmap, page_size: PageSize) -> Self {
        let worker = Worker::new(mmap, page_size);
        let (sender, receiver) = mpsc::channel::<HintingOrder>();
        let handle = Some(spawn(worker.start(receiver)));
        Self {
            sender,
            handle,
        }
    }

    /// Tells the [`Worker`] to prefault the given range of pages.
    ///
    /// # Panics
    ///
    /// If the `sender` of the channel is missing or
    /// if the WorkerHandle's thread exited
    fn prefault(&self, start: PageOffset, page_count: usize) {
        if let Err(_) = self.sender.send(HintingOrder::Prefault{start, page_count}) {
            panic!("internal invariant violated: worker thread terminated unexpectedly");
        }
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(HintingOrder::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

// Unwinding cannot leave WorkerHandle in an inconsistent state.
// The only state mutation is consuming the `JoinHandle` during `drop()`,
// which is an atomic ownership transfer and does not expose partially-updated invariants.
impl RefUnwindSafe for WorkerHandle {}

/// A handle to an immutable memory mapped buffer.
///
/// A `Mmap` is backed by a file, is created by `&File` reference,
/// and remains valid even after the `File` is dropped.
/// In other words, the `Mmap` handle is completely independent
/// of the `File` used to create it.
/// The memory will be unmapped when the Mmap handle is dropped.
///
/// Dereferencing and accessing the bytes of the buffer without
/// calling the appropriate hinting functions, ie [`will_need()`](Mmap::will_need)
/// and [`dont_need()`](Mmap::dont_need) may result in minor and or major page faults
/// (e.g. swapping the mapped pages into physical memory)
/// though the details of this are platform specific as
/// `Mmap` primarly targets `Unix` systems, though there is
/// some support for windows and other platforms.
///
/// `Mmap` is [`Sync`] and [`Send`].
///
/// # Safety
///
/// All file-backed memory map constructors are marked unsafe
/// because of the potential for Undefined Behavior (UB) using
/// the map if the underlying file is subsequently modified,
/// in or out of process. Applications must consider the risk
/// and take appropriate precautions when using file-backed maps.
/// Solutions such as file permissions, locks or process-private
/// (e.g. unlinked) files exist but are platform specific and limited.
pub(crate) struct Mmap {
    // Dropped first via order to prevent dangling pointers.
    /// The handle to send orders to the worker.
    worker_handle: WorkerHandle,
    /// The inner handle to the immutable memory mapped buffer.
    mmap: memmap2::Mmap,
    /// The size of page.
    page_size: PageSize,
    /// The maximum offset within the lenght of the `Mmap`.
    max_offset: usize,
}

impl Mmap {

    /// Returns a read-only memory map backed by a file.
    ///
    /// # Safety
    ///
    /// See the [`type-level`](Mmap) docs for why this function is unsafe.
    ///
    /// # Errors
    ///
    /// This method returns an error when the underlying system call fails,
    /// which can happen for a variety of reasons, such as when the file
    /// is not open with read permissions.
    ///
    /// Returns [`ErrorKind::Unsupported`](std::io::ErrorKind::Unsupported)
    /// on unsupported platforms.
    pub(crate) unsafe fn new(file: File, page_size: PageSize) -> Result<Arc<Self>, Error> {
        // Safety:
        // Safety of the operation is upheld by the user.
        let mmap = unsafe {
            memmap2::Mmap::map(&file)?
        };
        let worker_handle = WorkerHandle::new(&mmap, page_size);
        let max_offset = mmap.len().saturating_sub(1);
        Ok(Arc::new(Self{
            worker_handle,
            mmap,
            page_size,
            max_offset,
        }))
    }

    /// Advises the operating system that this byte range will likely be
    /// accessed soon.
    ///
    /// The range is rounded to whole pages. On Unix, this issues a
    /// `MADV_WILLNEED` hint and then prefaults the pages by reading one
    /// byte from each page.
    ///
    /// This is purely a performance optimization and does not change the
    /// observable contents of the mapping.
    ///
    /// # Panics
    ///
    /// Panics if the requested range lies outside the memory map or if the
    /// range calculation overflows.
    pub(crate) fn will_need(&self, byte_range: ByteRange) {
        let page_size = self.page_size;
        let start_offset = page_size.align_down(byte_range.start());
        let page_count = page_size.touched_page_count(byte_range.start(), byte_range.length());
        let page_length = page_size.page_length(page_count);
        let end_offset = start_offset.as_inner().checked_add(page_length)
            .expect(&format!("page range overflowed ({} + {})", start_offset.as_inner(), page_length));
        if end_offset > self.max_offset {
            panic!("hint offset {} exceeds maximum memory map offset {}", end_offset, self.max_offset)
        }
        #[cfg(unix)]
        {
            self.mmap.advise_range(Advice::WillNeed, start_offset.as_inner(), page_length);
        }
        self.worker_handle.prefault(start_offset, page_count);
    }

    /// Advises the operating system that this byte range is unlikely to be
    /// accessed again soon.
    ///
    /// The range is rounded to whole pages. On Unix, this issues a
    /// `MADV_DONTNEED`-style advisory hint. The mapping remains valid and
    /// may still be accessed normally, although future accesses may incur
    /// page faults if the operating system reclaims the pages. Unlike
    /// `will_need`, the final page is **only** included in the range if
    /// it is entirelly covered by the range.
    /// (aka the range ends on a page boundary)
    ///
    /// This is purely a performance optimization and does not change the
    /// observable contents of the mapping.
    ///
    /// # Panics
    ///
    /// Panics if the requested range lies outside the memory map or if the
    /// range calculation overflows.
    pub(crate) fn dont_need(&self, byte_range: ByteRange) {
        let page_size = self.page_size;
        let start_offset = page_size.align_down(byte_range.start());
        let page_count = page_size.reclaimable_pages_count(byte_range);
        let page_length = page_size.page_length(page_count);
        let end_offset = start_offset.as_inner().checked_add(page_length)
            .expect(&format!("page range overflowed ({} + {})", start_offset.as_inner(), page_length));
        if end_offset > self.max_offset {
            panic!("hint offset {} exceeds maximum memory map offset {}", end_offset, self.max_offset)
        }
        #[cfg(unix)]
        // Safety:
        // - `start_offset` and `page_length` fall within the bounds of the Mmap.
        // - `DontNeed` only advises the operating system that these pages may be
        //   reclaimed. The virtual address range remains mapped, and this method does
        //   not create, invalidate, or relocate Rust references. Any future access
        //   through the mapping remains valid according to the mmap abstraction.
        unsafe {
            self.mmap.unchecked_advise_range(UncheckedAdvice::DontNeed, start_offset.as_inner(), page_length);
        }
    }

    /// Returns the [`Buffer`] for a pod5 table given the `offset`
    /// and `length` found within the footer of the `source` pod5 file.
    ///
    /// # Panics
    ///
    /// Panics if the requested table range extends beyond the source buffer,
    /// or if `offset + length` overflows `usize`.
    pub(crate) fn table_buffer(source: &Arc<Self>, byte_range: ByteRange) -> Buffer {
        let end_offset = byte_range.start().checked_add(byte_range.length())
            .expect("Table range calculation overflowed usize");
        assert!(end_offset <= source.mmap.len(), "Table range exceeds source buffer bounds");

        // Safety:
        // - offset is within bounds due to the checked_add and assert above.
        // - source.mmap.as_mut_ptr() is never null on successful map.
        // - The `Arc<Self>` owner keeps the mmap alive for the liftime of the `Buffer`.
        unsafe {
            let ptr = source.mmap.as_ptr().add(byte_range.start());
            let ptr= NonNull::new_unchecked(ptr as *mut u8);
            Buffer::from_custom_allocation(ptr, byte_range.length(), source.clone())
        }
    }
}

impl Deref for Mmap {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.mmap.as_ref()
    }
}
