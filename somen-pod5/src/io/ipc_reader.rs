// standard
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    sync::Arc,
};
// third party
use crate::file::RowCount;
use crate::io::mmap::ByteRange;
use arrow::array::RecordBatch;
use arrow::buffer::Buffer;
use arrow::datatypes::Schema;
use arrow::error::ArrowError;
use arrow::ipc::convert::fb_to_schema;
use arrow::ipc::reader::{FileDecoder, read_footer_length};
use arrow::ipc::{Block, Footer, Message, root_as_footer, root_as_message};
use flatbuffers::InvalidFlatbuffer;
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
        write!(
            f,
            "expected {} {}, got {} {}",
            self.specifier, self.expected, self.specifier, self.found
        )
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
            return Err(IPCReaderError::MetaDataError(
                "No meta data found".to_string(),
            ));
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
            Err(IPCReaderError::MetaDataError(format!(
                "{}; {:?}",
                str, metadata
            )))
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
                },
                "MINKNOW:file_identifier" => match file_identifier {
                    None => file_identifier = Some(value.to_string()),
                    Some(_) => return error("Duplicate file identifier in the set"),
                },
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
    pub fn check<'a>(&'a self, other: &'a Metadata) -> Result<(), MetadataError<'a>> {
        if other.pod5_version != self.pod5_version {
            Err(MetadataError {
                specifier: MetadataName::Pod5_version,
                expected: &self.pod5_version,
                found: &other.pod5_version,
            })
        } else if other.software != self.software {
            Err(MetadataError {
                specifier: MetadataName::Software,
                expected: &self.software,
                found: &other.software,
            })
        } else if other.file_identifier != self.file_identifier {
            Err(MetadataError {
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
            IPCReaderError::NotYetImplemented(message) => {
                write!(f, "Not yet implemented: {}", message)
            }
            IPCReaderError::InvalidArgumentError(message) => {
                write!(f, "Invalid argument error: {}", message)
            }
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
            ArrowError::InvalidArgumentError(message) => {
                IPCReaderError::InvalidArgumentError(message)
            }
            ArrowError::SchemaError(message) => IPCReaderError::SchemaError(message),
            e => IPCReaderError::Unknown(e),
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

// Validate footer-provided ranges before passing them to Buffer or Arrow's decoder.
fn block_range(block: &Block, limit: usize) -> Result<(usize, usize, usize), IPCReaderError> {
    let invalid = || IPCReaderError::ParseError("invalid IPC block range".to_string());
    let start = usize::try_from(block.offset()).map_err(|_| invalid())?;
    let metadata_len = usize::try_from(block.metaDataLength()).map_err(|_| invalid())?;
    let body_len = usize::try_from(block.bodyLength()).map_err(|_| invalid())?;
    let len = metadata_len.checked_add(body_len).ok_or_else(invalid)?;
    let end = start.checked_add(len).ok_or_else(invalid)?;
    if end > limit {
        return Err(invalid());
    }
    Ok((start, metadata_len, len))
}

fn block_message(metadata: &[u8]) -> Result<Message<'_>, IPCReaderError> {
    let invalid = || IPCReaderError::ParseError("invalid IPC block message framing".to_string());
    let first: [u8; 4] = metadata
        .get(..4)
        .ok_or_else(invalid)?
        .try_into()
        .map_err(|_| invalid())?;
    let prefix_len = if first == [0xff; 4] { 8 } else { 4 };
    let length_bytes: [u8; 4] = metadata
        .get(prefix_len - 4..prefix_len)
        .ok_or_else(invalid)?
        .try_into()
        .map_err(|_| invalid())?;
    let message_len = usize::try_from(i32::from_le_bytes(length_bytes)).map_err(|_| invalid())?;
    let end = prefix_len.checked_add(message_len).ok_or_else(invalid)?;
    let message = metadata.get(prefix_len..end).ok_or_else(invalid)?;
    root_as_message(message).map_err(IPCReaderError::FlatbufferError)
}

// Arrow's flatbuffer verifier checks memory safety, not the semantic invariants
// assumed by fb_to_schema. Restrict conversion to the types POD5 consumes.
fn validate_schema(schema: arrow::ipc::Schema<'_>) -> Result<(), IPCReaderError> {
    fn field(f: arrow::ipc::Field<'_>, depth: usize) -> Result<(), IPCReaderError> {
        use arrow::ipc::{Precision, TimeUnit, Type};
        let invalid =
            || IPCReaderError::SchemaError("invalid or unsupported POD5 schema field".into());
        if depth > 3 {
            return Err(invalid());
        }
        let children = f.children();
        let count = children.as_ref().map_or(0, |c| c.len());
        if let Some(dict) = f.dictionary() {
            let index = dict.indexType().ok_or_else(invalid)?;
            if f.type_type() != Type::Utf8
                || !index.is_signed()
                || !matches!(index.bitWidth(), 8 | 16 | 32 | 64)
            {
                return Err(invalid());
            }
        }
        match f.type_type() {
            Type::Int if count == 0 => {
                let int = f.type_as_int().ok_or_else(invalid)?;
                if !matches!(int.bitWidth(), 8 | 16 | 32 | 64) {
                    return Err(invalid());
                }
            }
            Type::FloatingPoint if count == 0 => {
                if f.type_as_floating_point().ok_or_else(invalid)?.precision() != Precision::SINGLE
                {
                    return Err(invalid());
                }
            }
            Type::Timestamp if count == 0 => {
                if f.type_as_timestamp().ok_or_else(invalid)?.unit() != TimeUnit::MILLISECOND {
                    return Err(invalid());
                }
            }
            Type::FixedSizeBinary if count == 0 => {
                if f.type_as_fixed_size_binary()
                    .ok_or_else(invalid)?
                    .byteWidth()
                    != 16
                {
                    return Err(invalid());
                }
            }
            Type::Bool | Type::Utf8 | Type::LargeBinary if count == 0 => {
                if f.type_().is_none() {
                    return Err(invalid());
                }
            }
            Type::List | Type::LargeList | Type::Map if count == 1 => {
                if f.type_().is_none() {
                    return Err(invalid());
                }
                let child = children.unwrap().get(0);
                match f.type_type() {
                    Type::List
                        if child
                            .type_as_int()
                            .is_some_and(|i| i.bitWidth() == 64 && !i.is_signed()) =>
                    {
                        ()
                    }
                    Type::LargeList
                        if child
                            .type_as_int()
                            .is_some_and(|i| i.bitWidth() == 16 && i.is_signed()) =>
                    {
                        ()
                    }
                    Type::Map
                        if child.type_type() == Type::Struct_
                            && child.children().is_some_and(|c| c.len() == 2) =>
                    {
                        ()
                    }
                    _ => return Err(invalid()),
                }
                field(child, depth + 1)?;
            }
            Type::Struct_ => {
                if f.type_().is_none() {
                    return Err(invalid());
                }
                if let Some(children) = children {
                    for child in children {
                        field(child, depth + 1)?;
                    }
                }
            }
            _ => return Err(invalid()),
        }
        Ok(())
    }
    let fields = schema
        .fields()
        .ok_or_else(|| IPCReaderError::SchemaError("missing schema fields".into()))?;
    for f in fields {
        field(f, 0)?;
    }
    Ok(())
}

fn reject_compression(batch: arrow::ipc::RecordBatch<'_>) -> Result<(), IPCReaderError> {
    if batch.compression().is_some() {
        return Err(IPCReaderError::IpcError(
            "compressed IPC bodies are unsupported".into(),
        ));
    }
    Ok(())
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
    pub(crate) fn new(buffer: Buffer) -> Result<(Self, Arc<Schema>, Metadata), IPCReaderError> {
        // read ARROW_MAGIC (6 bytes) and footer length (4 bytes)
        let footer_end = match buffer.len().checked_sub(10) {
            Some(start) => start,
            None => {
                return Err(IPCReaderError::ParseError(String::from(
                    "buffer is too small",
                )));
            }
        };
        let trailer_buf: [u8; 10] = buffer[footer_end..].try_into().unwrap();
        let footer_len = read_footer_length(trailer_buf)?;
        let footer_start = match footer_end.checked_sub(footer_len) {
            Some(start) => start,
            None => {
                return Err(IPCReaderError::ParseError(String::from(
                    "buffer is too small",
                )));
            }
        };

        // read footer
        let footer_buf: &[u8] = &buffer[footer_start..footer_end];
        let footer = root_as_footer(footer_buf).map_err(|err| {
            IPCReaderError::ParseError(format!("unable to get root as footer: {err:?}"))
        })?;

        // get block quantity then save as Vec
        let blocks = footer.recordBatches().ok_or_else(|| {
            IPCReaderError::ParseError(String::from("unable to get record batches from IPC Footer"))
        })?;
        let total_blocks = blocks.len();
        if total_blocks > u32::MAX as usize {
            return Err(IPCReaderError::ParseError(
                "too many record batches".to_string(),
            ));
        }
        let blocks: Vec<_> = blocks.iter().copied().collect();
        for block in &blocks {
            block_range(block, footer_start)?;
        }

        let ipc_schema = footer.schema().ok_or_else(|| {
            IPCReaderError::ParseError(String::from("unable to get schema from IPC Footer"))
        })?;
        if !ipc_schema.endianness().equals_to_target_endianness() {
            return Err(IPCReaderError::IpcError(String::from(
                "the endianness of the source system does not match the endianness of the target system.",
            )));
        }

        validate_schema(ipc_schema)?;
        let schema = Arc::new(fb_to_schema(ipc_schema));
        let mut decoder = FileDecoder::new(schema.clone(), footer.version());
        //decoder.

        // Set up the dictionaries using the metadata, needed for the run info batches
        let metadata = Metadata::new(&footer)?;
        if let Some(dictionaries) = footer.dictionaries() {
            for block in dictionaries {
                let (start, metadata_len, total_len) = block_range(block, footer_start)?;
                let message = block_message(&buffer[start..start + metadata_len])?;
                let dictionary = message.header_as_dictionary_batch().ok_or_else(|| {
                    IPCReaderError::ParseError("expected dictionary batch header".to_string())
                })?;
                reject_compression(dictionary.data().ok_or_else(|| {
                    IPCReaderError::ParseError("missing dictionary data".to_string())
                })?)?;
                let block_buf = buffer.slice_with_length(start, total_len);

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
            let (start, metadata_len, _) = block_range(block, self.buffer.len())?;
            let message = block_message(&self.buffer[start..start + metadata_len])?;
            let batch = message.header_as_record_batch().ok_or_else(|| {
                IPCReaderError::ParseError("expected record batch header".to_string())
            })?;
            let rows = u64::try_from(batch.length()).map_err(|_| {
                IPCReaderError::ParseError("negative record batch length".to_string())
            })?;
            vec.push(RowCount::new(rows));
        }
        BatchIndexLookup::try_new(&vec).ok_or_else(|| {
            IPCReaderError::ParseError("record batch row count overflow".to_string())
        })
    }

    /// Return the [`RecordBatch`] at `batch_index`.
    ///
    /// This may return `None` if the IPC message was None
    pub(crate) fn get_batch(&self, batch_index: BatchIndex) -> Result<RecordBatch, IPCReaderError> {
        let block = self
            .blocks
            .get(Into::<usize>::into(batch_index))
            .ok_or_else(|| {
                IPCReaderError::InvalidArgumentError("batch index out of bounds".to_string())
            })?;
        let (start, metadata_len, block_len) = block_range(block, self.buffer.len())?;
        let message = block_message(&self.buffer[start..start + metadata_len])?;
        let batch = message.header_as_record_batch().ok_or_else(|| {
            IPCReaderError::ParseError("expected record batch header".to_string())
        })?;
        reject_compression(batch)?;
        let data = self.buffer.slice_with_length(start, block_len);
        match self.decoder.read_record_batch(block, &data) {
            Ok(Some(batch)) => Ok(batch),
            Ok(None) => Err(IPCReaderError::Empty),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int32Array, StringArray, UInt32Array};
    use arrow::datatypes::{DataType, Field};
    use arrow::ipc::writer::FileWriter;

    #[test]
    fn validates_struct_arity_and_map_entries() {
        fn check(data_type: DataType) -> bool {
            let schema = Schema::new(vec![Field::new("field", data_type, true)]);
            let mut builder = flatbuffers::FlatBufferBuilder::new();
            let offset = arrow::ipc::convert::schema_to_fb_offset(&mut builder, &schema);
            builder.finish(offset, None);
            validate_schema(arrow::ipc::root_as_schema(builder.finished_data()).unwrap()).is_ok()
        }
        for count in [0, 1, 2, 3] {
            let fields: Vec<_> = (0..count)
                .map(|i| Field::new(format!("f{i}"), DataType::Int32, true))
                .collect();
            assert!(check(DataType::Struct(fields.into())));
        }
        for count in [0, 1, 3] {
            let fields: Vec<_> = (0..count)
                .map(|i| Field::new(format!("f{i}"), DataType::Utf8, true))
                .collect();
            let entries = Field::new("entries", DataType::Struct(fields.into()), false);
            assert!(!check(DataType::Map(Arc::new(entries), false)));
        }
        let fields = vec![
            Field::new("key", DataType::Utf8, false),
            Field::new("value", DataType::Utf8, true),
        ];
        assert!(check(DataType::Map(
            Arc::new(Field::new(
                "entries",
                DataType::Struct(fields.into()),
                false
            )),
            false
        )));
    }

    #[test]
    fn rejects_malformed_schema_before_arrow_conversion() {
        use arrow::ipc;
        use flatbuffers::FlatBufferBuilder;
        for kind in [
            ipc::Type::Int,
            ipc::Type::List,
            ipc::Type::Union,
            ipc::Type::NONE,
        ] {
            let mut b = FlatBufferBuilder::new();
            let f = ipc::Field::create(
                &mut b,
                &ipc::FieldArgs {
                    type_type: kind,
                    ..Default::default()
                },
            );
            let fields = b.create_vector(&[f]);
            let schema = ipc::Schema::create(
                &mut b,
                &ipc::SchemaArgs {
                    fields: Some(fields),
                    ..Default::default()
                },
            );
            b.finish(schema, None);
            // Some malformed unions are rejected even earlier by the verifier.
            if let Ok(schema) = ipc::root_as_schema(b.finished_data()) {
                assert!(validate_schema(schema).is_err(), "accepted {kind:?}");
            }
        }
        let mut b = FlatBufferBuilder::new();
        let schema = ipc::Schema::create(&mut b, &ipc::SchemaArgs::default());
        b.finish(schema, None);
        assert!(validate_schema(ipc::root_as_schema(b.finished_data()).unwrap()).is_err());

        let mut b = FlatBufferBuilder::new();
        let int = ipc::Int::create(
            &mut b,
            &ipc::IntArgs {
                bitWidth: 7,
                is_signed: true,
            },
        );
        let f = ipc::Field::create(
            &mut b,
            &ipc::FieldArgs {
                type_type: ipc::Type::Int,
                type_: Some(int.as_union_value()),
                ..Default::default()
            },
        );
        let fields = b.create_vector(&[f]);
        let schema = ipc::Schema::create(
            &mut b,
            &ipc::SchemaArgs {
                fields: Some(fields),
                ..Default::default()
            },
        );
        b.finish(schema, None);
        assert!(validate_schema(ipc::root_as_schema(b.finished_data()).unwrap()).is_err());

        let mut b = FlatBufferBuilder::new();
        let utf8 = ipc::Utf8::create(&mut b, &ipc::Utf8Args::default());
        let dict = ipc::DictionaryEncoding::create(&mut b, &ipc::DictionaryEncodingArgs::default());
        let f = ipc::Field::create(
            &mut b,
            &ipc::FieldArgs {
                type_type: ipc::Type::Utf8,
                type_: Some(utf8.as_union_value()),
                dictionary: Some(dict),
                ..Default::default()
            },
        );
        let fields = b.create_vector(&[f]);
        let schema = ipc::Schema::create(
            &mut b,
            &ipc::SchemaArgs {
                fields: Some(fields),
                ..Default::default()
            },
        );
        b.finish(schema, None);
        assert!(validate_schema(ipc::root_as_schema(b.finished_data()).unwrap()).is_err());
    }

    #[test]
    fn rejects_compressed_batch_metadata() {
        use arrow::ipc;
        let mut b = flatbuffers::FlatBufferBuilder::new();
        let compression =
            ipc::BodyCompression::create(&mut b, &ipc::BodyCompressionArgs::default());
        let batch = ipc::RecordBatch::create(
            &mut b,
            &ipc::RecordBatchArgs {
                compression: Some(compression),
                ..Default::default()
            },
        );
        b.finish(batch, None);
        let batch = flatbuffers::root::<ipc::RecordBatch<'_>>(b.finished_data()).unwrap();
        assert!(reject_compression(batch).is_err());
    }

    #[test]
    fn reads_synthetic_pod5_embedded_tables() {
        let tables: [&[u8]; 3] = [
            include_bytes!("../../tests/fixtures/synthetic-read.arrow"),
            include_bytes!("../../tests/fixtures/synthetic-signal.arrow"),
            include_bytes!("../../tests/fixtures/synthetic-run_info.arrow"),
        ];
        let mut previous = None;
        for (index, bytes) in tables.into_iter().enumerate() {
            let (reader, schema, metadata) = IPCReader::new(Buffer::from(bytes.to_vec())).unwrap();
            if let Some(ref first) = previous {
                metadata.check(first).unwrap();
            } else {
                previous = Some(metadata);
            }
            assert_eq!(reader.num_batches(), 1);
            assert_eq!(
                reader
                    .get_batch_index_lookup()
                    .unwrap()
                    .search(crate::file::FileRowIndex::new(0)),
                Some(BatchIndex::new(0))
            );
            assert_eq!(
                reader
                    .get_batch_index_lookup()
                    .unwrap()
                    .search(crate::file::FileRowIndex::new(1)),
                None
            );
            let batch = reader.get_batch(BatchIndex::new(0)).unwrap();
            assert_eq!(batch.num_rows(), 1);
            assert_eq!(batch.schema(), schema);
            match index {
                0 => assert_eq!(
                    batch
                        .column_by_name("read_number")
                        .unwrap()
                        .as_any()
                        .downcast_ref::<UInt32Array>()
                        .unwrap()
                        .value(0),
                    1
                ),
                1 => assert_eq!(
                    batch
                        .column_by_name("samples")
                        .unwrap()
                        .as_any()
                        .downcast_ref::<UInt32Array>()
                        .unwrap()
                        .value(0),
                    4
                ),
                2 => assert_eq!(
                    batch
                        .column_by_name("acquisition_id")
                        .unwrap()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .unwrap()
                        .value(0),
                    "synthetic-acquisition"
                ),
                _ => unreachable!(),
            }
            assert!(reader.get_batch(BatchIndex::new(1)).is_err());
        }
    }

    #[test]
    fn rejects_corrupt_synthetic_pod5_ipc_messages_and_trailers() {
        let original = include_bytes!("../../tests/fixtures/synthetic-read.arrow");
        for len in [0, 1, 9, original.len() - 1] {
            assert!(IPCReader::new(Buffer::from(original[..len].to_vec())).is_err());
        }
        let (reader, _, _) = IPCReader::new(Buffer::from(original.to_vec())).unwrap();
        let start = reader.blocks[0].offset() as usize;
        let mut broken = original.to_vec();
        broken[start..start + 8].copy_from_slice(&[0xff; 8]);
        let (reader, _, _) = IPCReader::new(Buffer::from(broken)).unwrap();
        assert!(reader.get_batch_index_lookup().is_err());
        assert!(reader.get_batch(BatchIndex::new(0)).is_err());
    }

    #[test]
    fn rejects_corrupt_synthetic_pod5_dictionary_message() {
        let original = include_bytes!("../../tests/fixtures/synthetic-read.arrow");
        let footer_end = original.len() - 10;
        let footer_len = read_footer_length(original[footer_end..].try_into().unwrap()).unwrap();
        let footer = root_as_footer(&original[footer_end - footer_len..footer_end]).unwrap();
        let dictionaries = footer.dictionaries().unwrap();
        assert!(!dictionaries.is_empty());
        let start = dictionaries.get(0).offset() as usize;
        let mut broken = original.to_vec();
        broken[start..start + 8].copy_from_slice(&[0xff; 8]);
        assert!(IPCReader::new(Buffer::from(broken)).is_err());
    }

    #[test]
    fn rejects_invalid_block_ranges() {
        assert_eq!(block_range(&Block::new(4, 8, 12), 24).unwrap(), (4, 8, 20));
        for block in [
            Block::new(-1, 8, 0),
            Block::new(0, -1, 0),
            Block::new(0, 8, -1),
            Block::new(10, 8, 7),
            Block::new(i64::MAX, i32::MAX, i64::MAX),
        ] {
            assert!(block_range(&block, 24).is_err());
        }
    }

    #[test]
    fn rejects_bad_message_framing() {
        for data in [
            vec![],
            vec![0xff; 4],
            vec![0xff; 7],
            (-2i32).to_le_bytes().to_vec(),
            100i32.to_le_bytes().to_vec(),
            vec![0, 0, 0, 0],
        ] {
            assert!(block_message(&data).is_err(), "accepted {data:?}");
        }
    }

    #[test]
    fn reads_framed_record_batch_and_rejects_invalid_index() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int32, false)]));
        let batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(Int32Array::from(vec![1, 2]))])
                .unwrap();
        let mut bytes = Vec::new();
        {
            let mut writer = FileWriter::try_new(&mut bytes, &schema).unwrap();
            writer.write(&batch).unwrap();
            writer.finish().unwrap();
        }
        let footer_end = bytes.len() - 10;
        let footer_len = read_footer_length(bytes[footer_end..].try_into().unwrap()).unwrap();
        let footer = root_as_footer(&bytes[footer_end - footer_len..footer_end]).unwrap();
        let blocks: Vec<_> = footer.recordBatches().unwrap().iter().copied().collect();
        let decoder = FileDecoder::new(schema, footer.version());
        let reader = IPCReader {
            total_blocks: blocks.len(),
            blocks,
            buffer: Buffer::from(bytes),
            decoder,
        };
        assert!(reader.get_batch_index_lookup().is_ok());
        assert_eq!(reader.get_batch(BatchIndex::new(0)).unwrap(), batch);
        assert!(reader.get_batch(BatchIndex::new(1)).is_err());
    }

    #[test]
    fn supports_empty_batch_after_nonempty_batch() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int32, false)]));
        let batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(Int32Array::from(vec![1, 2]))])
                .unwrap();
        let empty = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(Vec::<i32>::new()))],
        )
        .unwrap();
        let mut bytes = Vec::new();
        {
            let mut writer = FileWriter::try_new(&mut bytes, &schema).unwrap();
            writer.write(&batch).unwrap();
            writer.write(&empty).unwrap();
            writer.finish().unwrap();
        }
        let footer_end = bytes.len() - 10;
        let footer_len = read_footer_length(bytes[footer_end..].try_into().unwrap()).unwrap();
        let footer = root_as_footer(&bytes[footer_end - footer_len..footer_end]).unwrap();
        let blocks: Vec<_> = footer.recordBatches().unwrap().iter().copied().collect();
        let decoder = FileDecoder::new(schema, footer.version());
        let reader = IPCReader {
            total_blocks: blocks.len(),
            blocks,
            buffer: Buffer::from(bytes),
            decoder,
        };
        let lookup = reader.get_batch_index_lookup().unwrap();
        assert_eq!(
            lookup.search(crate::file::FileRowIndex::new(1)),
            Some(BatchIndex::new(0))
        );
        assert_eq!(lookup.search(crate::file::FileRowIndex::new(2)), None);
        assert_eq!(reader.get_batch(BatchIndex::new(1)).unwrap(), empty);
    }
}
