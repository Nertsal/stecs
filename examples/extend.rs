use stecs::prelude::*;

struct World {
    units: StructOf<Dense<Unit>>,
}

#[derive(SplitFields)]
struct Unit {
    pos: f32,
}

#[derive(SplitFields)]
struct RenderUnit {
    size: f32,
}

fn main() {
    let mut world = World {
        units: Default::default(),
    };

    let _id = world.units.insert(Unit { pos: 2.0 });

    // Dynamically (at runtime) extend the Unit archetype with RenderUnit
    // That makes each Unit potentially also have RenderUnit,
    // with each field of RenderUnit in its own storage.
    // So it acts as a dynamic version of #[split(nested)]
    // world.units.extend_dyn(id, RenderUnit { size: 1.0 });

    // for (pos, size) in query!(world.units, (&pos, &dyn RenderUnit.size)) {
    //     // ...
    // }
}

// The `World` derive macro checks validity of foreign relationships.
// #[derive(World)]
// struct World {
//     units: StructOf<Dense<Unit>>,
//     #[world(foreign = "units")]
//     render_units: StructOf<Dense<RenderUnit>>,
// }

// #[derive(SplitFields)]
// struct Unit {
//     pos: f32,
// }

// #[derive(SplitFields)]
// struct RenderUnit {
//     #[split(foreign)]
//     unit: Unit,
//     size: f32,
// }

// fn main() {
//     let mut world = World {
//         units: default(),
//         render_units: default(),
//     };

//     for (size, pos) in query!(world.render_units, (&size, &unit.pos)) {
//         // ...
//     }
// }
