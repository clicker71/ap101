# AP-101 CODEC-PARITY: JPEG-LS DECODE AUDIT (STAGE 1 REPORT)

DATE: 2026-08-27
TARGET: pure_jpegls 1.0.0 (crates.io, MIT OR Apache-2.0, JP Fielding)
RESULT: 424/424 BYTE-IDENTICAL, ALL AUDIT GATES GREEN

ASCII-ONLY BY DISCIPLINE. NO EMOJI, NO NON-ASCII.

## 1. What was built (ap101, three new crates)

| crate | role |
|-------|------|
| examples/jpegls-driver | one DICOM/JPEG-LS file -> header line + raw u16 LE pixels on stdout; byte-exact, machine-independent output; links crates.io pure_jpegls 1.0.0 |
| examples/jpegls-parity | `codec-parity <dir> <ref-cmd> <test-cmd> [--max N] [--ext E]`: runs BOTH commands per file, byte-compares stdout (header + every pixel byte), prints an audit table (file / dims / ref-us / test-us / MATCH / note), exits non-zero on any mismatch |
| examples/jpegls-geometry | VENDORED COPY of pure_jpegls 1.0.0 sources (provenance: VENDOR.md) + audit gates G1-G4 |

## 2. The parity byte-diff is the authority

CRC/SEU machinery (FerriteCell, Checksum) is deliberately NOT used here:
a raw byte-diff of every decoded pixel buffer is the strongest check
available, and SEU injection is irrelevant for a codec hot path.

## 3. Stage-1 gate results (2026-08-27)

| gate | verdict | detail |
|------|---------|--------|
| corpus parity | 424/424 byte-identical | field DICOM/JPEG-LS corpus, 776x776x16-bit monochrome, 215 MB; ref wall 38.10 s vs test wall 36.22 s (release build) |
| BitReader geometry | one 64B cache line | 40 bytes; touched on EVERY bit read; zero split fields |
| Decoder geometry | one 64B cache line | 24 bytes; marker/byte walker |
| ContextModel geometry | 256 bytes, 4 lines, no padding | FINDING: `c` (bias correction Vec header, read every pixel as ctx.c[q]) sits ACROSS the 64B boundary at offset 48..72 |
| full-decode heap | exactly 7 allocations | pixels(1) + context A/B/C/N(4) + curr/prev row buffers(2); live bytes at return = pixels only |
| scan-loop heap | exactly 2 allocations | curr_line + prev_line, both freed before return; NOTHING allocated per pixel |
| included-vs-canonical | byte-identical | the include!-ed audit copy and the crate-root vendored copy agree |
| ap101 workspace | green | cargo test --workspace: all suites pass, incl. upstream pure_jpegls roundtrip tests run from the vendored copy |
| fmt | clean | cargo fmt --check |

## 4. How to run

    # driver
    cargo run --release -p jpegls-driver -- file.dcm > out.bin

    # parity over a corpus
    cargo run --release -p jpegls-parity -- \
        codec-parity /path/to/corpus \
        /path/to/reference-driver /path/to/tested-driver

    # audit gates (geometry + heap)
    cargo test -p jpegls-geometry

## 5. Finding: ContextModel.c crosses a cache line

Measured layout (Rust default field order, frozen in gate
`contextmodel_geometry`):

    a@0..24 (line 0), b@24..48 (line 0), c@48..72 (lines 0..1),
    n@72..96 (line 1), j@96..224 (lines 1..3), run_index@224..232,
    t1/t2/t3/max_val/reset@232..252 (line 3), size 256, align 8.

`c` is the bias-correction vector; its header (ptr/len/cap) is read on
every regular-mode pixel. A split header costs an extra line fill when
the header is evicted. Recorded as a candidate stage-2 layout anchor:
moving the four hot scalars before the four Vec headers keeps every
Vec header inside a single cache line.

## 6. Stage 2 outcome (measured 2026-08-27): NOT ATTEMPTED

Rule: profile first, code second; if the expected win is < 1.5x, no PR.

Profile (instrumented workdir copy, RDTSC counters, 5 largest corpus
files x 4 repeats, medians): read_golomb 38.2%, quantize_gradient 18.9%,
fill 14.5% (inside read_golomb), border branches 4.5%, duplicate row
write 0.17%, untouched pixel body (predictor + clamp + context update +
writes) 38.2%.

Measured experiment (anchors 1+2 only, private workdir): upstream tests
61/61, codec-parity 424/424, bench n=50 median 2338 -> 2153 ms = 1.086x.

Projection for all five anchors, optimistic ceilings: ~1.26x, at most
~1.3x - the 38% body caps everything below 1.5x.

VERDICT: profile showed < 1.5x - NOT DOING IT. No PR, no fork; the
codec-parity harness remains ap101's own verification tool.
