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

// #[macro_export]
// macro_rules! query {
//     ($structof: expr, $query: ident) => {{
//         let components = $crate::query_components!($structof, )
//     }}
// }

#[macro_export]
macro_rules! query_components {
    ($structof: expr, $components: ident, ($($fields: tt),*)) => {{
        $components {
            $($fields: &$structof.$fields),*
        }
    }};
}

// // Adapted from [soa_derive](https://github.com/lumol-org/soa-derive)
// #[macro_export]
// macro_rules! query {
//     // Query a struct that implements `StructQuery`.
//     // ($structof: expr, $query: ty) => {{
//     //     <$query>::query(&$structof)
//     // }};

//     // Query a tuple by field names.
//     ($structof: expr, ($($fields: tt)*) $(, $external: expr)* $(,)*) => {{
//         $crate::query_impl!(@munch $structof.inner, {$($fields)*} -> [] $($external ,)*)
//     }};
// }

// // Adapted from [soa_derive](https://github.com/lumol-org/soa-derive)
// #[macro_export]
// #[doc(hidden)]
// macro_rules! query_impl {
//     // @flatten creates a tuple-flattening closure for .map() call
//     // Finish recursion
//     (@flatten $p:pat => $tup:expr ) => {
//         |$p| $tup
//     };
//     // Eat an element ($_iter) and add it to the current closure. Then recurse
//     (@flatten $p:pat => ( $($tup:tt)* ) , $_iter:expr $( , $tail:expr )* ) => {
//         $crate::query_impl!(@flatten ($p, a) => ( $($tup)*, a ) $( , $tail )*)
//     };

//     // The main code is emmited here: we create an iterator, zip it and then
//     // map the zipped iterator to flatten it
//     (@last , $first: expr, $($tail: expr,)*) => {
//         ::std::iter::IntoIterator::into_iter($first)
//             $(
//                 .zip($tail)
//             )*
//             .map(
//                 $crate::query_impl!(@flatten a => (a) $( , $tail )*)
//             )
//     };

//     // Eat the last `mut $field` and then emit code
//     (@munch $self: expr, {mut $field: ident} -> [$($output: tt)*] $($ext: expr ,)*) => {
//         $crate::query_impl!(@last $($output)*, $self.$field.iter_mut(), $($ext, )*)
//     };
//     // Eat the last `$field` and then emit code
//     (@munch $self: expr, {$field: ident} -> [$($output: tt)*] $($ext: expr ,)*) => {
//         $crate::query_impl!(@last $($output)*, $self.$field.iter(), $($ext, )*)
//     };

//     // Eat the next `mut $field` and then recurse
//     (@munch $self: expr, {mut $field: ident, $($tail: tt)*} -> [$($output: tt)*] $($ext: expr ,)*) => {
//         $crate::query_impl!(@munch $self, {$($tail)*} -> [$($output)*, $self.$field.iter_mut()] $($ext, )*)
//     };
//     // Eat the next `$field` and then recurse
//     (@munch $self: expr, {$field: ident, $($tail: tt)*} -> [$($output: tt)*] $($ext: expr ,)*) => {
//         $crate::query_impl!(@munch $self, {$($tail)*} -> [$($output)*, $self.$field.iter()] $($ext, )*)
//     };
// }
