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

/// INJECT MULTI-BIT BURST ERROR (DRAM ROW UPSET SIMULATION).
///
/// FLIPS `bit_count` ADJACENT BITS STARTING AT `byte_offset`.
/// SIMULATES DRAM ROW DISTURBANCE WHERE 2–8 NEIGHBORING
/// BITS FLIP SIMULTANEOUSLY.
///
/// ## PANICS
///
/// IF `byte_offset + (bit_count / 8) + 1 >= data.len()`.
pub fn inject_burst_error(data: &mut [u8], byte_offset: usize, bit_count: u8) {
    assert!((2..=8).contains(&bit_count),
        "BURST ERROR: bit_count MUST BE 2..=8");
    let total_bits = bit_count as usize;
    let end_byte = byte_offset + (total_bits / 8) + 1;
    assert!(end_byte <= data.len(),
        "BURST ERROR: offset {} + bits {} exceeds data length {}",
        byte_offset, bit_count, data.len());

    let mut remaining = total_bits;
    let mut byte_idx = byte_offset;
    let mut bit_offset = 0u8;

    while remaining > 0 {
        let bits_in_this_byte = (8 - bit_offset).min(remaining as u8);
        let mask = ((1u16 << bits_in_this_byte) - 1) as u8;
        data[byte_idx] ^= mask << bit_offset;
        remaining -= bits_in_this_byte as usize;
        byte_idx += 1;
        bit_offset = 0;
    }
}

/// RANDOM f32 GENERATOR, EXCLUDING NaN AND INFINITY.
pub fn finite_f32() -> impl Strategy<Value = f32> {
    any::<f32>().prop_filter("must be finite", |f| f.is_finite())
}

/// RANDOM f64 GENERATOR, EXCLUDING NaN AND INFINITY.
pub fn finite_f64() -> impl Strategy<Value = f64> {
    any::<f64>().prop_filter("must be finite", |f| f.is_finite())
}
