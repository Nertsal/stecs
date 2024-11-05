use stecs::prelude::*;

struct World {
    units: StructOf<Vec<Unit>>,
}

// #[derive(DynamicComponent)]
#[derive(Debug, Clone)]
struct Poisoned {
    time: f32,
}

#[derive(SplitFields)]
#[split(debug, clone)]
struct Unit {
    name: String,
}

fn main() {
    let mut world = World {
        units: Default::default(),
    };

    let unit1 = world.units.insert(Unit {
        name: String::from("Alfred"),
    });
    let unit2 = world.units.insert(Unit {
        name: String::from("Olivia"),
    });

    world.units.insert_dyn(unit1, Poisoned { time: 3.0 });
    world.units.insert_dyn(unit2, Poisoned { time: 2.0 });
    world.units.remove_dyn::<Poisoned>(unit2).unwrap();

    println!("All units:");
    for unit in query!(world.units, (&name, &dyn Poisoned)) {
        println!("unit: {:?}", unit);
    }

    println!("Poisoned units:");
    for (name, poison) in query!(world.units, (&name, &mut dyn Poisoned.Some)) {
        poison.time -= 0.5;
        println!("unit: {:?}", (name, poison));
    }
}
