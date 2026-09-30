// standard
use std::{
    fmt::{
        Display,
        Formatter,
        Result,
    },
    error::Error,
};
// third party

// local


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The `RowIndex` within a whole [`Pod5`](crate::file::internal::backend::Pod5) table, also know as a whole IPC arrow file.
pub struct FileRowIndex(u64);

impl FileRowIndex {
    /// Returns a new `FileRowIndex`.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Converts a slice of `u64` into a slice of `FileRowIndexes`.
    pub fn from_slice(values: &[u64]) -> &[Self] {
        // Safety:
        // Layout is identical and ReadIterIndex has no invariants
        unsafe {
            std::slice::from_raw_parts(
                values.as_ptr() as *const Self,
                values.len()
            )
        }
    }

    /// Returns the value of the `FileRowIndex`.
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Returns the [`RowCount`] for this `RowIndex` based on the start
    /// `RowIndex` or None if it's not a valid `RowCount`.
    pub fn count_rows(&self, start: Self) -> Option<RowCount> {
        let difference = self.value() - start.value();
        let value = difference.try_into().ok()?;
        Some(RowCount::new(value))
    }

    /// Returns the [`BatchRowIndex`] for this `RowIndex` based on the start
    /// `RowIndex` or None if it's not a valid `BatchRowIndex`.
    pub fn local(&self, start: Self) -> Option<BatchRowIndex> {
        let difference = self.value() - start.value();
        let value = difference.try_into().ok()?;
        Some(BatchRowIndex::new(value))
    }

    /// Returns whether this `RowIndex` is within the range described by
    /// the start `RowIndex` and the length [`RowCount`], or None if the
    /// `RowIndex` is not a valid [`BatchRowIndex`] from `start`.
    pub fn in_range(&self, start: Self, length: RowCount) -> Option<bool> {
        let local = self.local(start)?.value() as u64;
        Some(local < length.value())
    }
}

impl From<u64> for FileRowIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<FileRowIndex> for usize {
    fn from(wrapper: FileRowIndex) -> Self {
        wrapper.0 as usize
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The `RowIndex` within a single [`Batch`](crate::record::batch::Batch).
pub struct BatchRowIndex(u32);

impl BatchRowIndex {
    /// Returns a new `BatchRowIndex`.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Converts a copied slice of `u32` into `BatchRowIndexes`.
    pub fn from_slice(values: &[u32]) -> Vec<Self> {
        values.iter().copied().map(BatchRowIndex::from).collect()
    }

    /// Returns the value of the `BatchRowIndex`.
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for BatchRowIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<BatchRowIndex> for usize {
    fn from(wrapper: BatchRowIndex) -> Self {
        wrapper.0 as usize
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The number of rows within a single [`Batch`](crate::record::batch::Batch).
pub struct RowCount(u64);

impl RowCount {
    /// Returns a new `RowCount`.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the value of the `RowCount`.
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for RowCount {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<RowCount> for usize {
    fn from(wrapper: RowCount) -> Self {
        wrapper.0 as usize
    }
}

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// An error where the user passes in an index which is too big.
pub struct IndexOutOfBounds(pub u64);

impl From<usize> for IndexOutOfBounds {
    fn from(u: usize) -> Self {
        Self(u as u64)
    }
}

impl Display for IndexOutOfBounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Index {} is out of bounds", self.0)
    }
}

impl Error for IndexOutOfBounds {}
