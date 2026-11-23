// standard

// third party

// local
use crate::{
    file::BatchRowIndex,
    io::reader::{
        BatchRefOf,
        ConcurrencyMode,
        Pod5ContextHandleOf,
    },
    record::{
        batch::columns::ReadColumns,
        ReadRecord,
    },
};

pub trait ReadIterContract<M: ConcurrencyMode>{}

/// Iterates the [`ReadRecord`]s of a `ReadBatch`, in row order.
pub struct ReadIter<M: ConcurrencyMode> {
    batch_ref: BatchRefOf<M, ReadColumns>,
    context: Pod5ContextHandleOf<M, ReadColumns>,
    cursor: u32,
    len: u32,
}

impl<M: ConcurrencyMode> ReadIter<M> {
    pub(crate) fn new(
        batch_ref: BatchRefOf<M, ReadColumns>,
        context: Pod5ContextHandleOf<M, ReadColumns>,
    ) -> Self {
        let len = u32::try_from(batch_ref.as_record_batch().num_rows()).unwrap_or(u32::MAX);
        Self {
            batch_ref,
            context,
            cursor: 0,
            len,
        }
    }
}

impl<M: ConcurrencyMode> Iterator for ReadIter<M> {
    type Item = ReadRecord<M::InitialReferenceModel<ReadColumns>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.len {
            return None;
        }
        let row = BatchRowIndex::new(self.cursor);
        self.cursor += 1;
        Some(ReadRecord::new(self.batch_ref.clone(), row, self.context.clone()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.cursor) as usize;
        (remaining, Some(remaining))
    }
}

impl<M: ConcurrencyMode> ReadIterContract<M> for ReadIter<M> {}
