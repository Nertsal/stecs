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

    // Query monsters' positions and damage (only Some variants)
    for (position, damage) in query!(world.monsters, (&mut position, &damage.Get.Some)) {
        println!("at {:?}, dealing {} damage", position, damage);
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
