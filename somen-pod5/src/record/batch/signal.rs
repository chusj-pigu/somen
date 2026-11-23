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
            RowCount,
        },
        io::reader::{BatchRefOf, ConcurrencyMode},
        record::internal::contracts::SignalRecordContract
    };
    // local
    use crate::record::batch::columns::SignalColumns;
    use crate::record::iter::{SignalBufferIter, SignalBufferIterContract};
    use crate::record::SignalRecord;

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

    /// The core implementation of the [`SignalBatch`], it pairs an owning
    /// reference to the underlying batch with the batch's starting row
    /// within the signal table, so that [`SignalRecord`]s can be produced
    /// without re-resolving it from the registry.
    pub struct SignalBatchCore<M: ConcurrencyMode> {
        /// The owning reference to the underlying batch.
        batch_ref: BatchRefOf<M, SignalColumns>,

        /// The starting row of this batch within the signal table.
        start: FileRowIndex,
    }

    impl<M: ConcurrencyMode> SignalBatchCore<M> {
        /// Creates a `SignalBatchCore` from a resolved batch reference and
        /// its starting row within the signal table.
        pub(crate) fn new(
            batch_ref: BatchRefOf<M, SignalColumns>,
            start: FileRowIndex,
        ) -> Self {
            Self { batch_ref, start }
        }
    }

    impl<M: ConcurrencyMode> SignalBatch<M> for SignalBatchCore<M> {
        type SignalRecord = SignalRecord<M::InitialReferenceModel<SignalColumns>>;
        type SignalBufferIter = SignalBufferIter<M>;

        fn single_row(&self, global_row: FileRowIndex) -> Result<Self::SignalRecord, IndexOutOfBounds> {
            let row_count = RowCount::new(self.batch_ref.as_record_batch().num_rows() as u64);
            let local = global_row.in_range(self.start, row_count)
                .filter(|in_range| *in_range)
                .and_then(|_| global_row.local(self.start))
                .ok_or(IndexOutOfBounds(global_row.value()))?;
            Ok(SignalRecord::new(self.batch_ref.clone(), local))
        }

        fn records(&self) -> Self::SignalBufferIter {
            SignalBufferIter::new(self.batch_ref.clone())
        }
    }
}

#[cfg_attr(feature = "backend", doc = "The single-threaded variant of the [`SignalBatch`](internal::SignalBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The single-threaded variant of a `SignalBatch`.")]
pub type SignalBatch = internal::SignalBatchCore<Local>;

#[cfg_attr(feature = "backend", doc = "The concurrent variant of the [`SignalBatch`](internal::SignalBatchCore) generic.")]
#[cfg_attr(not(feature = "backend"), doc = "The concurrent variant of a `SignalBatch`.")]
pub type ConcurrentSignalBatch = internal::SignalBatchCore<Atomic>;
