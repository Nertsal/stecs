#![allow(non_snake_case)]

use super::*;

#[component]
pub struct Data(f32);

#[component]
pub struct A(Option<f32>);
#[component]
pub struct B(Option<f32>);
#[component]
pub struct C(Option<f32>);
#[component]
pub struct D(Option<f32>);
#[component]
pub struct E(Option<f32>);
#[component]
pub struct F(Option<f32>);
#[component]
pub struct G(Option<f32>);
#[component]
pub struct H(Option<f32>);
#[component]
pub struct I(Option<f32>);
#[component]
pub struct J(Option<f32>);
#[component]
pub struct K(Option<f32>);
#[component]
pub struct L(Option<f32>);
#[component]
pub struct M(Option<f32>);
#[component]
pub struct N(Option<f32>);
#[component]
pub struct O(Option<f32>);
#[component]
pub struct P(Option<f32>);
#[component]
pub struct Q(Option<f32>);
#[component]
pub struct R(Option<f32>);
#[component]
pub struct S(Option<f32>);
#[component]
pub struct T(Option<f32>);
#[component]
pub struct U(Option<f32>);
#[component]
pub struct V(Option<f32>);
#[component]
pub struct W(Option<f32>);
#[component]
pub struct X(Option<f32>);
#[component]
pub struct Y(Option<f32>);
#[component]
pub struct Z(Option<f32>);

#[entity]
pub struct UnitFrag {
    pub data: Data,
    pub A: A,
    pub B: B,
    pub C: C,
    pub D: D,
    pub E: E,
    pub F: F,
    pub G: G,
    pub H: H,
    pub I: I,
    pub J: J,
    pub K: K,
    pub L: L,
    pub M: M,
    pub N: N,
    pub O: O,
    pub P: P,
    pub Q: Q,
    pub R: R,
    pub S: S,
    pub T: T,
    pub U: U,
    pub V: V,
    pub W: W,
    pub X: X,
    pub Y: Y,
    pub Z: Z,
}

macro_rules! create_entities {
    ($world:ident; $($variants:ident),*) => {
        $(
            for _ in 0..crate::N_ENTITIES_FRAG {
                $world.create(UnitFrag {
                    data: Data(1.0),
                    $variants: $variants(Some(0.0)),
                    ..Default::default()
                });
            }
        )*
    };
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();

        create_entities!(world;
            A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
        );

        Self(world)
    }

    pub fn run(&mut self) {
        run_frag_iter(&mut self.0, Query::new());
    }
}

#[system]
pub fn run_frag_iter(world: &mut World, query: Query<&mut Data>) {
    world.with_query_mut(query).iter_mut().for_each(|data| {
        data.0 *= 2.0;
    });
}
