// World
// - resources
// - archetypes (i.e. entity types): collections of components bundled together
//   specify the storage type for components (e.g. `Vec` or `Collection`)
//   - component storages
//     - component values (the actual data)

pub use ecs_derive::{query_components, StructOf, StructQuery};

pub mod archetype;
pub mod query;
pub mod storage;

pub mod prelude {
    pub use crate::{archetype::*, query::*, storage::*, StructOf, StructQuery};
}
