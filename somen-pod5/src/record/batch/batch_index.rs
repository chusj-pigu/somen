use crate::file::{FileRowIndex, RowCount};
use crate::io::reader::ConcurrencyMode;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The index corresponding to a single [`Batch`](crate::record::batch::Batch) with a IPC file.
pub struct BatchIndex(u32);

impl BatchIndex {
    /// Returns a new `BatchIndex`.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the value of the `BatchIndex`.
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for BatchIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<BatchIndex> for usize {
    fn from(wrapper: BatchIndex) -> Self {
        wrapper.0 as usize
    }
}

/// A lookup utility to find the corresponding [`BatchIndex`] for a given `FileRowIndex`.
///
/// This struct pre-calculates cumulative row bounds for each batch to allow
/// fast binary searches over the total row space.
pub struct BatchIndexLookup {
    /// The exclusive upper bounds of row indices for each batch,
    /// stored as cumulative running sums.
    row_indexes: Vec<u64>
}

impl BatchIndexLookup {
    /// Returns a new `BatchIndexLookup`.
    ///
    /// # Panics
    ///
    /// Panics if the total number of rows exceeds `u64::MAX`. Use
    /// [`Self::try_new`] when lengths come from untrusted input.
    pub fn new(batch_lengths: &[RowCount]) -> Self {
        Self::try_new(batch_lengths).expect("batch row count overflow")
    }

    /// Creates a lookup, returning `None` if the total row count overflows.
    pub fn try_new(batch_lengths: &[RowCount]) -> Option<Self> {
        let mut row_indexes =  Vec::with_capacity(batch_lengths.len());
        let mut total: u64 = 0;
        for row_count in batch_lengths {
            total = total.checked_add(row_count.value())?;
            row_indexes.push(total);
        }
        Some(Self { row_indexes })
    }

    /// Returns the `BatchIndex` in which the [`FileRowIndex`] fits,
    /// or None if the `FileRowIndex` doesn't fit in the bounds of any batch.
    pub fn search(&self, index: FileRowIndex) -> Option<BatchIndex> {
        let batch_index = self.row_indexes.partition_point(|&end| end <= index.value());
        if batch_index == self.row_indexes.len() {
            return None;
        }
        Some(BatchIndex::new(u32::try_from(batch_index).ok()?))
    }

    /// Returns the starting [`FileRowIndex`] and total [`RowCount`] of a batch.
    ///
    /// Returns `None` if the provided `BatchIndex` is out of bounds.
    pub fn get(&self, index: BatchIndex) -> Option<(FileRowIndex, RowCount)> {
        let index: usize = index.into();
        if self.row_indexes.len() <= index {
            return None;
        }
        let start = if index == 0 { 0 } else { self.row_indexes[index - 1] };
        let end = self.row_indexes[index];
        Some((FileRowIndex::new(start), RowCount::new(end - start)))
    }
}

/// A thread-safe, stateful cache for accelerating sequence-heavy [`BatchIndex`] lookups.
///
/// Avoids repeated O(log n) binary searches by caching the active batch metrics and
/// using fast branch checks for sequential, forward-moving row lookups.
pub struct BatchIndexCache<M: ConcurrencyMode> {
    /// Reference-counted background lookup metadata mapping row blocks to batch positions.
    source: M::RefCounted<BatchIndexLookup>,
    /// Currently cached batch index identifier.
    batch: Option<BatchIndex>,
    /// Starting row index boundary of the cached batch.
    start: Option<FileRowIndex>,
    /// Number of sequential rows contained in the cached batch.
    length: Option<RowCount>,
}

impl<M: ConcurrencyMode> BatchIndexCache<M> {
    /// Creates a new `BatchIndexCache` using a reference-counted lookup utility.
    pub fn new(source: M::RefCounted<BatchIndexLookup>) -> Self {
        Self {
            source,
            batch : None,
            start : None,
            length: None,
        }
    }

    /// Finds the corresponding [`BatchIndex`] for a given [`FileRowIndex`].
    ///
    /// Evaluates if the requested row resides in the current cached batch context or the
    /// immediate subsequent batch before resorting to a global binary search.
    pub fn find_batch_index(&mut self, index: FileRowIndex) -> Option<BatchIndex> {
        if let (Some(batch), Some(start), Some(length)) = (self.batch, self.start, self.length) {
            if index.value().checked_sub(start.value()).is_some_and(|local| local < length.value()) {
                return Some(batch);
            }
            if let Some(next) = batch.value().checked_add(1) {
                let batch = BatchIndex::new(next);
                if let Some((start, length)) = self.source.get(batch) {
                    if index.value().checked_sub(start.value()).is_some_and(|local| local < length.value()) {
                        self.batch = Some(batch);
                        self.start = Some(start);
                        self.length = Some(length);
                        return Some(batch);
                    }
                }
            }
        }
        let batch = self.source.search(index)?;
        let (start, length) = self.source.get(batch)?;
        self.batch = Some(batch);
        self.start = Some(start);
        self.length = Some(length);
        Some(batch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,)]
///
pub struct BatchRange {
    /// The `start` of the `BatchRange`.
    start: usize,
    /// The `length` of the `BatchRange`.
    length: usize,
}

impl BatchRange {
    /// Returns a new `BatchRange`.
    pub fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }

    /// Returns the `start` of the `BatchRange`.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the `length` of the `BatchRange`.
    pub fn length(&self) -> usize {
        self.length
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,)]
///
pub struct TableRange {
    /// The `start` of the `TableRange`.
    start: usize,
    /// The `length` of the `TableRange`.
    length: usize,
}

impl TableRange {
    /// Returns a new `TableRange`.
    pub fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }

    /// Returns the `start` of the `TableRange`.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the `length` of the `TableRange`.
    pub fn length(&self) -> usize {
        self.length
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::reader::Local;
    use std::rc::Rc;

    #[test]
    fn lookup_skips_empty_batches_and_handles_boundaries() {
        let lookup = BatchIndexLookup::new(&[0.into(), 2.into(), 0.into(), 3.into(), 0.into()]);
        for (row, batch) in [(0, 1), (1, 1), (2, 3), (4, 3)] {
            assert_eq!(lookup.search(FileRowIndex::new(row)), Some(BatchIndex::new(batch)));
        }
        assert_eq!(lookup.search(FileRowIndex::new(5)), None);
        assert_eq!(lookup.get(BatchIndex::new(0)), Some((FileRowIndex::new(0), RowCount::new(0))));
        assert_eq!(lookup.get(BatchIndex::new(2)), Some((FileRowIndex::new(2), RowCount::new(0))));
        assert_eq!(lookup.get(BatchIndex::new(3)), Some((FileRowIndex::new(2), RowCount::new(3))));
        assert_eq!(lookup.get(BatchIndex::new(5)), None);
        assert_eq!(BatchIndexLookup::new(&[0.into()]).search(FileRowIndex::new(0)), None);
        assert!(BatchIndexLookup::try_new(&[u64::MAX.into(), 1.into()]).is_none());
    }

    #[test]
    fn cache_handles_backwards_and_empty_batch_jumps() {
        let lookup = Rc::new(BatchIndexLookup::new(&[2.into(), 0.into(), 3.into()]));
        let mut cache = BatchIndexCache::<Local>::new(lookup);
        for (row, batch) in [(0, Some(0)), (2, Some(2)), (4, Some(2)), (0, Some(0)), (5, None)] {
            assert_eq!(cache.find_batch_index(FileRowIndex::new(row)), batch.map(BatchIndex::new));
        }
    }
}
