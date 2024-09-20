// World
// - resources
// - archetypes (i.e. entity types): collections of components bundled together
//   specify the storage type for components (e.g. `Vec` or `Collection`)
//   - component storages
//     - component values (the actual data)

pub use ecs_derive::{storage_get as get, storage_query as query};

pub mod archetype;
pub mod storage;

pub mod prelude {
    pub use crate::{
        archetype::{Archetype, SplitFields, StructOf, StructOfAble},
        get, query,
        storage::{Storage, StorageFamily},
    };
    pub use ecs_derive::SplitFields;
}
