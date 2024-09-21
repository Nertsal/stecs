//! # Static compiler-checked ECS*.
//!
//! For an introduction into the idea, see [this blogpost](https://nertsal.github.io/blog/so-i-wrote-my-own-ecs/).
//!
//! This library attempts to bridge the gap between
//! - compile-time guarantees, that are one of the important points of Rust;
//! - performance benefits of [SoA](https://en.wikipedia.org/wiki/AoS_and_SoA) (Struct of Array);
//! - and ease of use of ECS libraries
//!
//! *Note: technically this library likely does not qualify as a proper ECS.
//! What this library actually is, is a generalized SoA derive
//! (For an example of a non-general one, see [soa_derive](https://crates.io/crates/soa_derive) or [soa-rs](https://crates.io/crates/soa-rs/)).
//!
//! # Example
//!
//! See the [GitHub repository](https://github.com/geng-engine/ecs/) for more examples.
//!
//! ```
//! #[derive(SplitFields)]
//! struct Player {
//!     position: f64,
//!     health: Option<i64>,
//! }
//!
//! struct World {
//!     players: StructOf<Vec<Player>>,
//! }
//!
//! let mut world = World { players: Default::default() };
//! world.insert(Player {
//!     position: 1,
//!     health: Some(5),
//! });
//!
//! for (pos, health) in query!(world.players, (&position, &mut health.Get.Some)) {
//!     println!("player at {}; health: {}", position, health);
//!     *health -= 1;
//! }
//! ```
//!
//! # Archetypes
//!
//! Archetypes are entity types stored together for efficient access.
//! Here, the archetypes are static and defined by the user as regular structs with a derive macro.
//!
//! ```
//! #[derive(SplitFields)]
//! struct Monster {
//!     position: (f32, f32),
//!     health: f32,
//!     tick: usize,
//!     damage: Option<f32>,
//! }
//! ```
//!
//! The main thing [`SplitFields`] macro generates is an analogous struct where each field is inside an abstract [`Storage`](storage::Storage) (for example, Vec).
//!
//! ```
//! // Generated struct
//! struct MonsterStructOf<F: StorageFamily> {
//!     position: F::Storage<(f32, f32)>,
//!     health: F::Storage<f32>,
//!     tick: F::Storage<usize>,
//!     damage: F::Storage<Option<f32>>,
//! }
//! ```
//!
//! # Querying
//!
//! Archetypes form the basis of the library and can be used by themselves.
//! Though, accessing the required components manually may be inconvenient, so we offer more macros.
//!
//! Both [`query!`] and [`get!`] have almost identical syntax, and work by providing an archetype (or multiple, for a query), and then providing the target component view.
//!
//! The target view can be either a tuple or a struct (user-defined) with regular instantiation syntax, except for value expressions, which use optics.
//!
//! ```
//! // Get position of a specific entity
//! let pos = get!(world.units, id, (&position)).unwrap();
//!
//! // Querying into a tuple
//! for (pos, vel) in query!(world.units, (id, &mut position, &velocity)) { }
//!
//! // Equivalent query into a struct
//! for view in query!(world.units, TargetView { id, position: &mut position, velocity }) { }
//! ```
//!
//! # Optics
//!
//! For an overview of optics in general, see [this tutorial](https://www.schoolofhaskell.com/school/to-infinity-and-beyond/pick-of-the-week/a-little-lens-starter-tutorial) or [this Rust library](https://crates.io/crates/lens-rs).
//! This library provides only a very limited version of optics applicable to ECS component access.
//!
//! An optic has 3 distinguishable parts: **reference** type, **storage** access, and **component** access (optional).
//! The storage and component parts are separated by a `.Get` indicating access to a specific entity's component inside a storage, but can be omitted when not using the component part.
//!
//! **Note**: there's a special optic `id` (without `&`) that returns the id of the entity being queried.
//!
//! Take, for example, `&mut body.health.Get.Some`.
//!
//! 1. `&mut`. Each optic (except for `id`) must start by describing the **reference** type: either `&` or `&mut`.
//!
//! 2. `body.health`. The **storage** optic provides the path to the component storage.
//!    It is usually a single identifier that is the name of the component.
//!    But it can also be multiple dot-separated identifiers when querying inside a nested storage.
//!
//! 3. `.Some`. The **component** optic describes manipulations on the component value and starts after the `.Get`.
//!    Typically, the component optic is either omitted or used to filter out optional components: `.Some`.
//!
//! In general, there are 3 things you can do in an optic:
//! - access a field, like in normal Rust: `position.x`
//! - get the component from the storage: `.Get`
//! - filter out optional components: `.Some`
//!

/// Derive macro for the static archetypes.
///
/// Generates:
/// - impl [`SplitFields`](crate::archetype::SplitFields)
/// - `XStructOf`, an analogous structure to the one being derived, with fields being general storages (see example below)
/// - `Ref` struct that is used when iterating over the generated archetype
/// - `RefMut` struct that is used when mutably iterating over the generated archetype
///
/// You can annotate the struct with `#[split(debug)]` to derive a [`Debug`](trait@std::fmt::Debug) impl
/// for the `Ref` and `RefMut` structs, and with `#[split(clone)]` to derive [`Clone`](trait@std::clone::Clone).
///
/// Also, you can annotate fields with `#[split(nested)]`, if that field is another archetype, to also split its fields.
///
/// # Example
///
/// ```
/// # use ecs::prelude::*;
/// #[derive(SplitFields)]
/// #[split(debug, clone)]
/// struct Position {
///     x: f64,
///     y: f64,
/// }
///
/// #[derive(SplitFields)]
/// #[split(debug, clone)]
/// struct Projectile {
///     #[split(nested)]
///     position: Position,
///     lifetime: f64,
/// }
/// ```
///
pub use ecs_derive::SplitFields;

/// Get components of a specific entity.
///
/// Syntax is identical to [`query!`], with an additional `id` argument right after the archetype.
///
/// # Example
///
/// ```
/// get!(world.units, id, (&pos, &mut damage.Get.Some))
/// ```
/// ```
/// get!(
///     world.units,
///     id,
///     Target {
///         pos,
///         damage: &mut damage.Get.Some
///     }
/// )
/// ```
///
pub use ecs_derive::storage_get as get;

/// Query components from archetypes.
///
/// The general syntax for a query is: `query!(<archetype>, <view>)`.
///
/// # Example
///
/// ```
/// query!(world.units, (&pos, &mut damage.Get.Some))
///
/// query!(
///     world.units,
///     Target {
///         pos,
///         damage: &mut damage.Get.Some
///     }
/// )
/// ```
///
/// ## Archetypes
/// A single archetype is provided as an expression that evaluates to an [`Archetype`](archetype::Archetype)
/// In typical cases, it is the field in the world with the type of `StructOf<_>`.
///
/// Multiple archetypes can be queried at the same time, if the target views match all archetypes,
/// by listing them in an array: `query!([world.projectiles, world.monsters], <view>)`.
///
/// ## View
/// Queried components can be viewed in either tuple or struct form.
/// In either case, the syntax is the same as normal struct/tuple construction,
/// except for value expressions which use the [optics](crate#optics).
///
pub use ecs_derive::storage_query as query;

/// The traits for describing archetypes and split storages.
pub mod archetype;
/// The [`Storage`](storage::Storage) trait and basic implementors.
pub mod storage;

/// use `ecs::prelude::*;` to import all necessary traits, types, and macros.
pub mod prelude {
    pub use crate::{
        archetype::{Archetype, SplitFields, StructOf, StructOfAble as _},
        get, query,
        storage::{Storage, StorageFamily},
        SplitFields,
    };
}
