use super::*;

struct Data(f32);

macro_rules! create_entities {
    ($world:ident; $($variants:ident),*) => {
        $(
            #[allow(dead_code)]
            struct $variants(f32);

            $world.spawn_batch((0..crate::N_ENTITIES_FRAG).map(|_| {
                (
                    $variants(0.0),
                    Data(1.0),
                )
            }));
        )*
    };
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::new();
        create_entities!(
            world; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
        );
        Self(world)
    }

    pub fn run(&mut self) {
        for (_, data) in self.0.query_mut::<&mut Data>() {
            data.0 *= 2.0;
        }
    }
}
