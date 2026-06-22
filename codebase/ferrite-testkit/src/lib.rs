//--------------------------------------------------------------------
// MODULE:        ferrite-testkit/src/lib.rs
// PURPOSE:       TESTKIT ROOT. TWO MODULES: heap, strategy.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   STD REQUIRED. NOT FOR PRODUCTION BINARIES.
//--------------------------------------------------------------------

// HEAP — DETERMINISTIC ALLOC DETECTOR FOR TEST ISOLATION.
pub mod heap;

// STRATEGY — PROPTEST SEU STRATEGIES.
pub mod strategy;

// RE-EXPORT KEY SYMBOLS.
pub use heap::{
    execute_on_ferrite_core, execute_on_ferrite_core_with, set_global_allocator_ref,
    TestAllocator,
};
pub use strategy::{assert_seu_detected, bit_index, byte_offset, finite_f32, finite_f64, inject_burst_error};
