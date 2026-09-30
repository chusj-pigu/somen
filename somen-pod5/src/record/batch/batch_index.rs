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
    pub fn new(batch_lengths: &[RowCount]) -> Self {
        let mut row_indexes =  Vec::with_capacity(batch_lengths.len());
        let mut total: u64 = 0;
        for row_count in batch_lengths {
            total += row_count.value() as u64;
            // Note: storing total - 1 because binary_search is upper bound inclusive.
            // ex: with [2,4], inputing 0, 1 or 2 returns 0 while 3 or 4 return 1.
            row_indexes.push(total - 1);
        }
        Self { row_indexes }
    }

    /// Returns the `BatchIndex` in which the [`FileRowIndex`] fits,
    /// or None if the `FileRowIndex` doesn't fit in the bounds of any batch.
    pub fn search(&self, index: FileRowIndex) -> Option<BatchIndex> {
        let indexes = &self.row_indexes;
        let value = &index.value();
        if value > indexes.last()? {
            return None;
        }
        let result = indexes.binary_search(value);
        let batch_index = result.unwrap_or_else(|batch_index| batch_index);
        Some(BatchIndex::new(batch_index as u32))
    }

    /// Returns the starting [`FileRowIndex`] and total [`RowCount`] of a batch.
    ///
    /// Returns `None` if the provided `BatchIndex` is out of bounds.
    pub fn get(&self, index: BatchIndex) -> Option<(FileRowIndex, RowCount)> {
        let index: usize = index.into();
        if self.row_indexes.len() <= index {
            return None;
        }
        let start: FileRowIndex;
        if index == 0 {
            start = FileRowIndex::new(0)
        } else {
            start = {self.row_indexes[index - 1] + 1}.into();
        }
        let end: FileRowIndex = {self.row_indexes[index] + 1}.into();
        let end = end.count_rows(start)?;
        Some((start.into(),end))
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
            if index.in_range(start, length)? {
                return Some(batch);
            }
            let batch = BatchIndex::new(batch.value() + 1);
            let (start, length) = self.source.get(batch)?;
            if index.in_range(start, length)? {
                self.batch = Some(batch);
                self.start = Some(start);
                self.length = Some(length);
                return Some(batch);
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



/*

*/