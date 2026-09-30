// standard

// third party

// local

mod internal {
    // standard
    use std::{fmt, ops::Deref, rc::Rc, sync::Arc, error::Error, fmt::{
        Display,
        Formatter,
    }, rc, sync};
    //third party
    use arc_swap::Guard;
    // local
    use crate::{
        file::distribution::{
            ArcBatchRefTracker,
            BatchRefTracker,
            Capacity,
            BatchRegistry,
            RcBatchRefTracker,
        },
        record::batch::{
            internal::BatchColumns,
            BatchCore,
            BatchResult,
        },
    };
    use crate::record::batch::BatchError;
    use crate::io::decoder::SignalDecoder;

    mod sealed {
        /// todo
        pub trait RefCountedSeal {}

        /// todo
        pub trait ErrorRefSeal {}

        /// todo
        pub trait WeakRefCountedSeal {}

        /// todo
        pub trait BatchRefSeal {}

        /// todo
        pub trait ConcurrencySeal {}
    }

    /// todo
    pub trait WeakRefCounted<M: ConcurrencyMode, T>: sealed::WeakRefCountedSeal + Clone {
        /// todo
        type RefCounted: RefCounted<M, T>;

        /// todo
        fn upgrade(&self) -> Option<Self::RefCounted>;
    }

    impl<T> sealed::WeakRefCountedSeal for rc::Weak<T> {}

    impl<T> WeakRefCounted<Local, T> for rc::Weak<T> {
        type RefCounted = Rc<T>;

        fn upgrade(&self) -> Option<Self::RefCounted> {
            self.upgrade()
        }
    }

    impl<T> sealed::WeakRefCountedSeal for sync::Weak<T> {}

    impl<T> WeakRefCounted<Atomic, T> for sync::Weak<T> {
        type RefCounted = Arc<T>;

        fn upgrade(&self) -> Option<Self::RefCounted> {
            self.upgrade()
        }
    }

    /// A reference-counting pointer.
    ///
    /// `RefCounted<M, T>` provides shared ownership of a value of type `T`,
    /// allocated in the heap.
    /// Invoking clone on `RefCounted` produces a new `RefCounted` instance,
    /// which points to the same allocation on the heap as the source
    /// `RefCounted`, while increasing a reference count.
    /// When the last `RefCounted` pointer to a given allocation is destroyed,
    /// the value stored in that allocation (often referred to as "inner value")
    /// is also dropped.
    ///
    /// M describes whether the pointer is single-threaded only or thread safe.
    pub trait RefCounted<M: ConcurrencyMode, T>: sealed::RefCountedSeal + Clone + Deref {
        /// todo
        type WeakRefCounted: WeakRefCounted<M, T>;

        /// todo
        fn new(t: T) -> Self;

        /// todo
        fn downgrade(self) -> (Self, Self::WeakRefCounted);
    }

    impl<T> sealed::RefCountedSeal for Rc<T> {}
    impl<T> RefCounted<Local, T> for Rc<T>{
        type WeakRefCounted = rc::Weak<T>;

        fn new(t: T) -> Self {
            Rc::new(t)
        }

        fn downgrade(self) -> (Rc<T>, rc::Weak<T>) {
            let weak = Rc::downgrade(&self);
            (self, weak)
        }
    }

    impl<T> sealed::RefCountedSeal for Arc<T> {}
    impl<T> RefCounted<Atomic, T> for Arc<T>{
        type WeakRefCounted = sync::Weak<T>;

        fn new(t: T) -> Self {
            Arc::new(t)
        }

        fn downgrade(self) -> (Arc<T>, sync::Weak<T>) {
            let weak = Arc::downgrade(&self);
            (self, weak)
        }
    }

    /// A batch accessor.
    ///
    /// `BatchRef<M, C>` provides shared access to a Batch of column type `C`,
    /// allocated in the heap.
    /// When the last `BatchRef` pointer to a given batch is destroyed,
    /// the batch stored in that allocation is also dropped.
    pub trait BatchRef<C: BatchColumns>: Deref<Target = BatchCore<Self::ConcurrencyMode, C>> + sealed::BatchRefSeal {
        /// The [`ConcurrencyMode`] type associated with the `BatchRef`.
        type ConcurrencyMode: ConcurrencyMode;

        /// The [`StoredBatchRef`] type associated with the `BatchRef's` [`Batch`](crate::record::batch).
        type StoredBatchRef: StoredBatchRef<C>;

        /// Returns an owned Batch pointer to the same [`Batch`](crate::record::batch).
        fn into_stored(self) -> Self::StoredBatchRef;
    }

    /// todo
    pub trait ErrorRef: Deref<Target = BatchError> + sealed::ErrorRefSeal{}

    /// A local batch accessor.
    ///
    /// `LocalAccess<M, C>` provides thread-local access to a Batch of column type `C`,
    /// allocated in the heap.
    pub trait InitialBatchRef<C: BatchColumns>: BatchRef<C> {
        /// todo
        type Input;

        /// todo
        type ErrorRef: ErrorRef;

        /// todo
        fn new (input: Self::Input) -> Result<Self, Self::ErrorRef> where Self: Sized;
    }

    /// An owned batch pointer.
    ///
    /// `StoredBatchRef<M, C>` provides shared ownership to a Batch of column type `C`,
    /// allocated in the heap.
    ///
    /// A `StoredBatchRef` is sendable in multithreaded context, aka M = Atomic.
    pub trait StoredBatchRef<C: BatchColumns>: BatchRef<C> {}

    /// todo
    #[must_use]
    #[repr(transparent)]
    pub struct RcErrorRef<C: BatchColumns>(
        Rc<BatchResult<Local, C>>,
    );

    impl<C: BatchColumns> RcErrorRef<C> {
        /// todo
        pub fn new(input: Rc<BatchResult<Local, C>>) -> Result<Self, Rc<BatchResult<Local, C>>> {
            match input.deref() {
                Err(_) => Ok(Self(input)),
                Ok(_) => Err(input),
            }
        }
    }

    impl<C: BatchColumns> Deref for RcErrorRef<C> {
        type Target = BatchError;

        fn deref(&self) -> &BatchError {
            // Safety:
            // The Result is validated to be Err during
            // initialization and cannot be modified.
            unsafe { self.0.deref().as_ref().unwrap_err_unchecked() }
        }
    }

    impl<C: BatchColumns> sealed::ErrorRefSeal for RcErrorRef<C> {}

    impl<C: BatchColumns> ErrorRef for RcErrorRef<C> {}

    /// todo
    #[must_use]
    #[repr(transparent)]
    pub struct RcBatchRef<C: BatchColumns>(
        Rc<BatchResult<Local, C>>,
    );

    impl<C: BatchColumns> Deref for RcBatchRef<C> {
        type Target = BatchCore<Local, C>;

        fn deref(&self) -> &BatchCore<Local, C> {
            // Safety:
            // The Result is validated to be Ok during
            // initialization and cannot be modified.
            unsafe { self.0.deref().as_ref().unwrap_unchecked() }
        }
    }

    impl<C: BatchColumns> sealed::BatchRefSeal for RcBatchRef<C> {}

    impl<C: BatchColumns> BatchRef<C> for RcBatchRef<C> {
        type ConcurrencyMode = Local;

        type StoredBatchRef = Self;

        fn into_stored(self) -> Self {
            self
        }
    }

    impl<C: BatchColumns> StoredBatchRef<C> for RcBatchRef<C> {}

    impl<C: BatchColumns> InitialBatchRef<C> for RcBatchRef<C> {
        type Input = Rc<BatchResult<Local, C>>;

        type ErrorRef = RcErrorRef<C>;

        fn new(input: Rc<BatchResult<Local, C>>) -> Result<Self, RcErrorRef<C>> {
            match input.deref() {
                Ok(_) => Ok(Self(input)),
                Err(_) => {
                    match RcErrorRef::new(input) {
                        Ok(rc_error_rf) => Err(rc_error_rf),
                        Err(_) => panic!("RcErrorRef failed to maintain invariant, failed to initialize on input Err(_)"),
                    }
                },
            }
        }
    }

    /// todo
    #[must_use]
    #[repr(transparent)]
    pub struct GuardErrorRef<C: BatchColumns>(
        Guard<Option<Arc<BatchResult<Atomic, C>>>>,
    );

    impl<C: BatchColumns> GuardErrorRef<C> {
        /// todo
        pub fn new(input: Guard<Option<Arc<BatchResult<Atomic, C>>>>) -> Option<Result<Self, Guard<Option<Arc<BatchResult<Atomic, C>>>>>> {
            match input.as_ref()?.as_ref() {
                Err(_) => {Some(Ok(Self(input)))},
                Ok(_) => Some(Err(input)),
            }
        }
    }

    impl<C: BatchColumns> Deref for GuardErrorRef<C> {
        type Target = BatchError;

        fn deref(&self) -> &BatchError {
            // Safety:
            // The Result is validated to be Err during
            // initialization and cannot be modified.
            unsafe { self.0.as_ref().unwrap_unchecked().as_ref().as_ref().unwrap_err_unchecked() }
        }
    }

    impl<C: BatchColumns> sealed::ErrorRefSeal for GuardErrorRef<C> {}

    impl<C: BatchColumns> ErrorRef for GuardErrorRef<C> {}

    /// A temporary, thread-safe local view of a [`Batch`](crate::record::batch).
    ///
    /// `ArcGuard` wraps an [`arc_swap`](arc_swap)::[`Guard`].
    /// It dereferences directly to the underlying `Batch` in the `Option<Arc<Batch>>`,
    /// providing a guaranteed [`Some`] variant for read operations.
    ///
    /// Unlike a raw lock-free guard, this type enforces a design invariant where
    /// the underlying container is never empty.
    ///
    /// # Panics
    ///
    /// Functions and trait implementations on this type will panic if the underlying
    /// pointer unexpectedly evaluates to [`None`].
    #[must_use]
    #[repr(transparent)]
    pub struct GuardBatchRef<C: BatchColumns>(
        Guard<Option<Arc<BatchResult<Atomic, C>>>>,
    );

    impl<C: BatchColumns> Deref for GuardBatchRef<C> {
        type Target = BatchCore<Atomic, C>;

        fn deref(&self) -> &BatchCore<Atomic, C> {
            // Safety:
            // The Result is validated to be Some(Ok) during
            // initialization and cannot be modified.
            unsafe { self.0.as_ref().unwrap_unchecked().as_ref().as_ref().unwrap_unchecked() }
        }
    }

    impl<C: BatchColumns> sealed::BatchRefSeal for GuardBatchRef<C> {}

    impl<C: BatchColumns> BatchRef<C> for GuardBatchRef<C> {
        type ConcurrencyMode = Atomic;

        type StoredBatchRef = ArcBatchRef<C>;

        fn into_stored(self) -> ArcBatchRef<C> {
            // Safety:
            // The Result is validated to be Some(Ok) during
            // initialization and cannot be modified.
            unsafe {
                let inner_arc = Guard::into_inner(self.0).unwrap_unchecked();
                ArcBatchRef::new_unchecked(inner_arc)
            }
        }
    }

    impl<C: BatchColumns> InitialBatchRef<C> for GuardBatchRef<C> {
        type Input = Guard<Option<Arc<BatchResult<Atomic, C>>>>;

        type ErrorRef = GuardErrorRef<C>;

        fn new<'a>(input: Guard<Option<Arc<BatchResult<Atomic, C>>>>) -> Result<Self, GuardErrorRef<C>> {
            match input.as_ref() {
                Some(result) => match result.as_ref() {
                    Ok(_)=> Ok(Self(input)),
                    Err(_) => match GuardErrorRef::new(input) {
                        Some(result) => match result {
                            Ok(guard_error_ref) => Err(guard_error_ref),
                            Err(_) => panic!("GuardErrorRef failed to maintain invariant, failed to initialize on input Err(_) "),
                        },
                        None => panic!("GuardErrorRef failed to maintain invariant, failed to initialize on input Some(_) "),
                    },
                },
                None => panic!("GuardBatchRef invariant violated: Guard was None "),
            }
        }
    }

    /// todo
    #[must_use]
    #[repr(transparent)]
    pub struct ArcBatchRef<C: BatchColumns>(
        Arc<BatchResult<Atomic, C>>,
    );

    impl<C: BatchColumns> ArcBatchRef<C> {
        /// todo
        pub unsafe fn new_unchecked(input: Arc<BatchResult<Atomic, C>>) -> Self {
            Self(input)
        }
    }

    impl<C: BatchColumns> Deref for ArcBatchRef<C> {
        type Target = BatchCore<Atomic, C>;

        fn deref(&self) -> &BatchCore<Atomic, C> {
            // Safety:
            // The Result is guarantied to be Ok by the
            // initialization and cannot be modified.
            unsafe { self.0.deref().as_ref().unwrap_unchecked() }
        }
    }

    impl<C: BatchColumns> sealed::BatchRefSeal for ArcBatchRef<C> {}

    impl<C: BatchColumns> BatchRef<C> for ArcBatchRef<C> {
        type ConcurrencyMode = Atomic;

        type StoredBatchRef = Self;

        fn into_stored(self) -> Self {
            self
        }
    }

    impl<C: BatchColumns> StoredBatchRef<C> for ArcBatchRef<C> {}

    pub struct Pod5Context<M: ConcurrencyMode> {
        batch_registry: BatchRegistry<M>,
        signal_decoder: SignalDecoder<M>,
    }

    impl<M: ConcurrencyMode> Pod5Context<M> {
        fn new(
            batch_registry: BatchRegistry<M>,
            signal_decoder: SignalDecoder<M>,
        ) -> Self {
            Self {
                batch_registry,
                signal_decoder,
            }
        }
        pub fn batch_registry(&self) -> &BatchRegistry<M> {
            &self.batch_registry
        }
        pub fn signal_decoder(&self) -> &SignalDecoder<M> {
            &self.signal_decoder
        }
    }

    /// todo
    pub trait Pod5ContextHandle<M: ConcurrencyMode>: Clone + Deref<Target = Pod5Context<M>> {
        /// todo
        fn new(inner: M::RefCounted<Pod5Context<M>>) -> Self;

        /// todo
        fn into_stored(self) -> StoredPod5ContextHandle<M>;
    }

    /// todo
    pub struct InitialPod5ContextHandle<M: ConcurrencyMode> (Rc<M::RefCounted<Pod5Context<M>>>);

    impl<M: ConcurrencyMode> Clone for InitialPod5ContextHandle<M> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<M: ConcurrencyMode> Deref for InitialPod5ContextHandle<M> {
        type Target = Pod5Context<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<M: ConcurrencyMode> Pod5ContextHandle<M> for InitialPod5ContextHandle<M> {
        fn new(inner: M::RefCounted<Pod5Context<M>>) -> Self {
            Self(Rc::new(inner))
        }

        fn into_stored(self) -> StoredPod5ContextHandle<M> {
            let inner = self.0.as_ref();
            StoredPod5ContextHandle(inner.clone())
        }
    }

    /// todo
    pub struct StoredPod5ContextHandle<M: ConcurrencyMode> (M::RefCounted<Pod5Context<M>>);

    impl<M: ConcurrencyMode> Clone for StoredPod5ContextHandle<M> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<M: ConcurrencyMode> Deref for StoredPod5ContextHandle<M> {
        type Target = Pod5Context<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<M: ConcurrencyMode> Pod5ContextHandle<M> for StoredPod5ContextHandle<M> {
        fn new(inner: M::RefCounted<Pod5Context<M>>) -> Self {
            Self(inner)
        }

        fn into_stored(self) -> StoredPod5ContextHandle<M> {
            self
        }
    }

    /// todo
    pub trait ReferenceModel {
        /// todo
        type ConcurrencyMode: ConcurrencyMode;

        /// todo
        type Pod5ContextHandle: Pod5ContextHandle<Self::ConcurrencyMode>;

        /// todo
        type BatchRef<C: BatchColumns>: BatchRef<C, ConcurrencyMode = Self::ConcurrencyMode>;
        /// todo
        type StoredReferenceModel<C: BatchColumns>: StoredReferenceModel<
            C,
            ConcurrencyMode = Self::ConcurrencyMode,
            Pod5ContextHandle = StoredPod5ContextHandle<Self::ConcurrencyMode>,
            BatchRef<C> = <Self::BatchRef<C> as BatchRef<C>>::StoredBatchRef,
        >;
    }

    /// todo
    pub trait InitialReferenceModel<C: BatchColumns>: ReferenceModel<BatchRef<C>: InitialBatchRef<C>> {}

    /// todo
    pub trait StoredReferenceModel<C: BatchColumns>: ReferenceModel<BatchRef<C>: StoredBatchRef<C>, StoredReferenceModel<C> = Self> {}

    /// todo
    pub struct RcModel;

    impl ReferenceModel for RcModel {
        type ConcurrencyMode = Local;
        type Pod5ContextHandle = StoredPod5ContextHandle<Local>;
        type BatchRef<C: BatchColumns> = RcBatchRef<C>;
        type StoredReferenceModel<C: BatchColumns> = Self;
    }

    impl<C: BatchColumns> InitialReferenceModel<C> for RcModel {}

    impl<C: BatchColumns> StoredReferenceModel<C> for RcModel {}

    /// todo
    pub struct GuardModel;

    impl ReferenceModel for GuardModel {
        type ConcurrencyMode = Atomic;
        type Pod5ContextHandle = InitialPod5ContextHandle<Atomic>;
        type BatchRef<C: BatchColumns> = GuardBatchRef<C>;
        type StoredReferenceModel<C: BatchColumns> = ArcModel;
    }

    impl<C: BatchColumns> InitialReferenceModel<C> for GuardModel {}

    /// todo
    pub struct ArcModel;

    impl ReferenceModel for ArcModel {
        type ConcurrencyMode = Atomic;
        type Pod5ContextHandle = StoredPod5ContextHandle<Atomic>;
        type BatchRef<C: BatchColumns> = ArcBatchRef<C>;
        type StoredReferenceModel<C: BatchColumns> = Self;
    }

    impl<C: BatchColumns> StoredReferenceModel<C> for ArcModel {}

    /// todo
    pub trait ConcurrencyMode: sealed::ConcurrencySeal + Sized + 'static {
        /// todo
        type InitialReferenceModel<C: BatchColumns>: InitialReferenceModel<C, ConcurrencyMode = Self>;

        /// todo
        type RefCounted<T>: RefCounted<Self, T, WeakRefCounted = Self::WeakRefCounted<T>> + Deref<Target = T>;

        /// todo
        type WeakRefCounted<T>: WeakRefCounted<Self, T, RefCounted = Self::RefCounted<T>>;

        /// todo
        type Tracker<C: BatchColumns>: BatchRefTracker<Self, C>;
    }

    pub type BatchRefOf<M: ConcurrencyMode, C: BatchColumns> = <M::InitialReferenceModel<C> as ReferenceModel>::BatchRef<C>;
    pub type ErrorRefOf<M: ConcurrencyMode, C: BatchColumns> = <<M::InitialReferenceModel<C> as ReferenceModel>::BatchRef<C> as InitialBatchRef<C>>::ErrorRef;

    /// todo
    pub struct Local;

    impl sealed::ConcurrencySeal for Local {}

    impl ConcurrencyMode for Local {
        type InitialReferenceModel<C: BatchColumns> = RcModel;
        type RefCounted<T> = Rc<T>;
        type WeakRefCounted<T> = rc::Weak<T>;
        type Tracker<C: BatchColumns> = RcBatchRefTracker<C>;
    }

    /// todo
    pub struct Atomic;

    impl sealed::ConcurrencySeal for Atomic {}

    impl ConcurrencyMode for Atomic {
        type InitialReferenceModel<C: BatchColumns> = GuardModel;
        type RefCounted<T> = Arc<T>;
        type WeakRefCounted<T> = sync::Weak<T>;
        type Tracker<C: BatchColumns> = ArcBatchRefTracker<C>;
    }
}

use std::error::Error;
use std::fmt;
use std::fmt::{write, Display, Formatter};
use std::mem::MaybeUninit;
use std::ops::Add;
#[cfg(feature = "backend")]
pub use self::internal::{
    ArcBatchRef,
    Atomic,
    BatchRef,
    ConcurrencyMode,
    RcBatchRef,
    RcErrorRef,
    GuardBatchRef,
    GuardErrorRef,
    InitialBatchRef,
    Local,
    ReferenceModel,
    BatchRefOf,
    ErrorRefOf,
};

#[cfg(not(feature = "backend"))]
pub(crate) use self::internal::{
    ArcBatchRef,
    Atomic,
    BatchRef,
    ConcurrencyMode,
    RcBatchRef,
    RcErrorRef,
    GuardBatchRef,
    GuardErrorRef,
    InitialBatchRef,
    Local,
    ReferenceModel,
    RcModel,
    ArcModel,
    GuardModel,
    BatchRefOf,
    ErrorRefOf,
    Pod5ContextHandle,
    RefCounted,
    Pod5Context,
};


#[must_use]
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// A page-aligned byte offset.
///
/// Values of this type are guaranteed to lie on a page boundary.
pub struct PageOffset(usize);

impl PageOffset {

    /// Creates a `PageOffset` without validating the provided offset.
    ///
    /// The caller must ensure that `offset` is aligned to the page size used by the
    /// corresponding memory mapping.
    fn new(offset: usize) -> Self {
        Self(offset)
    }

    /// Returns the underlying byte offset.
    pub fn as_inner(&self) -> usize {
        self.0
    }
}

/// Failed to determine the operating system's memory page size.
#[derive(Debug)]
pub struct PageSizeError(String);

impl Display for PageSizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for PageSizeError {}

#[must_use]
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// The operating system's virtual memory page size.
///
/// `PageSize` provides page-aware arithmetic used throughout the crate,
/// including offset alignment, page counting, and page iteration.
pub struct PageSize(usize);

impl PageSize {
    /// Creates a `PageSize` without querying the operating system.
    ///
    /// The provided page size should match the actual page size used by the
    /// operating system.
    ///
    /// Supplying an incorrect value does not cause immediate undefined behavior,
    /// but it invalidates the assumptions made by the paging algorithms and may
    /// result in incorrect alignment, ineffective prefetching, or degraded
    /// performance.
    ///
    /// # Panics
    /// Panics if the provided `size` is not a power of two.
    pub fn custom(page_size: usize) -> Self {
        assert!(
            page_size.is_power_of_two(),
            "page size ({page_size}) must be a power of two"
        );
        assert_ne!(page_size, 0, "page size ({page_size}) must be a non-zero value");
        Self(page_size)
    }

    /// Returns the greatest page-aligned offset less than or equal to `offset`.
    pub fn align_down(&self, offset: usize) -> PageOffset {
        PageOffset::new(offset - (offset % self.0))
    }

    /// Returns the smallest page-aligned offset greater than or equal to `offset`.
    pub fn align_up(&self, offset: usize) -> PageOffset {
        let alignment = offset % self.0;
        if alignment == 0 {
            PageOffset::new(offset)
        } else {
            PageOffset::new(offset + (self.0 - alignment))
        }
    }

    /// Returns an iterator over consecutive page-aligned offsets as well as the final offset.
    ///
    /// The iterator begins at `start` and yields `page_count` page offsets.
    ///
    /// # Panics
    ///
    /// The total range covered by the pages must not overflow usize.
    pub fn iter_pages(&self, start: PageOffset, page_count: usize) -> (impl Iterator<Item = PageOffset>, PageOffset) {
        let start = start.as_inner();
        let page_size = self.0;
        let offset = page_count.checked_mul(page_size)
            .expect(&format!("page offset overflowed ({} * {})", page_count, page_size));
        let end = start.checked_add(offset)
            .expect(&format!("page range overflowed ({} + {})", start, offset));
        let iter = (0..page_count).map(move |page_count| {
            let offset = start + page_count * page_size;
            PageOffset::new(offset)
        });
        (iter, PageOffset::new(end))
    }

    /// Returns the number of bytes occupied by `page_count` pages.
    pub fn page_length(&self, page_count: usize) -> usize {
        self.0 * page_count
    }

    /// Returns the number of pages intersected by the byte range.
    ///
    /// Any page containing at least one byte of the range is included in the count.
    pub fn touched_page_count(&self, start: usize, length: usize) -> usize {
        if length == 0 {
            return 0;
        }
        let raw_end = start.checked_add(length)
            .expect(&format!("page range overflowed ({} + {})", start, length));
        let start = self.align_down(start);
        let end = self.align_up(raw_end);
        let span = end.as_inner() - start.as_inner();
        span >> self.0.trailing_zeros()
    }

    /// Returns the number of pages that may safely be reclaimed.
    ///
    /// Unlike [`touched_page_count`], a trailing partially covered page is excluded,
    /// since reclaiming it could discard bytes beyond the requested range.
    pub fn reclaimable_pages_count(&self, byte_range: ByteRange) -> usize {
        if byte_range.length() == 0 {
            return 0;
        }
        let raw_end = byte_range.start().checked_add(byte_range.length())
            .expect(&format!("page range overflowed ({} + {})", byte_range.start(), byte_range.length()));
        let start = self.align_down(byte_range.start());
        let end = self.align_down(raw_end);
        let span = end.as_inner() - start.as_inner();
        span >> self.0.trailing_zeros()
    }

    /// Returns whether `offset` lies on a page boundary.
    pub fn is_aligned(&self, offset: usize) -> bool {
        offset % self.0 == 0
    }
}

#[cfg(unix)]
use libc::{
    sysconf,
    _SC_PAGESIZE,
};

#[cfg(unix)]
impl PageSize {

    /// Returns the operating system's memory page size on `Unix`
    /// systems, or PageSizeError if an issue is encountered.
    pub fn initialize() -> Result<Self, PageSizeError> {
        // Safety:
        // Page size is a core value which should be available
        // on all Unix systems, and if it isn't, the error is handled.
        let raw_page_size = unsafe {
            sysconf(_SC_PAGESIZE)
        };
        if raw_page_size < 0 {
            Err(PageSizeError(format!("unknown error: {}", raw_page_size)))
        } else if raw_page_size == 0 {
            Err(PageSizeError("Page size found is invalid (0)".to_owned()))
        } else if !(raw_page_size as usize).is_power_of_two() {
            Err(PageSizeError("Page size found is invalid (not 2^n)".to_owned()))
        } else {
            Ok(Self(raw_page_size as usize))
        }
    }
}

#[cfg(windows)]
use windows_sys::Win32::System::SystemInformation::{
    GetSystemInfo,
    SYSTEM_INFO,
};
use crate::file::distribution::Capacity;
use crate::io::mmap::ByteRange;
use crate::record::batch::BatchIndex;

#[cfg(windows)]
impl PageSize {
    /// Returns the operating system's memory page size on `Windows`
    /// systems. This method does **not** ever return PageSizeError.
    pub fn initialize() -> Result<Self, PageSizeError> {
        let mut system_info = MaybeUninit::<SYSTEM_INFO>::uninit();
        // Safety:
        // - `system_info.as_mut_ptr()` provides a valid, aligned, writable pointer to
        //   sufficiently allocated memory (`MaybeUninit<SYSTEM_INFO>`).
        // - `GetSystemInfo` is guaranteed by Windows to populate this memory completely,
        //   making `assume_init()` safe to call immediately afterward.
        let page_size = unsafe {
            GetSystemInfo(system_info.as_mut_ptr());
            system_info.assume_init().dwPageSize as usize
        };
        if page_size == 0 {
            Err(PageSizeError("Page size found is invalid (0)".to_owned()))
        } else if !(page_size).is_power_of_two() {
            Err(PageSizeError("Page size found is invalid (not 2^n)".to_owned()))
        } else {
            Ok(Self(page_size))
        }
    }
}

#[cfg(not(any(unix, windows)))]
impl PageSize {
    /// Returns PageSizeError on non-`Unix` and non-`Windows` systems.
    /// This method does **not** ever return PageSize.
    /// [`custom()`](PageSize::custom) must be used instead.
    pub fn initialize() -> Result<Self, PageSizeError> {
        // On non-Unix/Windows systems, the unsafe custom page size **must** be used.
        Err(PageSizeError("Page size cannot be found, must use custom size.".to_owned()))
    }
}

/// Error raised when a HintThreshold is badly instantiated.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HintThresholdError {
    /// The provided value exceeds the maximum allowable threshold.
    OverFlow,

    /// The provided value falls below the minimum allowable threshold.
    UnderFlow,
}

impl Display for HintThresholdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::OverFlow => write!(f, "HintThreshold overflow: attempted new with threshold above max"),
            Self::UnderFlow => write!(f, "HintThreshold underflow: attempted new with threshold under min"),
        }
    }
}

impl Error for HintThresholdError {}

/// Represents the number of remaining uses at which point a hint
/// should be given to start loading the next batch.
/// The [`Capacity`](Capacity) of trackers
/// is compared against it by the internal
/// [`BatchManager`](crate::file::trackin::BatchManager).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HintThreshold(u64);
impl HintThreshold {
    /// The largest threshold count representable by this type.
    pub const MAX: u64 = Capacity::MAX;

    /// The smallest threshold count representable by this type.
    pub const MIN: u64 = Capacity::MIN;

    /// Returns `Ok(Capacity)`, or
    /// [`Err(HintThresholdError)`](HintThresholdError)
    /// if `threshold` exceeds [`MAX`](Capacity::MAX) or is `0`.
    pub fn new(threshold: u64) -> Result<Self, HintThresholdError> {
        if threshold > Self::MAX {
            return Err(HintThresholdError::OverFlow)
        }
        if threshold < Self::MIN {
            return Err(HintThresholdError::UnderFlow)
        }
        Ok(Self(threshold))
    }

    /// Returns `true` if the threshold has been hit.
    pub fn hit_by(&self, capacity: Capacity) -> bool {
        self.0 == capacity.count()
    }
}

impl From<HintThreshold> for u64 {
    fn from(threshold: HintThreshold) -> Self {
        threshold.0
    }
}

impl From<HintThreshold> for Capacity {
    fn from(threshold: HintThreshold) -> Capacity {
        // HintThreshold enforces identical bounds to Capacity at construction.
        Capacity::new(threshold.into()).unwrap()
    }
}

impl TryFrom<u64> for HintThreshold {
    type Error = HintThresholdError;
    fn try_from(threshold: u64) -> Result<Self, Self::Error> {
        HintThreshold::new(threshold)
    }
}

/// todo
#[derive(Debug, Copy, Clone)]
pub struct HintLookaheadError;

impl Display for HintLookaheadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Hint lookahead must be non-zero")
    }
}

impl Error for HintLookaheadError {}


/// todo
#[derive(Copy, Clone)]
pub struct HintLookahead(u32);

impl HintLookahead {
    /// todo
    pub fn new(u: u32) -> Result<Self, HintLookaheadError> {
        if u == 0 {
            Err(HintLookaheadError)
        } else {
            Ok(Self(u))
        }
    }

    pub fn initial_batch_indexes(&self) -> Vec<BatchIndex> {
        (0..self.0).map(|i| i.into()).collect()
    }

    /// todo
    pub fn next_batch_index(&self, batch_index: BatchIndex) -> BatchIndex {
        // Panic behavior used specifically because other limits, like
        // FlatBuffer's 2 GB serialized buffer limit, prevent reaching this
        // point through correct behavior.
        BatchIndex::new(batch_index.value().strict_add(self.0))
    }
}

pub struct ReaderBuilder;

pub struct Reader;