use stecs::prelude::*;

#[derive(Clone)]
pub struct World<'a> {
    pub units: StructOf<Dense<Unit<'a>>>,
}

#[derive(SplitFields)]
#[split(debug, to_owned, clone)]
pub struct Position<F: 'static> {
    pub x: F,
    pub y: F,
}

#[derive(SplitFields)]
#[split(debug, to_owned, clone)]
pub struct Unit<'a> {
    #[split(nested)]
    pub position: Position<f32>,
    pub name: &'a str,
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
