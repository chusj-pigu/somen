// standard

// third party

// local
use crate::{
    file::schema::ColumnSchema,
    record::batch::internal::arrays::FieldType
};

/// todo
#[derive(Debug, Copy, Clone)]
pub enum ReadSchema {
    /// Globally-unique identifier for the read, can be converted
    /// to a string form (using standard routines in other libraries)
    /// which matches how reads are identified elsewhere.
    ReadId,

    /// A list of zero-indexed row numbers in the Signal table.
    /// This must be all the rows in the Signal table that have a
    /// matching read_id, in order.
    /// It functions as an index for the Signal table.
    Signal,

    /// 1-indexed channel
    Channel,

    /// 1-indexed well (typically `1`, `2`, `3` or `4`)
    Well,

    /// Name of the pore type present in the well
    PoreType,

    /// Calibration offset used to scale raw ADC data into pA readings.
    CalibrationOffset,

    /// Calibration scale factor used to scale raw `ADC` data into `pA` readings.
    CalibrationScale,

    /// The read number on channel.
    /// This is increasing but typically not necessarily consecutive.
    ReadNumber,

    /// How many samples were taken on this channel before the read started
    /// (since the data acquisition period began).
    /// This can be combined with the sample rate to get a time in seconds
    /// for the start of the read relative to the start of data acquisition.
    Start,

    /// The level of current in the well before this read
    /// (typically the open pore level of the well).
    /// If the level is not known (eg: due to a mux change),
    /// this should be nulled out.
    MedianBefore,

    /// Scale for tracked read scaling values (based on previous reads shift)
    TrackedScalingScale,

    /// Shift for tracked read scaling values (based on previous reads shift)
    TrackedScalingShift,

    /// Scale for predicted read scaling values (based on this read's raw signal)
    PredictedScalingScale,

    /// Shift for predicted read scaling values (based on this read's raw signal)
    PredictedScalingShift,

    /// Number of selected reads since the last mux change on this reads channel
    NumReadsSinceMuxChange,

    /// Time in seconds since the last mux change on this reads channel
    TimeSinceMuxChange,

    /// Number of minknow events that the read contains
    NumMinknowEvents,

    /// The end reason, currently one of:
    /// `unknown`, `mux_change`, `unblock_mux_change`, `data_service_unblock_mux_change`,
    /// `signal_positive`, `signal_negative`, `api_request`, `device_data_error`,
    /// `analysis_config_change` or `paused`.
    EndReason,

    /// `True` if this read was ended 'forcibly'
    /// (eg: `mux_change`, `unblock`),
    /// `false` if it was a data-driven read break
    /// (`signal_positive`, `signal_negative`).
    ///
    /// This allows simple categorisation even in the presence
    /// of new reasons that reading code is unaware of.
    EndReasonForced,

    /// The run (acquisition) this read came from.
    /// Must match the acquisition_id field of exactly one entry in the run_info table.
    RunInfo,

    /// The full length of the signal for this read in samples
    /// (equal to the sum of all 'samples' fields of signal chunks)
    NumSamples,

    /// The open pore level for the read.
    /// A value in `pA` showing the open pore level of the well prior to the read starting.
    /// If the information is not available
    /// (feature not enabled in MinKNOW, or sequencing run on an old version)
    /// this value will be `NaN`.
    OpenPoreLevel,
}

impl ColumnSchema for ReadSchema {
    const COLUMNS: &'static [Self] = &[
        ReadSchema::ReadId,
        ReadSchema::Signal,
        ReadSchema::Channel,
        ReadSchema::Well,
        ReadSchema::PoreType,
        ReadSchema::CalibrationOffset,
        ReadSchema::CalibrationScale,
        ReadSchema::ReadNumber,
        ReadSchema::Start,
        ReadSchema::MedianBefore,
        ReadSchema::TrackedScalingScale,
        ReadSchema::TrackedScalingShift,
        ReadSchema::PredictedScalingScale,
        ReadSchema::PredictedScalingShift,
        ReadSchema::NumReadsSinceMuxChange,
        ReadSchema::TimeSinceMuxChange,
        ReadSchema::NumMinknowEvents,
        ReadSchema::EndReason,
        ReadSchema::EndReasonForced,
        ReadSchema::RunInfo,
        ReadSchema::NumSamples,
        ReadSchema::OpenPoreLevel,
    ];

    fn get_type(&self) -> FieldType {
        match self {
            ReadSchema::ReadId => FieldType::Uuid,
            ReadSchema::Signal => FieldType::List,
            ReadSchema::Channel => FieldType::UInt16,
            ReadSchema::Well => FieldType::Dictionary,
            ReadSchema::PoreType => FieldType::Float,
            ReadSchema::CalibrationOffset => FieldType::Float,
            ReadSchema::CalibrationScale => FieldType::UInt32,
            ReadSchema::ReadNumber => FieldType::UInt64,
            ReadSchema::Start => FieldType::Float,
            ReadSchema::MedianBefore => FieldType::Float,
            ReadSchema::TrackedScalingScale => FieldType::Float,
            ReadSchema::TrackedScalingShift => FieldType::Float,
            ReadSchema::PredictedScalingScale => FieldType::Float,
            ReadSchema::PredictedScalingShift => FieldType::Float,
            ReadSchema::NumReadsSinceMuxChange => FieldType::UInt32,
            ReadSchema::TimeSinceMuxChange => FieldType::Float,
            ReadSchema::NumMinknowEvents => FieldType::UInt64,
            ReadSchema::EndReason => FieldType::Dictionary,
            ReadSchema::EndReasonForced => FieldType::Bool,
            ReadSchema::RunInfo => FieldType::Dictionary,
            ReadSchema::NumSamples => FieldType::UInt64,
            ReadSchema::OpenPoreLevel => FieldType::Float,
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ReadSchema::ReadId => "read_id",
            ReadSchema::Signal => "signal",
            ReadSchema::Channel => "channel",
            ReadSchema::Well => "well",
            ReadSchema::PoreType => "pore_type",
            ReadSchema::CalibrationOffset => "calibration_offset",
            ReadSchema::CalibrationScale => "calibration_scale",
            ReadSchema::ReadNumber => "read_number",
            ReadSchema::Start => "start",
            ReadSchema::MedianBefore => "median_before",
            ReadSchema::TrackedScalingScale => "tracked_scaling_scale",
            ReadSchema::TrackedScalingShift => "tracked_scaling_shift",
            ReadSchema::PredictedScalingScale => "predicted_scaling_scale",
            ReadSchema::PredictedScalingShift => "predicted_scaling_shift",
            ReadSchema::NumReadsSinceMuxChange => "num_reads_since_mux_change",
            ReadSchema::TimeSinceMuxChange => "time_since_mux_change",
            ReadSchema::NumMinknowEvents => "num_minknow_events",
            ReadSchema::EndReason => "end_reason",
            ReadSchema::EndReasonForced => "end_reason_forced",
            ReadSchema::RunInfo => "run_info",
            ReadSchema::NumSamples => "num_samples",
            ReadSchema::OpenPoreLevel => "open_pore_level",
        }
    }
}