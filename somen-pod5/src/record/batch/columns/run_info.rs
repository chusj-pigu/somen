// standard

// third party
use arrow::array::RecordBatch;
// local
use crate::{
    file::schema::{
        ColumnSchema,
        RunInfoSchema,
        RunInfoSchema::*,
    },
    record::batch::{
        BatchError,
        columns::arrays::*,
        internal::BatchColumns,
    }
};

/// The set of [`columns`](BatchColumns) for 
/// [`RunInfoBatchCore`](crate::record::batch::internal::RunInfoBatchCore).
pub struct RunInfoColumns {
    acquisition_id: StringArray,
    acquisition_start_time: EpochMillisArray,
    adc_max: Int16Array,
    adc_min: Int16Array,
    context_tags: SliceMapArray,
    experiment_name: StringDictionary,
    flow_cell_id: StringDictionary,
    flow_cell_product_code: StringDictionary,
    protocol_name: StringDictionary,
    protocol_run_id: StringDictionary,
    protocol_start_time: EpochMillisArray,
    sample_id: StringDictionary,
    sample_rate: UInt16Array,
    sequencing_kit: StringDictionary,
    sequencer_position: StringDictionary,
    sequencer_position_type: StringDictionary,
    software: StringDictionary,
    system_name: StringDictionary,
    system_type: StringDictionary,
    tracking_id: SliceMapArray,
}

impl BatchColumns for RunInfoColumns {
    fn new(source: &RecordBatch) -> Result<Self, BatchError> {
        Ok(Self {
            acquisition_id: RunInfoSchema::try_from_array_ref(source, AcquisitionId)?,
            acquisition_start_time: RunInfoSchema::try_from_array_ref(source, AcquisitionStartTime)?,
            adc_max: RunInfoSchema::try_from_array_ref(source, AdcMax)?,
            adc_min: RunInfoSchema::try_from_array_ref(source, AdcMin)?,
            context_tags: RunInfoSchema::try_from_array_ref(source, ContextTags)?,
            experiment_name: RunInfoSchema::try_from_array_ref(source, ExperimentName)?,
            flow_cell_id: RunInfoSchema::try_from_array_ref(source, FlowCellId)?,
            flow_cell_product_code: RunInfoSchema::try_from_array_ref(source, FlowCellProductCode)?,
            protocol_name: RunInfoSchema::try_from_array_ref(source, ProtocolName)?,
            protocol_run_id: RunInfoSchema::try_from_array_ref(source, ProtocolRunId)?,
            protocol_start_time: RunInfoSchema::try_from_array_ref(source, ProtocolStartTime)?,
            sample_id: RunInfoSchema::try_from_array_ref(source, SampleId)?,
            sample_rate: RunInfoSchema::try_from_array_ref(source, SampleRate)?,
            sequencing_kit: RunInfoSchema::try_from_array_ref(source, SequencingKit)?,
            sequencer_position: RunInfoSchema::try_from_array_ref(source, SequencerPosition)?,
            sequencer_position_type: RunInfoSchema::try_from_array_ref(source, SequencerPositionType)?,
            software: RunInfoSchema::try_from_array_ref(source, Software)?,
            system_name: RunInfoSchema::try_from_array_ref(source, SystemName)?,
            system_type: RunInfoSchema::try_from_array_ref(source, SystemType)?,
            tracking_id: RunInfoSchema::try_from_array_ref(source, TrackingId)?,
        })
    }
}

impl RunInfoColumns {
    // --- Core Column ---
    /// Returns the stored down-casted acquisition identifier column.
    pub fn acquisition_id_column(&self) -> &StringArray {
        &self.acquisition_id
    }

    // --- Core Column ---
    /// Returns the stored down-casted acquisition start time column.
    pub fn acquisition_start_time_column(&self) -> &EpochMillisArray {
        &self.acquisition_start_time
    }

    // --- Core Column (Recoverable) ---
    /// Returns the stored down-casted maximum ADC value column.
    pub fn adc_max_column(&self) -> &Int16Array {
        &self.adc_max
    }

    // --- Core Column (Recoverable) ---
    /// Returns the stored down-casted minimum ADC value column.
    pub fn adc_min_column(&self) -> &Int16Array {
        &self.adc_min
    }

    /// Returns the stored down-casted run context tags column.
    pub fn context_tags_column(&self) -> &SliceMapArray {
        &self.context_tags
    }

    /// Returns the stored down-casted experiment name column.
    pub fn experiment_name_column(&self) -> &StringDictionary {
        &self.experiment_name
    }

    // --- Core Column ---
    /// Returns the stored down-casted flow cell identifier column.
    pub fn flow_cell_id_column(&self) -> &StringDictionary {
        &self.flow_cell_id
    }

    // --- Core Column ---
    /// Returns the stored down-casted flow cell product code column.
    pub fn flow_cell_product_code_column(&self) -> &StringDictionary {
        &self.flow_cell_product_code
    }

    /// Returns the stored down-casted protocol name column.
    pub fn protocol_name_column(&self) -> &StringDictionary {
        &self.protocol_name
    }

    // --- Core Column ---
    /// Returns the stored down-casted protocol run identifier column.
    pub fn protocol_run_id_column(&self) -> &StringDictionary {
        &self.protocol_run_id
    }

    // --- Core Column ---
    /// Returns the stored down-casted protocol start time column.
    pub fn protocol_start_time_column(&self) -> &EpochMillisArray {
        &self.protocol_start_time
    }

    /// Returns the stored down-casted sample identifier column.
    pub fn sample_id_column(&self) -> &StringDictionary {
        &self.sample_id
    }

    // --- Core Column ---
    /// Returns the stored down-casted acquisition sample rate column.
    pub fn sample_rate_column(&self) -> &UInt16Array {
        &self.sample_rate
    }

    // --- Core Column ---
    /// Returns the stored down-casted sequencing kit column.
    pub fn sequencing_kit_column(&self) -> &StringDictionary {
        &self.sequencing_kit
    }

    // --- Core Column ---
    /// Returns the stored down-casted sequencer position column.
    pub fn sequencer_position_column(&self) -> &StringDictionary {
        &self.sequencer_position
    }

    /// Returns the stored down-casted sequencer position type column.
    pub fn sequencer_position_type_column(&self) -> &StringDictionary {
        &self.sequencer_position_type
    }

    /// Returns the stored down-casted acquisition software description column.
    pub fn software_column(&self) -> &StringDictionary {
        &self.software
    }

    /// Returns the stored down-casted system name column.
    pub fn system_name_column(&self) -> &StringDictionary {
        &self.system_name
    }

    // --- Core Column ---
    /// Returns the stored down-casted system type column.
    pub fn system_type_column(&self) -> &StringDictionary {
        &self.system_type
    }

    /// Returns the stored down-casted run tracking information column.
    pub fn tracking_id_column(&self) -> &SliceMapArray {
        &self.tracking_id
    }
}