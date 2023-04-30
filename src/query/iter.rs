use super::*;

pub struct QueryIter<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> {
    pub(crate) ids: F::IdIter,
    pub(crate) components: &'iter Q::Components<'comp>,
}

pub struct QueryIterMut<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> {
    pub(crate) ids: F::IdIter,
    pub(crate) components: &'iter mut Q::Components<'comp>,
}

impl<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> Iterator
    for QueryIter<'comp, 'iter, Q, F>
{
    type Item = (
        F::Id,
        <Q::Components<'comp> as QueryComponents<F>>::ItemReadOnly<'iter>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.ids.next()?;
            if let Some(item) = self.components.get(id) {
                return Some((id, item));
            }
        }
    }
}

impl<'w, 'a, Q: StructQuery<F>, F: StorageFamily> IntoIterator for &'w Query<'a, Q, F> {
    type Item = <QueryIter<'a, 'w, Q, F> as Iterator>::Item;

    type IntoIter = QueryIter<'a, 'w, Q, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// TODO: impl Iterator
#[allow(clippy::should_implement_trait)]
#[allow(clippy::type_complexity)]
impl<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> QueryIterMut<'comp, 'iter, Q, F> {
    pub fn next(
        &mut self,
    ) -> Option<(
        F::Id,
        <Q::Components<'comp> as QueryComponents<F>>::Item<'_>,
    )> {
        loop {
            let id = self.ids.next()?;
            if self.components.get(id).is_some() {
                // To overcome borrow checker
                // Specifically long lasting mutable borrows from `get_mut`
                let item = self.components.get_mut(id).unwrap();
                return Some((id, item));
            }
        }
    }
}
