// standard

// third party

// local
use crate::{
    file::BatchRowIndex,
    io::reader::{
        BatchRefOf,
        ConcurrencyMode,
    },
    record::{
        batch::columns::SignalColumns,
        SignalRecord,
    },
};

pub trait SignalBufferIterContract<M: ConcurrencyMode>{}

/// Iterates the [`SignalRecord`]s of a `SignalBatch`, in row order.
pub struct SignalBufferIter<M: ConcurrencyMode> {
    batch_ref: BatchRefOf<M, SignalColumns>,
    cursor: u32,
    len: u32,
}

impl<M: ConcurrencyMode> SignalBufferIter<M> {
    pub(crate) fn new(batch_ref: BatchRefOf<M, SignalColumns>) -> Self {
        let len = u32::try_from(batch_ref.as_record_batch().num_rows()).unwrap_or(u32::MAX);
        Self {
            batch_ref,
            cursor: 0,
            len,
        }
    }
}

impl<M: ConcurrencyMode> Iterator for SignalBufferIter<M> {
    type Item = SignalRecord<M::InitialReferenceModel<SignalColumns>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.len {
            return None;
        }
        let row = BatchRowIndex::new(self.cursor);
        self.cursor += 1;
        Some(SignalRecord::new(self.batch_ref.clone(), row))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.cursor) as usize;
        (remaining, Some(remaining))
    }
}

impl<M: ConcurrencyMode> SignalBufferIterContract<M> for SignalBufferIter<M> {}
