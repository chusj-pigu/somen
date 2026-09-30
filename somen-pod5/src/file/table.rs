//! todo: table module doc

// standard
use std::{
    error::Error,
    fmt::{
        self,
        Display,
        Formatter,
        Debug
    },
};
// third party

// local
use crate::{
    file::schema::SchemaError,
    io::{
        ipc_reader::IPCReaderError,
        reader::{ConcurrencyMode, RefCounted}
    },
    record::{
        batch::internal::BatchColumns,
        Record
    }
};
use crate::io::mmap::ByteRange;
use crate::record::batch::{BatchIndex, BatchResult};

mod run_info;
mod read;
mod signal;

pub use self::{
    run_info::RunInfoTable,
    read::ReadTable,
    signal::SignalTable,
};

pub(crate) mod internal {
    pub use self::super::{
        read::ReadTableContract,
        run_info::RunInfoTableContract,
        signal::SignalTableContract,
    };
}

#[cfg(feature = "backend")]
pub mod backend {
    pub use self::internal::*;
}


#[derive(Debug)]
/// Errors encountered while accessing batch data.
pub enum TableError {
    /// todo
    SchemaError(SchemaError),

    /// The reader failed to decode the batch.
    DecodingFailure(IPCReaderError),
}

impl Display for TableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaError(_) => write!(f, "Schema error"),
            Self::DecodingFailure(_) => write!(f, "Decoding error"),
        }
    }
}

impl Error for TableError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SchemaError(e) => Some(e),
            Self::DecodingFailure(e) => Some(e),
        }
    }
}

impl From<IPCReaderError> for TableError {
    fn from(error: IPCReaderError) -> Self {
        TableError::DecodingFailure(error)
    }
}

impl From<SchemaError> for TableError {
    fn from(error: SchemaError) -> Self {
        TableError::SchemaError(error)
    }
}

pub(crate) trait BatchProvider<M: ConcurrencyMode, C: BatchColumns> {
    fn get_batch(&self, batch_index: BatchIndex) -> BatchResult<M, C>;
    fn get_batch_byte_range(&self, batch_index: BatchIndex) -> ByteRange;
}