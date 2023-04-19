use crate::{
    query::IdHolder,
    storage::{Storage, StorageFamily},
};

/// Type alias for the crazy access to the Id of items in the [Storage].
pub type ArchetypeId<A> = <<<A as Archetype>::Family as StorageFamily>::Storage<
    <A as Archetype>::Item,
> as Storage<<A as Archetype>::Item>>::Id;

/// A collection of components bundled together, or an entity type.
pub trait Archetype: IdHolder + Default {
    /// The type of the entity stored as components.
    type Item;
    /// The storage family used to store the components.
    type Family: StorageFamily;

    fn insert(&mut self, value: Self::Item) -> ArchetypeId<Self>;
    // fn get()
    // fn get_mut()
    fn remove(&mut self, id: ArchetypeId<Self>) -> Option<Self::Item>;
}

/// A wrapper around an [Archetype] for convenient usage in type definitions.
///
/// For example, `StructOf<Vec<Unit>>` would use `UnitStructOf<VecFamily>` underneath.
pub struct StructOf<S: StructOfAble> {
    pub inner: <S::Struct as SplitFields>::StructOf<S::Family>,
}

/// Implemented for "T's of structs" to convert into "structs of T's".
pub trait StructOfAble {
    /// The structure which should be split into components.
    type Struct: SplitFields;
    /// The storage family used to store the components.
    type Family: StorageFamily;
}

/// Implemented for structs to split into components.
pub trait SplitFields: Sized {
    /// The [Archetype] for the structure.
    type StructOf<F: StorageFamily>: Archetype;
}

impl<S: StructOfAble> std::ops::Deref for StructOf<S> {
    type Target = <S::Struct as SplitFields>::StructOf<S::Family>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: StructOfAble> std::ops::DerefMut for StructOf<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S: StructOfAble> Default for StructOf<S> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}
