# New and Unnamed ECS\*

This is an experimental ECS library intended to be more compiler-friendly. Archetypes are static, and queries are checked at compile-time. For an introduction into the idea, see [this blogpost](https://nertsal.github.io/blog/so-i-wrote-my-own-ecs/).

This library attempts to bridge the gap between
- compile-time guarantees, that are one of the important points of Rust;
- performance benefits of [SoA](https://en.wikipedia.org/wiki/AoS_and_SoA) (Struct of Array);
- and ease of use of ECS libraries

*Note: technically this library likely does not qualify as a proper ECS.
What this library actually is, is a generalized SoA derive
(For an example of a non-general one, see [soa_derive](https://crates.io/crates/soa_derive) or [soa-rs](https://crates.io/crates/soa-rs/)).

See [crate documentation](todo) for more information.

# Example

[See more examples here](examples/).

```rust
#[derive(SplitFields)]
struct Player {
    position: f64,
    health: Option<i64>,
}

struct World {
    players: StructOf<Vec<Player>>,
}

let mut world = World { players: Default::default() };
world.insert(Player {
    position: 1,
    health: Some(5),
});

for (pos, health) in query!(world.players, (&position, &mut health.Get.Some)) {
    println!("player at {}; health: {}", position, health);
    *health -= 1;
}
```

# Similar projects

Static ECS:
- [ecstatic](https://crates.io/crates/ecstatic)
- [gecs](https://crates.io/crates/gecs)

Similar to some parts of this library:
- [soa-rs](https://crates.io/crates/soa-rs/)
- [soa_derive](https://crates.io/crates/soa_derive)
- [soa-vec](https://crates.io/crates/soa-vec)
