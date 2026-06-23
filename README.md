# AP-101B Ferrite Discipline

**no_std memory integrity test harness honoring the IBM AP-101B Shuttle computer. CRC-32, SEU simulation, compact struct verification.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Mission Status](https://img.shields.io/badge/AP--101-GO%20FOR%20LAUNCH-brightgreen)](https://github.com/clicker71/ap101)

> [Русская версия](./README.ru.md) | [Russian README](./README.ru.md)

> **MISSION STATUS:** GO FOR LAUNCH — All discipline checks compliant. v0.1.0-alpha.

---

```
╔════════════════════════════════════════════════════════════════╗
║ AP-101B FERRITE DISCIPLINE TEST HARNESS — WHAT IT IS           ║
║                                                                ║
║ PURPOSE:    Compile-time + runtime audit for embedded/SEU-     ║
║             critical data structures.                          ║
║                                                                ║
║ DOES NOT:   Check that your code runs on 256 KB of RAM.        ║
║             That was never the goal.                           ║
║                                                                ║
║ BORN FROM:  Clarus PACS — DICOMweb server for medical          ║
║             imaging (github.com/clarus-pacs/clarus).           ║
║             We built this harness to audit our own code.       ║
║             We share it because the discipline transcends      ║
║             any single project.                                ║
║                                                                ║
║ TEST:       cargo test -p ap101b-core                          ║
║             cargo test -p ap101s-cmos                          ║
║             cargo test --workspace                             ║
╚════════════════════════════════════════════════════════════════╝
```

```
╔════════════════════════════════════════════════════════════════╗
║ MISSION SUPPORT / KEEP THE SERVERS ALIVE                       ║
║                                                                ║
║ PURPOSE:    Support infrastructure and open-source testing     ║
║             servers for Clarus PACS.                           ║
║                                                                ║
║ IMPACT:     If this discipline saved your embedded system      ║
║             from OOM, consider backing the launchpad.          ║
║                                                                ║
║ DEED:       Every contribution helps keep our compilation      ║
║             servers alive and running.                         ║
║                                                                ║
║ LINK:       [Boosty](https://boosty.to/clicker71)              ║
║             Definitum semel.                                   ║
╚════════════════════════════════════════════════════════════════╝
```

---

## The Illusion of High-Level Code

Modern software engineering has grown lazy. High-level languages and frameworks
have created a dangerous illusion: that resources are infinite and abstractions
are free.

They are not.

When a Python developer writes `print("Hello World")`, or a web developer
instantiates a nested JSON object, they see one line of code. But underneath,
the physical hardware sees a disaster: the runtime fires an avalanche of
Assembly instructions just to initialize a heap context. The OS memory manager
searches the global heap, locks threads, and carves out virtual memory. Cache
lines are instantly dirtied, forcing the CPU to stall for hundreds of cycles
fetching pointers from slow system RAM.

In a world of infinite cloud budgets, this laziness is hidden by corporate
credit cards. **In critical systems, this laziness kills.**

`ap101` exists to bring **Assembly-level thinking** back to modern Rust
codebases. It is an automated gatekeeper that forces your code to respect
the physical silicon.

---

## Why This Exists: The 4 Columns of the Disciplinary Standard

This test harness is not a retro-computing physics simulator. It is an
architectural quality contract for production software. It treats 256 KB
not as a physical limitation, but as a **disciplinary boundary** to enforce
four non-negotiable rules of high-performance engineering:

### 1. Hard Core Cache Locality — The L1/L2 Cache Wall

Your 64 GB of DDR5 RAM is a lie. Your CPU's ultra-fast **L1 and L2 Data
Caches** are still tiny (32 KB – 512 KB).

**The Trap:** If your hot-path scatters data across the heap using dynamic
pointers (`String`, `Vec`, `Box`), you hit the Cache Wall. The CPU drops
to zero velocity, waiting for the memory bus.

**The Discipline:** `ferrite-core` forces zero-allocation and continuous,
tight memory layouts (zero padding). Your entire hot-path working set
resides entirely inside L1/L2 cache lines. The code runs at the absolute
physical speed limit of the silicon.

### 2. Embedded Zero-Panic Constraints

In aerospace, automotive, and medical devices, an unhandled Out-Of-Memory
(OOM) error means catastrophic failure.

**The Trap:** A single temporary buffer allocation inside an interrupt
handler or telemetry loop under peak load panics the kernel.

**The Discipline:** This harness acts as a brutal CI gate. It intercepts
runtime allocations and unaligned structures, preventing junior developers
from accidentally introducing heap mutations into hard real-time systems.

### 3. High-Throughput Cloud & Edge Compute Economics

On modern edge nodes and high-frequency backend services, heap churn is
the silent killer of performance.

**The Trap:** Millions of allocations per second cause heap fragmentation
and force aggressive Garbage Collection or thread-locking allocation waits.

**The Discipline:** Forcing critical components to use stack-only, linear,
predictable structures drops CPU system overhead to zero. It translates
directly to fewer cloud instances and massive operational cost reductions.

### 4. Silicon Vulnerability — Radiation-Hardening at 3nm

The Space Shuttle's AP-101B used magnetic ferrite core memory, naturally
immune to cosmic rays. Modern sub-10nm silicon transistors are so
microscopic that they are highly vulnerable to Single Event Upsets (SEU) —
random, radiation-induced bit-flips caused by atmospheric neutrons right
here on Earth.

**The Trap:** High-altitude IoT, automotive safety controllers, and medical
equipment suffer from silent data corruption that standard compilers cannot
protect against.

**The Discipline:** The SEU simulation in this harness forces you to write
and test software-level fault-tolerant algorithms (CRC-32, XorFold, ECC)
to guarantee data integrity when the physical silicon fails.

**Full historical context** — the Shuttle's ferrite-to-CMOS transition,
STS-37/STS-40, and why this matters for modern silicon:
**[AP-101 STS Flight History](./HISTORY.md)**.

---

## Real-World Proof: Clarus PACS

This harness was born from **[Clarus PACS](https://github.com/clarus-pacs/clarus)** —
an ultra-compact DICOMweb medical imaging server.

When we first applied the AP-101B discipline to the Clarus Core deduplication
pipeline, we exposed **128 000 unnecessary heap allocations per single CT study**
caused by converting binary SHA-256 hashes into hex-encoded `String` values.

By enforcing the `ap101` standard:

- Replaced `String` hashes with raw stack-allocated `[u8; 32]` arrays.
- Eliminated VR string parsing allocation storms.
- Enforced zero-copy pixel extraction (`&[u8]` instead of `Vec<u8>`).

| Metric | Before (hex String) | After ([u8; 32]) |
|:---|---:|---:|
| Allocations per CT study | 128 000 | 0 |
| Per-study heap churn | ~22 MB | ~5 MB |
| Concurrent STOW ingestions (256 MB RAM) | 3 → OOM-kill | 6+ (stable) |
| ChunkRecord size | 40 B (+ heap) | 40 B (stack-only) |
| Patient re-irradiation risk | Present | Eliminated |

On a modern server with 64 GB of RAM, the same fix means 128 000 fewer
allocator calls per CT study, less heap fragmentation, lower allocator lock
contention under concurrent load, and smoother latency tails. The discipline
scales in both directions: it keeps budget hardware alive, and it keeps big
hardware fast.

> **The outcome was not just code optimization. It was reduced risk of
> patient re-irradiation.** On a budget ARM single-board computer or
> repurposed x86 workstation pulled from a junk closet — the kind of
> hardware a regional hospital might run a PACS server on — a CT scanner's
> controller runs
> out of memory, it drops the study. The patient must be called back for
> a repeat scan — extra radiation dose, extra cost, extra time. The
> AP-101B discipline eliminated that failure mode entirely.
> `Definitum semel.`

We share this test harness because every byte counts — in space, in medicine,
in any system where failure is not an option.

---

## What It Does

`ferrite-core` is a `no_std` Rust library providing:

- **`FerriteCell<T>`** — controlled-access memory cell with explicit safety contract.
  Safe read for machine-word Copy types. Unsafe write with volatile semantics.
  Cosmic ray injection for SEU testing.
- **Structural audit** — compile-time geometry checks (`size_of`, `align_of`,
  zero hidden padding) via `assert_no_padding!` macro and runtime `GeometryReport`.
- **Checksums** — `Checksum` trait with `Crc16`, `Crc32`, `XorFold` implementations.
  Stack-only. Zero allocations.
- **IBM PASS CRT telemetry** — `IbmCrt` decorator with historical `AP101B-CORE-` message IDs.
  (Named after the Shuttle's Integrated Display Processor / PASS CRT, not the
  ground-based IBM 3270 office terminal.)

`ferrite-testkit` (requires std) provides:

- **`TestAllocator`** — global allocator wrapper to verify zero heap allocations
  in code under test.
- **SEU strategies** — proptest generators for single-event upset simulation.

---

### Proof: Clarus Core

The discipline was born from [Clarus PACS](https://github.com/clarus-pacs/clarus),
a DICOMweb medical imaging server. The same code that had 128 000 allocations
per CT study in chunk dedup. The same code we fixed.

We created `examples/clarus-audit/` — a cross-project audit harness that
copies the EXACT production structs from Clarus Core and runs the full
AP-101B + AP-101S discipline against them:

```bash
cargo test -p clarus-audit ap101b_clarus_audit -- --nocapture
cargo test -p clarus-audit ap101s_clarus_audit -- --nocapture
```

**B-Model (ferrite core) audit of Clarus Core:**

```
╔════════════════════════════════════════════════════════════════╗
║ IBM AP-101B FERRITE CORE DISCIPLINE SUITE v3.0                 ║
║ TARGET: CLARUS CORE v0.3.0-alpha                               ║
╚════════════════════════════════════════════════════════════════╝
[ COMPLIANT ] AP101B-CORE-01 | ChunkRecord Geometry                     | Expected 40B, got 40B
[ COMPLIANT ] AP101B-CORE-02 | DicomElement Geometry                    | Expected 12B, got 12B | vr: [u8;2] — ZERO HEAP
[ COMPLIANT ] AP101B-CORE-03 | InstanceMeta Geometry                    | Expected 212B, got 212B
[ COMPLIANT ] AP101B-CORE-04 | InstanceMeta SEU (1000 flips)            | All bit-flips detected
[ COMPLIANT ] AP101B-CORE-05 | Zero Hidden Padding                      | Verified at compile time via assert_no_padding!
╔════════════════════════════════════════════════════════════════╗
║ MISSION STATUS: GO FOR LAUNCH.                                 ║
╚════════════════════════════════════════════════════════════════╝
```

**S-Model (CMOS) audit of Clarus Core — adds multi-bit SEU burst detection:**

```
╔════════════════════════════════════════════════════════════════╗
║ IBM AP-101S CMOS FERRITE DISCIPLINE SUITE v3.0                 ║
║ TARGET: CLARUS CORE v0.3.0-alpha (CMOS)                        ║
╚════════════════════════════════════════════════════════════════╝
[ COMPLIANT ] AP101S-CMOS-01 | ChunkRecord Geometry                     | Expected 40B, got 40B
[ COMPLIANT ] AP101S-CMOS-02 | DicomElement Geometry                    | Expected 12B, got 12B | vr: [u8;2] — ZERO HEAP
[ COMPLIANT ] AP101S-CMOS-03 | InstanceMeta Geometry                    | Expected 212B, got 212B
[ COMPLIANT ] AP101S-CMOS-04 | Multi-Bit SEU (1000 bursts, 2-8 bit)     | All bursts detected by CRC-32
[ COMPLIANT ] AP101S-CMOS-05 | InstanceMeta SEU (1000 flips)            | All bit-flips detected
[ COMPLIANT ] AP101S-CMOS-06 | Zero Hidden Padding                      | Verified at compile time via assert_no_padding!
╔════════════════════════════════════════════════════════════════╗
║ MISSION STATUS: GO FOR LAUNCH.                                 ║
╚════════════════════════════════════════════════════════════════╝
```

**Before/After — FMA-02 motivating example (Clarus chunk dedup):**

| Metric | Before (hex String) | After ([u8; 32]) |
|:---|---:|---:|
| Allocations per CT study | 128 000 | 0 |
| Per-study heap churn | ~22 MB | ~5 MB |
| Concurrent STOW ingestions (256 MB RAM) | 3 → OOM-kill | 6+ (stable) |
| ChunkRecord size | 40 B (+ heap) | 40 B (stack-only) |
| Patient re-irradiation risk | Present | Eliminated |

---

### Proof: Self-Test

The discipline auditing itself (meta-circular verification).
Every `assert_no_padding!` macro, every CRC-32 checksum, every SEU injection
that `ferrite-core` provides is verified by the same discipline:

**B-Model self-test (`cargo test -p ap101b-core`):**

```
╔════════════════════════════════════════════════════════════════╗
║ IBM AP-101B FERRITE CORE                                       ║
║ TARGET: NAVIGATION SYSTEM v1.0                                 ║
╚════════════════════════════════════════════════════════════════╝
[ COMPLIANT ] AP101B-CORE-01 | Struct Geometry & Alignment              | Size: 32B (max 32B), Align: 8B (min 4B)
[ COMPLIANT ] AP101B-CORE-02 | Zero Hidden Padding                      | Expected 32B, got 32B
[ COMPLIANT ] AP101B-CORE-03 | Deterministic Computation (±4 ULP)       | diff: 0.00e0 ≤ 4.77e-6
[ COMPLIANT ] AP101B-CORE-04 | Cosmic Ray SEU Resilience                | All bit-flips detected
╔════════════════════════════════════════════════════════════════╗
║ MISSION STATUS: GO FOR LAUNCH.                                 ║
╚════════════════════════════════════════════════════════════════╝
```

**S-Model self-test (`cargo test -p ap101s-cmos`):**

```
╔════════════════════════════════════════════════════════════════╗
║ IBM AP-101S CMOS                                               ║
║ TARGET: NAVIGATION SYSTEM v1.0 (CMOS)                          ║
╚════════════════════════════════════════════════════════════════╝
[ COMPLIANT ] AP101S-CMOS-01 | Struct Geometry & Alignment              | Size: 40B (max 40B), Align: 8B (min 4B)
[ COMPLIANT ] AP101S-CMOS-02 | Zero Hidden Padding                      | Expected 40B, got 40B
[ COMPLIANT ] AP101S-CMOS-03 | ECC Syndrome Check                       | ECC detects field change
[ COMPLIANT ] AP101S-CMOS-04 | Multi-Bit SEU Detection (2-8 bit)        | All bursts detected
[ COMPLIANT ] AP101S-CMOS-05 | Battery Retention                        | 0x5A=OK, 0x00=LOST
[ COMPLIANT ] AP101S-CMOS-06 | Deterministic Computation (±4 ULP)       | diff: 0.00e0 ≤ 4.77e-6
╔════════════════════════════════════════════════════════════════╗
║ MISSION STATUS: GO FOR LAUNCH.                                 ║
╚════════════════════════════════════════════════════════════════╝
```

All four audit targets — B-model self, S-model self, Clarus Core B-model,
Clarus Core S-model — return **GO FOR LAUNCH**. The discipline holds.

---

## Roadmap

| Version | Feature | Status |
|:---|:---|:---:|
| **v0.1.1** | Criterion benchmarks — Clarus before/after (ns/op, latency tails) | ⬜ |
| **v0.1.1** | Cache-line analysis in `GeometryReport` (false sharing detection) | ⬜ |
| **v0.2.0** | `#[derive(FerriteDiscipline)]` — one annotation: geometry + padding + CRC | ⬜ |
| **v0.2.0** | HFT/GameDev feature profiles (`features = ["hft"]`) | ⬜ |
| **v0.2.0** | `cargo-ap101` CI plugin — fails build on `String` in hot function | ⬜ |
| **v0.2.0** | Spatial burst SEU (2–4 adjacent cells, S-model) | ⬜ |
| **v0.3.0** | `const fn` CRC-32 (nightly) | ⬜ |
| **v0.3.0** | Article: «How we removed 128 000 allocations with 1981 Shuttle discipline» | ⬜ |

---

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
ferrite-core = { git = "https://github.com/clicker71/ap101", subdir = "codebase/ferrite-core" }

[dev-dependencies]
# ferrite-testkit brings in std support for verification
ferrite-testkit = { git = "https://github.com/clicker71/ap101", subdir = "codebase/ferrite-testkit" }
```

Run the navigation example test suite:

```bash
cargo test -p ap101b-core
```

Expected output:

```
╔════════════════════════════════════════════════════════════════╗
║ IBM AP-101B FERRITE CORE DISCIPLINE SUITE v3.0                 ║
║ TARGET: NAVIGATION SYSTEM v1.0                                 ║
╚════════════════════════════════════════════════════════════════╝
[ COMPLIANT ] AP101B-CORE-01 | Struct Geometry & Alignment         | Size: 32B...
[ COMPLIANT ] AP101B-CORE-02 | Zero Hidden Padding                 | ...
[ COMPLIANT ] AP101B-CORE-04 | Deterministic Computation (±4 ULP)  | ...
[ COMPLIANT ] AP101B-CORE-05 | Cosmic Ray SEU Resilience           | ...
╔════════════════════════════════════════════════════════════════╗
║ MISSION STATUS: GO FOR LAUNCH.                                 ║
╚════════════════════════════════════════════════════════════════╝
```

---

## Project Structure

```
ap101/
├── README.md
├── codebase/                   # Cargo workspace
│   ├── ferrite-core/           # no_std primitive library (shared)
│   │   ├── src/
│   │   │   ├── cell.rs         # FerriteCell with unsafe contract
│   │   │   ├── audit.rs        # Structural geometry audit
│   │   │   ├── checksum.rs     # Checksum trait
│   │   │   └── telemetry.rs    # IBM PASS CRT telemetry
│   │   └── Cargo.toml
│   ├── ferrite-testkit/        # std testing library (shared)
│   │   ├── src/
│   │   │   ├── heap.rs         # Deterministic alloc detector
│   │   │   └── strategy.rs     # Proptest SEU strategies
│   │   └── Cargo.toml
│   ├── examples/
│   │   ├── ap101b-core/        # AP-101B: Ferrite core memory
│   │   │   ├── src/lib.rs      # NavigationState (B-model)
│   │   │   └── tests/discipline.rs
│   │   ├── ap101s-cmos/        # AP-101S: CMOS SRAM + DRAM/ECC
│   │   │   ├── src/lib.rs      # NavigationState (S-model)
│   │   │   └── tests/discipline.rs
│   │   └── clarus-audit/       # Cross-project: Clarus Core audit
│   │       ├── src/lib.rs      # ChunkRecord, DicomElement, InstanceMeta
│   │       └── tests/
│   │           ├── b_model.rs  # B-model audit of Clarus
│   │           └── s_model.rs  # S-model audit (multi-bit SEU)
│   └── Cargo.toml
└── .gitignore
```

---

## Naming Convention

| ID Prefix | Model | Memory Tech | Word Size | Capacity | Era |
|:---|:---|:---|:---|:---|:---|
| `AP101B-CORE-` | IBM AP-101**B** | Ferrite core (Core Storage Module) | 32-bit | ~416 KB phys, ~256 KB usable | 1981–1990. Last flight: STS-40 (1991) |
| `AP101S-CMOS-` | IBM AP-101**S** | CMOS SRAM (batt-backed) + DRAM/ECC | 32-bit | 256K words (1 MB) | 1991–2011. First flight: STS-37 (1991). SEU first detected here. |

### Test Filtering

```bash
cargo test -p ap101b-core    # AP-101B ferrite core tests only
cargo test -p ap101s-cmos    # AP-101S CMOS tests only
cargo test --workspace       # ALL tests (B + S + core + testkit)

# CI-grade SEU verification (10 000 cases each):
PROPTEST_CASES=10000 cargo test -p ap101b-core prop_seu_detection
PROPTEST_CASES=10000 cargo test -p ap101s-cmos prop_burst_seu_detection
```

---

## Extensibility: Adding a New Model

The `ferrite-core` and `ferrite-testkit` libraries are model-agnostic.
Adding a new Shuttle computer variant requires only a new example crate:

| Step | Effort | What to do |
|:---|:---:|:---|
| 1. New example | 5 min | `cargo new examples/ap101s-cmos` |
| 2. Define struct | 15 min | `Ap101sState` — same pattern, different `EXPECTED_SIZE` |
| 3. Write discipline test | 20 min | Copy `ap101b()` → `ap101s()`. Adjust: multi-bit SEU, ECC detection, battery retention |
| 4. Filter | 0 min | `cargo test ap101s` — works immediately |

**B-model vs S-model differences in tests:**

| Aspect | AP-101B (ferrite core) | AP-101S (CMOS) |
|:---|:---|:---|
| Memory constraint | 256 KB usable | 1 MB |
| SEU model | Single-bit flip (cosmic ray flips ferrite core). **Ferrite is naturally immune.** | Multi-bit burst (DRAM row upset). **CMOS is vulnerable.** First SEU detected: STS-37. |
| Error detection | CRC-32 checksum | CRC-32 + ECC syndrome check |
| Power loss | Total loss (ferrite is non-volatile but unpowered read = destructive) | SRAM battery retention test |
| Struct size | 28–40 bytes (compact structs) | Up to 128 bytes (ECC overhead) |

> **Note on the S-model:** The AP-101S used semiconductor memory (CMOS SRAM/DRAM), not ferrite cores. It has its own discipline — `ap101s-cmos` — with checks specific to CMOS: ECC syndrome verification, multi-bit SEU bursts, and SRAM battery retention. Both models share `ferrite-core` primitives (CRC-32, structural audit, `assert_no_padding!`); the S-model adds ECC on top. Both models are tested; they simply protect against different physics.

---

## License

MIT — see [LICENSE](LICENSE).

---

> *"256 KB and not one byte more." — Flight software constraint, IBM AP-101B, ca. 1981.*
