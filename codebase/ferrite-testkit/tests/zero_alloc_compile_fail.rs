//! COMPILE-FAIL SELF-TESTS FOR THE `zero_alloc!` BACKSTOP.
//!
//! The gate proves itself in the NEGATIVE direction: every file under
//! `tests/ui/` wraps a block containing ONE deny-listed allocation and MUST
//! FAIL to compile with a `zero_alloc!` message. `trybuild` asserts the exact
//! compiler output against the checked-in `*.stderr` snapshots.
//!
//! The POSITIVE direction lives in [`zero_alloc_accepts_a_clean_block`]: a
//! block free of deny-listed constructs expands unchanged and compiles.

/// Every snippet in `tests/ui/` must be rejected by `zero_alloc!`.
///
/// Each snippet is a self-contained `fn main()` calling the macro with one
/// forbidden construct; the harness compiles them and compares the errors.
#[test]
fn zero_alloc_rejects_deny_listed_constructs() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

/// The positive direction: a clean block is accepted verbatim.
#[test]
fn zero_alloc_accepts_a_clean_block() {
    let value: u32 = ferrite_macros::zero_alloc!({
        let mut total: u32 = 0;
        let mut index: u32 = 0;
        while index < 4 {
            total += index;
            index += 1;
        }
        total
    });
    assert_eq!(value, 6);
}
