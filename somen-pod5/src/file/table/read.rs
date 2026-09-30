// standard

// third party
use arrow::buffer::Buffer;
// local
use crate::{
    io::{
        reader::{
            RefCounted,
            ConcurrencyMode,
        },
        ipc_reader::{
            IPCReader,
            Metadata,
        },
        mmap::Mmap,
    },
    record::{
        batch::{
            BatchCore,
            BatchError,
            BatchIndex,
            BatchIndexLookup,
            BatchResult,
            TableName,
            columns::{
                BatchColumns,
                ReadColumns,
            },
        },
        iter::{ReadIterContract, ReadIter},
        Record
    },
    file::{
        schema::{ColumnSchema, ReadSchema},
        table::TableError,
        FileRowIndex,
        Pod5,
    }
};
use crate::file::table::BatchProvider;
use crate::io::mmap::ByteRange;
use crate::record::ReadRecord;

/// todo
pub trait ReadTableContract<M: ConcurrencyMode> {
    /// todo
    type Iter: ReadIterContract<M>;

    /// todo
    type Record: Record<M::InitialReferenceModel<ReadColumns>, BatchColumns = ReadColumns>;

    /// todo
    fn iter(&self) -> Self::Iter;

    /// todo
    fn record(&self, index: FileRowIndex) -> Self::Record;

    /// todo
    fn find_batch(&self, row_index: FileRowIndex) -> BatchResult<M, ReadColumns>;

    /// todo
    fn get_batch(&self, batch_index: BatchIndex) -> BatchResult<M, ReadColumns>;
}

/// todo
pub struct ReadTable<M: ConcurrencyMode> {
    /// todo
    ipc_reader: IPCReader,

    /// todo
    batch_index_lookup: M::RefCounted<BatchIndexLookup>,

    /// todo
    pod5: M::WeakRefCounted<Pod5<M>>,

    /// todo
    mmap: M::RefCounted<Mmap>,
}

impl<M: ConcurrencyMode> ReadTable<M> {
    /// todo
    fn new(
        buffer: Buffer,
        pod5: M::WeakRefCounted<Pod5<M>>,
        mmap: M::RefCounted<Mmap>,
    ) -> Result<(Self, Metadata), TableError> {
        let (ipc_reader,schema,  metadata) = IPCReader::new(buffer)?;
        ReadSchema::validate_schema(schema)?;
        let batch_index_lookup = ipc_reader.get_batch_index_lookup()?;
        Ok((
            Self {
                ipc_reader,
                batch_index_lookup: M::RefCounted::new(batch_index_lookup),
                pod5,
                mmap,
            },
            metadata
        ))
    }
    
    pub(crate) fn get_ipc_reader(&self) -> &IPCReader {
        &self.ipc_reader
    }
}

impl<M: ConcurrencyMode> BatchProvider<M, ReadColumns> for ReadTable<M> {
    fn get_batch(&self, batch_index: BatchIndex) -> BatchResult<M, ReadColumns> {
        let batch = match self.ipc_reader.get_batch(batch_index) {
            Ok(batch) => batch,
            Err(e) => return Err(e.into()),
        };
        let byte_range = self.ipc_reader.get_batch_range(batch_index);
        BatchCore::new(batch, byte_range, self.mmap.clone())
    }

    fn get_batch_byte_range(&self, batch_index: BatchIndex) -> ByteRange {
        self.ipc_reader.get_batch_range(batch_index)
    }
}

impl<M: ConcurrencyMode> ReadTableContract<M> for ReadTable<M> {
    type Iter = ReadIter<M>;
    type Record = ReadRecord<M::InitialReferenceModel<ReadColumns>>;

    fn iter(&self) -> Self::Iter {
        todo!()
    }

    fn record(&self, index: FileRowIndex) -> Self::Record {
        todo!()
    }

    fn find_batch(&self, row_index: FileRowIndex) -> BatchResult<M, ReadColumns> {
        let index = match self.batch_index_lookup.search(row_index) {
            Some(index) => index,
            None => return Err(BatchError::OutOfBounds(row_index, TableName::Read ))
        };
        <Self as ReadTableContract<M>>::get_batch(self, index)
    }

    fn get_batch(&self, batch_index: BatchIndex) -> BatchResult<M, ReadColumns> {
        let byte_range = self.ipc_reader.get_batch_range(batch_index);
        self.mmap.will_need(byte_range);
        <Self as BatchProvider<M, ReadColumns>>::get_batch(self, batch_index)
    }
}