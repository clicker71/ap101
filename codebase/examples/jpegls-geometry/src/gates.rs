//--------------------------------------------------------------------
// MODULE:        jpegls-geometry/src/gates.rs
// PURPOSE:       AP-101 AUDIT GATES over the VENDORED pure_jpegls 1.0.0
//                sources. Three gate families:
//
//                G1 GEOMETRY (compile-time):
//                   BitReader    - touched on EVERY bit read; must fit
//                                  one 64B cache line, no split fields.
//                   Decoder      - marker/byte walker; one line.
//                   ContextModel - per-context A/B/C/N vecs must each
//                                  sit inside a single line; no padding
//                                  bloat; no split fields.
//
//                G2 FULL-DECODE HEAP ("the honest non-zero path"):
//                   decode() allocates EXACTLY the declared once-per-
//                   image buffers: pixels(1) + context(4) + curr/prev
//                   lines(2) = 7 allocations, byte count = formula.
//                   Nothing scales with the PIXEL COUNT of the image
//                   beyond those 7.
//
//                G3 SCAN-LOOP HEAP:
//                   decode_scan() (the per-pixel hot loop) allocates
//                   EXACTLY the two row buffers (curr_line, prev_line)
//                   and NOTHING inside the pixel loop.
//
//                PRIVACY NOTE: upstream BitReader/Decoder fields are
//                module-private. To measure spans without touching the
//                vendored files, bitstream.rs and decode.rs are
//                include!-ed into THIS module - the SAME byte-identical
//                declaration compiled at a different module path.
//                Layout of an identical declaration is identical.
//
//                NO FerriteCell/CRC/SEU: the parity byte-diff of the
//                jpegls-parity sibling is the correctness authority.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-08-27
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   Gates run under `cargo test -p jpegls-geometry`.
//--------------------------------------------------------------------

#![allow(dead_code, unused_imports, unused_variables)]
// The include!-ed VENDORED declarations may expose surfaces a specific
// gate does not use. This module is compiled ONLY under cfg(test).

use ferrite_core::{assert_no_padding, cache_line_fields};
use ferrite_testkit::TestAllocator;

/// x86_64 L1D line. The tool takes it as a parameter - no hardcoded
/// architecture assumption (ap101b-core discipline).
const LINE: usize = 64;

// ===========================================================================
// geom / scan: vendored declarations compiled into PRIVATE CHILD
// modules. Each child carries the gates that need access to ITS
// module-private items (children see parent-private fields/fns).
// ===========================================================================

mod geom {
    include!("vendor/bitstream.rs");

    #[cfg(test)]
    mod audit {
        use super::BitReader;
        use ferrite_core::cache_line_fields;

        const LINE: usize = 64;

        #[test]
        fn bitreader_fits_one_cache_line() {
            const SPANS: [ferrite_core::FieldSpan; 4] = cache_line_fields!(
                BitReader<'static>,
                LINE,
                data: &'static [u8],
                pos: usize,
                bits: u64,
                n_bits: i32
            );

            // BitReader is touched on EVERY bit read: it must live in one
            // cache line, with no field split across a line boundary.
            assert!(
                core::mem::size_of::<BitReader<'static>>() <= LINE,
                "BitReader size {} exceeds one cache line",
                core::mem::size_of::<BitReader<'static>>()
            );
            for span in SPANS {
                assert!(
                    !span.crosses_line(),
                    "BitReader.{} splits across cache lines ({:?})",
                    span.name,
                    span
                );
            }
        }
    }
}

mod scan {
    include!("vendor/decode.rs");

    #[cfg(test)]
    mod audit {
        use super::{decode, decode_scan, Decoder};
        use ferrite_core::cache_line_fields;
        use ferrite_testkit::TestAllocator;

        const LINE: usize = 64;

        #[test]
        fn decoder_fits_one_cache_line() {
            const SPANS: [ferrite_core::FieldSpan; 2] = cache_line_fields!(
                Decoder<'static>,
                LINE,
                data: &'static [u8],
                pos: usize
            );

            assert!(
                core::mem::size_of::<Decoder<'static>>() <= LINE,
                "Decoder size {} exceeds one cache line",
                core::mem::size_of::<Decoder<'static>>()
            );
            for span in SPANS {
                assert!(
                    !span.crosses_line(),
                    "Decoder.{} splits across cache lines",
                    span.name
                );
            }
        }

        #[test]
        fn decode_scan_allocates_only_two_row_buffers() {
            let image = super::super::synthetic_image_8bit();
            let w = 64usize;
            let h = 64usize;
            let max_val = super::super::max_val_of(*image.iter().max().unwrap());

            let mut buf: Vec<u8> = Vec::new();
            crate::encode::encode(&image, w as u32, h as u32, &mut buf).expect("encode");

            let scan_start = super::super::scan_data_offset(&buf);

            // All one-time state is constructed OUTSIDE the window - this
            // is "the honest non-zero path": declared buffers, honestly
            // accounted for before the measurement starts.
            let mut br = crate::bitstream::BitReader::new(&buf[scan_start..]);
            let mut ctx = crate::context::ContextModel::new(max_val, 0, 64);
            let mut pixels = vec![0u16; w * h];

            let alloc = TestAllocator::new();
            alloc.set_enabled(true);
            alloc.reset();

            // decode_scan is the include!-ed upstream copy (byte-identical).
            decode_scan(&mut br, &mut ctx, &mut pixels, w, h, max_val)
                .expect("scan decode must succeed");

            let (count, bytes) = alloc.snapshot();
            alloc.set_enabled(false);

            assert_eq!(pixels, image, "scan-level roundtrip parity");

            // DECLARED: curr_line + prev_line only, both freed before
            // decode_scan returns: count 2, live bytes 0. NOTHING per
            // pixel.
            assert_eq!(
                count, 2,
                "decode_scan must allocate exactly the two row buffers"
            );
            assert_eq!(bytes, 0, "row buffers must be freed before return");
        }

        #[test]
        fn included_decode_matches_canonical_decode() {
            // This module's include!-ed copy and the crate-root vendored
            // copy are compiled from the same file: identical input must
            // give identical output. Guards against divergence.
            let image = super::super::synthetic_image_8bit();
            let mut buf: Vec<u8> = Vec::new();
            crate::encode::encode(&image, 64, 64, &mut buf).expect("encode");

            let (canonical, _, _) = crate::decode::decode(&buf, 64, 64).expect("canonical decode");
            let (included, _, _) = decode(&buf, 64, 64).expect("included decode");

            assert_eq!(canonical, included);
        }
    }
}

// ===========================================================================
// G1 (continued): ContextModel geometry - fields are pub, measured on
// the canonical crate-root vendored type.
// ===========================================================================

#[test]
fn contextmodel_geometry() {
    // Fields of the root (canonical) vendored ContextModel are pub.
    const SPANS: [ferrite_core::FieldSpan; 11] = cache_line_fields!(
        crate::context::ContextModel,
        LINE,
        t1: i32,
        t2: i32,
        t3: i32,
        max_val: i32,
        a: Vec<i32>,
        b: Vec<i32>,
        c: Vec<i32>,
        n: Vec<i32>,
        reset: i32,
        j: [i32; 32],
        run_index: usize
    );

    // No padding bloat: declared fields fill the struct exactly.
    assert_no_padding!(
        crate::context::ContextModel,
        t1: i32,
        t2: i32,
        t3: i32,
        max_val: i32,
        a: Vec<i32>,
        b: Vec<i32>,
        c: Vec<i32>,
        n: Vec<i32>,
        reset: i32,
        j: [i32; 32],
        run_index: usize
    );

    // FROZEN MEASURED LAYOUT (Rust default field order, 2026-08-27):
    //   a@0..24 (line 0), b@24..48 (line 0), c@48..72 (lines 0..1),
    //   n@72..96 (line 1), j@96..224 (lines 1..3), run_index@224,
    //   t1/t2/t3/max_val/reset@232..252 (line 3), size 256, align 8.
    //
    // RECORDED FINDING: the bias-correction vector header `c` (read on
    // EVERY regular-mode pixel as ctx.c[q]) sits ACROSS the 64B
    // boundary. The gate freezes this fact - a stage-2 layout anchor
    // could move the four hot scalars first and keep every Vec header
    // inside a single cache line.
    let c = SPANS.iter().find(|s| s.name == "c").expect("span c");
    assert!(
        c.crosses_line(),
        "frozen finding: ContextModel.c must still cross lines 0..1"
    );

    // The other three statistics Vec headers sit inside single lines.
    for name in ["a", "b", "n"] {
        let span = SPANS.iter().find(|s| s.name == name).expect("vec span");
        assert!(
            !span.crosses_line(),
            "ContextModel.{name} must stay within one cache line"
        );
        assert!(span.size <= LINE, "ContextModel.{name} exceeds one line");
    }

    // Hot scalars (t1/t2/t3/max_val/reset) share line 3, never split.
    for name in ["t1", "t2", "t3", "max_val", "reset"] {
        let span = SPANS.iter().find(|s| s.name == name).expect("scalar span");
        assert!(!span.crosses_line(), "ContextModel.{name} splits");
    }

    // Whole struct: no more than 4 cache lines.
    assert!(
        core::mem::size_of::<crate::context::ContextModel>() <= 4 * LINE,
        "ContextModel grew past 4 cache lines"
    );
}

// ===========================================================================
// G2/G3 shared helpers
// ===========================================================================

/// Replicates upstream `effective_precision` (encode.rs) so tests can
/// reconstruct the SAME max_val the encoder embedded in the stream.
fn precision_of(max_pixel: u16) -> i32 {
    if max_pixel == 0 {
        return 8;
    }
    let bits_needed = 16 - max_pixel.leading_zeros(); // u16: 0..16, as upstream
    bits_needed.clamp(8, 16) as i32
}

fn max_val_of(max_pixel: u16) -> i32 {
    (1i32 << precision_of(max_pixel)) - 1
}

/// Deterministic synthetic image (ramp + plateaus) exercising borders,
/// gradients and context spread - 64x64, 8-bit range.
fn synthetic_image_8bit() -> Vec<u16> {
    const W: usize = 64;
    const H: usize = 64;
    let mut px = vec![0u16; W * H];
    for y in 0..H {
        for x in 0..W {
            px[y * W + x] = (((x * 3 + y * 5) % 200) + (y / 8) * 20) as u16;
        }
    }
    px
}

/// Locates the scan data of an encoded JPEG-LS stream: FF DA <len> ...
/// returns the byte offset where entropy-coded scan data begins.
fn scan_data_offset(buf: &[u8]) -> usize {
    for i in 0..buf.len().saturating_sub(4) {
        if buf[i] == 0xFF && buf[i + 1] == 0xDA {
            let len = u16::from_be_bytes([buf[i + 2], buf[i + 3]]) as usize;
            return i + 2 + len;
        }
    }
    panic!("SOS marker not found in encoded stream");
}

// ===========================================================================
// G2: FULL-DECODE HEAP - "the honest non-zero path"
// ===========================================================================

#[test]
fn full_decode_allocates_exactly_declared_buffers() {
    let image = synthetic_image_8bit();
    let w = 64u32;
    let h = 64u32;

    // Encode OUTSIDE the measurement window (one-time fixture).
    let mut buf: Vec<u8> = Vec::new();
    crate::encode::encode(&image, w, h, &mut buf).expect("encode fixture");

    let alloc = TestAllocator::new();
    alloc.set_enabled(true);
    alloc.reset();

    let (pixels, dw, dh) = crate::decode::decode(&buf, w, h).expect("decode must succeed");

    let (count, bytes) = alloc.snapshot();
    alloc.set_enabled(false);

    assert_eq!((dw, dh), (w, h));
    assert_eq!(pixels, image, "roundtrip parity through vendored codec");

    // DECLARED: 7 allocations total (pixels 1 + context A/B/C/N 4 +
    // curr/prev lines 2). The byte counter is LIVE-BYTES (dealloc
    // decrements), so at return only the returned pixels buffer
    // remains outstanding: bytes == 2*w*h. Every internal buffer was
    // freed before decode returned.
    let expect_bytes = 2 * (w as usize) * (h as usize);
    assert_eq!(
        count, 7,
        "decode allocation count diverged from the declared 7"
    );
    assert_eq!(
        bytes, expect_bytes,
        "decode allocation bytes diverged from the declared formula"
    );
}

#[test]
fn allocation_count_is_size_independent() {
    // If ANY allocation scaled with the pixel count, the count would
    // differ between a small and a large image. It must stay 7.
    let alloc = TestAllocator::new();

    for (w, h) in [(8u32, 8u32), (96u32, 96u32)] {
        let image: Vec<u16> = (0..w * h).map(|i| (i % 251) as u16).collect();
        let mut buf: Vec<u8> = Vec::new();
        crate::encode::encode(&image, w, h, &mut buf).expect("encode");

        alloc.set_enabled(true);
        alloc.reset();
        crate::decode::decode(&buf, w, h).expect("decode");
        let (count, _bytes) = alloc.snapshot();
        alloc.set_enabled(false);

        assert_eq!(
            count, 7,
            "{w}x{h}: decode allocation count must be size-independent (7)"
        );
    }
}
