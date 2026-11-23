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
        batch::columns::RunInfoColumns,
        RunInfoRecord,
    },
};

pub trait RunInfoIterContract<M: ConcurrencyMode>{}

/// Iterates the [`RunInfoRecord`]s of a `RunInfoBatch`, in row order.
pub struct RunInfoIter<M: ConcurrencyMode> {
    batch_ref: BatchRefOf<M, RunInfoColumns>,
    context: Pod5ContextHandleOf<M, RunInfoColumns>,
    cursor: u32,
    len: u32,
}

impl<M: ConcurrencyMode> RunInfoIter<M> {
    pub(crate) fn new(
        batch_ref: BatchRefOf<M, RunInfoColumns>,
        context: Pod5ContextHandleOf<M, RunInfoColumns>,
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

impl<M: ConcurrencyMode> Iterator for RunInfoIter<M> {
    type Item = RunInfoRecord<M::InitialReferenceModel<RunInfoColumns>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.len {
            return None;
        }
        let row = BatchRowIndex::new(self.cursor);
        self.cursor += 1;
        Some(RunInfoRecord::new(self.batch_ref.clone(), row, self.context.clone()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.cursor) as usize;
        (remaining, Some(remaining))
    }
}

impl<M: ConcurrencyMode> RunInfoIterContract<M> for RunInfoIter<M> {}
