use super::*;

struct Data(f32);

macro_rules! create_entities {
    ($world:ident; $($variants:ident),*) => {
        $(
            #[allow(dead_code)]
            struct $variants(f32);

            $world.extend((0..crate::N_ENTITIES_FRAG).map(|_| {
                (
                    $variants(0.0),
                    Data(1.0),
                )
            }));
        )*
    };
}

pub struct Benchmark(World, Query<Write<Data>>);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();

        create_entities!(
            world; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
        );

        let query = <Write<Data>>::query();

        Self(world, query)
    }

    pub fn run(&mut self) {
        self.1.for_each_mut(&mut self.0, |data| {
            data.0 *= 2.0;
        });
    }
}
