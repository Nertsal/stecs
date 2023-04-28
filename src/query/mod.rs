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
    type ItemReadOnly<'a>
    where
        Self: 'a;

    fn ids(&self) -> F::IdIter;
    fn get(&self, id: F::Id) -> Option<Self::ItemReadOnly<'_>>;
    fn get_mut(&mut self, id: F::Id) -> Option<Self::Item<'_>>;
}

pub struct Query<'a, Q: StructQuery<F>, F: StorageFamily> {
    components: Q::Components<'a>,
}

impl<'a, Q: StructQuery<F>, F: StorageFamily> Query<'a, Q, F> {
    pub fn get(
        &self,
        id: F::Id,
    ) -> Option<<Q::Components<'a> as QueryComponents<F>>::ItemReadOnly<'_>> {
        self.components.get(id)
    }

    pub fn get_mut(
        &mut self,
        id: F::Id,
    ) -> Option<<Q::Components<'a> as QueryComponents<F>>::Item<'_>> {
        self.components.get_mut(id)
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

    pub fn iter_mut(&mut self) -> QueryIterMut<'a, '_, Q, F>
    where
        Self: Sized,
    {
        QueryIterMut {
            ids: self.components.ids(),
            components: &mut self.components,
        }
    }

    pub fn values(
        &self,
    ) -> impl Iterator<Item = <Q::Components<'a> as QueryComponents<F>>::ItemReadOnly<'_>> {
        self.iter().map(|(_, v)| v)
    }
}
