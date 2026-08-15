//--------------------------------------------------------------------
// MODULE:        ferrite-core/src/audit.rs
// PURPOSE:       STRUCTURAL GEOMETRY AUDIT.
//                GeometryReport, audit_size_and_align, audit_exact_size,
//                assert_no_padding! MACRO.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   COMPILE-TIME WHERE POSSIBLE. ZERO HEAP.
//--------------------------------------------------------------------

use core::mem;

/// STRUCTURAL GEOMETRY AUDIT RESULT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryReport {
    pub type_name: &'static str,
    pub size_bytes: usize,
    pub align_bytes: usize,
    pub constraint_description: &'static str,
    pub constraint_max_size: Option<usize>,
    pub constraint_min_align: Option<usize>,
    pub constraint_exact_size: Option<usize>,
    /// Cache-line size the report was computed against (None = not requested).
    pub cache_line: Option<usize>,
    pub compliant: bool,
    pub details: &'static str,
}

/// VERIFY STRUCT SIZE ≤ `max_bytes`,
/// ALIGNMENT MULTIPLE OF `min_align`.
pub fn audit_size_and_align<T>(max_bytes: usize, min_align: usize) -> GeometryReport {
    let size = mem::size_of::<T>();
    let align = mem::align_of::<T>();
    let size_ok = size <= max_bytes;
    let align_ok = align >= min_align && align.is_multiple_of(min_align);

    GeometryReport {
        type_name: core::any::type_name::<T>(),
        size_bytes: size,
        align_bytes: align,
        constraint_description: "SIZE LE MAX, ALIGN GE MIN, ALIGN MULTIPLE OF MIN",
        constraint_max_size: Some(max_bytes),
        constraint_min_align: Some(min_align),
        constraint_exact_size: None,
        cache_line: None,
        compliant: size_ok && align_ok,
        details: if size_ok && align_ok {
            "WITHIN CONSTRAINTS"
        } else {
            "SIZE OR ALIGN CONSTRAINT VIOLATED"
        },
    }
}

/// VERIFY EXACT STRUCT SIZE MATCHES EXPECTED.
/// USEFUL FOR `#[repr(C)]` STRUCTS WITH KNOWN LAYOUT.
pub fn audit_exact_size<T>(expected: usize) -> GeometryReport {
    let actual = mem::size_of::<T>();
    GeometryReport {
        type_name: core::any::type_name::<T>(),
        size_bytes: actual,
        align_bytes: mem::align_of::<T>(),
        constraint_description: "EXACT SIZE MATCH",
        constraint_max_size: None,
        constraint_min_align: None,
        constraint_exact_size: Some(expected),
        cache_line: None,
        compliant: actual == expected,
        details: if actual == expected {
            "SIZE MATCHES EXPECTED"
        } else {
            "SIZE MISMATCH (HIDDEN PADDING SUSPECTED)"
        },
    }
}

/// COMPILE-TIME ZERO-PADDING CHECK FOR STRUCT.
///
/// ACCEPTS FIELD NAMES WITH EXPLICIT TYPE ANNOTATIONS.
/// VERIFIES: `size_of::<Struct>() == SUM(size_of::<FieldType>())`.
///
/// USES `core::mem::size_of::<FieldType>()` DIRECTLY —
/// NO NULL POINTER DEREFERENCE, NO UB.
///
/// ## EXAMPLE
///
/// ```ignore
/// #[repr(C)]
/// struct NavState {
///     x: f32,
///     y: f32,
///     flags: u32,
/// }
/// assert_no_padding!(NavState, x: f32, y: f32, flags: u32);
/// ```
#[macro_export]
macro_rules! assert_no_padding {
    ($struct:ty, $($field:ident: $ftype:ty),+ $(,)?) => {
        const _: () = {
            let struct_align: usize = core::mem::align_of::<$struct>();
            let mut offset: usize = 0;
            $(
                let field_align: usize = core::mem::align_of::<$ftype>();
                // ALIGN OFFSET TO FIELD ALIGNMENT
                offset = (offset + field_align - 1) & !(field_align - 1);
                offset += core::mem::size_of::<$ftype>();
            )+
            // ALIGN TO STRUCT ALIGNMENT (TAIL PADDING)
            let expected_size: usize = (offset + struct_align - 1) & !(struct_align - 1);
            let actual_size: usize = core::mem::size_of::<$struct>();
            if expected_size != actual_size {
                panic!(concat!(
                    "UNEXPECTED PADDING in struct ",
                    stringify!($struct),
                    ": check field layout — aligned sum ≠ size_of"
                ))
            }
        };
    };
}

/// ONE FIELD'S CACHE-LINE SPAN WITHIN ITS STRUCT.
///
/// AP101B-101 (V0.1.1). Offsets are compile-time facts via
/// `core::mem::offset_of!` (stable 1.77) — no reflection, no unsafe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldSpan {
    /// Field name as written in the macro invocation.
    pub name: &'static str,
    /// Byte offset of the field inside its struct.
    pub offset: usize,
    /// Field size in bytes.
    pub size: usize,
    /// Cache line containing the field's FIRST byte.
    pub start_line: usize,
    /// Cache line containing the field's LAST byte.
    pub end_line: usize,
}

impl FieldSpan {
    /// True when the field is split across a cache-line boundary.
    ///
    /// A split forces two line fills per access — the classic
    /// cross-cache-line penalty (AP101B-101 story).
    pub const fn crosses_line(&self) -> bool {
        self.start_line != self.end_line
    }

    /// True when both spans touch at least one common cache line.
    ///
    /// Two independently-written fields sharing a line are the
    /// FALSE-SHARING CANDIDATES: a write to one invalidates the other's
    /// line on every other core. Pairs must be checked by the discipline
    /// test against the actual ownership model (same owner = harmless).
    pub const fn shares_line_with(&self, other: &FieldSpan) -> bool {
        self.start_line <= other.end_line && other.start_line <= self.end_line
    }
}

/// COMPILE-TIME PER-FIELD CACHE-LINE SPANS FOR A STRUCT.
///
/// EXPANDS TO `[FieldSpan; N]`. FIELD TYPES MUST MATCH THE STRUCT
/// EXACTLY (SAME CONTRACT AS `assert_no_padding!`).
///
/// ## EXAMPLE
///
/// ```ignore
/// const SPANS: [FieldSpan; 2] =
///     cache_line_fields!(NavState, 64, x: f32, flags: u32);
/// ```
///
/// NOTE: `offset_of!` REQUIRES FIELD VISIBILITY AT THE INVOCATION SITE —
/// PRIVATE FIELDS MUST BE AUDITED INSIDE THE DEFINING CRATE.
#[macro_export]
macro_rules! cache_line_fields {
    ($ty:ty, $line:expr, $($field:ident: $ftype:ty),+ $(,)?) => {
        [
            $(
                $crate::audit::FieldSpan {
                    name: stringify!($field),
                    offset: core::mem::offset_of!($ty, $field),
                    size: core::mem::size_of::<$ftype>(),
                    start_line: core::mem::offset_of!($ty, $field) / $line,
                    end_line: {
                        let off = core::mem::offset_of!($ty, $field);
                        let size = core::mem::size_of::<$ftype>();
                        if size == 0 {
                            off / $line
                        } else {
                            (off + size - 1) / $line
                        }
                    },
                }
            ),+
        ]
    };
}
