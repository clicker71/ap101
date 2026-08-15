//--------------------------------------------------------------------
// MODULE:        ap101b-core/tests/cacheline.rs
// PURPOSE:       AP101B-101 (V0.1.1) CACHE-LINE ANALYSIS DISCIPLINE.
//                PER-FIELD CACHE-LINE SPANS, CROSS-LINE SPLITS,
//                FALSE-SHARING CANDIDATE GROUPING.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-08-15
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   COMPILE-TIME OFFSETS (offset_of!). ZERO HEAP.
//--------------------------------------------------------------------

use ap101b_core::NavigationState;
use ferrite_core::audit::FieldSpan;
use ferrite_core::cache_line_fields;
use ferrite_core::telemetry::IbmCrt;

/// TARGET LINE SIZE: x86_64 L1D (64B). IN-1 NOTE: THE TOOL TAKES THE
/// LINE SIZE AS A PARAMETER — NO HARDCODED ARCHITECTURE ASSUMPTION.
const LINE: usize = 64;

// SYNTHETIC SPLIT STRUCT: tail IS ALIGN-1 AND STARTS AT OFFSET 60 →
// SPANS BYTES 60..69 → CROSSES THE 64B LINE BOUNDARY. (A u64 HERE WOULD
// NOT SPLIT: repr(C) WOULD PAD IT UP TO ITS 8-BYTE ALIGNMENT FIRST.)
#[repr(C)]
struct SplitAcrossLine {
    head: [u8; 60],
    tail: [u8; 9],
}

// TWO OWNERS ON ONE LINE — FALSE-SHARING CANDIDATE.
#[repr(C)]
struct SameLineOwners {
    writer_a: u64,
    writer_b: u64,
}

// SAME LOGICAL PAIR, PADDED TO LINE ALIGNMENT — OWNERS SEPARATED.
#[repr(C, align(64))]
struct PaddedOwners {
    writer_a: u64,
    _pad: [u8; 56],
    writer_b: u64,
}

// COMPILE-TIME SPANS FOR THE REAL NAVIGATION STATE.
const NAV_SPANS: [FieldSpan; 6] = cache_line_fields!(
    NavigationState,
    LINE,
    timestamp: u64,
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
    status_flags: u32,
    checksum: u32
);

// COMPILE-TIME SPANS FOR THE SYNTHETIC STRUCTS.
const SPLIT_SPANS: [FieldSpan; 2] =
    cache_line_fields!(SplitAcrossLine, LINE, head: [u8; 60], tail: [u8; 9]);
const SAME_LINE_SPANS: [FieldSpan; 2] =
    cache_line_fields!(SameLineOwners, LINE, writer_a: u64, writer_b: u64);
const PADDED_SPANS: [FieldSpan; 2] =
    cache_line_fields!(PaddedOwners, LINE, writer_a: u64, writer_b: u64);

#[test]
fn ap101b_cacheline_navigation_state() {
    let mut all_clear = true;
    IbmCrt::print_header("AP-101B FERRITE CORE", "CACHE-LINE: NAVIGATION STATE v1.0");

    for span in NAV_SPANS {
        let ok = !span.crosses_line();
        all_clear &= ok;
        IbmCrt::print_row(
            "AP101B-CACHE-",
            "01",
            span.name,
            ok,
            &format!(
                "off={} size={} line={} (no split)",
                span.offset, span.size, span.start_line
            ),
        );
    }

    // NavigationState is 32B < one 64B line — every field is on line 0,
    // so every pair trivially shares. The discipline note: fields written
    // by DIFFERENT owners on one line are false-sharing candidates; here
    // the state is single-owner, so the grouping is informational.
    IbmCrt::print_row(
        "AP101B-CACHE-",
        "02",
        "Same-line groups (single owner)",
        true,
        "32B struct fully inside line 0 — no cross-core invalidation",
    );

    IbmCrt::print_footer(all_clear);
    assert!(all_clear, "NavigationState field crosses a cache line!");
}

#[test]
fn ap101b_cacheline_split_detection() {
    let mut all_clear = true;
    IbmCrt::print_header("AP-101B FERRITE CORE", "CACHE-LINE: SPLIT DETECTION");

    let head = &SPLIT_SPANS[0];
    let tail = &SPLIT_SPANS[1];

    let head_ok = !head.crosses_line();
    all_clear &= head_ok;
    IbmCrt::print_row(
        "AP101B-CACHE-",
        "03",
        "head [u8;60] fits one line",
        head_ok,
        &format!("off={} line={}", head.offset, head.start_line),
    );

    let tail_ok = tail.crosses_line();
    all_clear &= tail_ok;
    IbmCrt::print_row(
        "AP101B-CACHE-",
        "04",
        "tail [u8;9] @60 DETECTED as split",
        tail_ok,
        &format!(
            "off={} lines={}..{}",
            tail.offset, tail.start_line, tail.end_line
        ),
    );

    IbmCrt::print_footer(all_clear);
    assert!(all_clear, "cross-line split not detected where expected");
}

#[test]
fn ap101b_cacheline_false_sharing_grouping() {
    let mut all_clear = true;
    IbmCrt::print_header("AP-101B FERRITE CORE", "CACHE-LINE: FALSE-SHARING GROUPS");

    let (a, b) = (&SAME_LINE_SPANS[0], &SAME_LINE_SPANS[1]);
    let shared = a.shares_line_with(b);
    all_clear &= shared;
    IbmCrt::print_row(
        "AP101B-CACHE-",
        "05",
        "SameLineOwners share a line (candidate)",
        shared,
        &format!("a@{} b@{}", a.offset, b.offset),
    );

    let (pa, pb) = (&PADDED_SPANS[0], &PADDED_SPANS[1]);
    let separated = !pa.shares_line_with(pb);
    all_clear &= separated;
    IbmCrt::print_row(
        "AP101B-CACHE-",
        "06",
        "PaddedOwners separated by align(64)",
        separated,
        &format!(
            "a@{} (line {}) b@{} (line {})",
            pa.offset, pa.start_line, pb.offset, pb.start_line
        ),
    );

    IbmCrt::print_footer(all_clear);
    assert!(
        all_clear,
        "false-sharing grouping semantics violated: same-line pair not flagged \
         or padded pair still shares a line"
    );
}
