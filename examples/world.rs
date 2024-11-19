use stecs::prelude::*;

// #[derive(World)]
pub struct World {
    pub players: StructOf<Dense<Player>>,
    pub enemies: StructOf<Dense<Enemy>>,
    pub particles: StructOf<Dense<Particle>>,
}

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

fn main() {}
