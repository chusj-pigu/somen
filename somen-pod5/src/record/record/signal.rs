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
    io::{
        reader::{
            BatchRef,
            ReferenceModel,
        },
        vendor::VendorError
    },
    file::{
        BatchRowIndex,
        IndexOutOfBounds,
        schema::{
            ColumnSchema,
            SignalSchema,
        },
    },
    record::{
        batch::internal::{
            SignalColumns,
            arrays::{
                Indexable,
                InvalidUUIDLength,
            },
        },
        Record,
        types::{
            Uuid,
            LargeData,
        },
    },
};

/// Error raised when a mandatory column in a `Signal` contains an invalid null value.
///
/// Pod5 data layout requires these fields to be fully populated. A null value implies
/// either file corruption or a non-compliant file writer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SignalColumnError {
    // --- Structural Layout Columns ---

    /// The read identifier column contains an invalid null value.
    ReadId,

    // --- Core Data Columns ---

    /// The signal column contains an invalid null value.
    Signal,

    /// The samples column contains an invalid null value.
    Samples,
}

impl From<SignalColumnError> for SignalSchema {
    fn from(error: SignalColumnError) -> Self {
        match error {
            SignalColumnError::ReadId => Self::ReadId,
            SignalColumnError::Signal => Self::Signal,
            SignalColumnError::Samples => Self::Samples,
        }
    }
}

impl Display for SignalColumnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let variant = match self {
            Self::ReadId => "Structural",
            _ => "Core data",
        };
        write!(f, "{} column {} cannot contain null values", variant, SignalSchema::from(*self).get_name())
    }
}

impl Error for SignalColumnError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Error indicating that a [`SignalRecord`]'s field is not accessible.
pub enum SignalError {
    /// A mandatory field's source column contains an invalid `Null` value.
    FieldMissing(SignalColumnError),

    /// A UUID value did not contain exactly 16 bytes.
    InvalidUUIDLength(InvalidUUIDLength),

    /// The number of actual signals did not correspond with the metadata.
    WrongSignalQuantity{ expected: usize, found: usize },

    /// The number of actual signals did not correspond with the metadata.
    DecompressionError(VendorError),

    /// The requested row index falls outside the valid bounds of the batch.
    RowIndexOutOfBounds(IndexOutOfBounds),
}

impl Display for SignalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldMissing(_) => write!(f, "Field missing"),
            Self::InvalidUUIDLength(_) => write!(f, "Invalid UUID length"),
            Self::WrongSignalQuantity{expected, found} => write!(f, "Invalid quantity of signals, expected {expected}, found {found}"),
            Self::DecompressionError(_) => write!(f, "Decompression error"),
            Self::RowIndexOutOfBounds(_) => write!(f, "Row index out of bounds"),
        }
    }
}

impl Error for SignalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FieldMissing(err) => Some(err),
            Self::InvalidUUIDLength(err) => Some(err),
            Self::WrongSignalQuantity{..} => None,
            Self::DecompressionError(err) => Some(err),
            Self::RowIndexOutOfBounds(err) => Some(err),
        }
    }
}

/// Defines the operations of a `SignalRecord`.
pub trait SignalRecordContract<R: ReferenceModel>: Record<R> {
    /// The unique alphanumeric string identifying the specific read event.
    fn read_id(&self) -> Result<Uuid, SignalError>;

    /// The row indexes to the associated signal records.
    fn signal(&self) -> Result<&[i16], SignalError>;

    /// Number of signal items present within the `Signal`.
    fn samples(&self) -> Result<u32, SignalError>;
}

/// todo
pub enum SignalData {
    /// todo
    DecompressedVBZ(Vec<i16>),

    /// todo
    Raw,

    /// todo
    Error(SignalError),

    /// todo
    Unset,
}

impl From<SignalError> for SignalData {
    fn from(error: SignalError) -> Self {
        Self::Error(error)
    }
}

/// A zero-copy view over the signals of a read record within a POD5 file.
///
/// `SignalRecord` borrows the underlying data and provides typed access to
/// the metadata associated with a single sequencing read.
pub struct SignalRecord<R: ReferenceModel> {
    /// The batch backing this record.
    source: R::BatchRef<SignalColumns>,

    /// todo
    signal_data: SignalData,

    /// The row within the batch.
    row: BatchRowIndex,
}

impl<R: ReferenceModel> SignalRecord<R> {
    /// todo
    pub(super) fn new(
        source: R::BatchRef<SignalColumns>,
        row: BatchRowIndex,
    ) -> Self {
        Self {
            source,
            signal_data: SignalData::Unset,
            row,
        }
    }

    pub(crate) fn raw_signal(&self) -> Result<LargeData, SignalError> {
        self.unpack_strict(
            self.source.signal_column(),
            SignalColumnError::Signal,
        )
    }

    pub(crate) fn set_data(&mut self, data: SignalData) {
        self.signal_data = data;
    }

    pub(crate) fn is_compressed(&self) -> bool {
        match self.raw_signal() {
            Ok(LargeData::VBZ(_)) => true,
            _ => false,
        }
    }
    
    pub(crate) fn data(&self) -> &SignalData {
        &self.signal_data
    }
    
    pub(crate) fn is_set(&self) -> bool {
        match self.signal_data {
            SignalData::DecompressedVBZ(_) |
            SignalData::Raw | 
            SignalData::Error(_) => true,
            SignalData::Unset => false,
        }
    }

    fn unpack_strict<'a, T, I: Indexable<Value<'a>=T>>(&'a self, indexable: &'a I, error: SignalColumnError) -> Result<T, SignalError> {
        indexable.index(self.row)
            .map_err(SignalError::RowIndexOutOfBounds)?
            .ok_or(SignalError::FieldMissing(error))
    }
}

impl<R: ReferenceModel> Record<R> for SignalRecord<R> {
    type BatchColumns = SignalColumns;
    type StoredRecord = SignalRecord<R::StoredReferenceModel<SignalColumns>>;

    fn to_stored(self) -> Self::StoredRecord {
        Self::StoredRecord {
            source: self.source.into_stored(),
            signal_data: self.signal_data,
            row: self.row,
        }
    }
}

impl<R: ReferenceModel> SignalRecordContract<R> for SignalRecord<R> {
    fn read_id(&self) -> Result<Uuid, SignalError> {
        self.unpack_strict(
            self.source.read_id_column(),
            SignalColumnError::ReadId,
        )?.map_err(SignalError::InvalidUUIDLength)
    }

    fn signal(&self) -> Result<&[i16], SignalError> {
        match self.signal_data {
            SignalData::DecompressedVBZ(ref data) => Ok(data),
            SignalData::Raw => {
                let large_data: LargeData = self.unpack_strict(
                    self.source.signal_column(),
                    SignalColumnError::Signal,
                )?;
                match large_data {
                    LargeData::Raw(data) => Ok(data),
                    LargeData::VBZ(_) => panic!("cannot have compressed vbz signals with raw data type")
                }
            },
            SignalData::Error(e) => Err(e),
            SignalData::Unset => panic!("cannot get signal for unset"),
        }
    }

    fn samples(&self) -> Result<u32, SignalError> {
        self.unpack_strict(
            self.source.samples_column(),
            SignalColumnError::Signal,
        )
    }
}


//pub type SignalRecord = internal::SignalRecordCore<RcBatchRef<SignalColumns>>;
//pub type LeasedSignalRecord = internal::SignalRecordCore<GuardBatchRef<SignalColumns>>;
//pub type ConcurrentSignalRecord = internal::SignalRecordCore<ArcBatchRef<SignalColumns>>;