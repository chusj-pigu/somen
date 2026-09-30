// standard
use std::{
    error::Error,
    fmt::{
        self,
        Display,
        Formatter,
    },
};
// third party

// local
use crate::{
    io::reader::{
        BatchRef,
        ReferenceModel,
        Pod5ContextHandle,
    },
    file::{
        BatchRowIndex,
        IndexOutOfBounds,
        schema::{
            ColumnSchema,
            RunInfoSchema,
        },
    },
    record::{
        batch::internal::{
            RunInfoColumns,
            arrays::{
                Indexable,
            },
        },
        Record,
        types::FlatMap,
    },
};
use crate::io::reader::ConcurrencyMode;
use crate::record::batch::internal::ReadColumns;
use crate::record::ReadRecord;
use crate::record::record::read::ReadRecordContract;

/// Error raised when a mandatory column in a `RunInfoBatch` contains an invalid null value.
///
/// Pod5 data layout requires these fields to be fully populated. A null value implies
/// either file corruption or a non-compliant file writer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RunInfoColumnError {
    // --- Structural Layout Columns ---

    /// The acquisition identifier column contains an invalid null value.
    AcquisitionId,

    // --- Core Data Columns ---

    /// The acquisition start time column contains an invalid null value.
    AcquisitionStartTime,

    /// The maximum ADC column contains an invalid null value.
    AdcMax,

    /// The minimum ADC column contains an invalid null value.
    AdcMin,

    /// The flow cell identifier column contains an invalid null value.
    FlowCellId,

    /// The flow cell product code column contains an invalid null value.
    FlowCellProductCode,

    /// The protocol run identifier column contains an invalid null value.
    ProtocolRunId,

    /// The protocol start time column contains an invalid null value.
    ProtocolStartTime,

    /// The acquisition sample rate column contains an invalid null value.
    SampleRate,

    /// The sequencing kit column contains an invalid null value.
    SequencingKit,

    /// The sequencer position column contains an invalid null value.
    SequencerPosition,

    /// The system type column contains an invalid null value.
    SystemType
}

impl From<RunInfoColumnError> for RunInfoSchema {
    fn from(error: RunInfoColumnError) -> Self {
        match error {
            RunInfoColumnError::AcquisitionId => Self::AcquisitionId,
            RunInfoColumnError::AcquisitionStartTime => Self::AcquisitionStartTime,
            RunInfoColumnError::AdcMax => Self::AdcMax,
            RunInfoColumnError::AdcMin => Self::AdcMin,
            RunInfoColumnError::FlowCellId => Self::FlowCellId,
            RunInfoColumnError::FlowCellProductCode => Self::FlowCellProductCode,
            RunInfoColumnError::ProtocolRunId => Self::ProtocolRunId,
            RunInfoColumnError::ProtocolStartTime => Self::ProtocolStartTime,
            RunInfoColumnError::SampleRate => Self::SampleRate,
            RunInfoColumnError::SequencingKit => Self::SequencingKit,
            RunInfoColumnError::SequencerPosition => Self::SequencerPosition,
            RunInfoColumnError::SystemType => Self::SystemType,
        }
    }
}

impl Display for RunInfoColumnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let variant = match self {
            Self::AcquisitionId => "Structural",
            _ => "Core data",
        };
        write!(f, "{} column {} cannot contain null values", variant, RunInfoSchema::from(*self).get_name())
    }
}

impl Error for RunInfoColumnError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Error indicating that a [`RunInfoRecord`]'s field is not accessible.
pub enum RunInfoError {
    /// A mandatory field's source column contains an invalid `Null` value.
    FieldMissing(RunInfoColumnError),

    /// The requested row index falls outside the valid bounds of the batch.
    RowIndexOutOfBounds(IndexOutOfBounds),
}

impl Display for RunInfoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldMissing(_) => write!(f, "Field missing"),
            Self::RowIndexOutOfBounds(_) => write!(f, "Row index out of bounds"),
        }
    }
}

impl Error for RunInfoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FieldMissing(err) => Some(err),
            Self::RowIndexOutOfBounds(err) => Some(err),
        }
    }
}

/// Defines the operations of a `RunInfoRecord`.
pub trait RunInfoRecordContract<R: ReferenceModel>: Record<R> {
    /// The [`ReadRecordContract`] implementation returned by [`reads()`](Self::reads).
    type ReadRecord: ReadRecordContract<<R::ConcurrencyMode as ConcurrencyMode>::InitialReferenceModel<ReadColumns>>;

    /// The type of [`Error`] which can be returned.
    type RunInfoError: Error;

    /// Returns the [`ReadRecords`](Self::ReadRecord) of the run.
    fn reads() -> Self::ReadRecord;

    /// Returns the `SignalBuffers` of the [`ReadRecords`](Self::ReadRecord) of the run.
    fn signal_buffers() -> <Self::ReadRecord as ReadRecordContract<<R::ConcurrencyMode as ConcurrencyMode>::InitialReferenceModel<ReadColumns>>>::SignalBuffer;
    
    // --- Core Column ---
    /// Returns the stored down-casted acquisition identifier.
    fn acquisition_id(&self) -> Result<&str, Self::RunInfoError>;

    // --- Core Column ---
    /// Returns the stored down-casted acquisition start time.
    fn acquisition_start_time(&self) -> Result<i64, Self::RunInfoError>;

    // --- Core Column (Recoverable) ---
    /// Returns the stored down-casted maximum ADC value.
    fn adc_max(&self) -> Result<i16, Self::RunInfoError>;

    // --- Core Column (Recoverable) ---
    /// Returns the stored down-casted minimum ADC value.
    fn adc_min(&self) -> Result<i16, Self::RunInfoError>;
    
    /// Returns the stored down-casted run context tags.
    fn context_tags(&self) -> Result<Option<FlatMap>, Self::RunInfoError>;

    /// Returns the stored down-casted experiment name.
    fn experiment_name(&self) -> Result<Option<&str>, Self::RunInfoError>;

    // --- Core Column ---
    /// Returns the stored down-casted flow cell identifier.
    fn flow_cell_id(&self) -> Result<&str, Self::RunInfoError>;
    
    // --- Core Column ---
    /// Returns the stored down-casted flow cell product code.
    fn flow_cell_product_code(&self) -> Result<&str, Self::RunInfoError>;

    /// Returns the stored down-casted protocol name.
    fn protocol_name(&self) -> Result<Option<&str>, Self::RunInfoError>;

    // --- Core Column ---
    /// Returns the stored down-casted protocol run identifier.
    fn protocol_run_id(&self) -> Result<&str, Self::RunInfoError>;

    // --- Core Column ---
    /// Returns the stored down-casted protocol start time.
    fn protocol_start_time(&self) -> Result<i64, Self::RunInfoError>;

    /// Returns the stored down-casted sample identifier.
    fn sample_id(&self) -> Result<Option<&str>, Self::RunInfoError>;

    // --- Core Column ---
    /// Returns the stored down-casted acquisition sample rate.
    fn sample_rate(&self) -> Result<u16, Self::RunInfoError>;

    // --- Core Column ---
    /// Returns the stored down-casted sequencing kit.
    fn sequencing_kit(&self) -> Result<&str, Self::RunInfoError>;

    // --- Core Column ---
    /// Returns the stored down-casted sequencer position.
    fn sequencer_position(&self) -> Result<&str, Self::RunInfoError>;

    /// Returns the stored down-casted sequencer position type.
    fn sequencer_position_type(&self) -> Result<Option<&str>, Self::RunInfoError>;

    /// Returns the stored down-casted acquisition software description.
    fn software(&self) -> Result<Option<&str>, Self::RunInfoError>;

    /// Returns the stored down-casted system name.
    fn system_name(&self) -> Result<Option<&str>, Self::RunInfoError>;

    // --- Core Column ---
    /// Returns the stored down-casted system type.
    fn system_type(&self) -> Result<&str, Self::RunInfoError>;

    /// Returns the stored down-casted run tracking information.
    fn tracking_id(&self) -> Result<Option<FlatMap>, Self::RunInfoError>;
}

/// A zero-copy view over a run info record within a POD5 file.
///
/// `RunInfoRecord` borrows the underlying data and provides typed access to
/// the metadata associated with a single sequencing run.
pub struct RunInfoRecord<R: ReferenceModel> {
    /// The batch backing this record.
    source: R::BatchRef<RunInfoColumns>,

    /// The row within the batch.
    row: BatchRowIndex,

    /// Handle used during iteration calls to share access to the registry.
    pod5_context_handle: R::Pod5ContextHandle,
}

impl<R: ReferenceModel> RunInfoRecord<R> {
    /// todo
    pub(super) fn new(
        source: R::BatchRef<RunInfoColumns>,
        row: BatchRowIndex,
        pod5_context_handle: R::Pod5ContextHandle,
    ) -> Self {
        Self {
            source,
            row,
            pod5_context_handle,
        }
    }

    fn unpack_strict<'a, T, I: Indexable<Value<'a>=T>>(&'a self, indexable: &'a I, error: RunInfoColumnError) -> Result<T, RunInfoError> {
        self.unpack(indexable)?
            .ok_or(RunInfoError::FieldMissing(error))
    }

    fn unpack<'a, T, I: Indexable<Value<'a>=T>>(&'a self, indexable: &'a I) -> Result<Option<T>, RunInfoError> {
        indexable.index(self.row)
            .map_err(RunInfoError::RowIndexOutOfBounds)
    }
}

impl<R: ReferenceModel> Record<R> for RunInfoRecord<R> {
    type BatchColumns = RunInfoColumns;
    type StoredRecord = RunInfoRecord<R::StoredReferenceModel<RunInfoColumns>>;

    fn to_stored(self) -> Self::StoredRecord {
        Self::StoredRecord {
            source: self.source.into_stored(),
            row: self.row,
            pod5_context_handle: self.pod5_context_handle.into_stored(),
        }
    }
}

impl<R: ReferenceModel> RunInfoRecordContract<R> for RunInfoRecord<R> {
    type ReadRecord = ReadRecord<<R::ConcurrencyMode as ConcurrencyMode>::InitialReferenceModel<ReadColumns>>;
    
    type RunInfoError = RunInfoError;

    fn reads() -> Self::ReadRecord {
        todo!()
    }

    fn signal_buffers() -> <Self::ReadRecord as ReadRecordContract<<R::ConcurrencyMode as ConcurrencyMode>::InitialReferenceModel<ReadColumns>>>::SignalBuffer {
        todo!()
    }

    fn acquisition_id(&self) -> Result<&str, RunInfoError> {
        self.unpack_strict(
            self.source.acquisition_id_column(),
            RunInfoColumnError::AcquisitionId,
        )
    }

    fn acquisition_start_time(&self) -> Result<i64, RunInfoError> {
        self.unpack_strict(
            self.source.acquisition_start_time_column(),
            RunInfoColumnError::AcquisitionStartTime,
        )
    }

    fn adc_max(&self) -> Result<i16, RunInfoError> {
        self.unpack_strict(
            self.source.adc_max_column(),
            RunInfoColumnError::AdcMax,
        )
    }

    fn adc_min(&self) -> Result<i16, RunInfoError> {
        self.unpack_strict(
            self.source.adc_min_column(),
            RunInfoColumnError::AdcMin,
        )
    }

    fn context_tags(&self) -> Result<Option<FlatMap>, RunInfoError> {
        self.unpack(self.source.context_tags_column())
    }

    fn experiment_name(&self) -> Result<Option<&str>, RunInfoError> {
        self.unpack(self.source.experiment_name_column())
    }

    fn flow_cell_id(&self) -> Result<&str, RunInfoError> {
        self.unpack_strict(
            self.source.flow_cell_id_column(),
            RunInfoColumnError::FlowCellId,
        )
    }

    fn flow_cell_product_code(&self) -> Result<&str, RunInfoError> {
        self.unpack_strict(
            self.source.flow_cell_product_code_column(),
            RunInfoColumnError::FlowCellProductCode,
        )
    }

    fn protocol_name(&self) -> Result<Option<&str>, RunInfoError> {
        self.unpack(self.source.protocol_name_column())
    }

    fn protocol_run_id(&self) -> Result<&str, RunInfoError> {
        self.unpack_strict(
            self.source.protocol_run_id_column(),
            RunInfoColumnError::ProtocolRunId,
        )
    }

    fn protocol_start_time(&self) -> Result<i64, RunInfoError> {
        self.unpack_strict(
            self.source.protocol_start_time_column(),
            RunInfoColumnError::ProtocolStartTime,
        )
    }

    fn sample_id(&self) -> Result<Option<&str>, RunInfoError> {
        self.unpack(self.source.sample_id_column())
    }

    fn sample_rate(&self) -> Result<u16, RunInfoError> {
        self.unpack_strict(
            self.source.sample_rate_column(),
            RunInfoColumnError::SampleRate,
        )
    }

    fn sequencing_kit(&self) -> Result<&str, RunInfoError> {
        self.unpack_strict(
            self.source.sequencing_kit_column(),
            RunInfoColumnError::SequencingKit,
        )
    }

    fn sequencer_position(&self) -> Result<&str, RunInfoError> {
        self.unpack_strict(
            self.source.sequencer_position_column(),
            RunInfoColumnError::SequencerPosition,
        )
    }

    fn sequencer_position_type(&self) -> Result<Option<&str>, RunInfoError> {
        self.unpack(self.source.sequencer_position_type_column())
    }

    fn software(&self) -> Result<Option<&str>, RunInfoError> {
        self.unpack(self.source.software_column())
    }

    fn system_name(&self) -> Result<Option<&str>, RunInfoError> {
        self.unpack(self.source.system_name_column())
    }

    fn system_type(&self) -> Result<&str, RunInfoError> {
        self.unpack_strict(
            self.source.system_type_column(),
            RunInfoColumnError::SystemType,
        )
    }

    fn tracking_id(&self) -> Result<Option<FlatMap>, RunInfoError> {
        self.unpack(self.source.tracking_id_column())
    }
}