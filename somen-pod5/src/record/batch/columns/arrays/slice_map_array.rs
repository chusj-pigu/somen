// standard

// third party
use arrow::{
    array::{
        Array,
        ArrayRef,
        MapArray,
        StringArray,
    },
    datatypes::{
        DataType,
        Field,
        FieldRef,
        Fields,
    },
};
// local
use crate::file::{
    BatchRowIndex,
    IndexOutOfBounds,
};
use crate::file::schema::SchemaError;
// local
use crate::record::batch::columns::arrays::{DownCastFailure, FieldType, TryFromArrayRef};
use crate::record::batch::internal::arrays::Indexable;
// local
use crate::record::record::types::FlatMap;

/// Thin wrapper around Arrow's [`MapArray`].
///
/// This wrapper removes Arrow types from the public API while providing a
/// consistent indexing interface shared by the crate's array wrappers.
#[must_use]
pub struct SliceMapArray {
    /// The wrapped array.
    wrapped_array: MapArray,
    /// The downcasted keys array.
    keys_array: StringArray,
    /// The downcasted values array.
    values_array: StringArray
}

impl SliceMapArray {
    /// Creates a new wrapper around an Arrow [`MapArray`], [`StringArray`] and [`StringArray`].
    #[inline]
    pub fn from_raw_parts(wrapped_array: MapArray, keys_array: StringArray, values_array: StringArray) -> Self {
        Self {
            wrapped_array,
            keys_array,
            values_array,
        }
    }

    /// Consumes the wrapper and returns the underlying Arrow [`MapArray`], [`StringArray`] and [`StringArray`].
    #[inline]
    #[must_use]
    pub fn to_raw_parts(self) -> (MapArray, StringArray, StringArray) {
        (self.wrapped_array, self.keys_array, self.values_array)
    }

    /// Returns the value stored at the given batch row.
    ///
    /// Returns `Ok(None)` if the value is null.
    ///
    /// # Errors
    ///
    /// Returns an error if the row index is out of bounds.
    #[inline]
    pub fn index2(&self, index: BatchRowIndex) -> Result<Option<FlatMap>, IndexOutOfBounds> {
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

        let data = (start..start+len)
            .map(|i| (self.keys_array.value(i), self.values_array.value(i))).collect();
        Ok(Some(FlatMap::new(data)))
    }
}

impl TryFromArrayRef for SliceMapArray {
    #[inline]
    fn try_from_array_ref(array_ref: &ArrayRef) -> Result<Self, SchemaError> {
        FieldType::Map.validate(array_ref.data_type())?;
        let array: MapArray = array_ref.to_data().into();
        let keys = array.keys().to_data().clone();
        let values = array.values().to_data().clone();
        Ok(Self::from_raw_parts(array, keys.into(), values.into()))
    }
}

impl Indexable for SliceMapArray {
    type Value<'a> = FlatMap<'a>;

    fn index(&self, index: BatchRowIndex) -> Result<Option<FlatMap>, IndexOutOfBounds> {
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

        let data = (start..start+len)
            .map(|i| (self.keys_array.value(i), self.values_array.value(i))).collect();
        Ok(Some(FlatMap::new(data)))
    }
}