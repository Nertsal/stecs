use crate::storage::StorageFamily;

use std::collections::BTreeSet;

/// A collection of components bundled together, or an entity type.
pub trait Archetype<F: StorageFamily>: Default {
    /// The type of the entity stored as components.
    type Item;
    fn ids(&self) -> BTreeSet<F::Id>;
    fn insert(&mut self, value: Self::Item) -> F::Id;
    // fn get()
    // fn get_mut()
    fn remove(&mut self, id: F::Id) -> Option<Self::Item>;
}

/// A type synonym for an [Archetype] for convenient usage in type definitions.
///
/// For example, `StructOf<Vec<Unit>>` would turn into `UnitStructOf<VecFamily>`.
// pub type StructOf<S: StructOfAble> = <S::Struct as SplitFields<S::Family>>::StructOf;
pub type StructOf<S> =
    <<S as StructOfAble>::Struct as SplitFields<<S as StructOfAble>::Family>>::StructOf;

/// Implemented for "T's of structs" to convert into "structs of T's".
pub trait StructOfAble {
    /// The structure which should be split into components.
    type Struct: SplitFields<Self::Family>;
    /// The storage family used to store the components.
    type Family: StorageFamily;
}

/// Implemented for structs to split into components.
pub trait SplitFields<F: StorageFamily>: Sized {
    /// The [Archetype] for the structure.
    type StructOf: Archetype<F>;
}

pub trait StructRef {
    /// Type that holds immutable references to the fields.
    type Ref<'a>;
    /// Type that holds mutable references to the fields.
    type RefMut<'a>;
}

// impl<S: StructOfAble> StructOf<S> {
//     pub fn new() -> Self {
//         Self::default()
//     }
// }

// impl<S: StructOfAble> Archetype<S::Family> for StructOf<S> {
//     type Item = <<S::Struct as SplitFields<S::Family>>::StructOf as Archetype<S::Family>>::Item;
//     fn ids(&self) -> BTreeSet<<S::Family as StorageFamily>::Id> {
//         self.inner.ids()
//     }
//     fn insert(&mut self, value: Self::Item) -> <S::Family as StorageFamily>::Id {
//         self.inner.insert(value)
//     }
//     fn remove(&mut self, id: <S::Family as StorageFamily>::Id) -> Option<Self::Item> {
//         self.inner.remove(id)
//     }
// }

// impl<S: StructOfAble> std::ops::Deref for StructOf<S> {
//     type Target = <S::Struct as SplitFields<S::Family>>::StructOf;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl<S: StructOfAble> std::ops::DerefMut for StructOf<S> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

// impl<S: StructOfAble> Default for StructOf<S> {
//     fn default() -> Self {
//         Self {
//             inner: Default::default(),
//         }
//     }
// }

// impl<S: StructOfAble> Clone for StructOf<S>
// where
//     <S::Struct as SplitFields<S::Family>>::StructOf: Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//             inner: self.inner.clone(),
//         }
//     }
// }
