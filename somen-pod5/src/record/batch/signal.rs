// standard

// local
use crate::io::reader::{Atomic, Local};

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
        record::internal::contracts::SignalRecordContract
    };
    use crate::record::batch::BatchCore;
    // local
    use crate::record::batch::columns::SignalColumns;
    use crate::record::iter::{SignalBufferIter, SignalBufferIterContract};

    /// Defines the operations of a `SignalBatch`.
    pub trait SignalBatch<M: ConcurrencyMode> {
        /// The [`SignalRecord`] implementation returned by [`single_row()`](Self::single_row).
        type SignalRecord: SignalRecordContract<M::InitialReferenceModel<SignalColumns>>;

        /// The [`SignalBufferIter`] implementation returned by [`records()`](Self::records).
        type SignalBufferIter: SignalBufferIterContract<M>;

        /// Returns a single [`SignalRecord`](Self::SignalRecord) by
        /// its [`FileRowIndex`].
        ///
        /// # Errors
        ///
        /// Returns [`IndexOutOfBounds`] if `global_row` points outside the
        /// current batch's boundaries.
        fn single_row(&self, global_row: FileRowIndex) -> Result<Self::SignalRecord, IndexOutOfBounds>;

        /// Returns a variant of a [`SignalBufferIter`](Self::SignalBufferIter) of the records in this batch.
        fn records(&self) -> Self::SignalBufferIter;
    }

    pub type SignalBatchCore<M: ConcurrencyMode> = BatchCore<M, SignalColumns>;

    // impl<M: ConcurrencyMode> SignalBatch<M> for SignalBatchCore<M> {}
}

#[cfg_attr(feature = "backend", doc = "The single-threaded variant of the [`SignalBatch`](internal::SignalBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The single-threaded variant of a `SignalBatch`.")]
pub type SignalBatch = internal::SignalBatchCore<Local>;

#[cfg_attr(feature = "backend", doc = "The concurrent variant of the [`SignalBatch`](internal::SignalBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The concurrent variant of a `SignalBatch`.")]
pub type ConcurrentSignalBatch = internal::SignalBatchCore<Atomic>;