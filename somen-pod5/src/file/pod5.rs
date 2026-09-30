// standard

// third party

// local
use crate::io::reader::{ConcurrencyMode, HintLookahead, HintThreshold};
use crate::file::table::*;
use crate::file::table::internal::*;
use crate::io::mmap::Mmap;

pub trait Pod5Contract<M: ConcurrencyMode> {
     type RunInfoTable: RunInfoTableContract<M>;
     type ReadTable: ReadTableContract<M>;
     type SignalTable: SignalTableContract<M>;

     fn run_info_table(&self) -> &Self::RunInfoTable;
     fn read_table(&self) -> &Self::ReadTable;
     fn signal_table(&self) -> &Self::SignalTable;
}

pub struct Pod5<M: ConcurrencyMode>{
    hint_threshold: HintThreshold,
    hint_lookahead: HintLookahead,
    run_info_table: RunInfoTable<M>,
    read_table: ReadTable<M>,
    signal_table: SignalTable<M>,
    mmap: M::RefCounted<Mmap>,
}

impl<M: ConcurrencyMode> Pod5<M> {
    fn new(
        hint_threshold: HintThreshold,
        hint_lookahead: HintLookahead,
        run_info_table: RunInfoTable<M>,
        read_table: ReadTable<M>,
        signal_table: SignalTable<M>,
        mmap: M::RefCounted<Mmap>,
    ) -> Self {
        Self {
            hint_threshold,
            hint_lookahead,
            run_info_table,
            read_table,
            signal_table,
            mmap,
        }
    }
    
    pub(crate) fn hint_threshold(&self) -> HintThreshold {
        self.hint_threshold
    }

    pub(crate) fn hint_lookahead(&self) -> HintLookahead {
        self.hint_lookahead
    }

    pub(crate) fn mmap(&self) -> &Mmap {
        &self.mmap
    }
}

impl<M: ConcurrencyMode> Pod5Contract<M> for Pod5<M> {
    type RunInfoTable = RunInfoTable<M>;
    type ReadTable = ReadTable<M>;
    type SignalTable = SignalTable<M>;

    fn run_info_table(&self) -> &RunInfoTable<M> {
        &self.run_info_table
    }
    fn read_table(&self) -> &ReadTable<M> {
        &self.read_table
    }
    fn signal_table(&self) -> &SignalTable<M> {
        &self.signal_table
    }
}