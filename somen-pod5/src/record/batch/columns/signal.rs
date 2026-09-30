// standard

// third party
use arrow::array::RecordBatch;
// local
use crate::{
    file::schema::{
        ColumnSchema,
        SignalSchema,
        SignalSchema::*,
    },
    record::batch::{
        BatchError,
        columns::arrays::*,
        internal::BatchColumns,
    }
};

/// The set of [`columns`](BatchColumns) for 
/// [`SignalBatchCore`](crate::record::batch::internal::SignalBatchCore).
pub struct SignalColumns {
    read_id: UuidArray,
    signal: LargeDataSet,
    samples: UInt32Array,
}

impl BatchColumns for SignalColumns {
    fn new(source: &RecordBatch) -> Result<Self, BatchError> {
        Ok(Self{
            read_id: SignalSchema::try_from_array_ref(source, ReadId)?,
            signal: SignalSchema::try_from_array_ref(source, Signal)?,
            samples: SignalSchema::try_from_array_ref(source, Samples)?,
        })
    }
}

impl SignalColumns {
    // --- Core Column ---
    /// Returns the stored down-casted `read_id` column.
    pub fn read_id_column(&self) -> &UuidArray {
        &self.read_id
    }

    // --- Core Column ---
    /// Returns the stored down-casted `read_id` column.
    pub fn signal_column(&self) -> &LargeDataSet {
        &self.signal
    }

    // --- Core Column (Recoverable) ---
    /// Returns the stored down-casted `read_id` column.
    pub fn samples_column(&self) -> &UInt32Array {
        &self.samples
    }
}