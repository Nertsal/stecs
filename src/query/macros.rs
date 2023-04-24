#[macro_export]
macro_rules! query_components {
    ($structof: expr, $components: ident, ($($fields: tt)*)) => {
        $crate::query_components_impl!($structof.inner, $components, {$($fields)*} -> {})
    };
    ($structof: expr, $components: ident, ($($fields: tt)*), {$($extra_fields: ident: $extra_values: expr),* $(,)*}) => {
        $crate::query_components_impl!($structof.inner, $components, {$($fields)*} -> {$($extra_fields: $extra_values),*})
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! query_components_impl {
    // Return the collected fields
    ($structof: expr, $components: ident, {} -> {$($fields: ident: $values: expr),* $(,)*}) => {
        $components {
            $($fields: $values),*
        }
    };
    // Get the last `mut $field`
    ($structof: expr, $components: ident, {mut $field: ident} -> {$($tail_fields: ident: $tail_values: expr),* $(,)*}) => {
        $crate::query_components_impl!($structof, $components, {} -> {
            $field: &mut $structof.$field,
            $($tail_fields: $tail_values),*
        })
    };
    // Get the last `$field`
    ($structof: expr, $components: ident, {$field: ident} -> {$($tail_fields: ident: $tail_values: expr),* $(,)*}) => {
        $crate::query_components_impl!($structof, $components, {} -> {
            $field: &$structof.$field,
            $($tail_fields: $tail_values),*
        })
    };
    // Get the next `mut $field`
    ($structof: expr, $components: ident, {mut $field: ident, $($fields: tt)*} -> {$($tail_fields: ident: $tail_values: expr),* $(,)*}) => {
        $crate::query_components_impl!($structof, $components, {$($fields)+} -> {
            $field: &mut $structof.$field,
            $($tail_fields: $tail_values),*
        })
    };
    // Get the next `$field`
    ($structof: expr, $components: ident, {$field: ident, $($fields: tt)*} -> {$($tail_fields: ident: $tail_values: expr),* $(,)*}) => {
        $crate::query_components_impl!($structof, $components, {$($fields)+} -> {
            $field: &$structof.$field,
            $($tail_fields: $tail_values),*
        })
    };
}
