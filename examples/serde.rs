use serde::{Deserialize, Serialize};
use stecs::{prelude::*, storage::zero_vec::ZeroVec};

#[derive(SplitFields, Debug, Clone, PartialEq)]
#[split(serialize, deserialize)]
struct Unit {
    position: f32,
    health: Option<f32>,
}

#[derive(SplitFields, Debug, Clone, PartialEq)]
#[split(serialize, deserialize)]
struct Particle {
    position: f32,
    size: u32,
}

#[derive(Default, Serialize, Deserialize)]
struct World {
    units: StructOf<ZeroVec<Unit>>,
    particles: StructOf<ZeroVec<Particle>>,
}

// NOTE: SlotMap does not get properly deserialized from json
// but other formats can work fine (like ron)
// <https://github.com/orlp/slotmap/issues/124>

fn main() {
    let units = vec![
        Unit {
            position: 1.0,
            health: None,
        },
        Unit {
            position: 3.0,
            health: Some(10.0),
        },
        Unit {
            position: 2.0,
            health: Some(0.5),
        },
    ];

    let particles = vec![
        Particle {
            position: 0.0,
            size: 0,
        },
        Particle {
            position: 3.0,
            size: 10,
        },
    ];

    let mut world = World::default();

    for unit in &units {
        world.units.insert(unit.clone());
    }
    for particle in &particles {
        world.particles.insert(particle.clone());
    }

    let serialized = serde_json::to_string_pretty(&world).unwrap();
    println!("Serialized world:\n{}", serialized);

    let deserialized: World = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
        units,
        deserialized
            .units
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        particles,
        deserialized
            .particles
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
    );
}
