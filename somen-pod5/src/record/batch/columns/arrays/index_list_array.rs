// standard

// third party
use arrow::{
    array::{
        Array,
        ArrayRef,
        ListArray,
        PrimitiveArray,
        UInt64Array,
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
#[must_use]
pub struct IndexListArray {
    /// The wrapped array.
    wrapped_array: ListArray,
    /// The downcasted data array.
    data_array: UInt64Array,
}

impl IndexListArray {
    /// Verifies if the [`data_type`](DataType) corresponds to a `IndexListArray`,
    /// returning [`DownCastFailure`] otherwise.
    fn is_valid_index_list_array(data_type: &DataType) -> Result<(), DownCastFailure> {
        if let DataType::List(field_ref) = data_type {
            if *field_ref.data_type() == DataType::UInt64 {
                return Ok(());
            }
        }

        let expected = DataType::List(FieldRef::new(Field::new(
            "item",
            DataType::UInt64,
            false,
        )));

        Err(DownCastFailure {
            expected,
            actual: data_type.clone(),
        })
    }

    /// Creates a new wrapper around an Arrow [`ListArray`] and [`UInt64Array`].
    #[inline]
    pub fn from_raw_parts(wrapped_array: ListArray, data_array: UInt64Array) -> Self {
        Self {
            wrapped_array,
            data_array,
        }
    }

    /// Consumes the wrapper and returns the underlying Arrow [`ListArray`] and [`UInt64Array`].
    #[inline]
    #[must_use]
    pub fn to_raw_parts(self) -> (ListArray, UInt64Array) {
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
    pub fn index2(&self, index: BatchRowIndex) -> Result<Option<&[u64]>, IndexOutOfBounds> {
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

impl TryFromArrayRef for IndexListArray {
    #[inline]
    fn try_from_array_ref(array_ref: &ArrayRef) -> Result<Self, SchemaError> {
        FieldType::List.validate(array_ref.data_type())?;
        let array: ListArray = array_ref.to_data().into();
        let data = array.values().to_data().clone();
        Ok(Self::from_raw_parts(array, PrimitiveArray::from(data).into()))
    }
}

impl Indexable for IndexListArray {
    type Value<'a> = &'a [u64];

    fn index(&self, index: BatchRowIndex) -> Result<Option<&[u64]>, IndexOutOfBounds> {
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