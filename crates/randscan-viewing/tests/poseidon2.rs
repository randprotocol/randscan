//! Audit ZKV-2 / scan R2: the Poseidon2 round constants are the committed table
//! (`randscan-core/src/poseidon2_constants.rs`, compiled in by `#[path]`), never a seeded RNG
//! draw at runtime — `rand` does not promise its standard generator is stable across releases.

#[test]
fn the_crate_draws_no_constants_at_runtime() {
    let src = include_str!("../src/lib.rs");
    for pattern in [concat!("new_from", "_rng"), concat!("Std", "Rng"), concat!("seed_from", "_u64")] {
        assert!(!src.contains(pattern), "lib.rs still derives constants: {pattern}");
    }
}
