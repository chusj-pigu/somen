// standard
use std::{
    error::Error,
    fmt::{
        self,
        Display,
        Formatter,
    },
};
use std::fmt::Debug;
use std::ops::Deref;
// third party

// local
use crate::{
    io::reader::{
        BatchRef,
        ReferenceModel,
        Pod5ContextHandle,
        ArcBatchRef,
        GuardBatchRef,
        RcBatchRef,
    },
    file::{
        BatchRowIndex,
        IndexOutOfBounds,
        schema::{
            ColumnSchema,
            ReadSchema,
        },
    },
    record::{
        batch::internal::{
            ReadColumns,
            arrays::{
                Indexable,
                InvalidUUIDLength,
            },
        },
        Record,
        types::Uuid,
        SignalBuffer,
        internal::contracts::SignalBufferContract
    },
};
use crate::io::decoder::{DecoderError, SignalDecoder};
use crate::io::reader::Pod5Context;
use crate::record::batch::internal::SignalColumns;
use crate::io::reader::ConcurrencyMode;

/// Error indicating that a mandatory column in a `ReadBatch` contains an invalid null value.
///
/// Pod5 data layout requires these fields to be fully populated. A null value implies
/// either file corruption or a non-compliant file writer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReadColumnError {
    // --- Structural Layout Columns ---

    /// The read identifier column contains an invalid null value.
    ReadId,

    /// The row indexes column contains an invalid null value.
    Signal,

    /// The run_info identifier column contains an invalid null value.
    RunInfo,

    // --- Core Data Columns ---

    /// The channel identifier column contains an invalid null value.
    Channel,

    /// The pore well identifier column contains an invalid null value.
    Well,

    /// The calibration offset column contains an invalid null value.
    CalibrationOffset,

    /// The calibration scale column contains an invalid null value.
    CalibrationScale,

    /// todo
    ReadNumber,

    /// The samples before read start column contains an invalid null value.
    Start,

    /// The number of samples column contains an invalid null value.
    NumSamples,
}

impl From<ReadColumnError> for ReadSchema {
    fn from(error: ReadColumnError) -> Self {
        match error {
            ReadColumnError::ReadId => Self::ReadId,
            ReadColumnError::Signal => Self::Signal,
            ReadColumnError::RunInfo => Self::RunInfo,
            ReadColumnError::Channel => Self::Channel,
            ReadColumnError::Well => Self::Well,
            ReadColumnError::CalibrationOffset => Self::CalibrationOffset,
            ReadColumnError::CalibrationScale => Self::CalibrationScale,
            ReadColumnError::ReadNumber => Self::ReadNumber,
            ReadColumnError::Start => Self::Start,
            ReadColumnError::NumSamples => Self::NumSamples,
        }
    }
}

impl Display for ReadColumnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let variant = match self {
            Self::ReadId|Self::Signal|Self::RunInfo => "Structural",
            _ => "Core data",
        };
        write!(f, "{} column {} cannot contain null values", variant, ReadSchema::from(*self).get_name())
    }
}

impl Error for ReadColumnError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error indicating that a [`ReadRecord`]'s field is not accessible.
pub enum ReadError {
    /// A mandatory field's source column contains an invalid `Null` value.
    FieldMissing(ReadColumnError),

    /// A UUID value did not contain exactly 16 bytes.
    InvalidUUIDLength(InvalidUUIDLength),
    
    /// todo
    DecoderError(DecoderError),

    /// The requested row index falls outside the valid bounds of the batch.
    RowIndexOutOfBounds(IndexOutOfBounds),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldMissing(_) => write!(f, "Field missing"),
            Self::InvalidUUIDLength(_) => write!(f, "Invalid UUID length"),
            Self::DecoderError(_) => write!(f, "Decoder error"),
            Self::RowIndexOutOfBounds(_) => write!(f, "Row index out of bounds"),
        }
    }
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FieldMissing(err) => Some(err),
            Self::InvalidUUIDLength(err) => Some(err),
            Self::DecoderError(err) => Some(err),
            Self::RowIndexOutOfBounds(err) => Some(err),
        }
    }
}

/// Defines the operations of a `ReadRecord`.
pub trait ReadRecordContract<R: ReferenceModel>: Record<R> {     
    /// The [`SignalBuffer`] implementation returned by [`signals()`](Self::signals).
    type SignalBuffer: SignalBufferContract<<R::ConcurrencyMode as ConcurrencyMode>::InitialReferenceModel<SignalColumns>>;

    /// The type of [`Error`] which can be returned.
    type ReadError: Error;

    /// Returns the [`SignalBuffer`](Self::SignalBuffer) of the read.
    fn signals(&self) -> Result<Self::SignalBuffer, Self::ReadError>;

    /// The unique alphanumeric string identifying the specific read event.
    fn read_id(&self) -> Result<Uuid, Self::ReadError>;

    /// The row indexes to the associated signal records.
    fn signal_row_indexes(&self) -> Result<&[u64], Self::ReadError>;

    /// The hardware sequencer channel identifier.
    fn channel(&self) -> Result<u16, Self::ReadError>;

    /// The hardware pore well location identifier.
    fn well(&self) -> Result<u8, Self::ReadError>;

    /// The programmatic or state classification of the pore.
    fn pore_type(&self) -> Result<Option<&str>, Self::ReadError>;

    /// Digitisation offset scalar applied to normalize raw signal integers.
    fn calibration_offset(&self) -> Result<f32, Self::ReadError>;

    /// Scaling range factor applied to normalize raw signal integers.
    fn calibration_scale(&self) -> Result<f32, Self::ReadError>;

    /// Chronological execution index number of this read sequence.
    fn read_number(&self) -> Result<u32, Self::ReadError>;

    /// Absolute starting point (in samples) relative to the stream beginning.
    fn start(&self) -> Result<u64, Self::ReadError>;

    /// Contextual baseline median signal sequence captured prior to read activity.
    fn median_before(&self) -> Result<Option<f32>, Self::ReadError>;

    /// Dynamic scale variance adaptation tracking used by basecallers.
    fn tracked_scaling_scale(&self) -> Result<Option<f32>, Self::ReadError>;

    /// Dynamic local shift drift adaptation tracking used by basecallers.
    fn tracked_scaling_shift(&self) -> Result<Option<f32>, Self::ReadError>;

    /// Mathematically predicted scale properties for mapping algorithms.
    fn predicted_scaling_scale(&self) -> Result<Option<f32>, Self::ReadError>;

    /// Mathematically predicted offset shift properties for mapping algorithms.
    fn predicted_scaling_shift(&self) -> Result<Option<f32>, Self::ReadError>;

    /// Tally counts of distinct reads completed since the latest physical mux change.
    fn num_reads_since_mux_change(&self) -> Result<Option<u32>, Self::ReadError>;

    /// Exact running time elapsed since the physical mux layout transitioned.
    fn time_since_mux_change(&self) -> Result<Option<f32>, Self::ReadError>;
    
    /// Total count of discrete internal MinKNOW capture cycle notifications.
    fn num_minknow_events(&self) -> Result<Option<u64>, Self::ReadError>;

    /// Reason description detailing why the signal acquisition phase halted.
    fn end_reason(&self) -> Result<Option<&str>, Self::ReadError>;

    /// Indicates if data capture termination was forcefully overridden by system policy.
    fn end_reason_forced(&self) -> Result<Option<bool>, Self::ReadError>;

    /// Unique key referencing parent context details in global metadata arrays.
    fn run_info(&self) -> Result<&str, Self::ReadError>;

    /// Grand total size bounds of signal items present within the `Read`.
    fn num_samples(&self) -> Result<u64, Self::ReadError>;

    /// Metric score recording stable non-blocked open channel flow levels.
    fn open_pore_level(&self) -> Result<Option<f32>, Self::ReadError>;
}


/// A zero-copy view over a read record within a POD5 file.
///
/// `ReadRecord` borrows the underlying data and provides typed access to
/// the metadata associated with a single sequencing read.
pub struct ReadRecord<R: ReferenceModel> {
    /// The batch backing this record.
    source: R::BatchRef<ReadColumns>,

    /// The row within the batch.
    row: BatchRowIndex,

    /// Handle used during iteration calls to share access to the registry.
    pod5_context_handle: R::Pod5ContextHandle,
}

impl<R: ReferenceModel> ReadRecord<R> {
    /// todo
    pub(super) fn new(
        source: R::BatchRef<ReadColumns>,
        row: BatchRowIndex,
        pod5_context_handle: R::Pod5ContextHandle,
    ) -> Self {
        Self {
            source,
            row,
            pod5_context_handle,
        }
    }

    fn unpack_strict<'a, T, I: Indexable<Value<'a>=T>>(&'a self, indexable: &'a I, error: ReadColumnError) -> Result<T, ReadError> {
        self.unpack(indexable)?
            .ok_or(ReadError::FieldMissing(error))
    }

    fn unpack<'a, T, I: Indexable<Value<'a>=T>>(&'a self, indexable: &'a I) -> Result<Option<T>, ReadError> {
        indexable.index(self.row)
            .map_err(ReadError::RowIndexOutOfBounds)
    }
}

impl<R: ReferenceModel> Record<R> for ReadRecord<R> {
    type BatchColumns = ReadColumns;
    type StoredRecord = ReadRecord<R::StoredReferenceModel<ReadColumns>>;

    fn to_stored(self) -> Self::StoredRecord {
        Self::StoredRecord {
            source: self.source.into_stored(),
            row: self.row,
            pod5_context_handle: self.pod5_context_handle.into_stored(),
        }
    }
}

impl<R: ReferenceModel> ReadRecordContract<R> for ReadRecord<R> {
    type SignalBuffer = SignalBuffer<<R::ConcurrencyMode as ConcurrencyMode>::InitialReferenceModel<SignalColumns>>;

    type ReadError = ReadError;

    fn signals(&self) -> Result<Self::SignalBuffer, ReadError> {
        let uuid = self.read_id()?;
        let decoder:&Pod5Context<R::ConcurrencyMode> = self.pod5_context_handle.deref();
        decoder.signal_decoder().get_buffer(uuid).map_err(ReadError::DecoderError)
    }

    fn read_id(&self) -> Result<Uuid, ReadError> {
        self.unpack_strict(
            self.source.read_id_column(),
            ReadColumnError::ReadId,
        )?.map_err(ReadError::InvalidUUIDLength)
    }

    fn signal_row_indexes(&self) -> Result<&[u64], ReadError> {
        self.unpack_strict(
            self.source.signal_column(),
            ReadColumnError::Signal,
        )
    }

    fn channel(&self) -> Result<u16, ReadError> {
        self.unpack_strict(
            self.source.channel_column(),
            ReadColumnError::Channel,
        )
    }

    fn well(&self) -> Result<u8, ReadError> {
        self.unpack_strict(
            self.source.well_column(),
            ReadColumnError::Well,
        )
    }

    fn pore_type(&self) -> Result<Option<&str>, ReadError> {
        self.unpack(self.source.pore_type_column())
    }

    fn calibration_offset(&self) -> Result<f32, ReadError> {
        self.unpack_strict(
            self.source.calibration_offset_column(),
            ReadColumnError::CalibrationOffset,
        )
    }

    fn calibration_scale(&self) -> Result<f32, ReadError> {
        self.unpack_strict(
            self.source.calibration_scale_column(),
            ReadColumnError::CalibrationScale,
        )
    }

    fn read_number(&self) -> Result<u32, ReadError> {
        self.unpack_strict(
            self.source.read_number_column(),
            ReadColumnError::ReadNumber,
        )
    }

    fn start(&self) -> Result<u64, ReadError> {
        self.unpack_strict(
            self.source.start_column(),
            ReadColumnError::Start,
        )
    }

    fn median_before(&self) -> Result<Option<f32>, ReadError> {
        self.unpack(self.source.median_before_column())
    }

    fn tracked_scaling_scale(&self) -> Result<Option<f32>, ReadError> {
        self.unpack(self.source.tracked_scaling_scale_column())
    }

    fn tracked_scaling_shift(&self) -> Result<Option<f32>, ReadError> {
        self.unpack(self.source.tracked_scaling_shift_column())
    }

    fn predicted_scaling_scale(&self) -> Result<Option<f32>, ReadError> {
        self.unpack(self.source.predicted_scaling_scale_column())
    }

    fn predicted_scaling_shift(&self) -> Result<Option<f32>, ReadError> {
        self.unpack(self.source.predicted_scaling_shift_column())
    }

    fn num_reads_since_mux_change(&self) -> Result<Option<u32>, ReadError> {
        self.unpack(self.source.num_reads_since_mux_change_column())
    }

    fn time_since_mux_change(&self) -> Result<Option<f32>, ReadError> {
        self.unpack(self.source.time_since_mux_change_column())
    }

    fn num_minknow_events(&self) -> Result<Option<u64>, ReadError> {
        self.unpack(self.source.num_minknow_events_column())
    }

    fn end_reason(&self) -> Result<Option<&str>, ReadError> {
        self.unpack(self.source.end_reason_column())
    }

    fn end_reason_forced(&self) -> Result<Option<bool>, ReadError> {
        self.unpack(self.source.end_reason_forced_column())
    }

    fn run_info(&self) -> Result<&str, ReadError> {
        self.unpack_strict(
            self.source.run_info_column(),
            ReadColumnError::RunInfo,
        )
    }

    fn num_samples(&self) -> Result<u64, ReadError> {
        self.unpack_strict(
            self.source.num_samples_column(),
            ReadColumnError::NumSamples,
        )
    }

    fn open_pore_level(&self) -> Result<Option<f32>, ReadError> {
        self.unpack(self.source.open_pore_level_column())
    }
}