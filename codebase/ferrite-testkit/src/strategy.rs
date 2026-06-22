//--------------------------------------------------------------------
// MODULE:        ferrite-testkit/src/strategy.rs
// PURPOSE:       PROPTEST SEU STRATEGIES.
//                byte_offset, bit_index STRATEGIES.
//                assert_seu_detected, finite_f32, finite_f64 GENERATORS.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   PROPTEST 1.4. RAND 0.8. DETERMINISTIC SEEDS IN CI.
//--------------------------------------------------------------------

use ferrite_core::checksum::Checksum;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

/// STRATEGY: RANDOM BYTE OFFSET WITHIN STRUCT.
pub fn byte_offset<T: 'static>() -> impl Strategy<Value = usize> {
    let max = core::mem::size_of::<T>();
    (0..max).prop_map(move |i| i % max)
}

/// STRATEGY: RANDOM BIT INDEX (0..7).
pub fn bit_index() -> impl Strategy<Value = u8> {
    0u8..8
}

/// VERIFY ANY SINGLE BIT-FLIP CHANGES CHECKSUM.
///
/// ## PROPTEST USAGE
///
/// ```ignore
/// proptest! {
///     #[test]
///     fn test_seu_detection(
///         data in arb_navigation_state(),
///         byte_idx in byte_offset::<NavigationState>(),
///         bit in bit_index(),
///     ) {
///         assert_seu_detected::<Crc32, _>(&data, byte_idx, bit)?;
///     }
/// }
/// ```
pub fn assert_seu_detected<C: Checksum, T: AsRef<[u8]>>(
    data: &T,
    byte_idx: usize,
    bit: u8,
) -> Result<(), TestCaseError>
where
    <C as Checksum>::Output: std::fmt::Debug,
{
    let original = C::compute(data);

    // MUTATE COPY
    let mut corrupted = Vec::from(data.as_ref());
    if byte_idx < corrupted.len() {
        corrupted[byte_idx] ^= 1 << bit;
    }

    let corrupted_checksum = C::compute(&corrupted);
    prop_assert_ne!(
        original,
        corrupted_checksum,
        "SEU not detected: byte {}, bit {}",
        byte_idx,
        bit
    );
    Ok(())
}

/// RANDOM f32 GENERATOR, EXCLUDING NaN AND INFINITY.
pub fn finite_f32() -> impl Strategy<Value = f32> {
    any::<f32>().prop_filter("must be finite", |f| f.is_finite())
}

/// RANDOM f64 GENERATOR, EXCLUDING NaN AND INFINITY.
pub fn finite_f64() -> impl Strategy<Value = f64> {
    any::<f64>().prop_filter("must be finite", |f| f.is_finite())
}
