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
    io::reader::ReferenceModel,
    file::IndexOutOfBounds,
    record::batch::internal::BatchColumns,
};

pub mod types;
mod read;
mod run_info;
mod signal;
mod buffer;

pub use self::{
    read::{ReadRecord, ReadError, ReadColumnError},
    run_info::{RunInfoRecord, RunInfoError, RunInfoColumnError},
    signal::{SignalRecord, SignalError, SignalColumnError, SignalData},
    buffer::SignalBuffer
};

pub(crate) mod internal {
    pub mod contracts {
        pub use self::super::super::{
            read::ReadRecordContract,
            run_info::RunInfoRecordContract,
            signal::SignalRecordContract,
            buffer::SignalBufferContract,
        };
    }
}

#[cfg(feature = "backend")]
pub use self::internal::contracts;


/// A `Record` provides access to a specific row in a [`Batch`](crate::record::batch)
/// and describes how it can be turned into an owned variant which can live longer and
/// has additional properties.
///
///

/// A `Record` can  
pub trait Record<R: ReferenceModel>
{
    /// The type of [`Batch`] the record references.
    type BatchColumns: BatchColumns;

    /// The longer lived version of the [`BatchAccess`] pointer of the record.
    type StoredRecord: Record<R::StoredReferenceModel<Self::BatchColumns>>;

    /// Converts a record from local access to a longer-lived owned version.
    fn to_stored(self) -> Self::StoredRecord;
}