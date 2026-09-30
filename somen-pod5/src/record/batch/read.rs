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
                columns::ReadColumns,
                BatchCore,
            },
            internal::contracts::ReadRecordContract,
            iter::{ReadIterContract, ReadIter},
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

    /// The core implementation of the [`ReadBatch`], it owns a source
    /// [`RecordBatch`], but also the downcasted column refferences
    /// to avoid downcasting repeatidly.
    pub type ReadBatchCore<M: ConcurrencyMode> = BatchCore<M, ReadColumns>;
    
    // impl<M: ConcurrencyMode> ReadBatch<M> for BatchCore<M, ReadBatchColumns> {}
}

#[cfg_attr(feature = "backend", doc = "The single-threaded variant of the [`ReadBatch`](internal::ReadBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The single-threaded variant of a `ReadBatch`.")]
pub type ReadBatch = internal::ReadBatchCore<Local>;

#[cfg_attr(feature = "backend", doc = "The concurrent variant of the [`ReadBatch`](internal::ReadBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The concurrent variant of a `ReadBatch`.")]
pub type ConcurrentReadBatch = internal::ReadBatchCore<Atomic>;