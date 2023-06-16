use super::*;

type IdIter<T> = std::collections::hash_set::IntoIter<T>;

pub struct QueryIter<'comp: 'iter, 'iter, Q: QueryComponents<'comp>> {
    ids: IdIter<Q::Id>,
    components: &'iter Q,
    phantom_data: std::marker::PhantomData<&'comp Q>,
}

// pub struct QueryIterMut<'comp: 'iter, 'iter, Q: QueryComponents<F>, F: StorageFamily> {
//     pub(crate) ids: HashSet<Id>,
//     pub(crate) components: &'iter mut Q::Components<'comp>,
// }

impl<'comp: 'iter, 'iter, Q: QueryComponents<'comp>> QueryIter<'comp, 'iter, Q> {
    pub(super) fn new(components: &'iter Q) -> Self {
        Self {
            ids: components.ids().into_iter(),
            components,
            phantom_data: Default::default(),
        }
    }
}

impl<'comp: 'iter, 'iter, Q: QueryComponents<'comp>> Iterator for QueryIter<'comp, 'iter, Q> {
    type Item = (Q::Id, Q::Item<'iter>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.ids.next()?;
            if let Some(item) = self.components.get(id) {
                return Some((id, item));
            }
        }
    }
}

// // TODO: impl Iterator
// #[allow(clippy::should_implement_trait)]
// #[allow(clippy::type_complexity)]
// impl<'comp: 'iter, 'iter, Q: QueryComponents<F>, F: StorageFamily>
//     QueryIterMut<'comp, 'iter, Q, F>
// {
//     pub fn next(
//         &mut self,
//     ) -> Option<(
//         F::Id,
//         <Q::Components<'comp> as QueryComponents<F>>::Item<'_>,
//     )> {
//         loop {
//             let id = self.ids.next()?;
//             if self.components.get(id).is_some() {
//                 // To overcome borrow checker
//                 // Specifically long lasting mutable borrows from `get_mut`
//                 let item = self.components.get_mut(id).unwrap();
//                 return Some((id, item));
//             }
//         }
//     }
// }
