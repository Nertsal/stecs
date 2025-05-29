use slotmap::SlotMap;
use stecs::prelude::*;

struct World<'a> {
    units: StructOf<SlotMap<slotmap::DefaultKey, Unit<'a>>>,
}

#[derive(SplitFields)]
#[split(struct_ref(debug, to_owned))]
struct Position<F: 'static> {
    x: F,
    y: F,
}

#[derive(SplitFields)]
#[split(struct_ref(debug, to_owned))]
struct Unit<'a> {
    #[split(nested)]
    position: Position<f32>,
    name: &'a str,
}

fn main() {
    let unit_name1 = String::from("Alfred");
    let unit_name2 = String::from("Olivia");

    let mut world = World {
        units: Default::default(),
    };

    world.units.insert(Unit {
        position: Position { x: 1.0, y: 5.0 },
        name: &unit_name1,
    });
    world.units.insert(Unit {
        position: Position { x: -3.0, y: 0.0 },
        name: &unit_name2,
    });

    for name in query!(world.units, (&name)) {
        println!("unit: {:?}", name);
    }
}
