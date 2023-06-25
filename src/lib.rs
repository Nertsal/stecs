// World
// - resources
// - archetypes (i.e. entity types): collections of components bundled together
//   specify the storage type for components (e.g. `Vec` or `Collection`)
//   - component storages
//     - component values (the actual data)

pub use ecs_derive::query_components;

pub mod archetype;
#[cfg(feature = "arena")]
pub mod arena;
#[cfg(feature = "hashstorage")]
pub mod hashstorage;
pub mod storage;

pub mod prelude {
    pub use crate::{
        archetype::{Archetype, SplitFields, StructOf, StructOfAble},
        storage::{Storage, StorageFamily},
    };
    pub use ecs_derive::{storage_get as get, storage_query as query, SplitFields};
}
