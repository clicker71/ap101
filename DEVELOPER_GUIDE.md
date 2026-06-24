# AP-101B Ferrite Discipline — Developer Guide

**Form:** AP101-DG-01-EN | **Version:** 1.0 | **Date:** 2026-06-24

---

## 0. Overview

This guide assumes you have added `ferrite-core` and `ferrite-testkit` to your
project. If not:

```toml
[dependencies]
ferrite-core = { git = "https://github.com/clicker71/ap101", subdir = "codebase/ferrite-core" }

[dev-dependencies]
ferrite-testkit = { git = "https://github.com/clicker71/ap101", subdir = "codebase/ferrite-testkit" }
```

After adding, verify:

```bash
cargo test
```

The first run downloads and compiles the crates. No additional configuration
required. If `cargo test` passes, the harness is wired correctly.

---

## 1. The Three Pillars

Every AP-101 test suite exercises three independent properties of your code:

| Pillar | What it checks | Failure means |
|--------|---------------|---------------|
| **Geometry & Padding** | Struct size, alignment, hidden gaps | Your structs waste stack or leak layout |
| **Zero Heap** | Hot-path functions allocate nothing | Hidden `String`/`Vec`/`Box` in parser |
| **SEU Resilience** | Bit-flip detection works | Silent data corruption passes undetected |

You do not need all three. Pick the pillar relevant to your domain.

---

## 2. Pillar 1: Structural Geometry

### 2.1 Compile-time: `assert_no_padding!`

Catches hidden alignment gaps at compile time. Place it in your test module:

```rust
use ferrite_core::assert_no_padding;

#[repr(C)]
struct NavigationPacket {
    x: f64,
    y: f64,
    z: f64,
    status: u32,
}

assert_no_padding!(NavigationPacket);
```

If `NavigationPacket` has hidden padding (e.g. 4 bytes after `status`),
compilation fails with a clear message:

```
error: struct NavigationPacket has 4 bytes of hidden padding
```

**When to use:** Any `#[repr(C)]` struct crossing FFI boundaries, serialized
to disk/network, or mapped to hardware registers.

### 2.2 Runtime: `audit_exact_size`

Verifies struct size at runtime via `cargo test`:

```rust
use ferrite_core::audit::audit_exact_size;

#[test]
fn my_struct_has_correct_size() {
    let report = audit_exact_size::<NavigationPacket>(32);
    assert!(report.compliant);
}
```

**When to use:** Structs that must match a fixed byte budget (e.g. 256-byte
telemetry frames, 40-byte chunk records).

---

## 3. Pillar 2: Zero-Heap Enforcement

### 3.1 `TestAllocator` — Catch Hidden Allocations

Wrap your test with `TestAllocator` to ban heap usage:

```rust
use ferrite_testkit::heap::TestAllocator;

#[global_allocator]
static ALLOC: TestAllocator = TestAllocator;

#[test]
fn parse_packet_must_not_allocate() {
    let bytes: [u8; 64] = [0; 64];
    TestAllocator::enable();          // START surveillance
    let result = parse_packet(&bytes);
    TestAllocator::disable();         // STOP surveillance
    assert!(result.is_ok());
    // If parse_packet allocated a String/Vec/Box, the test panics
    // during TestAllocator::disable().
}
```

**When to use:** Hot-path parsing functions, interrupt handlers, telemetry
loops. Any code path where allocation = unacceptable latency or OOM risk.

### 3.2 What Counts as an Allocation

- `String::new()`, `String::from("...")`, `format!(...)`
- `Vec::new()`, `vec![...]`, `.collect::<Vec<_>>()`
- `Box::new(...)`
- Any type implementing `std::alloc::GlobalAlloc`

Stack-only types are fine: `[u8; N]`, `heapless::String<N>`, `heapless::Vec<T, N>`.

---

## 4. Pillar 3: SEU (Single-Event Upset) Resilience

### 4.1 `FerriteCell<T>` — Controlled-Access Memory Cell

```rust
use ferrite_core::cell::FerriteCell;

let cell = FerriteCell::new(0xDEADBEEFu32);
assert_eq!(cell.read_copy(), 0xDEADBEEF);

// SAFETY: single-threaded test, no concurrent readers.
unsafe { cell.write_volatile(0xCAFEBABE); }
assert_eq!(cell.read_copy(), 0xCAFEBABE);
```

### 4.2 `inject_cosmic_ray` — Simulate Bit-Flips

```rust
use ferrite_core::cell::FerriteCell;
use ferrite_testkit::strategy::seu_strategy;
use proptest::prelude::*;

proptest! {
    #[test]
    fn checksum_detects_any_single_bit_flip(
        flip_mask in seu_strategy::<u32>()
    ) {
        let original: u32 = 0x12345678;
        let cell = FerriteCell::new(original);
        unsafe { cell.inject_cosmic_ray(flip_mask); }
        let corrupted = cell.read_copy();

        let checksum_before = crc32(&original.to_le_bytes());
        let checksum_after  = crc32(&corrupted.to_le_bytes());

        assert_ne!(checksum_before, checksum_after,
            "CRC-32 failed to detect bit-flip: {} -> {}",
            original, corrupted);
    }
}
```

**When to use:** Storage integrity checks, telemetry CRC verification, ECC
validation. Any code path where undetected corruption = mission failure.

### 4.3 Running SEU Tests

SEU tests use `proptest`. By default, proptest runs 256 cases. For production
validation, run 10 000:

```bash
PROPTEST_CASES=10000 cargo test seu
```

This takes 2-15 seconds depending on struct size. CI should run with
`PROPTEST_CASES=10000` as a separate job (see Section 6).

---

## 5. Telemetry Output

AP-101 tests produce IBM PASS CRT-style telemetry. You do not need to format
output manually — use `IbmCrt`:

```rust
use ferrite_core::telemetry::IbmCrt;

IbmCrt::print_header("MY SYSTEM", "TARGET: CRITICAL PATH");

let report = audit_exact_size::<MyStruct>(64);
IbmCrt::print_row("MYSYS-", "01", "MyStruct Geometry", report.compliant,
    &format!("{}B", report.size_bytes));

IbmCrt::print_footer(true);  // true = GO FOR LAUNCH
```

Output:

```
[ COMPLIANT ] MYSYS-01 | MyStruct Geometry              | 64B
MISSION STATUS: GO FOR LAUNCH.
```

If a check fails:

```
[ANOMALY DET] MYSYS-01 | MyStruct Geometry              | 72B (expected 64B)
MISSION STATUS: ABORT. VIOLATION DETECTED.
```

---

## 6. CI/CD: GitHub Actions Template

Add to `.github/workflows/ci.yml`:

```yaml
name: AP-101 Ferrite Discipline

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo test

  no-std-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          target: thumbv7m-none-eabi
      - run: cargo check -p ferrite-core --no-default-features --target thumbv7m-none-eabi

  seu-10k:
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: PROPTEST_CASES=10000 cargo test seu
```

**Badge** (add to your README):

```markdown
[![AP-101](https://img.shields.io/badge/AP--101-GO%20FOR%20LAUNCH-brightgreen)](https://github.com/clicker71/ap101)
```

---

## 7. Understanding Test Failures

### 7.1 `ANOMALY DET` on Struct Geometry

```
[ANOMALY DET] AP101B-CORE-10 | Patient Geometry    | 112B
```

This means `audit_exact_size::<Patient>(0)` returned `compliant = false`.
The struct is larger than expected. Common causes:

- **String/Option fields:** These contain heap pointers (8 bytes each on
  64-bit). Not a bug if the struct is a DTO — domain models carrying
  variable-length data are expected to have String fields.
- **Hidden padding:** The compiler inserted alignment bytes between fields.
  Use `#[repr(C)]` or reorder fields (largest alignment first) to eliminate.
- **Oversized integer:** `usize` on 64-bit is 8 bytes vs `u32` at 4 bytes.

### 7.2 Test Panic During `TestAllocator::disable()`

```
panicked at 'ALLOCATION DETECTED: 24 bytes at 0x...'
```

A heap allocation occurred in the guarded code block. Check:

- Are you using `format!()` or `String::from()`?
- Did `.collect::<Vec<_>>()` sneak in?
- Is a dependency allocating internally?

Use `heapless::String<N>` and `heapless::Vec<T, N>` as replacements.

### 7.3 SEU Test Flakiness

If SEU tests sometimes pass and sometimes fail, the checksum algorithm may
have collisions. Verify:

- CRC-32 polynomial is correct (0xEDB88320 for standard CRC-32)
- All bytes are covered by the checksum
- Test uses sufficient `PROPTEST_CASES` (256 minimum, 10 000 recommended)

---

## 8. Feature Flags

`ferrite-core` is `no_std` by default. Enable `std` for telemetry output:

```toml
[dependencies]
ferrite-core = { git = "...", subdir = "...", features = ["std"] }
```

Without `std`:
- `IbmCrt::print_*` methods are unavailable (print to stdout requires std)
- `AuditReport` and `AuditResult` work fully in no_std (heapless-based)
- Compile on `thumbv7m-none-eabi` and other embedded targets

---

## 9. Common Pitfalls

| Pitfall | Symptom | Fix |
|---------|---------|-----|
| **Wrong `subdir` path** | Cargo: `package not found` | Use `subdir = "codebase/ferrite-core"` (not `ferrite-core`) |
| **Missing `std` feature** | `IbmCrt` not found | Add `features = ["std"]` to Cargo.toml |
| **`assert_no_padding!` on DTO** | Compile error on String struct | Use runtime `audit_exact_size` instead for DTOs |
| **`TestAllocator` in integration test** | `#[global_allocator]` conflict | Use in unit tests only (integration tests have their own allocator) |
| **SEU test timeout in CI** | `PROPTEST_CASES=10000` takes >60s | Set `PROPTEST_CASES=256` for PR checks, 10000 for merge queue |
| **Git dependency not updating** | Old code after `cargo update` | `cargo update -p ferrite-core` to force refresh |

---

## 10. Reference Projects

Study real-world AP-101 test suites:

| Project | What it audits | Where |
|---------|---------------|-------|
| **ap101b-core** | Self-test: geometry, padding, SEU, checksums | `codebase/examples/ap101b-core/` |
| **ap101s-cmos** | S-model: ECC, multi-bit SEU, battery retention | `codebase/examples/ap101s-cmos/` |
| **clarus-audit** | Clarus PACS: chunk store, DICOM elements, instance meta | `codebase/examples/clarus-audit/` |
| **Clarus integration test** | Cross-repo: domain models, storage, charset, JPEG2000 | `github.com/clarus-pacs/clarus` → `crates/clarus-testing/tests/ap101_audit.rs` |

---

## 11. Quick Reference Card

```
ADD DEPENDENCY:
  Cargo.toml: [dependencies] ferrite-core = { git = "...", subdir = "codebase/ferrite-core" }

CHECK GEOMETRY:
  assert_no_padding!(MyStruct);            // compile-time
  audit_exact_size::<MyStruct>(32);        // runtime

BAN HEAP:
  TestAllocator::enable();
  // ... code under test ...
  TestAllocator::disable();

SIMULATE SEU:
  unsafe { my_cell.inject_cosmic_ray(mask); }

PRINT TELEMETRY:
  IbmCrt::print_header("SYSTEM", "TARGET");
  IbmCrt::print_row("ID-", "01", "Check Name", passed, &detail);
  IbmCrt::print_footer(all_clear);

RUN SEU:
  PROPTEST_CASES=10000 cargo test seu
```
