// standard

// third party
use arrow::array::ArrayRef;
// local
use crate::file::{
    BatchRowIndex,
    IndexOutOfBounds,
};
use crate::file::schema::SchemaError;
// local
use crate::record::batch::columns::arrays::{
    DownCastFailure,
    SignalBinaryArray,
    SignalListArray,
};
use crate::record::batch::internal::arrays::{Indexable, TryFromArrayRef};
// local
use crate::record::record::types::LargeData;

/// Abstracts over the actual form of the signal data column, be it compressed or not.
#[must_use]
pub enum LargeDataSet {
    /// The compressed, unreadable form of the data.
    VBZ(SignalBinaryArray),

    /// The raw form of the data, directly readable.
    Raw(SignalListArray),
}

impl LargeDataSet {
    /// Returns the value stored at the given batch row.
    ///
    /// Returns `Ok(None)` if the value is null.
    ///
    /// # Errors
    ///
    /// Returns an error if the row index is out of bounds.
    pub fn index2(&self, index: BatchRowIndex) -> Result<Option<LargeData>, IndexOutOfBounds> {
        unimplemented!()
    }
}

impl TryFromArrayRef for LargeDataSet {
    #[inline]
    fn try_from_array_ref(array_ref: &ArrayRef) -> Result<Self, SchemaError> {
        let inner = SignalBinaryArray::try_from_array_ref(array_ref);
        if let Ok(inner) = inner{
            return Ok(LargeDataSet::VBZ(inner))
        }
        let inner = SignalListArray::try_from_array_ref(array_ref)?;
        Ok(LargeDataSet::Raw(inner))
    }
}

impl Indexable for LargeDataSet {
    type Value<'a> = LargeData<'a> where Self: 'a;
    fn index(&self, index: BatchRowIndex) -> Result<Option<LargeData>, IndexOutOfBounds> {
        match self {
            LargeDataSet::VBZ(array) => {
                match array.index(index) {
                    Ok(Some(data)) => Ok(Some(LargeData::VBZ(data))),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }
            LargeDataSet::Raw(array) => {
                match array.index(index) {
                    Ok(Some(data)) => Ok(Some(LargeData::Raw(data))),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }
        }
    }
}