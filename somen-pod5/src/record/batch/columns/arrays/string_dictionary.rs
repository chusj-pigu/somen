// standard

// third party
use arrow::{
    array::{
        types::{
            Int16Type,
            Int32Type,
            Int64Type,
            Int8Type,
        },
        Array,
        ArrayRef,
        DictionaryArray,
        StringArray,
    },
    datatypes::DataType,
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

/// A type-erased wrapper around [`DictionaryArray`] that unifies its index types.
#[must_use]
pub enum StringDictionaryArray {
    /// variant keyed with signed 8-bit integers
    Int8(DictionaryArray<Int8Type>),

    /// variant keyed with signed 16-bit integers
    Int16(DictionaryArray<Int16Type>),

    /// variant keyed with signed 32-bit integers
    Int32(DictionaryArray<Int32Type>),

    /// variant keyed with signed 64-bit integers
    Int64(DictionaryArray<Int64Type>),
}

impl StringDictionaryArray {
    /// Returns the length of the [`StringDictionaryArray`].
    pub fn len(&self) -> usize {
        match self {
            Self::Int8(dictionary) => dictionary.len(),
            Self::Int16(dictionary) => dictionary.len(),
            Self::Int32(dictionary) => dictionary.len(),
            Self::Int64(dictionary) => dictionary.len(),
        }
    }
    
    /// Returns whether the element of the [`StringDictionaryArray`] at `index` is null.
    pub fn is_null(&self, index: usize) -> bool {
        match self {
            Self::Int8(dictionary) => dictionary.is_null(index),
            Self::Int16(dictionary) => dictionary.is_null(index),
            Self::Int32(dictionary) => dictionary.is_null(index),
            Self::Int64(dictionary) => dictionary.is_null(index),
        }
    }

    /// Returns the index for the element of the [`StringDictionaryArray`] at `index`.
    /// 
    /// (The elements are stored in a separate array)
    pub fn key(&self, index: usize) -> usize {
        match self {
            Self::Int8(dictionary) => dictionary.keys().value(index) as usize,
            Self::Int16(dictionary) => dictionary.keys().value(index) as usize,
            Self::Int32(dictionary) => dictionary.keys().value(index) as usize,
            Self::Int64(dictionary) => dictionary.keys().value(index) as usize,
        }
    }
}

/// Thin wrapper around Arrow's [`DictionaryArray`].
///
/// This wrapper removes Arrow types from the public API while providing a
/// consistent indexing interface shared by the crate's array wrappers.
#[must_use]
pub struct StringDictionary {
    /// The wrapped array.
    wrapped_array: StringDictionaryArray,
    /// The downcasted data array.
    data_array: StringArray,
}

impl StringDictionary {
    /// Creates a new wrapper around an Arrow [`StringDictionaryArray`] and [`StringArray`].
    #[inline]
    pub fn from_raw_parts(wrapped_array: StringDictionaryArray, data_array: StringArray) -> Self {
        Self {
            wrapped_array,
            data_array,
        }
    }

    /// Consumes the wrapper and returns the underlying Arrow [`StringDictionaryArray`] and [`StringArray`].
    #[inline]
    #[must_use]
    pub fn to_raw_parts(self) -> (StringDictionaryArray, StringArray) {
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
    pub fn index2(&self, index: BatchRowIndex) -> Result<Option<&str>, IndexOutOfBounds> {
        let array = &self.wrapped_array;
        let index: usize = index.into();

        if index >= array.len() {
            return Err(IndexOutOfBounds::from(index));
        }

        if array.is_null(index) {
            return Ok(None);
        }

        let key = array.key(index);
        Ok(Some(self.data_array.value(key)))
    }
}

impl TryFromArrayRef for StringDictionary {
    #[inline]
    fn try_from_array_ref(array_ref: &ArrayRef) -> Result<Self, SchemaError> {
        FieldType::Dictionary.validate(array_ref.data_type())?;
        let dictionary = array_ref.to_data();
        let data = dictionary.child_data()[0].clone();
        let array = match array_ref.data_type() {
            DataType::Dictionary(key_type, _) => {
                match **key_type {
                    DataType::Int8 => StringDictionaryArray::Int8(dictionary.into()),
                    DataType::Int16 => StringDictionaryArray::Int16(dictionary.into()),
                    DataType::Int32 => StringDictionaryArray::Int32(dictionary.into()),
                    DataType::Int64 => StringDictionaryArray::Int64(dictionary.into()),
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        };
        Ok(Self::from_raw_parts(array, data.into()))
    }
}

impl Indexable for StringDictionary {
    type Value<'a> = &'a str;

    fn index(&self, index: BatchRowIndex) -> Result<Option<&str>, IndexOutOfBounds> {
        let array = &self.wrapped_array;
        let index: usize = index.into();

        if index >= array.len() {
            return Err(IndexOutOfBounds::from(index));
        }

        if array.is_null(index) {
            return Ok(None);
        }

        let key = array.key(index);
        Ok(Some(self.data_array.value(key)))
    }
}