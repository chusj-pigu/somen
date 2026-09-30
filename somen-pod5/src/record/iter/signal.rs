// standard

// third party

// local
use crate::io::reader::ConcurrencyMode;

pub trait SignalBufferIterContract<M: ConcurrencyMode>{}

pub struct SignalBufferIter<M: ConcurrencyMode> {
    concurrency_mode: M,
}

impl<M: ConcurrencyMode> SignalBufferIterContract<M> for SignalBufferIter<M> {}