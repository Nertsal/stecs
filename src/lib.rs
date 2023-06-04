// World
// - resources
// - archetypes (i.e. entity types): collections of components bundled together
//   specify the storage type for components (e.g. `Vec` or `Collection`)
//   - component storages
//     - component values (the actual data)

pub use ecs_derive::{query_components, StructOf, StructQuery};

mod archetype;
#[cfg(feature = "arena")]
pub mod arena;
#[cfg(feature = "hashstorage")]
pub mod hashstorage;
mod query;
mod storage;

pub use archetype::*;
pub use query::*;
pub use storage::*;

pub mod prelude {
    pub use crate::{archetype::*, query::*, storage::*, StructOf, StructQuery};
}
