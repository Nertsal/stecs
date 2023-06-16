mod iter;

pub use iter::*;

use crate::storage::{Storage, StorageFamily};

use std::collections::HashSet;

pub trait QueryComponents<'comp> {
    type Id: Copy;
    type Item<'a>
    where
        'comp: 'a,
        Self: 'a;

    fn ids(&self) -> HashSet<Self::Id>;
    fn get<'a>(&'a self, id: Self::Id) -> Option<Self::Item<'a>>
    where
        'comp: 'a;
}

pub struct Query<'a, Q: QueryComponents<'a>> {
    components: Q,
    phantom_data: std::marker::PhantomData<&'a Q>,
}

impl<'a, Q: QueryComponents<'a>> Query<'a, Q> {
    /// Supposed to be used by the `query!` macro.
    pub fn new(components: Q) -> Self {
        Self {
            components,
            phantom_data: Default::default(),
        }
    }

    pub fn get(&self, id: Q::Id) -> Option<Q::Item<'_>> {
        self.components.get(id)
    }

    // pub fn get_mut(&mut self, id: F::Id) -> Option<Q::Item<'_>> {
    //     self.components.get_mut(id)
    // }

    pub fn iter(&self) -> QueryIter<'a, '_, Q>
    where
        Self: Sized,
    {
        QueryIter::new(&self.components)
    }

    // pub fn iter_mut(&mut self) -> QueryIterMut<'a, '_, Q, F>
    // where
    //     Self: Sized,
    // {
    //     QueryIterMut {
    //         ids: self.ids.clone(),
    //         components: &mut self.components,
    //     }
    // }

    // pub fn values(&self) -> impl Iterator<Item = Q::ItemReadOnly<'_>> {
    //     self.iter().map(|(_, v)| v)
    // }
}

// impl<'w, 'a, Q: QueryComponents<'a, F>, F: StorageFamily> IntoIterator for &'w Query<'a, Q, F> {
//     type Item = <QueryIter<'a, 'w, Q, F> as Iterator>::Item;

//     type IntoIter = QueryIter<'a, 'w, Q, F>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

impl<'comp, S: Storage<T>, T> QueryComponents<'comp> for (&'comp S, std::marker::PhantomData<T>) {
    type Id = S::Id;
    type Item<'a> = &'a T
    where
        'comp: 'a,
        Self: 'a;

    fn ids(&self) -> HashSet<Self::Id> {
        crate::storage::Storage::ids(self.0)
    }

    fn get<'a>(&'a self, id: Self::Id) -> Option<Self::Item<'a>>
    where
        'comp: 'a,
    {
        let t = crate::storage::Storage::get(self.0, id)?;
        Some(t)
    }
}

// impl<'comp, F: StorageFamily, T0, T1> QueryComponents<'comp, F>
//     for (&'comp F::Storage<T0>, &'comp F::Storage<T1>)
// {
//     type Item<'a> = (&'a T0, &'a T1)
//     where
//         'comp: 'a,
//         Self: 'a;

//     fn ids(&self) -> HashSet<<F as StorageFamily>::Id> {
//         crate::prelude::Storage::ids(self.0)
//     }

//     fn get<'a>(&'a self, id: <F as StorageFamily>::Id) -> Option<Self::Item<'a>>
//     where
//         'comp: 'a,
//     {
//         let t0 = crate::prelude::Storage::get(self.0, id)?;
//         let t1 = crate::prelude::Storage::get(self.1, id)?;
//         Some((t0, t1))
//     }
// }
