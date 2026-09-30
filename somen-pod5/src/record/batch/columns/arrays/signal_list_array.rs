// standard

// third party
use arrow::{
    array::{
        Array,
        ArrayRef,
        Int16Array,
        ListArray,
        PrimitiveArray,
    },
    datatypes::{
        DataType,
        Field,
        FieldRef,
    },
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

/// Thin wrapper around Arrow's [`ListArray`].
///
/// This wrapper removes Arrow types from the public API while providing a
/// consistent indexing interface shared by the crate's array wrappers.
pub struct SignalListArray {
    /// The wrapped array.
    wrapped_array: ListArray,
    /// The downcasted data array.
    data_array: Int16Array,
}

impl SignalListArray {
    /// Creates a new wrapper around an Arrow [`ListArray`] and [`Int16Array`].
    #[inline]
    #[must_use]
    pub fn from_raw_parts(wrapped_array: ListArray, data_array: Int16Array) -> Self {
        Self {
            wrapped_array,
            data_array,
        }
    }

    /// Consumes the wrapper and returns the underlying Arrow [`ListArray`] and [`Int16Array`].
    #[inline]
    #[must_use]
    pub fn to_raw_parts(self) -> (ListArray, Int16Array) {
        (self.wrapped_array, self.data_array)
    }

    /// Returns the value stored at the given batch row.
    ///
    /// Returns `Ok(None)` if the value is null.
    ///
    /// # Errors
    ///
    /// Returns an error if the row index is out of bounds.
    #[inline]
    pub fn index2(&self, index: BatchRowIndex) -> Result<Option<&[i16]>, IndexOutOfBounds> {
        let array = &self.wrapped_array;
        let index: usize = index.into();

        if index >= array.len() {
            return Err(IndexOutOfBounds(index as u64));
        }

        if array.is_null(index) {
            return Ok(None);
        }

        let offsets = array.offsets();
        let start = offsets[index] as usize;
        let len = array.value_length(index) as usize;

        Ok(Some(&self.data_array.values()[start..start + len]))
    }
}

impl TryFromArrayRef for SignalListArray {
    #[inline]
    fn try_from_array_ref(array_ref: &ArrayRef) -> Result<Self, SchemaError> {
        FieldType::LargeList.validate(array_ref.data_type())?;
        let array: ListArray = array_ref.to_data().into();
        let data = array.values().to_data().clone();
        Ok(Self::from_raw_parts(array, PrimitiveArray::from(data).into()))
    }
}

impl Indexable for SignalListArray {
    type Value<'a> = &'a [i16];

    fn index(&self, index: BatchRowIndex) -> Result<Option<&[i16]>, IndexOutOfBounds> {
        let array = &self.wrapped_array;
        let index: usize = index.into();

        if index >= array.len() {
            return Err(IndexOutOfBounds::from(index));
        }

        if array.is_null(index) {
            return Ok(None);
        }

        let offsets = array.offsets();
        let start = offsets[index] as usize;
        let len = array.value_length(index) as usize;

        Ok(Some(&self.data_array.values()[start..start + len]))
    }
}