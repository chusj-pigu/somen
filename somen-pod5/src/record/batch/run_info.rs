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
        },
        io::reader::ConcurrencyMode,
        record::{
            batch::{
                columns::RunInfoColumns,
                BatchCore,
            },
            internal::contracts::RunInfoRecordContract,
            iter::{RunInfoIterContract, RunInfoIter},
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

    pub type RunInfoBatchCore<M: ConcurrencyMode> = BatchCore<M, RunInfoColumns>;

    // impl<M: ConcurrencyMode> RunInfoBatch<M> for RunInfoBatchCore<M> {}
}

#[cfg_attr(feature = "backend", doc = "The single-threaded variant of the [`RunInfoBatch`](internal::RunInfoBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The single-threaded variant of a `RunInfoBatch`.")]
pub type RunInfoBatch = internal::RunInfoBatchCore<Local>;

#[cfg_attr(feature = "backend", doc = "The concurrent variant of the [`RunInfoBatch`](internal::RunInfoBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The concurrent variant of a `RunInfoBatch`.")]
pub type ConcurrentRunInfoBatch = internal::RunInfoBatchCore<Atomic>;