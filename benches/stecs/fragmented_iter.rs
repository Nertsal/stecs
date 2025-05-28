#![allow(non_snake_case)]

use slotmap::SlotMap;
use stecs::prelude::*;

struct Data(f32);

macro_rules! unit {
    ($(($variant:ident, $field:ident)),*) => {
        $(
            #[allow(dead_code)]
            struct $variant(f32);
        )*

        #[derive(SplitFields)]
        struct Unit {
            $(
                $field: Option<$variant>,
            )*
            data: Data,
        }

        impl Default for Unit {
            fn default() -> Self {
                Self {
                    $(
                        $field: None,
                    )*
                    data: Data(1.0),
                }
            }
        }
    };
}

macro_rules! create_entities {
    ($world:ident; $(($variant:ident, $field:ident)),*) => {
        $(
            for _ in 0..crate::N_ENTITIES_FRAG {
                $world.units.insert(Unit {
                    $field: Some($variant(0.0)),
                    ..Default::default()
                });
            }
        )*
    };
}

unit!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j),
    (K, k),
    (L, l),
    (M, m),
    (N, n),
    (O, o),
    (P, p),
    (Q, q),
    (R, r),
    (S, s),
    (T, t),
    (U, u),
    (V, v),
    (W, w),
    (X, x),
    (Y, y),
    (Z, z)
);

struct World {
    units: StructOf<SlotMap<slotmap::DefaultKey, Unit>>,
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World {
            units: Default::default(),
        };

        create_entities!(world;
            (A, a),
            (B, b),
            (C, c),
            (D, d),
            (E, e),
            (F, f),
            (G, g),
            (H, h),
            (I, i),
            (J, j),
            (K, k),
            (L, l),
            (M, m),
            (N, n),
            (O, o),
            (P, p),
            (Q, q),
            (R, r),
            (S, s),
            (T, t),
            (U, u),
            (V, v),
            (W, w),
            (X, x),
            (Y, y),
            (Z, z)
        );

        Self(world)
    }

    pub fn run(&mut self) {
        for data in query!(self.0.units, (&mut data)) {
            data.0 *= 2.0;
        }
    }
}
