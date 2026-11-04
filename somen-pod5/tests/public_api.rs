//! Public-facing checks compiled as an external crate in both feature configurations.

use somen_pod5::{file::FileRowIndex, record::record::types::FlatMap};

#[test]
fn ordinary_api_is_usable_with_either_feature_configuration() {
    let index = FileRowIndex::new(42);
    assert_eq!(index.value(), 42);
    let map = FlatMap::new(vec![("key", "first"), ("key", "second")]);
    assert_eq!(map.get("key"), Some("first"));
    assert!(map.has_duplicate_keys());
}

#[cfg(feature = "backend")]
#[test]
fn backend_exports_are_usable_by_external_generic_code() {
    use somen_pod5::{
        file::{
            Pod5,
            backend::Pod5Contract,
            table::{
                ReadTable, RunInfoTable, SignalTable,
                backend::{ReadTableContract, RunInfoTableContract, SignalTableContract},
            },
        },
        io::reader::{
            Atomic, BatchRef, BatchRefOf, ConcurrencyMode, GuardBatchRef, Local, RcBatchRef,
        },
        record::batch::columns::ReadColumns,
    };

    fn pod5<M: ConcurrencyMode, T: Pod5Contract<M>>() {}
    fn tables<
        M: ConcurrencyMode,
        R: ReadTableContract<M>,
        I: RunInfoTableContract<M>,
        S: SignalTableContract<M>,
    >() {
    }
    fn batch_ref<T: BatchRef<ReadColumns>>() {}
    fn same_type<T>(_: std::marker::PhantomData<T>, _: std::marker::PhantomData<T>) {}

    pod5::<Local, Pod5<Local>>();
    pod5::<Atomic, Pod5<Atomic>>();
    tables::<Local, ReadTable<Local>, RunInfoTable<Local>, SignalTable<Local>>();
    tables::<Atomic, ReadTable<Atomic>, RunInfoTable<Atomic>, SignalTable<Atomic>>();
    batch_ref::<RcBatchRef<ReadColumns>>();
    batch_ref::<GuardBatchRef<ReadColumns>>();
    same_type(
        std::marker::PhantomData::<BatchRefOf<Local, ReadColumns>>,
        std::marker::PhantomData::<RcBatchRef<ReadColumns>>,
    );
    same_type(
        std::marker::PhantomData::<BatchRefOf<Atomic, ReadColumns>>,
        std::marker::PhantomData::<GuardBatchRef<ReadColumns>>,
    );
}
