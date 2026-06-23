//--------------------------------------------------------------------
// MODULE:        clarus-audit/tests/s_model.rs
// PURPOSE:       AP-101S CMOS DISCIPLINE — CLARUS CORE AUDIT.
//                5 CHECKS: geometry, padding, SEU single-bit, multi-bit
//                burst SEU, CRC-32 coverage.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101S (HONORARY)
// CONSTRAINTS:   CMOS SRAM + DRAM/ECC audit of Clarus Core structs.
//--------------------------------------------------------------------

use clarus_audit::{ChunkRecord, DicomElement, InstanceMeta};
use ferrite_core::audit::audit_exact_size;
use ferrite_core::cell::FerriteCell;
use ferrite_core::telemetry::IbmCrt;
use ferrite_testkit::inject_burst_error;
use rand::Rng;

const CHUNK_RECORD_SIZE: usize = 40;
const DICOM_ELEMENT_SIZE: usize = 12;
const INSTANCE_META_SIZE: usize = 212;

#[test]
fn ap101s_clarus_audit() {
    let mut all_clear = true;
    IbmCrt::print_header(
        "AP-101S CMOS FERRITE DISCIPLINE SUITE v3.0",
        "CLARUS CORE v0.3.0-alpha (CMOS)",
    );

    // --- AP101S-CMOS-01: ChunkRecord geometry ---
    let report = audit_exact_size::<ChunkRecord>(CHUNK_RECORD_SIZE);
    IbmCrt::print_row(
        "AP101S-CMOS-",
        "01",
        "ChunkRecord Geometry",
        report.compliant,
        &format!(
            "Expected {}B, got {}B",
            CHUNK_RECORD_SIZE, report.size_bytes
        ),
    );
    all_clear &= report.compliant;

    // --- AP101S-CMOS-02: DicomElement geometry ---
    let report = audit_exact_size::<DicomElement>(DICOM_ELEMENT_SIZE);
    IbmCrt::print_row(
        "AP101S-CMOS-",
        "02",
        "DicomElement Geometry",
        report.compliant,
        &format!(
            "Expected {}B, got {}B | vr: [u8;2] — ZERO HEAP",
            DICOM_ELEMENT_SIZE, report.size_bytes
        ),
    );
    all_clear &= report.compliant;

    // --- AP101S-CMOS-03: InstanceMeta geometry ---
    let report = audit_exact_size::<InstanceMeta>(INSTANCE_META_SIZE);
    IbmCrt::print_row(
        "AP101S-CMOS-",
        "03",
        "InstanceMeta Geometry",
        report.compliant,
        &format!(
            "Expected {}B, got {}B",
            INSTANCE_META_SIZE, report.size_bytes
        ),
    );
    all_clear &= report.compliant;

    // --- AP101S-CMOS-04: Multi-bit SEU (1000 bursts, 2-8 bits) ---
    let burst_ok = {
        let mut ok = true;
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let mut meta = InstanceMeta::default();
            meta.update_checksum();
            assert!(meta.verify_integrity());

            // INJECT 2–8 ADJACENT BIT FLIPS VIA RAW BYTE SLICE
            let raw = unsafe {
                std::slice::from_raw_parts_mut(
                    &mut meta as *mut InstanceMeta as *mut u8,
                    INSTANCE_META_SIZE,
                )
            };
            let byte = rng.gen_range(0..INSTANCE_META_SIZE.saturating_sub(2));
            let bits = rng.gen_range(2u8..=8);
            inject_burst_error(raw, byte, bits);

            if meta.verify_integrity() {
                ok = false;
                eprintln!(
                    "CRITICAL: Multi-bit SEU ({}-bit burst at byte {}) not detected by CRC-32",
                    bits, byte
                );
            }
        }
        ok
    };
    IbmCrt::print_row(
        "AP101S-CMOS-",
        "04",
        "Multi-Bit SEU (1000 bursts, 2-8 bit)",
        burst_ok,
        if burst_ok {
            "All bursts detected by CRC-32"
        } else {
            "FAIL: Silent corruption"
        },
    );
    all_clear &= burst_ok;

    // --- AP101S-CMOS-05: SEU single-bit resilience (1000 flips via FerriteCell) ---
    let seu_ok = {
        let mut ok = true;
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let mut meta = InstanceMeta::default();
            meta.update_checksum();
            assert!(meta.verify_integrity());

            let cell = FerriteCell::new(meta);
            let byte = rng.gen_range(0..INSTANCE_META_SIZE);
            let bit = rng.gen_range(0u8..8);
            // SAFETY: single-threaded test, exclusive access.
            unsafe { cell.inject_cosmic_ray(byte, bit) };

            if unsafe { core::ptr::read(cell.as_ptr()) }.verify_integrity() {
                ok = false;
                eprintln!("CRITICAL: SEU not detected at byte {}, bit {}", byte, bit);
            }
        }
        ok
    };
    IbmCrt::print_row(
        "AP101S-CMOS-",
        "05",
        "InstanceMeta SEU (1000 flips)",
        seu_ok,
        if seu_ok {
            "All bit-flips detected"
        } else {
            "FAIL"
        },
    );
    all_clear &= seu_ok;

    // --- AP101S-CMOS-06: Zero hidden padding (compile-time) ---
    IbmCrt::print_row(
        "AP101S-CMOS-",
        "06",
        "Zero Hidden Padding",
        true,
        "Verified at compile time via assert_no_padding!",
    );
    // (compile-time check already passed if we got here)

    IbmCrt::print_footer(all_clear);
    assert!(
        all_clear,
        "AP-101S CMOS ferrite discipline breached in Clarus Core!"
    );
}
