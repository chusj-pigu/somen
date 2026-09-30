//! A `Batch` is the fundamental unit of work for processing POD5 data.
//!
//! Each batch owns a columnar Arrow [`RecordBatch`] together with typed column
//! wrappers, allowing records to be accessed without repeatedly downcasting
//! Arrow arrays.
//!
//! Individual records are produced on demand by indexing into the batch rather
//! than being stored directly.
//!
//! Although `Batch` is part of the public API, it primarily exists to support
//! higher-level iteration and record access.
//! todo: review batch module doc

// standard
use std::{
    error::Error,
    fmt::{
        self,
        Debug,
        Display,
        Formatter
    },
    ops::Deref
};
// third party
use arrow::array::RecordBatch;
// local
use crate::{
    io::{
        reader::ConcurrencyMode,
        ipc_reader::IPCReaderError,
        mmap::{
            ByteRange,
            Mmap,
        },
    },
    record::batch::columns::{
        BatchColumns,
    },
    file::{
        FileRowIndex,
        schema::SchemaError,
    },
};

mod run_info;
mod read;
mod signal;
mod batch_index;
pub mod columns;

pub use self::{
    batch_index::*,
    read::{
        ConcurrentReadBatch,
        ReadBatch,
    },
    run_info::{
        ConcurrentRunInfoBatch,
        RunInfoBatch,
    },
    signal::{
        ConcurrentSignalBatch,
        SignalBatch,
    },
};


pub(crate) mod internal {
    pub use self::super::{
        columns::*,
        read::internal::*,
        run_info::internal::*,
        signal::internal::*,
    };
}

#[cfg(feature = "backend")]
pub mod backend {
    pub use self::internal::*;
}


mod sealed {
    pub trait Seal {}
}


/// The result of initializing a [`Batch`](crate::record::batch).
pub type BatchResult<M: ConcurrencyMode, C: BatchColumns> = Result<BatchCore<M,C>, BatchError>;

/// The shared implementation underlying all [`Batch`](crate::record::batch) types.
///
/// `BatchCore` owns the Arrow [`RecordBatch`] together with its typed column
/// wrappers in the form of [`BatchColumns`], providing the storage from which
/// batch-specific record types are produced without repeated Arrow downcasts.
#[derive(Debug)]
pub struct BatchCore<M: ConcurrencyMode, C: BatchColumns> {
    /// The underlying columnar storage.
    record: RecordBatch,

    /// The typed column wrappers obtained by downcasting the Arrow arrays.
    columns: C,

    /// The information required when dropping a batch
    drop_information: (
        ByteRange,
        M::RefCounted<Mmap>,
    ),
}

impl<M: ConcurrencyMode, C: BatchColumns> BatchCore<M, C> {
    /// Creates a `BatchCore` from an Arrow [`RecordBatch`].
    pub(crate) fn new(record: RecordBatch, byte_range: ByteRange, mmap: M::RefCounted<Mmap>) -> BatchResult<M, C> {
        let columns = C::new(&record)?;
        Ok(Self {
            record,
            columns,
            drop_information: (byte_range, mmap),
        })
    }

    /// Returns a reference to the underlying arrow [`RecordBatch`].
    pub fn as_record_batch(&self) -> &RecordBatch {
        &self.record
    }
}

impl<M: ConcurrencyMode, C: BatchColumns> Drop for BatchCore<M, C> {
    fn drop(&mut self) {
        let (range, mmap) = &self.drop_information;
        mmap.dont_need(*range);
    }
}

impl<M: ConcurrencyMode, C: BatchColumns> Deref for BatchCore<M, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.columns
    }
}

/// The names of the tables of a pod5 file.
#[derive(Debug)]
pub enum TableName {
    RunInfo,
    Read,
    Signal,
}

impl Display for TableName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RunInfo => f.write_str("run-info"),
            Self::Read => f.write_str("read"),
            Self::Signal => f.write_str("signal"),
        }
    }
}

#[derive(Debug)]
/// Errors encountered while accessing batch data.
pub enum BatchError {
    /// todo
    SchemaError(SchemaError),

    /// The reader failed to decode the batch.
    DecodingFailure(IPCReaderError),

    /// todo
    OutOfBounds(FileRowIndex, TableName),
}

impl Display for BatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaError(_) => write!(f, "Schema error"),
            Self::DecodingFailure(_) => write!(f, "Decoding error"),
            Self::OutOfBounds(e, n) => write!(f, "Row index {} out of bounds for the {n} table", e.value()),
        }
    }
}

impl Error for BatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SchemaError(e) => Some(e),
            Self::DecodingFailure(e) => Some(e),
            _ => None,
        }
    }
}

impl From<IPCReaderError> for BatchError {
    fn from(error: IPCReaderError) -> Self {
        BatchError::DecodingFailure(error)
    }
}

impl From<SchemaError> for BatchError {
    fn from(error: SchemaError) -> Self {
        BatchError::SchemaError(error)
    }
}