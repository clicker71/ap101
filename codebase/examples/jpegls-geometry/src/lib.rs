//--------------------------------------------------------------------
// MODULE:        jpegls-geometry/src/lib.rs
// PURPOSE:       Wire the VERBATIM pure_jpegls 1.0.0 sources (src/vendor/)
//                into this crate as path-attributed modules so the AP-101
//                audit gates can reach crate-private structs (BitReader,
//                ContextModel, decode_scan) WITHOUT editing upstream code.
//                THE VENDORED FILES ARE BYTE-IDENTICAL TO THE CRATES.IO
//                RELEASE (provenance in VENDOR.md). THIS IS NOT A FORK.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-08-27
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   Upstream files untouched. All ap101 code lives in
//                gates.rs; nothing here modifies vendored semantics.
//--------------------------------------------------------------------

#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    unused_assignments
)]
// VENDORED UPSTREAM CODE (src/vendor/) KEEPS ITS OWN STYLE: ap101 LINT
// SILENCE HERE IS DELIBERATE AND DOCUMENTED (VENDOR.md). OUR OWN GATES
// IN gates.rs ARE LINT-CLEAN BY REVIEW.

// Test builds install the AP-101 deterministic allocator so the G2/G3
// heap gates observe every allocation of the measuring thread.
#[cfg(test)]
#[global_allocator]
static ALLOCATOR: ferrite_testkit::TestAllocator = ferrite_testkit::TestAllocator::new();

#[path = "vendor/bitstream.rs"]
mod bitstream;
#[path = "vendor/context.rs"]
mod context;
#[path = "vendor/decode.rs"]
mod decode;
#[path = "vendor/encode.rs"]
mod encode;
#[path = "vendor/error.rs"]
mod error;
#[path = "vendor/predictor.rs"]
mod predictor;
#[path = "vendor/run_mode.rs"]
mod run_mode;

// THE AUDIT GATES. Unit tests live here: they can see crate-private
// vendor items, which integration tests (tests/) could not.
#[cfg(test)]
pub mod gates;
