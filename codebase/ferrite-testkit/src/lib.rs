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
