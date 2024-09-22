use stecs::prelude::*;

struct World {
    blocks: StructOf<Vec<Block>>,
}

#[derive(SplitFields)]
#[split(debug)]
struct Position {
    x: i64,
}

#[derive(SplitFields)]
#[split(debug)]
struct Block {
    #[split(nested)]
    position: Position,
    height: i64,
}

fn main() {
    let mut world = World {
        blocks: Default::default(),
    };

    world.blocks.insert(Block {
        position: Position { x: 0 },
        height: 2,
    });

    // Print
    for (_, block) in world.blocks.iter() {
        println!("{:?}", block);
    }

    // Iterate over the whole archetype
    println!("position.x += height");
    for (_, block) in world.blocks.iter_mut() {
        *block.position.x += *block.height;
    }

    // Print
    for (_, block) in world.blocks.iter() {
        println!("{:?}", block);
    }

    // Iterate over a storage
    println!("height += 1");
    for x in world.blocks.height.iter_mut() {
        *x += 1;
    }

    // Print
    for (_, block) in world.blocks.iter() {
        println!("{:?}", block);
    }

    // Iterate over a whole nested archetype
    println!("position.x += 1");
    for (_, position) in world.blocks.position.iter_mut() {
        *position.x += 1;
    }

    // Print
    for (_, block) in world.blocks.iter() {
        println!("{:?}", block);
    }
}
