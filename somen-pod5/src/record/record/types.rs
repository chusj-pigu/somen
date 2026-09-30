//! POD5 Types
//!
//! This module provides lightweight wrapper types used by POD5 records.
//!
//! These types borrow data directly from the underlying record without copying
//! or allocating, while providing a more convenient and strongly typed API than
//! the raw storage representation.
//!
//! Most users will encounter these types through the accessors on [`Record`](crate::record::record).

mod flat_map;
mod uuid;
mod large_data;

pub use self::{
    flat_map::*,
    uuid::*,
    large_data::*,
};