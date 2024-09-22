use stecs::prelude::*;

struct World<'a> {
    units: StructOf<Vec<Unit<'a>>>,
}

#[derive(SplitFields)]
#[split(debug, clone)]
struct Position<F: 'static> {
    x: F,
    y: F,
}

#[derive(SplitFields)]
#[split(debug, clone)]
struct Unit<'a> {
    #[split(nested)]
    position: Position<f32>,
    name: &'a str,
}

fn main() {
    let mut world = World {
        units: Default::default(),
    };

    let unit_name1 = String::from("Alfred");
    let unit_name2 = String::from("Olivia");

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
