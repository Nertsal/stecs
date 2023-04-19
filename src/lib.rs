// World
// - resources
// - archetypes (i.e. entity types): collections of components bundled together
//   specify the storage type for components (e.g. `Vec` or `Collection`)
//   - component storages
//     - component values (the actual data)

// -- Macros --

// Adapted from [soa_derive](https://github.com/lumol-org/soa-derive)
#[macro_export]
macro_rules! query {
    // Query a struct that implements `StructQuery`.
    // ($structof: expr, $query: ty) => {{
    //     <$query>::query(&$structof)
    // }};

    // Query a tuple by field names.
    ($structof: expr, ($($fields: tt)*) $(, $external: expr)* $(,)*) => {{
        $crate::query_impl!(@munch $structof.inner, {$($fields)*} -> [] $($external ,)*)
    }};
}

// Adapted from [soa_derive](https://github.com/lumol-org/soa-derive)
#[macro_export]
#[doc(hidden)]
macro_rules! query_impl {
    // @flatten creates a tuple-flattening closure for .map() call
    // Finish recursion
    (@flatten $p:pat => $tup:expr ) => {
        |$p| $tup
    };
    // Eat an element ($_iter) and add it to the current closure. Then recurse
    (@flatten $p:pat => ( $($tup:tt)* ) , $_iter:expr $( , $tail:expr )* ) => {
        $crate::query_impl!(@flatten ($p, a) => ( $($tup)*, a ) $( , $tail )*)
    };

    // The main code is emmited here: we create an iterator, zip it and then
    // map the zipped iterator to flatten it
    (@last , $first: expr, $($tail: expr,)*) => {
        ::std::iter::IntoIterator::into_iter($first)
            $(
                .zip($tail)
            )*
            .map(
                $crate::query_impl!(@flatten a => (a) $( , $tail )*)
            )
    };

    // Eat the last `mut $field` and then emit code
    (@munch $self: expr, {mut $field: ident} -> [$($output: tt)*] $($ext: expr ,)*) => {
        $crate::query_impl!(@last $($output)*, $self.$field.iter_mut(), $($ext, )*)
    };
    // Eat the last `$field` and then emit code
    (@munch $self: expr, {$field: ident} -> [$($output: tt)*] $($ext: expr ,)*) => {
        $crate::query_impl!(@last $($output)*, $self.$field.iter(), $($ext, )*)
    };

    // Eat the next `mut $field` and then recurse
    (@munch $self: expr, {mut $field: ident, $($tail: tt)*} -> [$($output: tt)*] $($ext: expr ,)*) => {
        $crate::query_impl!(@munch $self, {$($tail)*} -> [$($output)*, $self.$field.iter_mut()] $($ext, )*)
    };
    // Eat the next `$field` and then recurse
    (@munch $self: expr, {$field: ident, $($tail: tt)*} -> [$($output: tt)*] $($ext: expr ,)*) => {
        $crate::query_impl!(@munch $self, {$($tail)*} -> [$($output)*, $self.$field.iter()] $($ext, )*)
    };
}

mod archetype;
mod query;
mod storage;

pub mod prelude {
    pub use crate::{archetype::*, query, query::*, storage::*};
}
