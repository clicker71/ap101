//--------------------------------------------------------------------
// MODULE:        ferrite-testkit/src/heap.rs
// PURPOSE:       DETERMINISTIC ALLOC DETECTOR FOR TEST ISOLATION.
//                TestAllocator -- GLOBAL ALLOCATOR WRAPPER.
//                execute_on_ferrite_core -- VERIFY ZERO ALLOCATIONS.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   PER-THREAD COUNTERS. FOREIGN THREADS DO NOT POLLUTE.
//--------------------------------------------------------------------

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

// PER-THREAD ALLOCATION COUNTERS.
//
// WHY THREAD-LOCAL: THE PREVIOUS GLOBAL `AtomicUsize` COUNTERS WERE
// POLLUTED BY ALLOCATIONS FROM OTHER THREADS IN THE SAME PROCESS
// (LIBTEST HARNESS OUTPUT/CAPTURE THREADS), CAUSING RARE FLAKY
// FAILURES IN SINGLE-THREADED ZERO-ALLOC TESTS. A MEASUREMENT WINDOW
// NOW OBSERVES ONLY ALLOCATIONS PERFORMED ON THE MEASURING THREAD.
thread_local! {
    static COUNTERS: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
    static ENABLED: Cell<bool> = const { Cell::new(true) };
}

/// GLOBAL ALLOCATOR FOR TEST ISOLATION.
///
/// COUNTERS ARE **PER-THREAD**. FOREIGN THREADS NO LONGER POLLUTE
/// THE SNAPSHOT COMPARISON.
///
/// ## TRADEOFF
///
/// ALLOCATIONS PERFORMED BY A WORKER THREAD SPAWNED INSIDE `f` ARE
/// NOT ATTRIBUTED TO THE MEASURING THREAD. THE ZERO-ALLOC GATE COVERS
/// SINGLE-THREADED HOTPATHS; MULTITHREADED CODE STILL REQUIRES AN
/// ISOLATED PROCESS (API CONTRACT, UNCHANGED).
///
/// ## USAGE
///
/// ```ignore
/// #[global_allocator]
/// static ALLOCATOR: ferrite_testkit::TestAllocator = ferrite_testkit::TestAllocator::new();
/// ```
///
/// THEN CALL `execute_on_ferrite_core` TO VERIFY
/// YOUR CODE PERFORMS ZERO ALLOCATIONS.
pub struct TestAllocator {
    system: System,
}

impl TestAllocator {
    pub const fn new() -> Self {
        Self { system: System }
    }

    /// ENABLE/DISABLE TRACKING ON THE CURRENT THREAD.
    pub fn set_enabled(&self, enabled: bool) {
        ENABLED.with(|e| e.set(enabled));
    }

    /// RESET THE CURRENT THREAD'S COUNTERS.
    pub fn reset(&self) {
        COUNTERS.with(|c| c.set((0, 0)));
    }

    /// RETURN THE CURRENT THREAD'S SNAPSHOT.
    pub fn snapshot(&self) -> (usize, usize) {
        COUNTERS.with(|c| c.get())
    }
}

impl Default for TestAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: TestAllocator wraps System allocator. System is safe for global use.
unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ENABLED.with(|e| {
            if e.get() {
                COUNTERS.with(|c| {
                    let (n, b) = c.get();
                    c.set((n.saturating_add(1), b.saturating_add(layout.size())));
                });
            }
        });
        self.system.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SATURATING SUB: A CROSS-THREAD FREE (ALLOCATED ON ANOTHER
        // THREAD) MUST NOT UNDERFLOW THIS THREAD'S BYTE COUNTER.
        ENABLED.with(|e| {
            if e.get() {
                COUNTERS.with(|c| {
                    let (n, b) = c.get();
                    c.set((n, b.saturating_sub(layout.size())));
                });
            }
        });
        self.system.dealloc(ptr, layout)
    }
}

/// GLOBAL ALLOCATOR REFERENCE FOR `execute_on_ferrite_core`.
///
/// SET VIA `set_global_allocator_ref` BEFORE CALLING
/// `execute_on_ferrite_core` WITHOUT EXPLICIT ALLOCATOR ARGUMENT.
static GLOBAL_ALLOCATOR_REF: std::sync::OnceLock<&'static TestAllocator> =
    std::sync::OnceLock::new();

/// REGISTER GLOBAL ALLOCATOR REFERENCE.
///
/// CALL ONCE AT TEST SETUP.
pub fn set_global_allocator_ref(allocator: &'static TestAllocator) {
    let _ = GLOBAL_ALLOCATOR_REF.set(allocator);
}

/// EXECUTE CLOSURE AND VERIFY ZERO ALLOCATIONS.
///
/// ## IMPORTANT
///
/// NESTING `execute_on_ferrite_core` INSIDE `f` IS DISCOURAGED —
/// THE INNER WINDOW SHADOWS THE OUTER ONE ON THE SAME THREAD.
///
/// FOR MULTITHREADED CODE USE SEPARATE PROCESS WITH ISOLATED ALLOCATOR.
///
/// ## PREREQUISITE
///
/// `set_global_allocator_ref` MUST BE CALLED FIRST.
pub fn execute_on_ferrite_core<F, R>(f: F) -> (bool, R)
where
    F: FnOnce() -> R,
{
    let allocator = GLOBAL_ALLOCATOR_REF
        .get()
        .expect("GLOBAL ALLOCATOR REF NOT SET. CALL set_global_allocator_ref FIRST.");
    execute_on_ferrite_core_with(allocator, f)
}

/// VERSION OF execute_on_ferrite_core WITH EXPLICIT ALLOCATOR REF.
///
/// PREFER THIS OVER `execute_on_ferrite_core` — IT AVOIDS
/// THE GLOBAL OnceLock AND WORKS WITH ANY `&TestAllocator`.
pub fn execute_on_ferrite_core_with<F, R>(allocator: &TestAllocator, f: F) -> (bool, R)
where
    F: FnOnce() -> R,
{
    let snap_before = allocator.snapshot();
    let result = f();
    let snap_after = allocator.snapshot();
    (snap_before == snap_after, result)
}
