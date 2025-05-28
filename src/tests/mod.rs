use crate::{self as stecs, archetype::SplitFields, prelude::*};

/// Test that when an entity is removed from the archetype,
/// its identifier cannot be used to access data of entities created after.
#[test]
fn entity_remove_id_invalid() {
    #[derive(SplitFields)]
    struct Unit {
        a: (),
    }

    fn test_storage<F: StorageFamily>() {
        let mut units = <Unit as SplitFields<F>>::StructOf::default();
        let id = units.insert(Unit { a: () });
        units.remove(id);
        units.insert(Unit { a: () });
        assert!(units.get(id).is_none());
    }

    #[cfg(feature = "slotmap")]
    test_storage::<crate::storage::slotmap::SlotMapFamily<slotmap::DefaultKey>>();
    #[cfg(feature = "zero_vec")]
    test_storage::<crate::storage::zero_vec::ZeroVecFamily>();
}
