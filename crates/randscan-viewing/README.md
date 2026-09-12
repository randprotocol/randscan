# randscan-viewing

The open side of the shielded chain's disclosure layer, compiled to WebAssembly so a viewing key
or a per-transaction key pasted into randscan.org is used in the visitor's browser and never
sent anywhere.

It re-implements, byte for byte, what the fullnode's vendored zkVM does when a wallet opens an
envelope (`crates/shrugg-zkvm/src/{hash,notes,viewing,call_envelope}.rs` at commit `01dc23d`):
the Poseidon2 sponge with the node's seeded round constants, `nk -> pk / ovk / ML-KEM seed`,
the note layout and commitment, ChaCha20-Poly1305 with the commitment as associated data, and
the three openings (receiver through ML-KEM-768, sender through `ovk`, or the transaction key),
plus a call's sealed input transcript checked against the receipt's `H_IN`. No sealing, no
randomness, no ledger. `tests/vectors.json` was produced by the fullnode crate itself; the
tests open it.

Built outside the workspace because the Plonky3 crates need Rust 1.98 (`rust-toolchain.toml`):

```bash
cd crates/randscan-viewing
cargo test
wasm-pack build --release --target web --out-dir ../../frontend/public/viewing --out-name randscan_viewing
```

The artifacts under `frontend/public/viewing/` are committed; the frontend loads them at runtime
(`src/lib/viewing.ts`). Re-run the build (and refresh the vectors) whenever the fullnode changes
the envelope format, the note layout or the hash.
