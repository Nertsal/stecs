use crate::storage::StorageFamily;

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

pub struct QueryIter<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> {
    ids: F::IdIter,
    components: &'iter Q::Components<'comp>,
}

// -- Query impl --

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

impl<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> Iterator
    for QueryIter<'comp, 'iter, Q, F>
{
    type Item = <Q::Components<'comp> as QueryComponents<F>>::Item<'iter>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.ids.next()?;
        self.components.get(id)
    }
}

// -- Macro --

#[macro_export]
macro_rules! query_components {
    ($structof: expr, $components: ident, ($($fields: tt),*), {$($extra_fields: ident: $extra_values: expr),*}) => {{
        $components {
            $($fields: &$structof.inner.$fields),*,
            $($extra_fields: $extra_values),*
        }
    }};
}
