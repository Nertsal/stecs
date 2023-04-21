// World
// - resources
// - archetypes (i.e. entity types): collections of components bundled together
//   specify the storage type for components (e.g. `Vec` or `Collection`)
//   - component storages
//     - component values (the actual data)

// -- Macros --

pub use ecs_derive::{StructOf, StructQuery};

mod archetype;
mod query;
mod storage;

pub mod prelude {
    pub use crate::{archetype::*, query::*, query_components, storage::*, StructOf, StructQuery};
}
