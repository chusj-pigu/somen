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
            batch::columns::RunInfoColumns,
            internal::contracts::RunInfoRecordContract,
            iter::{RunInfoIterContract, RunInfoIter},
            RunInfoRecord,
        }
    };
    // local

    /// Defines the operations of a `SignalBatch`.
    pub trait RunInfoBatch<M: ConcurrencyMode> {
        /// The [`RunInfoRecordContract`] implementation returned by [`single_row()`](Self::single_row).
        type RunInfoRecord: RunInfoRecordContract<M::InitialReferenceModel<RunInfoColumns>>;

        /// The [`RunInfoIter`] implementation returned by [`records()`](Self::records).
        type RunInfoIter: RunInfoIterContract<M>;

        /// Returns a single [`RunInfoRecord`](Self::RunInfoRecord) by
        /// its [`FileRowIndex`].
        ///
        /// # Errors
        ///
        /// Returns [`IndexOutOfBounds`] if `global_row` points outside the
        /// current batch's boundaries.
        fn single_row(&self, global_row: FileRowIndex) -> Result<Self::RunInfoRecord, IndexOutOfBounds>;

        /// Returns a variant of a [`RunInfoIter`](Self::RunInfoIter) of the records in this batch.
        fn records(&self) -> Self::RunInfoIter;
    }

    /// The core implementation of the [`RunInfoBatch`], it pairs an owning
    /// reference to the underlying batch with the batch's starting row
    /// within the run-info table and a handle to the shared decoding
    /// context, so that [`RunInfoRecord`]s can be produced without
    /// re-resolving either from the registry.
    pub struct RunInfoBatchCore<M: ConcurrencyMode> {
        /// The owning reference to the underlying batch.
        batch_ref: BatchRefOf<M, RunInfoColumns>,

        /// The starting row of this batch within the run-info table.
        start: FileRowIndex,

        /// Handle used to share access to the registry when producing records.
        context: Pod5ContextHandleOf<M, RunInfoColumns>,
    }

    impl<M: ConcurrencyMode> RunInfoBatchCore<M> {
        /// Creates a `RunInfoBatchCore` from a resolved batch reference, its
        /// starting row within the run-info table, and the shared decoding context.
        pub(crate) fn new(
            batch_ref: BatchRefOf<M, RunInfoColumns>,
            start: FileRowIndex,
            context: Pod5ContextHandleOf<M, RunInfoColumns>,
        ) -> Self {
            Self { batch_ref, start, context }
        }
    }

    impl<M: ConcurrencyMode> RunInfoBatch<M> for RunInfoBatchCore<M> {
        type RunInfoRecord = RunInfoRecord<M::InitialReferenceModel<RunInfoColumns>>;
        type RunInfoIter = RunInfoIter<M>;

        fn single_row(&self, global_row: FileRowIndex) -> Result<Self::RunInfoRecord, IndexOutOfBounds> {
            let row_count = RowCount::new(self.batch_ref.as_record_batch().num_rows() as u64);
            let local = global_row.in_range(self.start, row_count)
                .filter(|in_range| *in_range)
                .and_then(|_| global_row.local(self.start))
                .ok_or(IndexOutOfBounds(global_row.value()))?;
            Ok(RunInfoRecord::new(self.batch_ref.clone(), local, self.context.clone()))
        }

        fn records(&self) -> Self::RunInfoIter {
            RunInfoIter::new(self.batch_ref.clone(), self.context.clone())
        }
    }
}

#[cfg_attr(feature = "backend", doc = "The single-threaded variant of the [`RunInfoBatch`](internal::RunInfoBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The single-threaded variant of a `RunInfoBatch`.")]
pub type RunInfoBatch = internal::RunInfoBatchCore<Local>;

#[cfg_attr(feature = "backend", doc = "The concurrent variant of the [`RunInfoBatch`](internal::RunInfoBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The concurrent variant of a `RunInfoBatch`.")]
pub type ConcurrentRunInfoBatch = internal::RunInfoBatchCore<Atomic>;
