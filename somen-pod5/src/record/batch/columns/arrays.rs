//! Arrays serve to provide convenient methods for interacting with
//! various arrow [`Arrays`](arrow::array::Array) which themselves do not provide
//! direct indexing methods.
//!
//! Although `Arrays` are part of the public API, they primarily exists to support
//! higher-level iteration and record access.
//! 
//! todo: review array module doc

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
use arrow::{
    datatypes::{
        DataType,
        Field,
        FieldRef,
        Fields,
        TimeUnit
    },
    array::ArrayRef,
};
// local
use crate::file::{
    BatchRowIndex,
    IndexOutOfBounds,
    schema::SchemaError,
};


mod index_list_array;
mod large_dataset;
mod signal_list_array;
mod simple_arrays;
mod slice_map_array;
mod string_dictionary;
mod uuid_array;

pub use {
    index_list_array::*,
    large_dataset::*,
    signal_list_array::*,
    simple_arrays::*,
    slice_map_array::*,
    string_dictionary::*,
    uuid_array::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// An error where an attempt to downcast a
/// [`RecordBatch's`](arrow::array::RecordBatch)
/// to a specific type did not work, because the
/// [`DataTypes`](DataType) were not compatible.
pub struct DownCastFailure {
    /// The Arrow DataType the column was downcast to.
    pub expected: DataType,
    /// The Arrow DataType of the column.
    pub actual: DataType,
}

impl Display for DownCastFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Downcast failed: expected column of type {:?}, but got {:?}.",
            self.expected, self.actual
        )
    }
}

impl Error for DownCastFailure {}

/// todo
#[derive(Debug, Copy, Clone)]
pub enum FieldType {
    Bool,
    Int16,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Utf8,
    Uuid,
    Timestamp,
    List,
    LargeList,
    Map,
    Dictionary,
    LargeBinary,
    LargeData,
}

impl FieldType {
    fn simple(actual: &DataType, expected: DataType) -> Result<(), SchemaError> {
        if *actual == expected {
            Ok(())
        } else {
            Err(SchemaError::DownCastFailure {
                expected,
                actual: actual.clone(),
            })
        }
    }

    fn timestamp(actual: &DataType) -> Result<(), SchemaError> {
        if let DataType::Timestamp(TimeUnit::Millisecond, _) = &actual {
            Ok(())
        } else {
            Err(SchemaError::DownCastFailure{
                expected: DataType::Timestamp(TimeUnit::Millisecond, None),
                actual: actual.clone(),
            })
        }
    }

    fn list(actual: &DataType) -> Result<(), SchemaError> {
        if let DataType::List(field_ref) = &actual {
            if *field_ref.data_type() == DataType::UInt64 {
                return Ok(());
            }
        }

        let expected = DataType::List(FieldRef::new(Field::new(
            "item",
            DataType::UInt64,
            false,
        )));

        Err(SchemaError::DownCastFailure {
            expected,
            actual: actual.clone(),
        })
    }

    fn large_list(actual: &DataType) -> Result<(), SchemaError> {
        if let DataType::LargeList(field_ref) = &actual {
            if *field_ref.data_type() == DataType::Int16 {
                return Ok(());
            }
        }

        let expected = DataType::List(FieldRef::new(Field::new(
            "item",
            DataType::Int16,
            false,
        )));

        Err(SchemaError::DownCastFailure {
            expected,
            actual: actual.clone(),
        })
    }

    fn map(actual: &DataType) -> Result<(), SchemaError> {
        if let DataType::Map(field_ref, _) = &actual {
            if let DataType::Struct(fields) = field_ref.data_type() {
                if fields.len() == 2
                    && *fields[0].data_type() == DataType::Utf8
                    && *fields[1].data_type() == DataType::Utf8 {
                    return Ok(());
                }
            }
        }

        let expected = DataType::Map(
            FieldRef::new(Field::new(
                "entries",
                DataType::Struct(Fields::from(vec![
                    Field::new("key", DataType::Utf8, false),
                    Field::new("value", DataType::Utf8, true),
                ])),
                false,
            )),
            false,
        );

        Err(SchemaError::DownCastFailure {
            expected,
            actual: actual.clone(),
        })
    }

    fn dictionary(actual: &DataType) -> Result<(), SchemaError> {
        let key_type = if let DataType::Dictionary(key_type, value_type) = actual {
            if **value_type == DataType::Utf8 {
                match **key_type {
                    DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => return Ok(()),
                    _ => {}
                }
            }
            match **key_type {
                DataType::Int8 => DataType::Int8,
                DataType::Int16 => DataType::Int16,
                DataType::Int64 => DataType::Int64,
                DataType::Int32 | _ => DataType::Int32,
            }
        } else {
            DataType::Int32
        };

        let expected = DataType::Dictionary(Box::new(key_type), Box::new(DataType::Utf8));

        Err(SchemaError::DownCastFailure {
            expected,
            actual: actual.clone(),
        })
    }

    fn large_data(actual: &DataType) -> Result<(), SchemaError> {
        if let Ok(()) = Self::LargeBinary.validate(actual) {
            Ok(())
        } else {
            Self::LargeList.validate(actual)
        }
    }

    pub fn validate(&self, data_type: &DataType) -> Result<(), SchemaError> {
        match self {
            Self::Bool => Self::simple(data_type, DataType::Boolean),
            Self::Int16 => Self::simple(data_type, DataType::Int16),
            Self::UInt8 => Self::simple(data_type, DataType::UInt8),
            Self::UInt16 => Self::simple(data_type, DataType::UInt16),
            Self::UInt32 => Self::simple(data_type, DataType::UInt32),
            Self::UInt64 => Self::simple(data_type, DataType::UInt64),
            Self::Float => Self::simple(data_type, DataType::Float32),
            Self::Utf8 => Self::simple(data_type, DataType::Utf8),
            Self::Uuid => Self::simple(data_type, DataType::FixedSizeBinary(16)),
            Self::Timestamp => Self::timestamp(data_type),
            Self::List => Self::list(data_type),
            Self::LargeList => Self::large_list(data_type),
            Self::Map => Self::map(data_type),
            Self::Dictionary => Self::dictionary(data_type),
            Self::LargeBinary => Self::simple(data_type, DataType::LargeBinary),
            Self::LargeData => Self::large_data(data_type),
        }
    }
}

/// todo
pub trait TryFromArrayRef {
    /// Attempts to create a new `TryFromArrayRef` from an Arrow [`ArrayRef`],
    /// returning [`DownCastFailure`] otherwise.
    fn try_from_array_ref(array_ref: &ArrayRef) -> Result<Self, SchemaError> where Self: Sized;
}

/// todo
pub trait Indexable {
    /// todo
    type Value<'a> where Self: 'a;

    /// Returns the value stored at the given batch row.
    ///
    /// Returns `Ok(None)` if the value is null.
    ///
    /// # Errors
    ///
    /// Returns an error if the row index is out of bounds.
    fn index<'a>(&'a self, index: BatchRowIndex) -> Result<Option<Self::Value<'a>>, IndexOutOfBounds>;
}