// standard

// third party

// local
 use crate::io::reader::ConcurrencyMode;

pub trait ReadIterContract<M: ConcurrencyMode>{}

pub struct ReadIter<M: ConcurrencyMode> {
    concurrency_mode: M,
}

impl<M: ConcurrencyMode> ReadIterContract<M> for ReadIter<M> {}