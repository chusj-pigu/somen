//! **somen-pod5** handles the reading and writing of the POD5 format.
//!
//! POD5 is a format used to store raw electrical signal data from DNA and RNA sequencing.
//! Developed by *Oxford Nanopore Technologies*, it is produced by their devices
//! such as *MinION*, *GridION* and *PromethION*.
//!
//! A POD5 file contains three categories of data: run information, reads, and
//! signal samples. Signal samples are exposed as buffers associated with each read, while run
//! information and reads are exposed as individual records.

extern crate core;

#[cfg(not(any(unix, windows)))]
compile_error!("This crate only supports Windows and Unix-based operating systems.");
pub mod io;
pub mod file;
pub mod record;
