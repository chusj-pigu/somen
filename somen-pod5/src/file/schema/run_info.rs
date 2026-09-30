// standard

// third party

// local
use crate::{
    file::schema::ColumnSchema,
    record::batch::internal::arrays::FieldType
};

/// todo
#[derive(Debug, Copy, Clone)]
pub enum RunInfoSchema {
    /// A unique identifier for the run (acquisition).
    /// This is the same identifier that MinKNOW uses
    /// to identify an acquisition within a protocol.
    AcquisitionId,

    /// This is the clock time for sample 0, and can be used
    /// together with sample_rate and the :start read field to
    /// calculate a clock time for when a given read was acquired.
    /// The timezone should be set.
    /// MinKNOW will set this to the local timezone on file creation.
    /// When merging files that have different timezones, merging code
    /// will have to pick a timezone (possibly defaulting to 'UTC').
    AcquisitionStartTime,

    /// The maximum ADC value that might be encountered.
    /// This is a hardware constraint.
    AdcMax,

    /// The minimum ADC value that might be encountered.
    /// This is a hardware constraint.
    /// adc_max - adc_min + 1 is the digitisation.
    AdcMin,

    /// The context tags for the run. For compatibility with fast5.
    /// Readers must not make any assumptions about the contents of this field.
    ContextTags,

    /// A user-supplied name for the experiment being run.
    ExperimentName,

    /// Uniquely identifies the flow cell the data was captured on.
    /// This is written on the flow cell case.
    FlowCellId,

    /// Identifies the type of flow cell the data was captured on.
    FlowCellProductCode,

    /// The name of the protocol that was run.
    ProtocolName,

    /// A unique identifier for the protocol run that produced this data.
    ProtocolRunId,

    /// When the protocol that the acquisition was part of started.
    /// The same considerations apply as for acquisition_start_time.
    ProtocolStartTime,

    /// A user-supplied name for the sample being analysed.
    SampleId,

    /// The number of samples acquired each second on each channel.
    /// This can be used to convert numbers of samples into time durations.
    SampleRate,

    /// The type of sequencing kit used to prepare the sample.
    SequencingKit,

    /// The sequencer position the data was collected on.
    /// For removable positions, like MinION Mk1Bs, this is unique
    /// (e.g. `MN12345`), while for integrated positions it is not
    /// (e.g. `X1` on a GridION).
    SequencerPosition,

    /// The type of sequencing hardware the data was collected on.
    /// For example: `MinION Mk1B` or `GridION` or `PromethION`.
    SequencerPositionType,

    /// A description of the software that acquired the data.
    /// For example: `MinKNOW 21.05.12 (Bream 5.1.6, Configurations 16.2.1, Core 5.1.9, Guppy 4.2.3)`.
    Software,

    /// The name of the system the data was collected on.
    /// This might be a sequencer serial (eg: `GXB1234`) or a host name (e.g. `Lab PC`).
    SystemName,

    /// The type of system the data was collected on.
    /// For example, `GridION Mk1` or `PromethION P48`.
    /// If the system is not a Nanopore sequencer with built-in compute,
    /// this will be a description of the operating system (e.g. `Ubuntu 20.04`).
    SystemType,

    /// The tracking id for the run.
    /// For compatibility with fast5.
    /// Readers must not make any assumptions about the contents of this field.
    TrackingId,
}

impl ColumnSchema for RunInfoSchema {
    const COLUMNS: &'static [Self] = &[
        RunInfoSchema::AcquisitionId,
        RunInfoSchema::AcquisitionStartTime,
        RunInfoSchema::AdcMax,
        RunInfoSchema::AdcMin,
        RunInfoSchema::ContextTags,
        RunInfoSchema::ExperimentName,
        RunInfoSchema::FlowCellId,
        RunInfoSchema::FlowCellProductCode,
        RunInfoSchema::ProtocolName,
        RunInfoSchema::ProtocolRunId,
        RunInfoSchema::ProtocolStartTime,
        RunInfoSchema::SampleId,
        RunInfoSchema::SampleRate,
        RunInfoSchema::SequencingKit,
        RunInfoSchema::SequencerPosition,
        RunInfoSchema::SequencerPositionType,
        RunInfoSchema::Software,
        RunInfoSchema::SystemName,
        RunInfoSchema::SystemType,
        RunInfoSchema::TrackingId,
    ];

    fn get_type(&self) -> FieldType {
        match self {
            RunInfoSchema::AcquisitionId => FieldType::Utf8,
            RunInfoSchema::AcquisitionStartTime => FieldType::Timestamp,
            RunInfoSchema::AdcMax => FieldType::Int16,
            RunInfoSchema::AdcMin => FieldType::Int16,
            RunInfoSchema::ContextTags => FieldType::Map,
            RunInfoSchema::ExperimentName => FieldType::Utf8,
            RunInfoSchema::FlowCellId => FieldType::Utf8,
            RunInfoSchema::FlowCellProductCode => FieldType::Utf8,
            RunInfoSchema::ProtocolName => FieldType::Utf8,
            RunInfoSchema::ProtocolRunId => FieldType::Utf8,
            RunInfoSchema::ProtocolStartTime => FieldType::Timestamp,
            RunInfoSchema::SampleId => FieldType::Utf8,
            RunInfoSchema::SampleRate => FieldType::UInt16,
            RunInfoSchema::SequencingKit => FieldType::Utf8,
            RunInfoSchema::SequencerPosition => FieldType::Utf8,
            RunInfoSchema::SequencerPositionType => FieldType::Utf8,
            RunInfoSchema::Software => FieldType::Utf8,
            RunInfoSchema::SystemName => FieldType::Utf8,
            RunInfoSchema::SystemType => FieldType::Utf8,
            RunInfoSchema::TrackingId => FieldType::Map,
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            RunInfoSchema::AcquisitionId => "acquisition_id",
            RunInfoSchema::AcquisitionStartTime => "acquisition_start_time",
            RunInfoSchema::AdcMax => "adc_max",
            RunInfoSchema::AdcMin => "adc_min",
            RunInfoSchema::ContextTags => "context_tags",
            RunInfoSchema::ExperimentName => "experiment_name",
            RunInfoSchema::FlowCellId => "flow_cell_id",
            RunInfoSchema::FlowCellProductCode => "flow_cell_product_code",
            RunInfoSchema::ProtocolName => "protocol_name",
            RunInfoSchema::ProtocolRunId => "protocol_run_id",
            RunInfoSchema::ProtocolStartTime => "protocol_start_time",
            RunInfoSchema::SampleId => "sample_id",
            RunInfoSchema::SampleRate => "sample_rate",
            RunInfoSchema::SequencingKit => "sequencing_kit",
            RunInfoSchema::SequencerPosition => "sequencer_position",
            RunInfoSchema::SequencerPositionType => "sequencer_position_type",
            RunInfoSchema::Software => "software",
            RunInfoSchema::SystemName => "system_name",
            RunInfoSchema::SystemType => "system_type",
            RunInfoSchema::TrackingId => "tracking_id",
        }
    }
}