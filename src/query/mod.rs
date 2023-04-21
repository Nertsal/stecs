use crate::storage::StorageFamily;

mod iter;
mod macros;

pub use iter::*;

pub trait StructQuery<F: StorageFamily> {
    /// Reference to the storages being queried.
    type Components<'a>: QueryComponents<F>;

    fn query(components: Self::Components<'_>) -> Query<'_, Self, F>
    where
        Self: Sized,
    {
        Query { components }
    }
}

pub trait QueryComponents<F: StorageFamily> {
    type Item<'a>
    where
        Self: 'a;

    fn ids(&self) -> F::IdIter;
    fn get(&self, id: F::Id) -> Option<Self::Item<'_>>;
}

pub struct Query<'a, Q: StructQuery<F>, F: StorageFamily> {
    components: Q::Components<'a>,
}

impl<'a, Q: StructQuery<F>, F: StorageFamily> Query<'a, Q, F> {
    pub fn get(&self, id: F::Id) -> Option<<Q::Components<'a> as QueryComponents<F>>::Item<'_>> {
        self.components.get(id)
    }

    pub fn iter(&self) -> QueryIter<'a, '_, Q, F>
    where
        Self: Sized,
    {
        QueryIter {
            ids: self.components.ids(),
            components: &self.components,
        }
    }
}
