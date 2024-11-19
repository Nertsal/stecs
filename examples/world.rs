use stecs::prelude::*;

#[derive(World)]
pub struct World {
    #[world(groups = ["actor"])]
    pub players: StructOf<Dense<Player>>,
    #[world(groups = ["actor"])]
    pub enemies: StructOf<Dense<Enemy>>,
    #[world(groups = [])]
    pub particles: StructOf<Dense<Particle>>,
}

// macro_rules! query_all {
//     ($world:expr, $args:tt) => {
//         query!([$world.players, $world.enemies, $world.particles], $args)
//     };
// }

// macro_rules! actors {
//     ($world:expr) => {
//         [$world.players, $world.enemies]
//     };
// }

#[derive(SplitFields)]
pub struct Position {
    pub x: f32,
}

#[derive(SplitFields)]
pub struct Particle {
    #[split(nested)]
    pub position: Position,
    pub size: f32,
}

#[derive(SplitFields)]
pub struct Actor {}

pub enum EnemyAi {
    Crawler,
}

#[derive(SplitFields)]
pub struct Player {
    #[split(nested)]
    pub position: Position,
    #[split(nested)]
    pub actor: Actor,
}

#[derive(SplitFields)]
pub struct Enemy {
    #[split(nested)]
    pub position: Position,
    #[split(nested)]
    pub actor: Actor,
    pub ai: EnemyAi,
}

fn main() {
    let world = World::default();

    for position in query_all!(world, (&position.x)) {
        println!("entity at position: {}", position);
    }
}
