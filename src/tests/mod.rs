use crate as stecs;
use crate::prelude::*;

use slotmap::SlotMap;

/// Test that when an entity is removed from the archetype,
/// its identifier cannot be used to access data of entities created after.
#[test]
fn entity_remove_id_invalid() {
    #[derive(SplitFields)]
    struct Unit {
        a: (),
    }

    let mut units = StructOf::<SlotMap<slotmap::DefaultKey, Unit>>::new();
    let id = units.insert(Unit { a: () });
    units.remove(id);
    units.insert(Unit { a: () });
    assert!(units.get(id).is_none());
}
