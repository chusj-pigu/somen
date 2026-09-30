// standard
use std::{
    error::Error,
    fmt,
};
// third party
use arrow::{
    array::{
        Array,
        ArrayRef,
        FixedSizeBinaryArray,
    },
    datatypes::DataType
};
// local
use crate::file::{
    BatchRowIndex,
    IndexOutOfBounds,
};
use crate::file::schema::SchemaError;
// local
use crate::record::batch::columns::arrays::{DownCastFailure, FieldType};
use crate::record::batch::internal::arrays::{Indexable, TryFromArrayRef};
// local
use crate::record::record::types::Uuid;

/// A non-null UUID value did not contain exactly 16 bytes.
///
/// This indicates that the underlying Arrow array contains invalid data.
/// The contained value is the number of bytes that were found.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidUUIDLength(usize);

impl fmt::Display for InvalidUUIDLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UUID value contains {} bytes instead of 16.", self.0)
    }
}

impl Error for InvalidUUIDLength {}

/// Errors returned when accessing values from a [`UuidArray`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UuidArrayError {
    /// The requested row index is outside the bounds of the array.
    RowIndexOutOfBounds(IndexOutOfBounds),

    /// A non-null UUID value did not contain exactly 16 bytes.
    InvalidUUIDLength(InvalidUUIDLength),
}

impl fmt::Display for UuidArrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RowIndexOutOfBounds(_) => write!(f, "Uuid array index error"),
            Self::InvalidUUIDLength(_) => write!(f, "Uuid array length error"),
        }
    }
}

impl Error for UuidArrayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RowIndexOutOfBounds(e) => Some(e),
            Self::InvalidUUIDLength(e) => Some(e),
        }
    }
}

/// Thin wrapper around Arrow's [`FixedSizeBinaryArray`].
///
/// This wrapper removes Arrow types from the public API while providing a
/// consistent indexing interface shared by the crate's array wrappers.
///
/// Each non-null element is expected to contain the data of a 16-byte UUID.
#[repr(transparent)]
#[must_use]
pub struct UuidArray(FixedSizeBinaryArray);

impl UuidArray {
    /// Creates a new wrapper around an Arrow [`FixedSizeBinaryArray`].
    #[inline]
    pub fn from_raw_parts(array: FixedSizeBinaryArray) -> Self {
        Self(array)
    }

    /// Consumes the wrapper and returns the underlying Arrow [`FixedSizeBinaryArray`].
    #[inline]
    #[must_use]
    pub fn to_raw_parts(self) -> FixedSizeBinaryArray {
        self.0
    }

    /// Returns the UUID stored at the given batch row.
    ///
    /// Returns `Ok(None)` if the value is null.
    ///
    /// # Errors
    ///
    /// Returns an error if the row index is out of bounds or if a non-null
    /// value does not contain exactly 16 bytes.
    #[inline]
    pub fn index2(&self, index: BatchRowIndex) -> Result<Option<Uuid>, UuidArrayError> {
        let array = &self.0;
        let index: usize = index.into();

        if index >= array.len() {
            return Err(UuidArrayError::RowIndexOutOfBounds(IndexOutOfBounds::from(index)));
        }

        if array.is_null(index) {
            return Ok(None);
        }

        let value = array.value(index);
        let len = value.len();

        if let Ok(data) = <&[u8; 16]>::try_from(value) {
            return Ok(Some(Uuid::from_bytes(data)));
        }

        Err(UuidArrayError::InvalidUUIDLength(InvalidUUIDLength(len)))
    }
}

impl TryFromArrayRef for UuidArray {
    #[inline]
    fn try_from_array_ref(array_ref: &ArrayRef) -> Result<Self, SchemaError> {
        FieldType::Uuid.validate(array_ref.data_type())?;
        Ok(Self::from_raw_parts(array_ref.to_data().into()))
    }
}

impl Indexable for UuidArray {
    type Value<'a> = Result<Uuid<'a>, InvalidUUIDLength>;

    fn index(&self, index: BatchRowIndex) 
        -> Result<Option<
            Result<Uuid, InvalidUUIDLength>
        >, IndexOutOfBounds> {
        let array = &self.0;
        let index: usize = index.into();

        if index >= array.len() {
            return Err(IndexOutOfBounds::from(index));
        }

        if array.is_null(index) {
            return Ok(None);
        }

        let value = array.value(index);
        let len = value.len();

        if let Ok(data) = <&[u8; 16]>::try_from(value) {
            return Ok(Some(Ok(Uuid::from_bytes(data))));
        }

        Ok(Some(Err(InvalidUUIDLength(len))))
    }
}