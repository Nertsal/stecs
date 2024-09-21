mod iter;

pub use self::iter::*;

use crate::storage::StorageFamily;

/// A collection of components bundled together, or an entity type, or a generic SoA (struct of arrays).
pub trait Archetype<F: StorageFamily>: Default {
    /// The type of the entity stored as components.
    type Item;
    /// Return id's of all active entities.
    fn ids(&self) -> impl Iterator<Item = F::Id>;
    /// Insert a new entity, returning its id.
    fn insert(&mut self, value: Self::Item) -> F::Id;
    /// Remove an entity with a given id.
    fn remove(&mut self, id: F::Id) -> Option<Self::Item>;
}

/// A type synonym for a specific implementor of [Archetype] for convenient usage in type definitions.
///
/// For example, `StructOf<Vec<Unit>>` would turn into `UnitStructOf<VecFamily>`.
pub type StructOf<S> =
    <<S as StructOfAble>::Struct as SplitFields<<S as StructOfAble>::Family>>::StructOf;

/// Implemented for "T's of structs" to convert into "structs of T's" (e.g. AoS to SoA).
pub trait StructOfAble {
    /// The structure (static archetype) which should be split into components.
    type Struct: SplitFields<Self::Family>;
    /// The storage family used to store the components.
    type Family: StorageFamily;
}

/// Implemented for structs (static archetypes) to split into components.
pub trait SplitFields<F: StorageFamily>: Sized {
    /// The [Archetype] for the structure.
    type StructOf: Archetype<F>;
}

/// The trait describing what types act as a borrowed and mutably borrowed versions.
pub trait StructRef {
    /// Type that holds immutable references to the fields.
    type Ref<'a>;
    /// Type that holds mutable references to the fields.
    type RefMut<'a>;
}
