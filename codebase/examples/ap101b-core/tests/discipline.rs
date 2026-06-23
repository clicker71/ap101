//--------------------------------------------------------------------
// MODULE:        ap101b-core/tests/discipline.rs
// PURPOSE:       AP-101B FERRITE CORE DISCIPLINE TEST SUITE.
//                5 CHECKS: GEOMETRY, PADDING, DETERMINISM, SEU, PBT.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   256 KB USABLE CORE. RAND 0.8. PROPTEST 1.4.
//--------------------------------------------------------------------

use ap101b_core::NavigationState;
use ferrite_core::audit::{audit_exact_size, audit_size_and_align};
use ferrite_core::cell::FerriteCell;
use ferrite_core::telemetry::IbmCrt;
use ferrite_testkit::{bit_index, byte_offset};
use proptest::prelude::*;
use rand::Rng;

// EXPECTED SIZE: u64(8) + f32*3(12) + u32(4) + u32(4) = 28 BYTES + 4 TAIL PADDING (ALIGN 8) = 32
// TAIL PADDING IS REQUIRED BY #[repr(C)] ABI: STRUCT SIZE MUST BE MULTIPLE OF LARGEST FIELD ALIGNMENT.
const EXPECTED_SIZE: usize = 32;

#[test]
fn ap101b() {
    let mut all_clear = true;
    IbmCrt::print_header("AP-101B FERRITE CORE", "NAVIGATION SYSTEM v1.0");

    // --- AP101B-CORE-01: STRUCT GEOMETRY ---
    let report = audit_size_and_align::<NavigationState>(EXPECTED_SIZE, 4);
    IbmCrt::print_row(
        "AP101B-CORE-",
        "01",
        "Struct Geometry & Alignment",
        report.compliant,
        &format!(
            "Size: {}B (max {}B), Align: {}B (min 4B)",
            report.size_bytes, EXPECTED_SIZE, report.align_bytes
        ),
    );
    all_clear &= report.compliant;

    // --- AP101B-CORE-02: ZERO HIDDEN PADDING ---
    // NOTE: VERIFIED AT COMPILE TIME VIA assert_no_padding! IN lib.rs.
    // RUNTIME CHECK FOR CI.
    let report = audit_exact_size::<NavigationState>(EXPECTED_SIZE);
    IbmCrt::print_row(
        "AP101B-CORE-",
        "02",
        "Zero Hidden Padding",
        report.compliant,
        &format!("Expected {}B, got {}B", EXPECTED_SIZE, report.size_bytes),
    );
    all_clear &= report.compliant;

    // --- AP101B-CORE-03: DETERMINISTIC COMPUTATION ---
    let state_a = NavigationState {
        velocity_x: 10.0,
        ..Default::default()
    };
    let state_b = NavigationState {
        velocity_x: 10.0,
        ..Default::default()
    };
    let speed_a = state_a.velocity_magnitude();
    let speed_b = state_b.velocity_magnitude();
    let diff = (speed_a - speed_b).abs();
    let eps = 4.0 * f32::EPSILON * speed_a.max(speed_b); // 4 ULP
    let deterministic = diff <= eps;

    IbmCrt::print_row(
        "AP101B-CORE-",
        "03",
        "Deterministic Computation (±4 ULP)",
        deterministic,
        &format!("diff: {:.2e} ≤ {:.2e}", diff, eps),
    );
    all_clear &= deterministic;

    // --- AP101B-CORE-04: COSMIC RAY SEU RESILIENCE ---
    let seu_ok = {
        let mut ok = true;
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let mut state = NavigationState::default();
            state.timestamp = rng.gen();
            state.velocity_x = rng.gen::<f32>().abs() + 0.1;
            state.velocity_y = rng.gen::<f32>().abs() + 0.1;
            state.velocity_z = rng.gen::<f32>().abs() + 0.1;
            state.status_flags = rng.gen();
            state.update_checksum();

            let cell = FerriteCell::new(state);
            // SAFETY: single-threaded test, exclusive access.
            assert!(unsafe { core::ptr::read(cell.as_ptr()) }.verify_integrity());

            // INJECT RANDOM SINGLE-EVENT UPSET
            let byte = rng.gen_range(0..EXPECTED_SIZE);
            let bit = rng.gen_range(0u8..8);
            unsafe { cell.inject_cosmic_ray(byte, bit) };

            if unsafe { core::ptr::read(cell.as_ptr()) }.verify_integrity() {
                ok = false;
                eprintln!("CRITICAL: SEU not detected at byte {}, bit {}", byte, bit);
            }
        }
        ok
    };

    IbmCrt::print_row(
        "AP101B-CORE-",
        "04",
        "Cosmic Ray SEU Resilience",
        seu_ok,
        if seu_ok {
            "All bit-flips detected"
        } else {
            "FAIL: Silent data corruption possible"
        },
    );
    all_clear &= seu_ok;

    IbmCrt::print_footer(all_clear);
    assert!(all_clear, "Ferrite discipline breached!");
}

// PROPERTY-BASED TEST FOR CI (RUN SEPARATELY WITH HIGHER ITERATION COUNT)
proptest! {
    #[test]
    fn prop_seu_detection(
        byte_idx in byte_offset::<NavigationState>(),
        bit in bit_index(),
    ) {
        let mut state = NavigationState::default();
        state.update_checksum();
        assert!(state.verify_integrity());

        let cell = FerriteCell::new(state);
        unsafe { cell.inject_cosmic_ray(byte_idx, bit) };

        prop_assert!(
            !unsafe { core::ptr::read(cell.as_ptr()) }.verify_integrity(),
            "SEU not detected at byte {}, bit {}",
            byte_idx, bit
        );
    }
}
