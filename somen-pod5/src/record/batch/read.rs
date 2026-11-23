// standard

// third party

// local
use crate::io::reader::{
    Atomic,
    Local,
};

pub(crate) mod internal {
    // standard

    // third party

    // local
    use crate::{
        file::{
            FileRowIndex,
            IndexOutOfBounds,
            RowCount,
        },
        io::reader::{
            BatchRefOf,
            ConcurrencyMode,
            Pod5ContextHandleOf,
        },
        record::{
            batch::columns::ReadColumns,
            internal::contracts::ReadRecordContract,
            iter::{ReadIterContract, ReadIter},
            ReadRecord,
        }
    };

    /// Defines the operations of a `ReadBatch`.
    pub trait ReadBatch<M: ConcurrencyMode> {
        /// The [`ReadRecord`] implementation returned by [`single_row()`](Self::single_row).
        type ReadRecord: ReadRecordContract<M::InitialReferenceModel<ReadColumns>>;

        /// The [`ReadIter`] implementation returned by [`records()`](Self::records).
        type ReadIter: ReadIterContract<M>;

        /// Returns a single [`ReadRecord`](Self::ReadRecord) by
        /// its [`FileRowIndex`].
        ///
        /// # Errors
        ///
        /// Returns [`IndexOutOfBounds`] if `global_row` points outside the
        /// current batch's boundaries.
        fn single_row(&self, global_row: FileRowIndex) -> Result<Self::ReadRecord, IndexOutOfBounds>;

        /// Returns a variant of a [`ReadIter`](Self::ReadIter) of the records in this batch.
        fn records(&self) -> Self::ReadIter;
    }

    /// The core implementation of the [`ReadBatch`], it pairs an owning
    /// reference to the underlying batch with the batch's starting row
    /// within the read table and a handle to the shared decoding context,
    /// so that [`ReadRecord`]s can be produced without re-resolving either
    /// from the registry.
    pub struct ReadBatchCore<M: ConcurrencyMode> {
        /// The owning reference to the underlying batch.
        batch_ref: BatchRefOf<M, ReadColumns>,

        /// The starting row of this batch within the read table.
        start: FileRowIndex,

        /// Handle used to share access to the registry when producing records.
        context: Pod5ContextHandleOf<M, ReadColumns>,
    }

    impl<M: ConcurrencyMode> ReadBatchCore<M> {
        /// Creates a `ReadBatchCore` from a resolved batch reference, its
        /// starting row within the read table, and the shared decoding context.
        pub(crate) fn new(
            batch_ref: BatchRefOf<M, ReadColumns>,
            start: FileRowIndex,
            context: Pod5ContextHandleOf<M, ReadColumns>,
        ) -> Self {
            Self { batch_ref, start, context }
        }
    }

    impl<M: ConcurrencyMode> ReadBatch<M> for ReadBatchCore<M> {
        type ReadRecord = ReadRecord<M::InitialReferenceModel<ReadColumns>>;
        type ReadIter = ReadIter<M>;

        fn single_row(&self, global_row: FileRowIndex) -> Result<Self::ReadRecord, IndexOutOfBounds> {
            let row_count = RowCount::new(self.batch_ref.as_record_batch().num_rows() as u64);
            let local = global_row.in_range(self.start, row_count)
                .filter(|in_range| *in_range)
                .and_then(|_| global_row.local(self.start))
                .ok_or(IndexOutOfBounds(global_row.value()))?;
            Ok(ReadRecord::new(self.batch_ref.clone(), local, self.context.clone()))
        }

        fn records(&self) -> Self::ReadIter {
            ReadIter::new(self.batch_ref.clone(), self.context.clone())
        }
    }
}

#[cfg_attr(feature = "backend", doc = "The single-threaded variant of the [`ReadBatch`](internal::ReadBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The single-threaded variant of a `ReadBatch`.")]
pub type ReadBatch = internal::ReadBatchCore<Local>;

#[cfg_attr(feature = "backend", doc = "The concurrent variant of the [`ReadBatch`](internal::ReadBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The concurrent variant of a `ReadBatch`.")]
pub type ConcurrentReadBatch = internal::ReadBatchCore<Atomic>;
