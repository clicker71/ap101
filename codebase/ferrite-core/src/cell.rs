//--------------------------------------------------------------------
// MODULE:        ferrite-core/src/cell.rs
// PURPOSE:       FERRITE MEMORY CELL WITH UNSAFE CONTRACT.
//                FerriteCell<T> -- CONTROLLED-ACCESS WRAPPER AROUND
//                UnsafeCell. SAFE READ FOR COPY TYPES. UNSAFE WRITE.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   NO HEAP. NO STD. PURE CORE::CELL.
//--------------------------------------------------------------------

use core::cell::UnsafeCell;

/// CONTROLLED-ACCESS MEMORY CELL.
///
/// UNLIKE RAW `UnsafeCell`, PROVIDES EXPLICIT METHODS FOR SAFE
/// READ OF COPY TYPES AND UNSAFE MUTATION.
///
/// ## SAFETY CONTRACT
///
/// - `read_copy` IS SAFE ONLY FOR TYPES IMPLEMENTING `FerriteSafeRead`
///   (ATOMIC PRIMITIVES, COPY TYPES OF SIZE ≤ MACHINE WORD).
/// - `write_volatile` AND `inject_cosmic_ray` REQUIRE `&self`
///   WITH EXTERNAL SYNCHRONIZATION. CALLER MUST ENSURE NO CONCURRENT
///   READERS DURING WRITE.
///
/// ## EXAMPLE
///
/// ```ignore
/// let cell = FerriteCell::new(42u32);
/// assert_eq!(cell.read_copy(), 42);
/// // SAFETY: single-threaded context, no concurrent readers.
/// unsafe { cell.write_volatile(43); }
/// assert_eq!(cell.read_copy(), 43);
/// ```
pub struct FerriteCell<T> {
    core: UnsafeCell<T>,
}

// MANUAL Send/Sync WITH EXPLICIT CONTRACT.
// SAFE BECAUSE:
// - Send: NEVER EXPOSE &mut T WITHOUT CONTROL.
// - Sync: read_copy OPERATES ATOMICALLY ON PRIMITIVES.
//   write_volatile REQUIRES EXTERNAL SYNCHRONIZATION (CALLER DUTY).
unsafe impl<T: Send> Send for FerriteCell<T> {}
unsafe impl<T: Send> Sync for FerriteCell<T> {}

impl<T> FerriteCell<T> {
    /// CONSTRUCT NEW CELL.
    pub const fn new(val: T) -> Self {
        Self {
            core: UnsafeCell::new(val),
        }
    }

    /// RETURN RAW POINTER TO DATA.
    ///
    /// ## Safety
    ///
    /// CALLER MUST ENSURE NO DATA RACES.
    pub unsafe fn as_ptr(&self) -> *const T {
        self.core.get()
    }

    /// RETURN MUTABLE RAW POINTER TO DATA.
    ///
    /// ## Safety
    ///
    /// CALLER MUST ENSURE EXCLUSIVE ACCESS.
    pub unsafe fn as_mut_ptr(&self) -> *mut T {
        self.core.get()
    }

    /// SAFE COPY OF VALUE (MACHINE-WORD COPY TYPES ONLY).
    /// FOR COMPLEX TYPES USE UNSAFE READ.
    pub fn read_copy(&self) -> T
    where
        T: Copy + FerriteSafeRead,
    {
        // SAFE: FerriteSafeRead TYPES GUARANTEE ATOMIC
        // OR SINGLE-BYTE READ SIZE.
        unsafe { core::ptr::read_volatile(self.core.get()) }
    }

    /// WRITE VALUE WITH VOLATILE SEMANTICS.
    ///
    /// ## Safety
    ///
    /// CALLER MUST ENSURE NO CONCURRENT READERS
    /// DURING WRITE. THIS IS A NON-ATOMIC WRITE.
    pub unsafe fn write_volatile(&self, val: T) {
        core::ptr::write_volatile(self.core.get(), val);
    }

    /// INJECT SINGLE-EVENT UPSET: FLIP BIT `bit_index` AT BYTE `byte_offset`.
    ///
    /// ## Safety
    ///
    /// - `byte_offset` MUST BE < `size_of::<T>()`.
    /// - `bit_index` MUST BE < 8.
    /// - CALLER MUST ENSURE EXCLUSIVE ACCESS.
    ///   THIS IS A NON-ATOMIC READ-MODIFY-WRITE — TWO CONCURRENT
    ///   CALLS ON THE SAME BYTE = LOST UPDATE.
    pub unsafe fn inject_cosmic_ray(&self, byte_offset: usize, bit_index: u8) {
        debug_assert!(byte_offset < core::mem::size_of::<T>());
        debug_assert!(bit_index < 8);

        let ptr = self.core.get() as *mut u8;
        let target = ptr.add(byte_offset);
        let current = core::ptr::read_volatile(target);
        core::ptr::write_volatile(target, current ^ (1 << bit_index));
    }

    /// CONSUME CELL. RETURN INNER VALUE.
    pub fn into_inner(self) -> T {
        self.core.into_inner()
    }
}

// NOTE: NO SAFE `store` METHOD EXISTS.
//
// WRITING THROUGH A SHARED REFERENCE (&self) IS A DATA RACE.
// USE `unsafe fn write_volatile(&self, val: T)` WITH EXTERNAL
// SYNCHRONIZATION GUARANTEES INSTEAD.
//
// RATIONALE: FerriteCell IS NOT A SYNCHRONIZATION PRIMITIVE.
// FOR ATOMIC STORE USE core::sync::atomic::Atomic* TYPES.

/// MARKER TRAIT FOR TYPES SAFE TO READ THROUGH `FerriteCell`.
///
/// ## SAFETY CRITERIA
///
/// TYPE MUST BE:
/// - PRIMITIVE INTEGER/BOOLEAN (SIZE ≤ usize). CPU READS ATOMICALLY.
/// - OR COPY + SIZE = 1 BYTE (u8, i8, bool).
///
/// ## RATIONALE
///
/// READING u64 ON 32-BIT PLATFORM MAY BE NON-ATOMIC.
/// READING STRUCT WITH f64 MAY CAUSE TORN READ.
/// THIS TRAIT GUARANTEES `read_copy` IS SAFE.
///
/// # Safety
///
/// Implementors must guarantee that `read_volatile` on this type
/// produces a valid bit pattern and is atomic with respect to
/// concurrent writers on the target platform.
pub unsafe trait FerriteSafeRead: Copy {}

// IMPLEMENTATIONS FOR SAFE TYPES
unsafe impl FerriteSafeRead for u8 {}
unsafe impl FerriteSafeRead for i8 {}
unsafe impl FerriteSafeRead for bool {}
unsafe impl FerriteSafeRead for u16 {}
unsafe impl FerriteSafeRead for i16 {}
unsafe impl FerriteSafeRead for u32 {}
unsafe impl FerriteSafeRead for i32 {}
unsafe impl FerriteSafeRead for usize {}
unsafe impl FerriteSafeRead for isize {}

// f32 AND f64 EXCLUDED — READ MAY BE NON-ATOMIC
// ON SOME PLATFORMS (E.G. VIA FPU STACK).
