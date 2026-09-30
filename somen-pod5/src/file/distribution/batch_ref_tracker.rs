// standard
use std::{
    error::Error,
    fmt::{
        self,
        Display,
        Formatter,
    },
    rc::Rc,
    cell::Cell,
    sync::{
        atomic::{
            AtomicU64,
            Ordering,
        },
        Arc,
    },
};
// third party
use arc_swap::ArcSwapOption;
// local
use crate::{
    record::batch::{
        BatchResult,
        internal::BatchColumns,
    },
    io::reader::{
        ConcurrencyMode,
        Local,
        Atomic,
        BatchRefOf,
        RcBatchRef,
        GuardBatchRef,
        ErrorRefOf,
        RcErrorRef,
        GuardErrorRef,
        InitialBatchRef,
    }
};
/// The expected number of times a batch will be acquired.
///
/// This value is computed before iteration begins and is
/// used to initialize [`BatchUsageTrackers`](BatchRefTracker).
// While a u32 would be sufficient for the count,
// this allows for the value to be a single word,
// and lets the value carry bit flags for the concurrent tracker.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Capacity(u64);
impl Capacity {
    /// The largest usage count representable by this type.
    pub const MAX: u64 = u64::MAX >> 3;

    /// The smallest usage count representable by this type.
    pub const MIN: u64 = 1u64;

    /// Bit mask for the load bit.
    pub const IS_LOADED: u64 = 1 << 63;

    /// Bit mask for the unload bit.
    pub const WAS_UNLOADED: u64 = 1 << 62;

    /// Bit mask for the lock bit.
    pub const IS_LOCKED: u64 = 1 << 61;

    /// Bit mask for the count bits.
    const COUNT_MASK: u64 = Self::MAX;

    /// Returns `Some(Capacity)`, or `None` if `capacity`
    /// exceeds [`Self::MAX`] or is `0`.
    pub fn new(capacity: u64) -> Option<Self> {
        if capacity > Self::MAX {
            return None
        }
        if capacity < Self::MIN {
            return None;
        }
        Some(Self(capacity))
    }

    /// Returns whether the load was successfully logged.
    pub fn load(&mut self) -> bool {
        if self.is_loaded() {
            return false
        }
        self.0 |= Self::IS_LOADED;
        true
    }

    /// Updates the capacity's unload bit to true.
    pub fn unload(&mut self) {
        self.0 |= Self::WAS_UNLOADED;
    }

    /// Returns whether the usage was succesfully logged.
    pub fn take(&mut self) -> bool {
        if self.is_exhausted() {
            return false
        }
        self.0 = self.0.strict_sub(1);
        true
    }

    /// Returns the count of the capacity.
    pub fn count(&self) -> u64 {
        self.0 & Self::COUNT_MASK
    }

    /// Returns whether the data is currently loaded or not.
    pub fn is_loaded(&self) -> bool {
        self.0 & Self::IS_LOADED != 0
    }

    /// Returns whether the data was ever unloaded or not.
    pub fn was_unloaded(&self) -> bool {
        self.0 & Self::WAS_UNLOADED != 0
    }

    /// Returns whether the lock bit is set or not.
    pub fn is_locked(&self) -> bool {
        self.0 & Self::IS_LOCKED != 0
    }

    /// Returns whether the capacity is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.count() == 0
    }
}

impl From<Capacity> for u64 {
    fn from(capacity: Capacity) -> Self {
        capacity.0
    }
}

impl Display for Capacity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,
               "IS_LOADED: {}, WAS_UNLOADED: {}, IS_LOCKED: {}, usage count: {}",
               self.is_loaded(), self.was_unloaded(), self.is_locked(), self.count()
        )
    }
}

/// todo
#[derive(Debug, Copy, Clone)]
pub struct CapacityError;

impl Display for CapacityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Capacity already exhausted")
    }
}

impl Error for CapacityError {}


/// A trait defining the functions of the `UsageTrackers` used by the
/// [`BatchManager`](crate::file::manager::BatchManager)
/// to manage [`Batches`](Batch).
pub trait BatchRefTracker<M: ConcurrencyMode, C: BatchColumns>{
    /// Creates a new `BatchUsageTracker`.
    fn with_capacity(capacity: Capacity) -> Result<Self, CapacityError> where Self: Sized;

    /// Sets the `BatchUsageTracker` to track the [`Batch`](Batch)
    /// [`source`](crate::io::reader::BatchHandle)
    /// and returns the
    /// [`Batch accessor`](crate::io::reader::LocalAccessOld)
    /// as well as the hint indictor.
    ///
    /// # Panics
    ///
    /// Panics if the `BatchUsageTracker` is already set or has previously been set.
    fn set(&self, result: BatchResult<M, C>) -> Result<(BatchRefOf<M,C>, Capacity), ErrorRefOf<M,C>>;

    /// Returns `Some` [`accessor`](crate::io::reader::LocalAccessOld)
    /// as well as the hint indictor if a [`Batch`](Batch) is
    /// currently stored, decreasing the `BatchUsageTracker`'s
    /// capacity by one, or returns `None`.
    ///
    /// # Panics
    ///
    /// Panics if the `BatchUsageTracker` has already exhausted its capacity.
    fn acquire(&self) -> Option<Result<(BatchRefOf<M,C>, Capacity), ErrorRefOf<M,C>>>;
}


/// A non-thread-safe batch [`UsageTracker`](BatchRefTracker).
///
/// This type is neither `Send` nor `Sync`.
pub struct RcBatchRefTracker<C: BatchColumns>{
    pointer: Cell<Option< Rc<BatchResult<Local, C>>>>,
    capacity: Cell<Option<Capacity>>,
}

impl<C: BatchColumns> BatchRefTracker<Local, C> for RcBatchRefTracker<C> {
    fn with_capacity(mut capacity: Capacity) -> Result<Self, CapacityError> {
        if capacity.take() { // the usage from setting the batch is static
            return Err(CapacityError);
        }
        Ok(Self {
            pointer: Cell::new(None),
            capacity: Cell::new(Some(capacity))
        })
    }

    fn set(&self, batch: BatchResult<Local, C>) -> Result<(RcBatchRef<C>, Capacity), RcErrorRef<C>> {
        let mut capacity = self.capacity.get().unwrap();
        if capacity.load() {
            unreachable!("RcBatchUsageTracker state mismanagement: attempted set when already set")
        }
        if !capacity.was_unloaded() {
            unreachable!("RcBatchUsageTracker state mismanagement: attempted set when unloaded")
        }
        let pointer = Rc::new(batch);
        self.pointer.replace(Some(pointer.clone()));
        self.capacity.replace(Some(capacity));
        match RcBatchRef::new(pointer) {
            Ok(rc_batch_ref) => Ok((rc_batch_ref, capacity)),
            Err(rc_error_ref) => Err(rc_error_ref),
        }
    }

    fn acquire(&self) -> Option<Result<(RcBatchRef<C>, Capacity), RcErrorRef<C>>> {
        let mut capacity = self.capacity.get().unwrap();
        if capacity.is_exhausted() {
            unreachable!("RcBatchUsageTracker state mismanagement: attempted acquire when capacity exhausted")
        }
        if !capacity.is_loaded() {
            if capacity.was_unloaded() {
                unreachable!("RcBatchUsageTracker state mismanagement: attempted acquire when unloaded")
            }
            return None;
        }
        let pointer;
        if capacity.count() == 1 {
            pointer = self.pointer.replace(None)?;
            capacity.unload();
        } else {
            pointer = self.pointer.take()?;
            self.pointer.set(Some(pointer.clone()));
        }
        capacity.take();
        self.capacity.set(Some(capacity));
        match RcBatchRef::new(pointer) {
            Ok(rc_batch_ref) => Some(Ok((rc_batch_ref, capacity))),
            Err(rc_error_ref) => Some(Err(rc_error_ref)),
        }
    }
}


/// A thread-safe batch [`UsageTracker`](BatchRefTracker).
///
/// Provides lock-free reads and atomic replacement of the stored batch
///
/// This type is `Send + Sync`.
pub struct ArcBatchRefTracker<C: BatchColumns> {
    pointer: ArcSwapOption<BatchResult<Atomic, C>>,
    capacity: AtomicU64,
}

impl<C: BatchColumns> BatchRefTracker<Atomic, C> for ArcBatchRefTracker<C> {
    fn with_capacity(capacity: Capacity) -> Result<ArcBatchRefTracker<C>, CapacityError> {
        let count: u64 = capacity.into();
        if count == 0 {
            return Err(CapacityError);
        }
        Ok(Self {
            pointer: ArcSwapOption::new(None),
            capacity: AtomicU64::new(count - 1), // the usage from setting the batch is static
        })
    }

    /*
    This is a *simplification*, but the ordering is safe with relaxed
    because the count operations are all on the same atomic value, so cannot be reordered.
    The block of count operations itself cannot leak over the pointer operations because
    they use SeqCst, which acts like a barrier in both reordering directions.
    */

    fn set(&self, batch: BatchResult<Atomic, C>) -> Result<(GuardBatchRef<C>, Capacity), GuardErrorRef<C>> {
        let val = self.capacity.load(Ordering::Relaxed);
        let capacity = Capacity::new(val).unwrap();
        if !capacity.is_locked() {
            unreachable!("ArcBatchUsageTracker state mismanagement: attempted set when not locked")
        }
        if capacity.is_loaded() {
            unreachable!("ArcBatchUsageTracker state mismanagement: attempted set when already set")
        }
        if capacity.was_unloaded() {
            unreachable!("ArcBatchUsageTracker state mismanagement: attempted set when unloaded")
        }
        let pointer = Arc::new(batch);
        self.pointer.store(Some(pointer));
        let guard = self.pointer.load();
        let val = self.capacity.fetch_xor(Capacity::IS_LOADED|Capacity::IS_LOCKED, Ordering::Relaxed);
        let mut capacity = Capacity::new(val).unwrap();
        if capacity.is_exhausted() {
            self.pointer.store(None);
            self.capacity.fetch_or(Capacity::WAS_UNLOADED, Ordering::Relaxed);
            capacity.unload();
        }
        match GuardBatchRef::new(guard) {
            Ok(guard_batch_ref) => Ok((guard_batch_ref, capacity)),
            Err(guard_error_ref) => Err(guard_error_ref),
        }
    }

    fn acquire(&self) -> Option<Result<(GuardBatchRef<C>, Capacity), GuardErrorRef<C>>> {
        let val = self.capacity.load(Ordering::Relaxed);
        let capacity = Capacity::new(val).unwrap();
        if capacity.is_exhausted() {
            unreachable!("ArcBatchUsageTracker state mismanagement: attempted acquire when exhausted")
        }
        if capacity.was_unloaded() {
            unreachable!("ArcBatchUsageTracker state mismanagement: attempted acquire when unloaded")
        }
        if !capacity.is_loaded() {
            if !capacity.is_locked() {
                let val = self.capacity.fetch_or(Capacity::IS_LOCKED, Ordering::Relaxed);
                if !Capacity::new(val).unwrap().is_locked() {
                    return None
                }
            }
            let mut spins = 0u8;
            while {
                let val = self.capacity.load(Ordering::Relaxed);
                Capacity::new(val).unwrap().is_locked()
            } {
                if spins < 50 {
                    core::hint::spin_loop();
                    spins += 1;
                } else {
                    spins = 0;
                    std::thread::yield_now();
                }
            }
        };
        let guard = self.pointer.load();
        let val = self.capacity.fetch_sub(1, Ordering::Relaxed);
        let mut capacity = Capacity::new(val).unwrap();
        if capacity.is_exhausted() {
            self.pointer.store(None);
            self.capacity.fetch_or(Capacity::WAS_UNLOADED, Ordering::Relaxed);
            capacity.unload()
        }
        match GuardBatchRef::new(guard) {
            Ok(guard_batch_ref) => Some(Ok((guard_batch_ref, capacity))),
            Err(guard_error_ref) => Some(Err(guard_error_ref),)
        }
    }
}