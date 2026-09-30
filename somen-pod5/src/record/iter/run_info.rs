// standard

// third party

// local
use crate::io::reader::ConcurrencyMode;

pub trait RunInfoIterContract<M: ConcurrencyMode>{}

pub struct RunInfoIter<M: ConcurrencyMode> {
    concurrency_mode: M,
}

impl<M: ConcurrencyMode> RunInfoIterContract<M> for RunInfoIter<M> {}