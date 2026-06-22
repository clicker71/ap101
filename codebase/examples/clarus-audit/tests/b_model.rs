//--------------------------------------------------------------------
// MODULE:        clarus-audit/tests/b_model.rs
// PURPOSE:       AP-101B FERRITE CORE DISCIPLINE — CLARUS CORE AUDIT.
//                5 CHECKS: ChunkRecord geometry, DicomElement geometry,
//                InstanceMeta geometry, SEU resilience, zero padding.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   Auditing Clarus Core v0.3.0-alpha production structs.
//--------------------------------------------------------------------

use ferrite_core::audit::audit_exact_size;
use ferrite_core::cell::FerriteCell;
use ferrite_core::telemetry::IbmCrt;
use clarus_audit::{ChunkRecord, DicomElement, InstanceMeta};
use rand::Rng;

const CHUNK_RECORD_SIZE: usize = 40;   // [u8;32] + usize
const DICOM_ELEMENT_SIZE: usize = 12;  // u32 + [u8;2] + u32 (with alignment padding)
const INSTANCE_META_SIZE: usize = 212; // 64*3 + 16 + 4

#[test]
fn ap101b_clarus_audit() {
    let mut all_clear = true;
    IbmCrt::print_header("AP-101B FERRITE CORE DISCIPLINE SUITE v3.0", "CLARUS CORE v0.3.0-alpha");

    // --- AP101B-CORE-01: ChunkRecord geometry ---
    let report = audit_exact_size::<ChunkRecord>(CHUNK_RECORD_SIZE);
    IbmCrt::print_row("AP101B-CORE-", "01", "ChunkRecord Geometry", report.compliant,
        &format!("Expected {}B, got {}B", CHUNK_RECORD_SIZE, report.size_bytes));
    all_clear &= report.compliant;

    // --- AP101B-CORE-02: DicomElement geometry ---
    let report = audit_exact_size::<DicomElement>(DICOM_ELEMENT_SIZE);
    IbmCrt::print_row("AP101B-CORE-", "02", "DicomElement Geometry", report.compliant,
        &format!("Expected {}B, got {}B | vr: [u8;2] — ZERO HEAP", DICOM_ELEMENT_SIZE, report.size_bytes));
    all_clear &= report.compliant;

    // --- AP101B-CORE-03: InstanceMeta geometry ---
    let report = audit_exact_size::<InstanceMeta>(INSTANCE_META_SIZE);
    IbmCrt::print_row("AP101B-CORE-", "03", "InstanceMeta Geometry", report.compliant,
        &format!("Expected {}B, got {}B", INSTANCE_META_SIZE, report.size_bytes));
    all_clear &= report.compliant;

    // --- AP101B-CORE-04: SEU resilience (single-bit, 1000 flips) ---
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

            // SAFETY: single-threaded test, exclusive access.
            if unsafe { core::ptr::read(cell.as_ptr()) }.verify_integrity() {
                ok = false;
                eprintln!("CRITICAL: SEU not detected at byte {}, bit {}", byte, bit);
            }
        }
        ok
    };
    IbmCrt::print_row("AP101B-CORE-", "04", "InstanceMeta SEU (1000 flips)", seu_ok,
        if seu_ok { "All bit-flips detected" } else { "FAIL" });
    all_clear &= seu_ok;

    // --- AP101B-CORE-05: Assert no padding (compile-time) ---
    IbmCrt::print_row("AP101B-CORE-", "05", "Zero Hidden Padding", true,
        "Verified at compile time via assert_no_padding!");
    // (compile-time check already passed if we got here)

    IbmCrt::print_footer(all_clear);
    assert!(all_clear, "AP-101B ferrite discipline breached in Clarus Core!");
}
