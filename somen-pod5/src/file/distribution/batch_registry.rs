// standard

// third party

// local
use crate::{
    file::{
        Pod5,
        distribution::{
            BatchRefTracker,
            Capacity,
            CapacityError,
        },
        pod5::Pod5Contract,
        table::{
            internal::{
                ReadTableContract,
                RunInfoTableContract,
                SignalTableContract,
            },
            ReadTable,
            RunInfoTable,
            SignalTable,
        }
    },
    io::{
        reader::{
            BatchRefOf,
            ConcurrencyMode,
            ErrorRefOf,
        },
        mmap::ByteRange,
    },
    record::batch::{
        BatchIndex,
        BatchResult,
        internal::{
            RunInfoColumns,
            ReadColumns,
            SignalColumns,
            BatchColumns
        },
    },
};
use crate::file::table::BatchProvider;

/// todo
pub(crate) struct BatchRegistry<M: ConcurrencyMode> {
    pod5: M::RefCounted<Pod5<M>>,
    run_info_batch_trackers: Vec<M::Tracker<RunInfoColumns>>,
    read_batch_trackers: Vec<M::Tracker<ReadColumns>>,
    signal_batch_trackers: Vec<M::Tracker<SignalColumns>>,
}

impl<M: ConcurrencyMode> BatchRegistry<M> {
    /// todo
    pub(crate) fn new(
        pod5: M::RefCounted<Pod5<M>>,
        run_info_capacities: Vec<Capacity>,
        read_capacities: Vec<Capacity>,
        signal_capacities: Vec<Capacity>,
    ) -> Result<Self, CapacityError> {
        let run_info_batch_trackers = run_info_capacities
            .into_iter()
            .map(M::Tracker::with_capacity)
            .collect::<Result<Vec<_>, CapacityError>>()?;
        let read_batch_trackers = read_capacities
            .into_iter()
            .map(M::Tracker::with_capacity)
            .collect::<Result<Vec<_>, CapacityError>>()?;
        let signal_batch_trackers = signal_capacities
            .into_iter()
            .map(M::Tracker::with_capacity)
            .collect::<Result<Vec<_>, CapacityError>>()?;

        let run_info_table = pod5.run_info_table().get_ipc_reader();
        let read_table = pod5.read_table().get_ipc_reader();
        let signal_table = pod5.signal_table().get_ipc_reader();
        
        for index in pod5.hint_lookahead().initial_batch_indexes() {
            let range = run_info_table.get_batch_range(index);
            pod5.mmap().will_need(range);
            let range = read_table.get_batch_range(index);
            pod5.mmap().will_need(range);
            let range = signal_table.get_batch_range(index);
            pod5.mmap().will_need(range);
        }

        Ok(Self {
            pod5,
            run_info_batch_trackers,
            read_batch_trackers,
            signal_batch_trackers,
        })
    }

    /// todo
    fn get_batch<C: BatchColumns, P: BatchProvider<M, C>>(
        &self,
        index:BatchIndex,
        trackers: &Vec<M::Tracker<C>>,
        batch_provider: &P,
    ) -> Result<BatchRefOf<M, C>, ErrorRefOf<M, C>> {
        let tracker = &trackers[usize::from(index)];
        let result = match tracker.acquire() {
            Some(result) => result,
            None => tracker.set(batch_provider.get_batch(index)),
        };
        let (batch_ref, capacity) = result?;
        if self.pod5.hint_threshold().hit_by(capacity) {
            let next_index = self.pod5.hint_lookahead().next_batch_index(index);
            let range = batch_provider.get_batch_byte_range(next_index);
            self.pod5.mmap().will_need(range);
        }
        Ok(batch_ref)
    }

    /// todo
    pub(crate) fn get_run_info_batch(&self, index: BatchIndex) -> Result<BatchRefOf<M, RunInfoColumns>, ErrorRefOf<M, RunInfoColumns>> {
        self.get_batch(
            index,
            &self.run_info_batch_trackers,
            self.pod5.run_info_table()
        )
    }

    /// todo
    pub(crate) fn get_read_batch(&self, index: BatchIndex) -> Result<BatchRefOf<M, ReadColumns>, ErrorRefOf<M, ReadColumns>> {
        self.get_batch(
            index,
            &self.read_batch_trackers,
            self.pod5.read_table(),
        )
    }

    /// todo
    pub(crate) fn get_signal_batch(&self, index: BatchIndex) -> Result<BatchRefOf<M, SignalColumns>, ErrorRefOf<M, SignalColumns>> {
        self.get_batch(
            index,
            &self.signal_batch_trackers,
            self.pod5.signal_table(),
        )
    }
}