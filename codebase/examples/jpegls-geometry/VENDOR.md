# VENDOR.md: provenance of src/vendor/

## What is vendored

`src/vendor/` contains a BYTE-IDENTICAL copy of the source files of the
crates.io package `pure_jpegls` version 1.0.0:

| file            | upstream path (crate root) |
|-----------------|----------------------------|
| error.rs        | src/error.rs               |
| predictor.rs    | src/predictor.rs           |
| bitstream.rs    | src/bitstream.rs           |
| context.rs      | src/context.rs             |
| run_mode.rs     | src/run_mode.rs            |
| decode.rs       | src/decode.rs              |
| encode.rs       | src/encode.rs              |

Upstream `src/lib.rs` is NOT vendored: this crate wires the modules
itself (see src/lib.rs) so the AP-101 audit gates can reach
crate-private items.

## Why

The audit gates (src/gates.rs) measure cache-line geometry and heap
discipline of crate-private structs (`BitReader`, `Decoder`,
`decode_scan`). The crates.io library does not export them. A vendored
copy is the only way to measure the real declarations without forking.

## This is NOT a fork

- No vendored file is modified except ONE mechanical, documented class
  of change: module-level inner doc comment lines (`//! ...`) were
  rewritten to ordinary comments (`// ...`) in 6 files (bitstream,
  context, decode, encode, predictor, run_mode). Reason: `include!`
  cannot import inner doc comments into a module body (compile error
  E0753). The comment TEXT is unchanged; no code token is touched.
- No `[patch]`, no renamed package, no published crate.
- The vendored code exists solely to be AUDITED and to run upstream's
  own roundtrip tests inside `cargo test -p jpegls-geometry`.

## License

Upstream license (from Cargo.toml): MIT OR Apache-2.0.
Upstream author: JP Fielding <jp.fielding@gmail.com>.
Upstream repository: https://github.com/jpfielding/dicos.rs
Upstream manifest + README copies: vendor_upstream_cargo_toml.orig,
vendor_upstream_readme.md (kept for audit trail).

## Lint policy

Upstream code keeps its own style. Lint silence at the crate level
(src/lib.rs inner attributes) is deliberate; ap101's own gates are
lint-clean by review.
