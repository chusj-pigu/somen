// standard
use std::{sync::atomic::{
    AtomicBool,
    Ordering,
}, thread::{
    JoinHandle,
    spawn,
    park,
}, ops::{
    Deref,
    DerefMut,
}, fmt};
use std::cell::UnsafeCell;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
// third party

// local
use crate::{
    record::{
        SignalData,
        SignalError,
        SignalRecord,
        batch::internal::SignalColumns,
        internal::contracts::SignalRecordContract,
        types::LargeData
    },
    io::{
        reader::ConcurrencyMode,
        vendor::decode,
    },
};
use crate::io::reader::RefCounted;
use crate::record::{ReadError, SignalBuffer};
use crate::record::types::Uuid;

struct BoxRef<T> (*mut T);

impl<T> BoxRef<T> {
    fn new(source: &mut Box<T>) -> Self {
        Self (source.as_mut() as *mut T)
    }
}

impl<T> Deref for BoxRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> DerefMut for BoxRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

/// Safety:
/// mutations are done by a single thread and
/// are guarantied to be finished before the record is released.
unsafe impl<T> Send for BoxRef<T> {}

type Task<M: ConcurrencyMode> = SignalRecord<M::InitialReferenceModel<SignalColumns>>;

enum Operation<M: ConcurrencyMode> {
    Decompress(BoxRef<Task<M>>),
    Pause,
    Stop,
}

/// todo
struct Queue<M: ConcurrencyMode> {
    /// todo
    current: usize,

    /// todo
    last: usize,

    /// todo
    tasks: Vec<Option<BoxRef<Task<M>>>>,

    /// todo
    stop: bool,
}

impl<M: ConcurrencyMode> Queue<M> {
    /// todo
    fn with_capacity(capacity: usize) -> Self {
        let mut tasks = Vec::with_capacity(capacity);
        tasks.resize_with(capacity, || None);
        Self {
            current: 0,
            last: 0,
            tasks,
            stop: false,
        }
    }

    fn count(&self) -> usize {
        if self.last >= self.current {
            self.last - self.current
        } else {
            self.last + self.tasks.capacity() - self.current
        }
    }

    /// todo
    fn next(&mut self) -> Operation<M> {
        if self.stop {
            return Operation::Stop;
        }
        if self.current == self.last {
            return Operation::Pause;
        }
        self.current = self.current.strict_add(1);
        if self.current == self.tasks.capacity() {
                self.current = 0;
        }

        Operation::Decompress(self.tasks.remove(self.current).expect("None in tracked bounds"))
    }

    fn fit(&mut self, count: usize) {
        let old_count = if self.last >= self.current {
            // Safety:
            // just checked that it will not underflow
            unsafe {
                self.last.unchecked_sub(self.current)
            }
        } else {
            self.last.strict_add(self.tasks.capacity().strict_sub(self.current))
        };
        let total_count = count - old_count;
        if total_count > self.tasks.capacity() {
            let mut tasks = Vec::with_capacity(total_count);
            while self.current != self.last {
                self.current = self.current.strict_add(1);
                if self.current == old_count {
                    self.current = 0;
                }
                tasks.push(self.tasks[self.current].take());
            }
            for _ in old_count..total_count {
                tasks.push(None)
            }
            self.current = 0;
            self.last = old_count;
            self.tasks = tasks;
        }
    }

    fn push(&mut self, task: BoxRef<Task<M>>) {
        let mut current = self.current.strict_add(1);
        if current == self.tasks.capacity() {
            current = 0;
        }
        if current == self.last {
            panic!("tasks capacity exceeded");
        }
        self.current = current;
        self.tasks[current].replace(task);
    }

    fn push_many(&mut self, task_count: usize, tasks: impl Iterator<Item = BoxRef<Task<M>>>) {
        self.fit(task_count);
        for task in tasks {
            self.push(task);
        }
    }
}

macro_rules! get_lock_and_work {
    ($self:expr, $body: block) => {
        let mut spins = 0u8;
        loop {
            if !($self.locked).fetch_or(true, Ordering::Acquire) {
                $body
            }
            if spins < 255 {
                core::hint::spin_loop();
            } else {
                std::thread::yield_now();
            }
            spins += 1;
        }
    };
}

/// todo
struct Worker<M: ConcurrencyMode> {
    /// todo
    queue: BoxRef<Queue<M>>,

    /// todo
    parked: BoxRef<bool>,

    /// todo
    locked: BoxRef<AtomicBool>,
}

unsafe impl<M: ConcurrencyMode> Send for Worker<M> {}

impl<M: ConcurrencyMode> Worker<M> {
    fn decompress(task: &BoxRef<Task<M>>) -> SignalData {
        match task.raw_signal() {
            Ok(LargeData::VBZ(bytes)) => {
                let data = match decode(bytes).map_err(SignalError::DecompressionError) {
                    Ok(data) => data,
                    Err(e) => return e.into(),
                };

                let len = match task.samples() {
                    Ok(len) => len as usize,
                    Err(e) => return e.into(),
                };

                if data.len() != len {
                    return SignalError::WrongSignalQuantity {
                        expected: len,
                        found: data.len()
                    }.into();
                }

                SignalData::DecompressedVBZ(data)
            },
            _ => unreachable!("raw data and errors should never be passed to the decompression pipeline"),
        }
    }
    fn start(mut self) -> impl FnOnce() {
        move || {
            get_lock_and_work!(&self, {
                match self.queue.next() {
                    Operation::Decompress(mut task) => {
                        self.locked.store(false, Ordering::Release);
                        let data = Self::decompress(&task);
                        get_lock_and_work!(&self, {
                            task.set_data(data);
                            self.locked.store(false, Ordering::Release);
                            break;
                        });
                    },
                    Operation::Pause => {
                        *self.parked = true;
                        self.locked.store(false, Ordering::Release);
                        park();
                    },
                    Operation::Stop => {
                        self.locked.store(false, Ordering::Release);
                        break;
                    },
                }
            });
        }
    }

    /// todo
    fn new(queue: &mut Box<Queue<M>>, lock: &mut Box<AtomicBool>) -> WorkerHandle {
        let mut parked_original = Box::new(false);

        let worker = Worker {
            queue: BoxRef::new(queue),
            parked: BoxRef::new(&mut parked_original),
            locked: BoxRef::new(lock),
        };

        let handle = spawn(worker.start());

        WorkerHandle::new(handle, parked_original)
    }
}

/// todo
struct WorkerHandle {
    /// todo
    handle: JoinHandle<()>,

    /// todo
    parked: Box<bool>,
}

impl WorkerHandle {
    fn new(handle: JoinHandle<()>, parked: Box<bool>) -> WorkerHandle {
        WorkerHandle {
            handle,
            parked,
        }
    }

    fn notify(&mut self) {
        if *self.parked {
            *self.parked = false;
            self.handle.thread().unpark();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// todo
pub enum DecoderError {
    /// todo
    SignalRecords(Arc<Vec<SignalError>>),

    /// todo
    ReadIdNotFound([u8;16]),
}

impl Display for DecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignalRecords(ref_counted) => {
                write!(f, "signal-records refs: ")?;
                for e in ref_counted.iter() {
                    write!(f, "{:?}", e)?;
                }
                write!(f, ")")
            },
            Self::ReadIdNotFound(id) => write!(f, "invalid ID: {}", Uuid::from_bytes(id)),
        }
    }
}

impl Error for DecoderError {}

/// todo
pub(crate) struct Order<M: ConcurrencyMode> {
    /// todo
    signal_total: u64,

    /// todo
    uuid: [u8; 16],

    /// todo
    data: Vec<Box<Task<M>>>,

    /// todo
    task_count: usize,
}

impl<M: ConcurrencyMode> Order<M> {
    // I think that we chose o have an itterator to not have to do copying. we should be able to do
    // indexes.to_iter.map(index->record) (using the registry) and pass it in
    pub(crate) fn new(
        signal_total: u64,
        uuid: Uuid,
        task_count: usize,
        signal_records: impl Iterator<Item = Task<M>>
    ) -> Option<Self> {
        let mut data = Vec::with_capacity(task_count);
        for task in signal_records {
            data.push(Box::new(task));
        }
        if data.is_empty() {
            None
        } else {
            Some(Self {
                signal_total,
                uuid: uuid.to_bytes(),
                data,
                task_count,
            })
        }


    }

    fn id_matches(&self, uuid: Uuid) -> bool {
        uuid == self.uuid
    }

    fn buffer_ready(&self) -> bool {
        for task in &self.data {
            if let SignalData::Unset = task.data() {
                return false
            }
        }
        true
    }

    /// Safety: caller must ensure buffer is ready to be consumed
    pub(crate) unsafe fn to_tasks(self) -> Vec<Box<Task<M>>> {
        self.data
    }

    fn finish_task(&mut self) {
        self.task_count -= 1;
    }

    fn finished(&self) -> bool {
        self.task_count == 0
    }
}

/// todo
pub struct SignalDecoder<M: ConcurrencyMode> {
    /// todo
    workers: UnsafeCell<Vec<WorkerHandle>>,

    /// todo
    orders: UnsafeCell<Vec<Order<M>>>,

    /// todo
    queue: UnsafeCell<Box<Queue<M>>>,

    /// todo
    locked: Box<AtomicBool>,
}

impl<M: ConcurrencyMode> SignalDecoder<M> {
    fn new(worker_count: usize) -> Self {
        let orders = UnsafeCell::new(
            Vec::with_capacity(
                worker_count.unbounded_shl(1)
            )
        );
        let mut queue = UnsafeCell::new(
            Box::new(
                Queue::with_capacity(
                    worker_count.unbounded_shl(2)
                )
            )
        );
        let mut locked = Box::new(AtomicBool::new(true));
        let mut workers = UnsafeCell::new(
            Vec::with_capacity(worker_count)
        );
        for _ in 0 ..worker_count {
            workers.get_mut().push(Worker::new(queue.get_mut(), &mut locked));
        }
        locked.store(false, Ordering::Release);
        Self {
            orders,
            queue,
            workers,
            locked,
        }
    }

    unsafe fn add_order(&self, order: Order<M>) {
        // Safety
        let orders = unsafe {
            &mut *self.orders.get()
        };
        orders.push(order);
        // Safety:
        // Vec is not empty.
        let order = unsafe {
            orders.last_mut().unwrap_unchecked()
        };
        let count = order.data
            .iter()
            .filter(|signal_record| signal_record.is_compressed())
            .count();

        let vbz_records = order.data
            .iter_mut()
            .map(|signal_record| {
                match signal_record.raw_signal() {
                    Ok(LargeData::VBZ(_)) => {},
                    Ok(LargeData::Raw(_)) => signal_record.set_data(SignalData::Raw),
                    Err(e) => signal_record.set_data(SignalData::Error(e)),
                }
                signal_record
            })
            .filter(|signal_record| signal_record.is_compressed())
            .map(|signal_box| {BoxRef::new(signal_box)});
        // Safety
        unsafe {
            &mut *self.queue.get()
        }.push_many(count, vbz_records)
    }
    
    fn decompress_fallback(&self) {
        get_lock_and_work!(&self, {
            // Safety;
            let queue = unsafe {
                &mut *self.queue.get()
            };
            match queue.next() {
                Operation::Decompress(mut task) => {
                    self.locked.store(false, Ordering::Release);
                    let data = Worker::<M>::decompress(&task);
                    get_lock_and_work!(&self, {
                        task.set_data(data);
                        self.locked.store(false, Ordering::Release);
                        break;
                    });
                },
                _ => {
                    self.locked.store(false, Ordering::Release);
                    break
                },
            }
        });
    }
    
    pub(crate) fn decompress(&self, order: Order<M>) {
        let mut self_work = false;
        get_lock_and_work!(&self, {
            // Safety;
            // Called under lock
            unsafe {
                self.add_order(order)
            }
            // Safety;
            let workers = unsafe {
                &mut *self.workers.get()
            };
            if !workers.is_empty() {
                for worker in workers {
                    worker.notify();
                }
                self.locked.store(false, Ordering::Release);
                break;
            } else {
                self_work = true;
            }
            break
        });
        if self_work {
            self.decompress_fallback()
        }
    }

    pub(crate) fn get_buffer(&self, uuid: Uuid) -> Result<SignalBuffer<M::InitialReferenceModel<SignalColumns>>, DecoderError> {
        get_lock_and_work!(&self, {
            // Safety;
            let orders = unsafe {
                &mut *self.orders.get()
            };
            let index = orders.iter()
            .position(|order| order.id_matches(uuid))
            .ok_or(DecoderError::ReadIdNotFound(uuid.to_bytes()))?;

            if orders[index].buffer_ready() {
                let order = orders.remove(index);
                self.locked.store(false,Ordering::Release);

                // Safety:
                // Just checked that the buffer is ready, while under lock.
                return unsafe {
                    Ok(SignalBuffer::<M::InitialReferenceModel<SignalColumns>>::try_from_order(order)?)
                };
            }
        });
    }
}

/*
variants to consider for future update;

struct ByteSlice {
    start: *mut i16,
    length: usize,
}

struct Queue {
    current: usize,
    last: usize,
    tasks: Vec<(*mut SignalRecord, ByteSlice)>
}

struct Decompressor {
    data: Vec<(SignalTotal, Uuid, Vec<SignalRecord>, Vec<i16>)>,
    queue: Queue,
    workers: Vec<WorkerHandle>,
    lock: AtomicBool,
}

TLDR: moving data storage to singular continuous vector to remove secondary copying step
to obtain continuous vector in VBZ compressed files requires adjusting the svb library to use
mutable slices instead of mutable vectors, as well as introducing relevant errors where necessary.

This change mostly leaves the api identical from the user side, only adding new error types.
This should therefore pass relativelly easilly.
*/
