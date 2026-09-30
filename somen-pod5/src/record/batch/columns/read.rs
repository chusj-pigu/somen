// standard

// third party
use arrow::array::RecordBatch;
// local
use crate::{
    file::schema::{
        ColumnSchema,
        ReadSchema,
        ReadSchema::*,
    },
    record::batch::{
        BatchError,
        columns::arrays::*,
        internal::BatchColumns,
    }
};

/// The set of [`columns`](BatchColumns) for
/// [`ReadBatchCore`](crate::record::batch::internal::ReadBatchCore).
pub struct ReadColumns {
    read_id: UuidArray,
    signal: IndexListArray,
    channel: UInt16Array,
    well: UInt8Array,
    pore_type: StringDictionary,
    calibration_offset: Float32Array,
    calibration_scale: Float32Array,
    read_number: UInt32Array,
    start: UInt64Array,
    median_before: Float32Array,
    tracked_scaling_scale: Float32Array,
    tracked_scaling_shift: Float32Array,
    predicted_scaling_scale: Float32Array,
    predicted_scaling_shift: Float32Array,
    num_reads_since_mux_change: UInt32Array,
    time_since_mux_change: Float32Array,
    num_minknow_events: UInt64Array,
    end_reason: StringDictionary,
    end_reason_forced: BooleanArray,
    run_info: StringDictionary,
    num_samples: UInt64Array,
    open_pore_level: Float32Array,
}

impl BatchColumns for ReadColumns {
    fn new(source: &RecordBatch) -> Result<Self, BatchError> {
        Ok(Self {
            read_id: ReadSchema::try_from_array_ref(source, ReadId)?,
            signal: ReadSchema::try_from_array_ref(source, Signal)?,
            channel: ReadSchema::try_from_array_ref(source, Channel)?,
            well: ReadSchema::try_from_array_ref(source, Well)?,
            pore_type: ReadSchema::try_from_array_ref(source, PoreType)?,
            calibration_offset: ReadSchema::try_from_array_ref(source, CalibrationOffset)?,
            calibration_scale: ReadSchema::try_from_array_ref(source, CalibrationScale)?,
            read_number: ReadSchema::try_from_array_ref(source, ReadNumber)?,
            start: ReadSchema::try_from_array_ref(source, Start)?,
            median_before: ReadSchema::try_from_array_ref(source, MedianBefore)?,
            tracked_scaling_scale: ReadSchema::try_from_array_ref(source, TrackedScalingScale)?,
            tracked_scaling_shift: ReadSchema::try_from_array_ref(source, TrackedScalingShift)?,
            predicted_scaling_scale: ReadSchema::try_from_array_ref(source, PredictedScalingScale)?,
            predicted_scaling_shift: ReadSchema::try_from_array_ref(source, PredictedScalingShift)?,
            num_reads_since_mux_change: ReadSchema::try_from_array_ref(source, NumReadsSinceMuxChange)?,
            time_since_mux_change: ReadSchema::try_from_array_ref(source, TimeSinceMuxChange)?,
            num_minknow_events: ReadSchema::try_from_array_ref(source, NumMinknowEvents)?,
            end_reason: ReadSchema::try_from_array_ref(source, EndReason)?,
            end_reason_forced: ReadSchema::try_from_array_ref(source, EndReasonForced)?,
            run_info: ReadSchema::try_from_array_ref(source, RunInfo)?,
            num_samples: ReadSchema::try_from_array_ref(source, NumSamples)?,
            open_pore_level: ReadSchema::try_from_array_ref(source, OpenPoreLevel)?,
        })
    }
}

impl ReadColumns {
    // --- Core Column ---
    /// Returns the stored down-casted `read_id` column.
    pub fn read_id_column(&self) -> &UuidArray {
        &self.read_id
    }

    // --- Core Column (Recoverable) ---
    /// Returns the stored down-casted signal [`FileRowIndexes`](FileRowIndex) column.
    pub fn signal_column(&self) -> &IndexListArray {
        &self.signal
    }

    // --- Core Column ---
    /// Returns the stored down-casted sequencing channel identifier column.
    pub fn channel_column(&self) -> &UInt16Array {
        &self.channel
    }

    // --- Core Column ---
    /// Returns the stored down-casted pore well identifier column.
    pub fn well_column(&self) -> &UInt8Array {
        &self.well
    }

    /// Returns the stored down-casted pore type classification column.
    pub fn pore_type_column(&self) -> &StringDictionary {
        &self.pore_type
    }

    // --- Core Column ---
    /// Returns the stored down-casted calibration offset (applied to signal values) column.
    pub fn calibration_offset_column(&self) -> &Float32Array {
        &self.calibration_offset
    }

    // --- Core Column ---
    /// Returns the stored down-casted calibration scale (applied to signal values) column.
    pub fn calibration_scale_column(&self) -> &Float32Array {
        &self.calibration_scale
    }

    // --- Core Column ---
    /// Returns the stored down-casted sequential read number column.
    pub fn read_number_column(&self) -> &UInt32Array {
        &self.read_number
    }

    // --- Core Column ---
    /// Returns the stored down-casted start position (in samples) column.
    pub fn start_column(&self) -> &UInt64Array {
        &self.start
    }

    /// Returns the stored down-casted median signal value (before each read begins) column.
    pub fn median_before_column(&self) -> &Float32Array {
        &self.median_before
    }

    /// Returns the stored down-casted tracked scaling factor (applied during basecalling) column.
    pub fn tracked_scaling_scale_column(&self) -> &Float32Array {
        &self.tracked_scaling_scale
    }

    /// Returns the stored down-casted tracked scaling shift (applied during basecalling) column.
    pub fn tracked_scaling_shift_column(&self) -> &Float32Array {
        &self.tracked_scaling_shift
    }

    /// Returns the stored down-casted predicted scaling factor column.
    pub fn predicted_scaling_scale_column(&self) -> &Float32Array {
        &self.predicted_scaling_scale
    }

    /// Returns the stored down-casted predicted scaling shift column.
    pub fn predicted_scaling_shift_column(&self) -> &Float32Array {
        &self.predicted_scaling_shift
    }

    /// Returns the stored down-casted column of the number of reads since the last mux change.
    pub fn num_reads_since_mux_change_column(&self) -> &UInt32Array {
        &self.num_reads_since_mux_change
    }

    /// Returns the stored down-casted column of the time elapsed since the last mux change.
    pub fn time_since_mux_change_column(&self) -> &Float32Array {
        &self.time_since_mux_change
    }

    /// Returns the stored down-casted column of the number of MinKNOW events associated with each read.
    pub fn num_minknow_events_column(&self) -> &UInt64Array {
        &self.num_minknow_events
    }

    /// Returns the stored down-casted column of the end reason classification for each read.
    pub fn end_reason_column(&self) -> &StringDictionary {
        &self.end_reason
    }

    /// Returns the stored down-casted column of whether the end reason was forced.
    pub fn end_reason_forced_column(&self) -> &BooleanArray {
        &self.end_reason_forced
    }

    // --- Core Column ---
    /// Returns the stored down-casted run information identifier column.
    pub fn run_info_column(&self) -> &StringDictionary {
        &self.run_info
    }

    // --- Core Column (Recoverable) ---
    /// Returns the stored down-casted column of the number of samples in each read.
    pub fn num_samples_column(&self) -> &UInt64Array {
        &self.num_samples
    }

    /// Returns the stored down-casted open pore level measurement column.
    pub fn open_pore_level_column(&self) -> &Float32Array {
        &self.open_pore_level
    }
}