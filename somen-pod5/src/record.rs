//! Pod5 Record
//!
//! This module provides types for accessing the records contained within a POD5
//! file.
//!
//! Records represent the smallest logical units of data exposed by the crate.
//! They are grouped into batches, which are grouped within tables in a POD5
//! file.
//!
//! This module also provides supporting types for efficiently traversing the file,
//! like iterators.

pub mod record;
pub mod iter;
pub mod batch;

pub use self::{
    record::*,
};