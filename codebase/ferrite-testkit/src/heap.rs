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

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// GLOBAL ALLOCATOR FOR TEST ISOLATION.
///
/// ## KNOWN LIMITATION: BYTE COUNTER DRIFT
///
/// IF `set_enabled(false)` IS CALLED BETWEEN `alloc` AND `dealloc`,
/// THE `bytes` COUNTER IS NOT DECREMENTED, CAUSING PERMANENT DRIFT.
/// USE `reset()` TO ZERO COUNTERS BEFORE EACH TEST.
/// FOR MULTITHREADED TESTS, USE `execute_on_ferrite_core_with`
/// WITH A FRESH `TestAllocator` INSTANCE PER THREAD.
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
    count: AtomicUsize,
    bytes: AtomicUsize,
    enabled: AtomicBool,
    system: System,
}

impl TestAllocator {
    pub const fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
            bytes: AtomicUsize::new(0),
            enabled: AtomicBool::new(true),
            system: System,
        }
    }

    /// ENABLE/DISABLE TRACKING.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// RESET COUNTERS.
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.bytes.store(0, Ordering::Relaxed);
    }

    /// RETURN CURRENT SNAPSHOT.
    pub fn snapshot(&self) -> (usize, usize) {
        (
            self.count.load(Ordering::Acquire),
            self.bytes.load(Ordering::Acquire),
        )
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
        if self.enabled.load(Ordering::Relaxed) {
            self.count.fetch_add(1, Ordering::Release);
            self.bytes.fetch_add(layout.size(), Ordering::Release);
        }
        self.system.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // DESIGN NOTE: IF set_enabled(false) BETWEEN alloc AND dealloc,
        // BYTES COUNTER WILL NOT DECREMENT — PERMANENT DRIFT.
        // CALLER MUST RESET AFTER DISABLE/REENABLE CYCLES.
        if self.enabled.load(Ordering::Relaxed) {
            self.bytes.fetch_sub(layout.size(), Ordering::Release);
        }
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
/// THIS FUNCTION IS **NOT** REENTRANT. IF `f` CALLS
/// `execute_on_ferrite_core`, COUNTERS MAY CORRUPT.
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
