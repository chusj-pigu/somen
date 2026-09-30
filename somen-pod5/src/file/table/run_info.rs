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
                RunInfoColumns,
            },
        },
        iter::{RunInfoIterContract, RunInfoIter},
        Record
    },
    file::{
        schema::{ColumnSchema, RunInfoSchema},
        table::TableError,
        FileRowIndex,
        Pod5,
    }
};
use crate::file::table::BatchProvider;
use crate::io::mmap::ByteRange;
use crate::record::RunInfoRecord;

/// todo
pub trait RunInfoTableContract<M: ConcurrencyMode> {
    /// todo
    type Iter: RunInfoIterContract<M>;

    /// todo
    type Record: Record<M::InitialReferenceModel<RunInfoColumns>, BatchColumns = RunInfoColumns>;

    /// todo
    fn iter(&self) -> Self::Iter;

    /// todo
    fn record(&self, index: FileRowIndex) -> Self::Record;

    /// todo
    fn find_batch(&self, row_index: FileRowIndex) -> BatchResult<M, RunInfoColumns>;

    /// todo
    fn get_batch(&self, batch_index: BatchIndex) -> BatchResult<M, RunInfoColumns>;
}

/// todo
pub struct RunInfoTable<M: ConcurrencyMode> {
    /// todo
    ipc_reader: IPCReader,

    /// todo
    batch_index_lookup: M::RefCounted<BatchIndexLookup>,

    /// todo
    pod5: M::WeakRefCounted<Pod5<M>>,

    /// todo
    mmap: M::RefCounted<Mmap>,
}

impl<M: ConcurrencyMode> RunInfoTable<M> {
    /// todo
    fn new(
        buffer: Buffer,
        pod5: M::WeakRefCounted<Pod5<M>>,
        mmap: M::RefCounted<Mmap>,
    ) -> Result<(Self, Metadata), TableError> {
        let (ipc_reader,schema,  metadata) = IPCReader::new(buffer)?;
        RunInfoSchema::validate_schema(schema)?;
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

impl<M: ConcurrencyMode> BatchProvider<M, RunInfoColumns> for RunInfoTable<M> {
    fn get_batch(&self, batch_index: BatchIndex) -> BatchResult<M, RunInfoColumns> {
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

impl<M: ConcurrencyMode> RunInfoTableContract<M> for RunInfoTable<M> {
    type Iter = RunInfoIter<M>;
    type Record = RunInfoRecord<M::InitialReferenceModel<RunInfoColumns>>;

    fn iter(&self) -> Self::Iter {
        todo!()
    }

    fn record(&self, index: FileRowIndex) -> Self::Record {
        todo!()
    }

    fn find_batch(&self, row_index: FileRowIndex) -> BatchResult<M, RunInfoColumns> {
        let index = match self.batch_index_lookup.search(row_index) {
            Some(index) => index,
            None => return Err(BatchError::OutOfBounds(row_index, TableName::RunInfo ))
        };
        <Self as RunInfoTableContract<M>>::get_batch(self, index)
    }

    fn get_batch(&self, batch_index: BatchIndex) -> BatchResult<M, RunInfoColumns> {
        let byte_range = self.ipc_reader.get_batch_range(batch_index);
        self.mmap.will_need(byte_range);
        <Self as BatchProvider<M, RunInfoColumns>>::get_batch(self, batch_index)
    }
}
















/*
temp: figuring out if we can functionnally provide an iter of signal buffers.
from the top of my head, I don't see any reason why we couldn't, but let's think about this.

So the first thing is that we don't deal with any signal stuff untill we know the user is interested with that.
Basically, this means we wait untill the user attempts to use a record signal buffer to start loading the data and decopmresssing if necessary.

One thing to remember is that here, we  want to think in terms of rows and not batches.
there no point decompressing the whole batch if only one row will ever be used.

That seems to imply that it would be a iter-wise system, since the manager only thinks in terms of batches.

I'm not entirelly sure that agrees with me... but it's not the end of the world either.

I'm questioning why pod5 internals has to be a thing actually.
was it that it simplified the pod5 source where it was being used or something like that?

ok, so a couple things: internal cotracts is a stupid concept, don't do it again.
Signal buffer iter is not impossible, per say, but I think I need to think about how signal buffers work from the get go.

The basic idea for how signal buffers work is that they take in a set of signal records upon need.
This however does not work with our decompression strategy.
We want to decompress signals in advance, in parallel.
This imposes two things.
- We firstly need to have a good way to know in advance what signals rows are needed.
    Important: signalBuffers are ONLY accessible through records.
    This means if you CAN obtain a signal buffer, you have a read record,
        meaning you have the coordinates for the attached signals.
    How this plays out, I think, is that we add a parameter N which defines how many reads in advance
        the user wants us to decompress, along with the thread and worker counts.
    Then, we simply load all of these read records at the same time.
    IF and WHEN the user request signal buffers, if the function realizes the signal buffer is
        compressed, we pass a signal somehow back to the read iter.
    That tells it to immediatly make the next N records, and also to immediatly send their signal
        records to the decompressor.
    In fact, the way the signal is propagated would be that the record would hold a refcounted of
        the iter, through which it would call a function of the iter.
    Said function would then be used by the read to obtain the signal buffers.
    It would take in the signal rows, I suppose?
    Some context.
    I think it's better to have the iter provide the signal buffer in this context mainly because
        then the iter is the only one having to interact with the decompressor.
    I suppose in either case, we still will be doing an rc<rcp<T>>,
        to safelly pass the accesssor to the record, so in the end it makes little difference
        in terms of decompressor.
    But we also need access to the iter to trigger the pre-stuff.
    So in total, we need access to;
     - the iter to warn it that it needs to preloads stuff (for the read records)
     - the batch registry to pass it to new iters (for the read and run_info records at least)
     - the decompressor (for the read records) to request for its signal buffer's data
    I'm wondering if this can all be done by interacting with the parent iter.
    The parent iter would have;
     - itself to be warned
     - its own registry to clone
     - access to the decompressor
    This could be exposed as three functions, I think.
    I'm also wondering about wether I really care about the read ids of the signal records and
    all that. I suppose it's better to check them, same with the signal count, for accuracy.
    So yeah, I think it would be sensible to replace the BatchRegistryHandle with a IterHandle.
    tho the iter would be unset/dynamic.

    Now, with the above decided, when a read record's signals function is called and the signals
        are compressed, here's what happens;
     - Firstly, the first read record needs to obtain its signal records.
      - It has a list of the indexs required, filewise.
      - It has a ref of its iter
      - Its iter has a ref of the manager
    [I'm interupting this. I'm wondering if I actually do want a signal iter.]
    [It might be simpler to have the read iter have an internal function able to provide them.]
      - Its iter can provide the signal records for the record
      - Its iter has a ref to the decompressor.
      - If the first signl records are compressed, it sets an internal flag
      - It then sends the signal data to decompressor as a pair (ReadId , Vec[records])
     - Once the Iter is aware that it is dealing with compressed data, it reads N of its records.
      - These are stored in an internal buffer (a circular one most likely).
      - Their signal data is also sent to the decompressor as pairs, in order of reading.
     - The decompressor handles the compressed data
      - The way the decompressor works from the outisde is pretty simple.
      - You pass in (lenght, vec<signal records>)
     -
     -
    .

    mmaps errror can probably bubble up to being an IPC error stored in the registry.
    Going to take a pause of the above stuff to just note down some stuff about the decompression.
    I don't think I actually need rayon for the level of stuff I'm tackling.
    In short, we would pass in three things I think.
    We would pass in the read id, the read's total signal lenght and a vec of the signal records.
    The "give tasks" function basically makes a vector/array of bytes.
    We then make a mutable slice of the vector for each signal's boundaries.
    We then log tasks into a vec<task>, where a task hold the signal record and the slice for it.
    Not quite that easy it seems. slices and refs aren't sendable.
    they aren't, but my orders will, with manual send/sync implementation.
    The whole decompressor will also be, to store the handles.
    I'm unsure as of yet as to how exatlcy I'll handle flagging it all though.
    probably a vec of booleans, sent as mutables refs too allong with the slices in the orders.
    so we basically have three vecs then? one for data, one for tasks, one for workers?
    yes, but not quite. Tasks will be provided via a circular buffer rather than a simple vec,
    probably something like 100 for initial size. The thing is, we cannot updates the vec willy
    nilly, or we might lose our position. actually frick that we totally can,
    it's just that it loses the certatinty that we just need to go up a level...
    I don't even know wth I'm talking about anymore

    Let me go back over it.
    The idea with the order struct is that we want workers to be able to just call the same
    function always, without worrying about the actual index.
    I think that what would work would be a vec<(atomic boolean, option<task>)> (multithread wise)
    Here's how it would all work. You have two main functions, add_tasks and get_task.
    upon add_tasks(), we take in a set of tasks.
    We have a certain index. this index tells us where about in the circle we are. We go at that index

    We want to modify and read at the same time. there is not really easy way to do this.
    Realistically, The

    1+1+signal record+&[], length
    22/14+signal record

    signal record = 8/8/24 + 8 + 8/8/16

    so 22/14+24/48 so 70b at highest level and 46 at the lower one.
    if we could have rfp of the record that would be 30/22, which would be super nice in comparison.
    I'm just not sure I can really do that reasonably?
    the heck, I can just pass a ref, Not like I wasn't alreay doing that.

    so we CAN send orders which are triple ref. We cannot however, have a vec of that.

    We would prefer to have a structure r


*/