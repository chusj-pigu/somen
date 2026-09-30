// standard
use std::{
    error::Error,
    fmt::{
        self,
        Display,
        Formatter,
    },
    sync::Arc,
};
// third party
use arrow::{
    array::RecordBatch,
    datatypes::{DataType, Fields, Schema},
};
// local
use crate::record::batch::internal::arrays::{
    TryFromArrayRef,
    FieldType,
};

mod run_info;
mod read;
mod signal;

pub use self::{
    run_info::*,
    read::*,
    signal::*,
};

#[derive(Debug)]
/// todo
pub enum SchemaError {
    /// todo
    InvalidColumnSet(Fields),

    /// An error where an attempt to downcast a
    /// [`RecordBatch's`](arrow::array::RecordBatch)
    /// to a specific type did not work, because the
    /// [`DataTypes`](DataType) were not compatible.
    DownCastFailure{
        /// The Arrow DataType the column was downcast to.
        expected: DataType,
        /// The Arrow DataType of the column.
        actual: DataType,
    },

    /// todo
    MissingColumn(&'static str, Fields)
}

impl Display for SchemaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidColumnSet(fields) => write!(f, "Invalid column set: {:?}", fields),
            Self::DownCastFailure{expected, actual} => write!(f, "Downcast failed: expected column of type {:?}, but got {:?}.", expected, actual),
            Self::MissingColumn(name, fields) => write!(f, "Missing column {} in {:?}", name, fields),
        }
    }
}

impl Error for SchemaError {}

/// Describes a set of columns composing the schema for a table of the pod5 file.
pub(crate) trait ColumnSchema: Sized + Copy where Self: 'static {
    /// The set of columns associated with the table of this schema.
    const COLUMNS: &'static [Self];

    /// Returns the type of a column.
    fn get_type(&self) -> FieldType;

    /// Returns the name of a column.
    fn get_name(&self) -> &'static str;

    /// Validates a [`Schema`] against the column set.
    /// 
    /// # Errors
    /// 
    /// Return SchemaError if the `Schema` is not conform to the column set.
    fn validate_schema(schema: Arc<Schema>) -> Result<(), SchemaError> {
        for column in Self::COLUMNS {
            let field = match schema.column_with_name(column.get_name()) {
                Some((_, field)) => field,
                None => return Err(SchemaError::MissingColumn(column.get_name(), schema.fields.clone())),
            };
            column.get_type().validate(field.data_type())?;
        }

        Ok(())
    }

    fn try_from_array_ref<T: TryFromArrayRef>(source: &RecordBatch, column: Self) -> Result<T, SchemaError> {
        let schema = source.schema();
        let index = match schema.column_with_name(column.get_name()) {
            Some((index, _)) => index,
            None => return Err(SchemaError::MissingColumn(column.get_name(), schema.fields.clone())),
        };
        T::try_from_array_ref(&source.columns()[index])
    }
}