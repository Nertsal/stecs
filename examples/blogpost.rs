#![allow(dead_code)]

use ecs::prelude::*;
use generational_arena::Arena;

// Define an Archetype
#[derive(SplitFields)]
#[split(debug, clone)] // derive Debug and Clone for generated reference types
struct Monster {
    position: (f32, f32),
    health: f32,
    tick: usize,
    damage: Option<f32>,
}

#[derive(SplitFields)]
struct Corpse {
    #[split(nested)]
    monster: Monster,
    time: f32,
}

struct World {
    monsters: StructOf<Arena<Monster>>,
    corpses: StructOf<Arena<Corpse>>,
}

fn main() {
    let mut world = World {
        monsters: Default::default(),
        corpses: Default::default(),
    };

    // Insert a new Monster
    let id = world.monsters.insert(Monster {
        position: (0.0, 0.0),
        health: 10.0,
        tick: 7,
        damage: None,
    });

    // Remove a Monster by id
    let monster: Monster = world.monsters.remove(id).unwrap();

    world.monsters.insert(monster);
    world.monsters.insert(Monster {
        position: (1.0, 3.0),
        health: 5.0,
        tick: 5,
        damage: Some(1.0),
    });

    world.corpses.insert(Corpse {
        monster: Monster {
            position: (-5.0, 0.0),
            health: 0.0,
            tick: 10,
            damage: None,
        },
        time: 5.0,
    });

    // Query monsters' positions and damage (only Some variants)
    for (id, position, damage) in query!(world.monsters, (id, &mut position, &damage.Get.Some)) {
        println!("[{:?}] at {:?}, dealing {} damage", id, position, damage);
    }

    // Querying into a struct

    // 1. define the struct
    #[derive(Debug)]
    struct MonsterRef<'a> {
        position: &'a (f32, f32),
        damage: &'a f32,
    }

    // 2. query
    for monster in query!(
        world.monsters,
        MonsterRef {
            position,
            damage: &damage.Get.Some,
        }
    ) {
        println!("{:?}", monster);
    }

    for (time, position) in query!(world.corpses, (&time, &monster.position)) {
        println!("{} - {:?}", time, position)
    }
}
