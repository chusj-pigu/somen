//! todo: column module doc

// standard

// third party
use arrow::array::RecordBatch;
// local
use crate::record::batch::BatchError;


mod read;
mod run_info;
mod signal;
pub mod arrays;

pub use self::{
    read::ReadColumns,
    run_info::RunInfoColumns,
    signal::SignalColumns,
};

mod sealed {
    pub trait Seal {}
}

/// A `BatchColumns` provides typed column wrappers for a
/// [`BatchCore`](crate::record::batch::BatchCore).
///
/// Constructs and provides the typed column wrappers used by a
/// [`BatchCore`](crate::record::batch::BatchCore).
///
/// Each implementation defines the column layout expected for a specific batch
/// type and performs the Arrow downcasts required to access its columns upon initialization.
/// This is not a full schema validation.
pub trait BatchColumns: sealed::Seal {
    /// Attempts to construct a typed column set from an Arrow [`RecordBatch`].
    ///
    /// Returns `None` if the record batch does not contain the expected columns or
    /// if any column cannot be downcast to its expected type.

    /// Returns a new `BatchColumns` built from the
    /// downcast columns of the [`RecordBatch`], or
    /// `None` if the `RecordBatch` doesn't match the
    /// downcast requirements.
    fn new(source: &RecordBatch) -> Result<Self, BatchError> where Self: Sized;
}

impl sealed::Seal for ReadColumns {}
impl sealed::Seal for RunInfoColumns {}
impl sealed::Seal for SignalColumns {}