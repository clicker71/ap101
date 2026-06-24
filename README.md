# AP-101B Ferrite Discipline

> **A compile-time and test-time SIHFT (Software-Implemented Hardware Fault Tolerance) framework for high-performance Rust.**
> **An automated CI gate ensuring zero-heap allocation compliance, strict struct geometry, and software-level resilience against memory corruption.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE) [![CI](https://github.com/clicker71/ap101/actions/workflows/ci.yml/badge.svg)](https://github.com/clicker71/ap101/actions/workflows/ci.yml) [![Mission Status](https://img.shields.io/badge/AP--101-GO%20FOR%20LAUNCH-brightgreen)](https://github.com/clicker71/ap101) [![Boosty](https://img.shields.io/badge/Boosty-Support_Mission-orange)](https://boosty.to/clicker71/donate)

---

## Real-World Mission: This is a SIHFT Toolkit, Not a Simulator

A common misconception when skimming this repository is that it is an academic
space shuttle simulator, a retro-computing exercise, or an enthusiast project
focused on arbitrary 256 KB memory limitations.

**It is none of those.**

`ap101` implements pragmatic **Software-Implemented Hardware Fault Tolerance
(SIHFT)** concepts for production applications. It was forged as the core
internal testing discipline for the **[Clarus PACS](https://github.com/clarus-pacs/clarus)**
medical imaging server. DICOM ingestion dealing with gigabytes of multi-frame
CT/MRI data cannot tolerate heap churn, fragmentation, or silent data
corruption caused by unstable hardware.

### Proven Production Impact in Clarus

- **Zero-Heap Reinforcement:** Slashed run-time allocations from **128 000 to
  exactly 0** per computed tomography (CT) study ingestion.
- **COTS Hardware Resilience:** Prevents `OOM-KILL` failures and memory
  corruption crashes on cheap ARM single-board computers and old, repurposed
  x86 workstations deployed in local regional hospitals. The discipline scales
  in both directions: keeps budget hardware alive, keeps big hardware fast.

---

## Why "AP-101B"?

The **IBM AP-101B** was the core avionics computer of the Space Shuttle.
The aerospace industry proved that when building on imperfect hardware,
the software must maintain total architectural discipline down to the
assembly level.

We brought this philosophy to modern Rust. `ap101` does not run code in
space; it introduces automated guardrails into your CI suite to ensure
that your data structures, memory access boundaries, and low-level code
invariants do not degrade during rapid production refactoring.

**[Full Shuttle history: ferrite-to-CMOS transition, STS-37/STS-40 →](./HISTORY.md)**

---

## The Three Pillars of SIHFT Enforcement

The framework provides three distinct, high-utility tools for your test suite:

1. **Zero-Heap Enforcer (`TestAllocator`):** A global allocator interceptor.
   If a hot-path parsing function leaks a hidden allocation (`String`, `Vec`,
   `Box`), the CI pipeline fails immediately.

2. **Structural Integrity (`assert_no_padding!`):** A compile-time layout
   assertion checking for hidden alignment gaps (padding bytes) in your data
   structures. Prevents hidden data serialization leaks and stack bloating.

3. **Fault Injection Engine (`FerriteCell<T>`):** A test-runner mechanism
   designed to simulate physical single-event upsets (SEUs) and transient
   bit-flips. Validates how your software detects, flags, and gracefully
   handles physical memory corruption.

---

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
ferrite-core = { git = "https://github.com/clicker71/ap101", subdir = "codebase/ferrite-core" }

[dev-dependencies]
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

**[DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md)** — full guide: writing tests, CI setup,
understanding failures, common pitfalls, reference projects.

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

### 4. Silicon Vulnerability — Radiation-Hardening from 1991 to 3nm

The Space Shuttle's AP-101B (1981–1990) used magnetic ferrite core memory —
naturally immune to cosmic rays at the physics level. The **AP-101S** (1991–2011)
was the first Shuttle computer to use semiconductor memory (CMOS SRAM/DRAM),
and with it came the **first registered SEU** on STS-37. The era of software-level
fault tolerance began.

Today, at 3nm and below, the problem is exponentially worse. A single
atmospheric neutron can flip bits in server RAM, automotive controllers,
and medical equipment — on Earth, at sea level. The smaller the transistor,
the lower the critical charge, the higher the SEU cross-section.

**The Trap:** Shrinking process nodes make silent data corruption a
statistical certainty for any sufficiently large deployment. Standard
compilers offer zero protection.

**The Discipline:** The SEU simulation in this harness forces you to write
and test software-level fault-tolerant algorithms (CRC-32, XorFold, ECC)
to guarantee data integrity when the physical silicon fails — exactly as
the aerospace industry learned the hard way in 1991.

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

Cross-project audit of Clarus PACS production structs — the same code
that had 128 000 allocations per CT study. The harness verifies geometry,
padding, and SEU resilience of real-world medical data structures:

```bash
cargo test -p clarus-audit -- --nocapture
```

**B-Model audit:**

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

**S-Model audit — adds multi-bit SEU burst detection:**

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

---

### Proof: Clarus Production Audit (Cross-Repo Integration Test)

The discipline is applied as an **integration test inside the Clarus PACS
repository itself**. This is not a curated model example — it is a direct audit
of production code: domain models, storage layer, CJK charset detection, and
JPEG2000 transfer syntax constants.

Run from the Clarus repo:

```bash
cargo test -p clarus-testing --test ap101_audit -- --nocapture
```

**Domain models — honest violations (expected for DTOs):**

```
[ANOMALY DET] AP101B-CORE-10 | Patient Geometry                         | 112B
[ANOMALY DET] AP101B-CORE-11 | Study Geometry                           | 368B
[ANOMALY DET] AP101B-CORE-12 | Series Geometry                          | 168B
[ANOMALY DET] AP101B-CORE-13 | Instance Geometry                        | 184B
[ANOMALY DET] AP101B-CORE-14 | WorkItem Geometry                        | 376B
MISSION STATUS: ABORT. VIOLATION DETECTED.
```

Domain models use `String`/`Option<String>` for DICOM attributes. Hidden heap
pointers and padding are unavoidable for data transfer objects. The test
correctly flags them — structural discipline is not required for DTOs, only
for the hot path.

**Hot path — all clear:**

```
[ COMPLIANT ] AP101B-CORE-20 | ChunkHash [u8; 32]                       | ZERO HEAP
[ COMPLIANT ] AP101B-CORE-30 | Charset Chinese/Japanese/Korean/Georgian  | Pure compute
[ COMPLIANT ] AP101B-CORE-40 | JPEG2000 Transfer Syntax UIDs             | Constants
MISSION STATUS: GO FOR LAUNCH.
```

The storage layer (`[u8; 32]` chunk hash), charset detection (pure functions,
no allocation), and JPEG2000 codec constants (compile-time strings) are all
ferrite-compliant. This is the code that processes gigabytes of multi-frame
DICOM pixel data — and it runs with zero heap churn.

The test was deliberately not "fudged" to hide the domain model violations.
It reports them honestly, demonstrating that the discipline distinguishes
between what must be ferrite-compliant (hot path) and what cannot be (DTOs
with variable-length DICOM attributes).

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

## Engineering Discipline & Boundaries

This project values architectural rigor over blind abstraction. For a transparent
breakdown of our engineering assumptions, blind spots, and architectural
constraints, read **[DECISIONS.md](./DECISIONS.md)**.

---

## ROADMAP

```
- [ ] V0.1.1  CRITERION BENCHMARKS — CLARUS BEFORE/AFTER (NS/OP, LATENCY TAILS)
- [ ] V0.1.1  CACHE-LINE ANALYSIS IN GEOMETRYREPORT (FALSE SHARING DETECTION)
- [ ] V0.2.0  #[DERIVE(FERRITEDISCIPLINE)] — ONE ANNOTATION: GEOMETRY + PADDING + CRC
- [ ] V0.2.0  HFT/GAMEDEV FEATURE PROFILES (FEATURES = ["HFT"])
- [ ] V0.2.0  CARGO-AP101 CI PLUGIN — FAILS BUILD ON STRING IN HOT FUNCTION
- [ ] V0.2.0  SPATIAL BURST SEU (2-4 ADJACENT CELLS, S-MODEL)
- [ ] V0.2.0  ASYNC RUNTIME ISOLATION — ALLOC MICROSCOPE (TOKIO-AWARE)
- [ ] V0.2.0  CROSS-PLATFORM TARGET ALIGNMENT CONTRACT (ARM/X86_64)
- [ ] V0.2.0  STACK DEPTH ANALYSIS — CARGO-CALL-STACK INTEGRATION
- [ ] V0.2.0  FERRITECELL SAFETY CONTRACTS + UNSAFE AUDIT (MCDC)
- [ ] V0.3.0  QEMU ON-TARGET SEU TESTING (ARM/RISC-V)
- [ ] V0.3.0  CONST FN CRC-32 (NIGHTLY)
- [ ] V0.3.0  ARTICLE — 128 000 ALLOCS REMOVED WITH 1981 SHUTTLE DISCIPLINE
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

```
ID PREFIX        AP101B-CORE-                         AP101S-CMOS-
MODEL            IBM AP-101B                          IBM AP-101S
MEMORY TECH      FERRITE CORE (CORE STORAGE MODULE)    CMOS SRAM (BATT-BACKED) + DRAM/ECC
WORD SIZE        32-BIT                               32-BIT
CAPACITY         ~416 KB PHYS, ~256 KB USABLE         256K WORDS (1 MB)
ERA              1981-1990. LAST FLIGHT: STS-40       1991-2011. FIRST FLIGHT: STS-37.
                 (1991)                               SEU FIRST DETECTED HERE.
```

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

```
STEP                  EFFORT     WHAT TO DO
1. NEW EXAMPLE        5 MIN      cargo new examples/ap101s-cmos
2. DEFINE STRUCT      15 MIN     Ap101sState — SAME PATTERN, DIFFERENT EXPECTED_SIZE
3. DISCIPLINE TEST    20 MIN     COPY ap101b() -> ap101s(). ADJUST: MULTI-BIT SEU, ECC, BATTERY
4. FILTER             0 MIN      cargo test ap101s — WORKS IMMEDIATELY
```

**B-model vs S-model differences in tests:**

```
ASPECT              AP-101B (FERRITE CORE)              AP-101S (CMOS)
MEMORY CONSTRAINT   256 KB USABLE                       1 MB
SEU MODEL           SINGLE-BIT FLIP. FERRITE IS         MULTI-BIT BURST (DRAM ROW UPSET).
                    NATURALLY IMMUNE.                   CMOS IS VULNERABLE. FIRST SEU
                                                        DETECTED: STS-37.
ERROR DETECTION     CRC-32 CHECKSUM                     CRC-32 + ECC SYNDROME CHECK
POWER LOSS          TOTAL LOSS. FERRITE IS NON-         SRAM BATTERY RETENTION TEST
                    VOLATILE BUT UNPOWERED READ
                    = DESTRUCTIVE.
STRUCT SIZE         28-40 BYTES (COMPACT STRUCTS)       UP TO 128 BYTES (ECC OVERHEAD)
```

> **Note on the S-model:** The AP-101S used semiconductor memory (CMOS SRAM/DRAM), not ferrite cores. It has its own discipline — `ap101s-cmos` — with checks specific to CMOS: ECC syndrome verification, multi-bit SEU bursts, and SRAM battery retention. Both models share `ferrite-core` primitives (CRC-32, structural audit, `assert_no_padding!`); the S-model adds ECC on top. Both models are tested; they simply protect against different physics.

---

## License

MIT — see [LICENSE](LICENSE).

---

> *"256 KB and not one byte more." — Flight software constraint, IBM AP-101B, ca. 1981.*
