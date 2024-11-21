mod game;
mod logic;
mod render;

#[macroquad::main("Breakout Example")]
async fn main() {
    game::World::new().run().await
}
