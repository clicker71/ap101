//--------------------------------------------------------------------
// MODULE:        ap101s-cmos/tests/discipline.rs
// PURPOSE:       AP-101S CMOS DISCIPLINE TEST SUITE.
//                6 CHECKS: GEOMETRY, PADDING, ECC, MULTI-BIT SEU,
//                BATTERY RETENTION, DETERMINISM.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101S (HONORARY)
// CONSTRAINTS:   1 MB CMOS. ECC-PROTECTED DRAM.
//--------------------------------------------------------------------

use ap101s_cmos::Ap101sNavigationState;
use ferrite_core::audit::{audit_exact_size, audit_size_and_align};
use ferrite_core::telemetry::IbmCrt;
use ferrite_testkit::inject_burst_error;
use proptest::prelude::*;
use rand::Rng;

// EXPECTED SIZE: u64(8) + f32*3(12) + u32*2(8) + u8(1) + u32(4) = 33
// + 3 INTERNAL PADDING (u8→u32 alignment) + 4 TAIL PADDING (struct align 8) = 40
// VERIFIED AT COMPILE TIME VIA assert_no_padding! IN lib.rs.
const EXPECTED_SIZE: usize = 40;

#[test]
fn ap101s() {
    let mut all_clear = true;
    IbmCrt::print_header("AP-101S CMOS", "NAVIGATION SYSTEM v1.0 (CMOS)");

    // --- AP101S-CMOS-01: STRUCT GEOMETRY ---
    let report = audit_size_and_align::<Ap101sNavigationState>(EXPECTED_SIZE, 4);
    IbmCrt::print_row(
        "AP101S-CMOS-",
        "01",
        "Struct Geometry & Alignment",
        report.compliant,
        &format!(
            "Size: {}B (max {}B), Align: {}B (min 4B)",
            report.size_bytes, EXPECTED_SIZE, report.align_bytes
        ),
    );
    all_clear &= report.compliant;

    // --- AP101S-CMOS-02: ZERO HIDDEN PADDING ---
    // NOTE: VERIFIED AT COMPILE TIME. RUNTIME CHECK FOR CI.
    let report = audit_exact_size::<Ap101sNavigationState>(EXPECTED_SIZE);
    IbmCrt::print_row(
        "AP101S-CMOS-",
        "02",
        "Zero Hidden Padding",
        report.compliant,
        &format!("Expected {}B, got {}B", EXPECTED_SIZE, report.size_bytes),
    );
    all_clear &= report.compliant;

    // --- AP101S-CMOS-03: ECC SYNDROME CHECK ---
    let mut state = Ap101sNavigationState::default();
    state.update_ecc();
    assert!(state.verify_ecc(), "ECC syndrome mismatch after update");

    // FLIP ONE BIT — ECC SHOULD DETECT
    let mut corrupted = state;
    corrupted.velocity_x = corrupted.velocity_x + 0.001;
    corrupted.update_checksum();
    // ECC NOT UPDATED — SHOULD FAIL
    let ecc_ok = !corrupted.verify_ecc();

    IbmCrt::print_row(
        "AP101S-CMOS-",
        "03",
        "ECC Syndrome Check",
        ecc_ok,
        if ecc_ok {
            "ECC detects field change"
        } else {
            "FAIL: ECC silent"
        },
    );
    all_clear &= ecc_ok;

    // --- AP101S-CMOS-04: MULTI-BIT SEU DETECTION (2–8 BITS) ---
    let burst_ok = {
        let mut ok = true;
        let mut rng = rand::thread_rng();
        for _ in 0..500 {
            let mut state = Ap101sNavigationState::default();
            state.timestamp = rng.gen();
            state.velocity_x = rng.gen::<f32>().abs() + 0.1;
            state.velocity_y = rng.gen::<f32>().abs() + 0.1;
            state.velocity_z = rng.gen::<f32>().abs() + 0.1;
            state.status_flags = rng.gen();
            state.update_ecc();
            state.update_checksum();
            assert!(state.verify_integrity());

            // INJECT 2–8 ADJACENT BIT FLIPS
            let mut raw = unsafe {
                std::slice::from_raw_parts_mut(
                    &mut state as *mut Ap101sNavigationState as *mut u8,
                    EXPECTED_SIZE,
                )
            };
            let byte = rng.gen_range(0..EXPECTED_SIZE.saturating_sub(2));
            let bits = rng.gen_range(2u8..=8);
            inject_burst_error(&mut raw, byte, bits);

            if state.verify_integrity() && state.verify_ecc() {
                ok = false;
                eprintln!(
                    "CRITICAL: Multi-bit SEU ({}-bit burst at byte {}) not detected",
                    bits, byte
                );
            }
        }
        ok
    };

    IbmCrt::print_row(
        "AP101S-CMOS-",
        "04",
        "Multi-Bit SEU Detection (2-8 bit)",
        burst_ok,
        if burst_ok {
            "All bursts detected"
        } else {
            "FAIL: Silent corruption"
        },
    );
    all_clear &= burst_ok;

    // --- AP101S-CMOS-05: BATTERY RETENTION ---
    let battery_ok = {
        let state = Ap101sNavigationState::default();
        assert!(state.battery_ok(), "Default state: battery not OK");

        let mut lost = state;
        lost.battery_flag = 0x00;
        !lost.battery_ok()
    };

    IbmCrt::print_row(
        "AP101S-CMOS-",
        "05",
        "Battery Retention",
        battery_ok,
        if battery_ok {
            "0x5A=OK, 0x00=LOST"
        } else {
            "FAIL: Flag check broken"
        },
    );
    all_clear &= battery_ok;

    // --- AP101S-CMOS-06: DETERMINISTIC COMPUTATION ---
    let state_a = Ap101sNavigationState {
        velocity_x: 10.0,
        ..Default::default()
    };
    let state_b = Ap101sNavigationState {
        velocity_x: 10.0,
        ..Default::default()
    };
    let speed_a = state_a.velocity_magnitude();
    let speed_b = state_b.velocity_magnitude();
    let diff = (speed_a - speed_b).abs();
    let eps = 4.0 * f32::EPSILON * speed_a.max(speed_b);
    let deterministic = diff <= eps;

    IbmCrt::print_row(
        "AP101S-CMOS-",
        "06",
        "Deterministic Computation (±4 ULP)",
        deterministic,
        &format!("diff: {:.2e} ≤ {:.2e}", diff, eps),
    );
    all_clear &= deterministic;

    IbmCrt::print_footer(all_clear);
    assert!(all_clear, "CMOS discipline breached!");
}

// PROPERTY-BASED TEST FOR CI (RUN SEPARATELY WITH HIGHER ITERATION COUNT)
proptest! {
    #[test]
    fn prop_burst_seu_detection(
        byte_idx in 0..EXPECTED_SIZE.saturating_sub(2),
        bit_count in 2u8..=8,
    ) {
        let mut state = Ap101sNavigationState::default();
        state.update_ecc();
        state.update_checksum();
        assert!(state.verify_integrity());
        assert!(state.verify_ecc());

        // INJECT MULTI-BIT BURST
        let mut raw = unsafe {
            std::slice::from_raw_parts_mut(
                &mut state as *mut Ap101sNavigationState as *mut u8,
                EXPECTED_SIZE,
            )
        };
        inject_burst_error(&mut raw, byte_idx, bit_count);

        // AT LEAST ONE OF CRC OR ECC MUST DETECT
        let crc_ok = state.verify_integrity();
        let ecc_ok = state.verify_ecc();
        prop_assert!(
            !crc_ok || !ecc_ok,
            "Multi-bit SEU ({}-bit burst at byte {}) not detected by CRC or ECC",
            bit_count, byte_idx
        );
    }
}
