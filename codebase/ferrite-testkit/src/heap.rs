//--------------------------------------------------------------------
// MODULE:        ferrite-testkit/src/heap.rs
// PURPOSE:       DETERMINISTIC ALLOC DETECTOR FOR TEST ISOLATION.
//                TestAllocator -- GLOBAL ALLOCATOR WRAPPER.
//                execute_on_ferrite_core -- VERIFY ZERO ALLOCATIONS.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   NOT REENTRANT. MULTITHREADING REQUIRES ISOLATED PROCESS.
//--------------------------------------------------------------------

// IMPLEMENTATION: Sprint P3B.
