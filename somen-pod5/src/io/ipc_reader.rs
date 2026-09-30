// standard
use std::{
    collections::HashMap,
    error::Error,
    fmt::{
        self,
        Formatter,
        Display,
    },
    sync::Arc,
};
// third party
use arrow::datatypes::{ArrowNativeType, Schema};
use arrow::array::{RecordBatch};
use arrow::error::{ArrowError};
use arrow::buffer::Buffer;
use arrow::ipc::convert::fb_to_schema;
use arrow::ipc::reader::{FileDecoder, read_footer_length};
use arrow::ipc::{Block, root_as_footer, Footer, root_as_message};
use flatbuffers::InvalidFlatbuffer;
use crate::file::RowCount;
use crate::io::mmap::ByteRange;
// local
use crate::record::batch::{BatchIndex, BatchIndexLookup, BatchRange};

/// todo
#[derive(Debug)]
pub enum MetadataName {
    /// todo
    Pod5_version,
    /// todo
    Software,
    /// todo
    File_identifier,
}

impl Display for MetadataName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pod5_version => write!(f, "pod5 version"),
            Self::Software => write!(f, "software"),
            Self::File_identifier => write!(f, "file identifier"),
        }
    }
}

/// todo
#[derive(Debug)]
pub struct MetadataError<'a> {
    /// todo
    specifier: MetadataName,
    /// todo
    expected: &'a String,
    /// todo
    found: &'a String,
}

impl Display for MetadataError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {} {}, got {} {}", self.specifier, self.expected, self.specifier, self.found)
    }
}

impl Error for MetadataError<'_> {}

/// todo
pub struct Metadata {
    /// todo
    pod5_version: String,
    /// todo
    software: String,
    /// todo
    file_identifier: String,
}

impl Metadata {
    /// todo
    pub fn new(footer: &Footer) -> Result<Self, IPCReaderError> {
        if let None = footer.custom_metadata() {
            return Err(IPCReaderError::MetaDataError("No meta data found".to_string()));
        }
        let metadata = footer.custom_metadata().unwrap();

        let error = |str: &str| {
            let mut key_values = Vec::new();
            for kv in metadata {
                key_values.push((
                    kv.key().unwrap_or("No Key"),
                    kv.value().unwrap_or("No Value"),
                ));
            }
            Err(IPCReaderError::MetaDataError(format!("{}; {:?}", str, metadata)))
        };

        if metadata.len() != 3 {
            return error("Incorrect number of fields");
        }

        let mut pod5_version = None;
        let mut software = None;
        let mut file_identifier = None;

        for kv in metadata {
            let key = match kv.key() {
                Some(k) => k,
                None => return error("Missing key in the set"),
            };
            let value = match kv.value() {
                Some(k) => k,
                None => return error("Missing value in the set"),
            };

            match key {
                "MINKNOW:pod5_version" => match pod5_version {
                    None => pod5_version = Some(value.to_string()),
                    Some(_) => return error("Duplicate pod5 version in the set"),
                },
                "MINKNOW:software" => match software {
                    None => software = Some(value.to_string()),
                    Some(_) => return error("Duplicate software in the set"),
                }
                "MINKNOW:file_identifier" => match file_identifier {
                    None => file_identifier = Some(value.to_string()),
                    Some(_) => return error("Duplicate file identifier in the set"),
                }
                _ => return error("Unrecognized key in the set"),
            }
        }

        Ok(Self {
            pod5_version: pod5_version.unwrap(),
            software: software.unwrap(),
            file_identifier: file_identifier.unwrap(),
        })
    }

    /// todo
    pub fn check<'a>(&'a self, other: &'a Metadata) -> Result<(),MetadataError<'a>> {
        if other.pod5_version != self.pod5_version {
            Err(MetadataError{
                specifier: MetadataName::Pod5_version,
                expected: &self.pod5_version,
                found: &other.pod5_version,
            })
        } else if other.software != self.software {
            Err(MetadataError{
                specifier: MetadataName::Software,
                expected: &self.software,
                found: &other.software,
            })
        } else if other.file_identifier != self.file_identifier {
            Err(MetadataError{
                specifier: MetadataName::Pod5_version,
                expected: &self.file_identifier,
                found: &other.file_identifier,
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
/// Error indicating failure to parse an IPC file to create a [`IPCReader`].
pub enum IPCReaderError {
    /// Error during parsing from a string.
    ParseError(String),

    /// Error during IPC operations in `arrow-ipc` or `arrow-flight`.
    IpcError(String),

    /// Returned when functionality is not yet available.
    NotYetImplemented(String),

    /// Error indicating that an unexpected or bad argument was passed to a function.
    InvalidArgumentError(String),

    //// Error during schema-related operations.
    SchemaError(String),

    /// The message header for the `RecordBatch` was not found.
    Empty,

    //// Error indicating the expected metadata content was not found.
    MetaDataError(String),

    /// Error indicating a flatbuffer is invalid
    FlatbufferError(InvalidFlatbuffer),

    /// Unhandled or unknown errors.
    Unknown(ArrowError),
}

impl Display for IPCReaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            IPCReaderError::ParseError(message) => write!(f, "Parser error: {}", message),
            IPCReaderError::IpcError(message) => write!(f, "Ipc error: {}", message),
            IPCReaderError::NotYetImplemented(message) => write!(f, "Not yet implemented: {}", message),
            IPCReaderError::InvalidArgumentError(message) => write!(f, "Invalid argument error: {}", message),
            IPCReaderError::SchemaError(message) => write!(f, "Schema error: {}", message),
            IPCReaderError::Empty => write!(f, "Message header not found"),
            IPCReaderError::MetaDataError(message) => write!(f, "Meta data error: {}", message),
            IPCReaderError::FlatbufferError(err) => write!(f, "Unknown Flatbuffer error"),
            IPCReaderError::Unknown(_) => write!(f, "Unexpected arrow error"),
        }
    }
}

impl From<ArrowError> for IPCReaderError {
    fn from(error: ArrowError) -> Self {
        match error {
            ArrowError::ParseError(message) => IPCReaderError::ParseError(message),
            ArrowError::IpcError(message) => IPCReaderError::IpcError(message),
            ArrowError::NotYetImplemented(message) => IPCReaderError::NotYetImplemented(message),
            ArrowError::InvalidArgumentError(message) => IPCReaderError::InvalidArgumentError(message),
            ArrowError::SchemaError(message) => IPCReaderError::SchemaError(message),
            e => IPCReaderError::Unknown(e)
        }
    }
}

impl Error for IPCReaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IPCReaderError::FlatbufferError(error) => Some(error),
            IPCReaderError::Unknown(error) => Some(error),
            _ => None,
        }
    }
}

/// Incrementally decodes [`RecordBatches`](RecordBatch) from a memory-mapped IPC file stored in a Arrow
/// [`Buffer`] using the [`FileDecoder`] API.
pub(crate) struct IPCReader {
    /// Memory mapped Buffer with the data
    buffer: Buffer,

    /// The decoder
    decoder: FileDecoder,

    /// The blocks in the file
    ///
    /// A block indicates the regions in the file to read to get data
    blocks: Vec<Block>,

    /// The total number of blocks, which may contain record batches and other types
    total_blocks: usize,
}

impl IPCReader {
    //cuz I'll forget again, we are exporting the schema to verify it against what it should be
    //and the metadata to validate that all three tables have the same metadata.
    pub(crate) fn new (buffer: Buffer) -> Result<(Self, Arc<Schema>, Metadata), IPCReaderError> {
        // read ARROW_MAGIC (6 bytes) and footer length (4 bytes)
        let footer_end = match buffer.len().checked_sub(10) {
            Some(start) => start,
            None => return Err(IPCReaderError::ParseError(String::from("buffer is too small"))),
        };
        let trailer_buf: [u8; 10] = buffer[footer_end..].try_into().unwrap();
        let footer_len = read_footer_length(trailer_buf)?;
        let footer_start = match footer_end.checked_sub(footer_len) {
            Some(start) => start,
            None => return Err(IPCReaderError::ParseError(String::from("buffer is too small"))),
        };

        // read footer
        let footer_buf: &[u8] = &buffer[footer_start..footer_end];
        let footer = root_as_footer(footer_buf).map_err(
            |err| IPCReaderError::ParseError(format!("unable to get root as footer: {err:?}")),
        )?;

        // get block quantity then save as Vec
        let blocks = footer.recordBatches().ok_or_else(|| {
            IPCReaderError::ParseError(String::from("unable to get record batches from IPC Footer"))
        })?;
        let total_blocks = blocks.len();
        let blocks = blocks.iter().copied().collect();

        let ipc_schema = footer.schema().ok_or_else(|| {
            IPCReaderError::ParseError(String::from("unable to get schema from IPC Footer"))
        })?;
        if !ipc_schema.endianness().equals_to_target_endianness() {
            return Err(IPCReaderError::IpcError(
                String::from("the endianness of the source system does not match the endianness of the target system.")
            ));
        }

        let schema = Arc::new(fb_to_schema(ipc_schema));
        let mut decoder = FileDecoder::new(schema.clone(), footer.version());
        //decoder.

        // Set up the dictionaries using the metadata, needed for the run info batches
        let metadata = Metadata::new(&footer)?;
        if let Some(dictionaries) = footer.dictionaries() {
            for block in dictionaries {
                let body_len = block.bodyLength().to_usize().unwrap();
                let metadata_len = block.metaDataLength().to_usize().unwrap();
                let total_len = body_len.checked_add(metadata_len).unwrap();

                let block_buf = buffer.slice_with_length(
                    block.offset() as usize,
                    total_len,
                );

                decoder.read_dictionary(block, &block_buf)?
            }
        }

        let reader = Self {
            buffer,
            blocks,
            total_blocks,
            decoder,
        };
        Ok((reader, schema, metadata))
    }

    /// Return the number of [`RecordBatch`]es in this buffer
    pub(crate) fn num_batches(&self) -> usize {
        self.total_blocks
    }

    /// Return the [`RecordBatch`] at `batch_index`.
    ///
    /// This may return `None` if the IPC message was None
    pub(crate) fn get_batch_range(&self, batch_index: BatchIndex) -> ByteRange {
        let block = &self.blocks[Into::<usize>::into(batch_index)];
        let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
        let start = block.offset() as usize;
        ByteRange::new(start, block_len)
    }

    /// todo
    pub(crate) fn get_batch_index_lookup(&self) -> Result<BatchIndexLookup, IPCReaderError> {
        let mut vec = Vec::with_capacity(self.total_blocks);
        for batch_index in 0..self.total_blocks {
            let block = &self.blocks[batch_index];
            let metadata_len = block.metaDataLength() as usize;
            let metadata = self.buffer.slice_with_length(
                block.offset() as usize,
                metadata_len,
            );
            let message = root_as_message(metadata.as_ref())
                .map_err(IPCReaderError::FlatbufferError)?;
            let batch = message.header_as_record_batch().unwrap();
            vec.push(RowCount::new(batch.length() as u64));
        }
        Ok(BatchIndexLookup::new(&vec))
    }

    /// Return the [`RecordBatch`] at `batch_index`.
    ///
    /// This may return `None` if the IPC message was None
    pub(crate) fn get_batch(&self, batch_index: BatchIndex) -> Result<RecordBatch, IPCReaderError> {
        let block = &self.blocks[Into::<usize>::into(batch_index)];
        let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
        let data = self
            .buffer
            .slice_with_length(block.offset() as _, block_len);
        match self.decoder.read_record_batch(block, &data) {
            Ok(Some(batch)) => Ok(batch),
            Ok(None) => Err(IPCReaderError::Empty),
            Err(error) => Err(error.into()),
        }
    }
}