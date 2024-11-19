use stecs::prelude::*;

pub struct World {
    pub units: StructOf<Dense<Unit>>,
}

#[derive(Debug)]
pub struct Poisoned {
    pub time: f32,
}

#[derive(SplitFields)]
#[split(debug)]
pub struct Unit {
    pub name: String,
}

#[derive(SplitFields)]
#[split(debug)]
pub struct UnitExtension {
    pub hue: f32,
    pub size: f32,
}

fn main() {
    let mut world = World {
        units: Default::default(),
    };

    let unit1 = world.units.insert(Unit {
        name: String::from("Alfred"),
    });
    let unit2 = world.units.insert(Unit {
        name: String::from("Olivia"),
    });

    // Insert dynamic components
    world.units.insert_dyn(unit1, Poisoned { time: 3.0 });
    world.units.insert_dyn(unit2, Poisoned { time: 2.0 });
    world.units.remove_dyn::<Poisoned>(unit2).unwrap();

    println!("All units:");
    for unit in query!(world.units, (&name, &dyn Poisoned)) {
        println!("unit: {:?}", unit);
    }

    println!("Poisoned units:");
    for (name, poison) in query!(world.units, (&name, &mut dyn Poisoned.Some)) {
        poison.time -= 0.5;
        println!("unit: {:?}", (name, poison));
    }

    // Extend unit1 with extra fields
    world.units.extend_dyn(
        unit1,
        UnitExtension {
            hue: 0.0,
            size: 2.5,
        },
    );

    println!("Extended units:");
    for (name, hue, size) in {
        use ::stecs::storage::Storage;
        #[allow(non_snake_case)]
        {
            fn extract<F: ::stecs::storage::StorageFamily>(
                ext: &mut <UnitExtension as ::stecs::archetype::SplitFields<F>>::Split,
            ) -> (&mut F::Storage<f32>, &F::Storage<f32>) {
                (&mut ext.hue, &ext.size)
            }
            let (__ext_hue, __ext_size) = match world.units.r#dyn.get_ext_mut::<UnitExtension>() {
                Some(__ext) => {
                    let (a, b) = extract(__ext);
                    (Some(a), Some(b))
                }
                None => (None, None),
            };
            let __field0 = ::stecs::storage::IdGenerator::ids(&world.units.ids).map(|__ID| {
                let value = world.units.inner.name.get(__ID);
                value.expect("`id` must be valid")
            });
            let __field1 = unsafe {
                match __ext_hue {
                    Some(__ext_hue) => __ext_hue.get_many_unchecked_mut(
                        ::stecs::storage::IdGenerator::ids(&world.units.ids),
                    ),
                    None => default(&__ext_hue).get_many_unchecked_mut(
                        ::stecs::storage::IdGenerator::ids(&world.units.ids),
                    ),
                }
            };
            let __field2 = {
                ::stecs::storage::IdGenerator::ids(&world.units.ids)
                    .map(|__ID| __ext_size.and_then(|ext| ext.get(__ID)))
            };
            __field0
                .zip(__field1)
                .zip(__field2)
                .filter_map(|((__field0, __field1), __field2)| Some((__field0, __field1, __field2)))
        }
    } {
        println!("unit: {:?}", (name, hue, size));
    }
}

fn default<T: Default>(_: &Option<&mut T>) -> T {
    T::default()
}
