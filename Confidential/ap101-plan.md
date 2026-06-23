# AP-101B Ferrite Discipline — Master Sprint Plan

**Project:** AP-101B CORE — test harness honoring Shuttle ferrite core memory
**Repository:** `github.com/clicker71/ap101` (public, MIT)
**Stack:** Rust (edition 2021), `no_std` (ferrite-core), `std` (ferrite-testkit), proptest
**Methodology:** [AI_Driven_Development_Methodology.md](./AI_Driven_Development_Methodology.md) v1.0
**Working folder:** `D:\ap101`
**Public folder:** `D:\ap101\codebase`
**Confidential:** `D:\ap101\Confidential`

---

## Sprint Overview

| Sprint | Week | Theme | Lines | Status |
|:---|:---|:---|:---:|:---:|
| **P0** | 1 (day 1) | Git init, CI/CD, cargo workspace | 30 | ✅ |
| **P1** | 1 (days 2–3) | Fix 4 critical blockers (B1, B3, N1, N3) | ~80 | ✅ |
| **P2** | 1 (day 4) | Fix remaining findings (B2, B4, B5, N2, N4–N8) | ~60 | ✅ |
| **P3B** | 1 (day 5) | Architect re-review → Writer generates AP-101B `.rs` files | ~500 | ✅ |
| **P3S** | 1 (day 5) | Architect designs AP-101S → Writer generates AP-101S `.rs` files | ~200 | ✅ |
| **P4** | 2 (days 1–2) | Tests, clippy, miri, SEU verification (B + S) | ~200 | ✅ |
| **P5** | 2 (day 3) | Docs, SBOM, README polish (B + S) | — | ✅ |
| **P6B** 🔴 | 2 (day 4) | **RELEASE: AP-101B ferrite-audit Clarus Core** | — | ✅ |
| **P6S** 🔴 | 2 (day 4) | **RELEASE: AP-101S CMOS-audit Clarus Core** | — | ✅ |

**Total:** ~1070 lines. 2 weeks. B + S in parallel: +3 hours over B-only.

---

## Sprint P0: Infrastructure (Week 1, Day 1)

**Goal:** Git, CI/CD, cargo workspace, ferrite-core skeleton.

> **Thinking Effort:** `None` — infrastructure.

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101B-000 | Setup | **Cargo workspace** | `Cargo.toml` at `ap101-bringup/` root with `[workspace]` members: `ferrite-core`, `ferrite-testkit`, `examples/navigation`. `edition = "2021"`. |
| AP101B-001 | Setup | **Git init + remote** | `git init` in `ap101-bringup/`. `git remote add origin`. First commit: `.gitignore`, `Cargo.toml`, `README.md`. |
| AP101B-002 | Setup | **CI/CD** | GitHub Actions `ci.yml`: `cargo test`, `cargo clippy -- -D warnings`, `cargo miri test` (ferrite-core only). |
| AP101B-003 | Setup | **`ferrite-core` skeleton** | `cargo init --lib ferrite-core`. `#![no_std]`. Empty modules: `cell`, `audit`, `checksum`, `telemetry`. |

### Definition of Done

- [x] `git init` in `ap101-bringup/`, remote configured
- [x] `cargo build` succeeds (empty workspace)
- [x] CI green on first push
- [x] `Confidential/` not in git

---

## Sprint P1: Fix Critical Blockers (Week 1, Days 2–3)

**Goal:** Resolve 4 findings blocking Writer handoff: B1, B3, N1, N3.

> **Thinking Effort:** `High` — UB fix in macro, struct layout, dependency resolution.

| ID | Type | Name | Fix | Lines |
|:---|:---|:---|:---|:---:|
| AP101B-010 | Task | **B1: Fix `assert_no_padding!` UB** | Replace `&*core::ptr::null::<T>()` with `core::mem::size_of::<FieldType>()`. Must work in `const` context. | ~20 |
| AP101B-011 | Task | **B3: Add `heapless` dependency** | `cargo add heapless --features "serde"`. Alternative: replace with fixed arrays to keep zero-deps. Evaluate in Architect prompt. | ~5 |
| AP101B-012 | Task | **N1: Fix `NavigationState` size** | `EXPECTED_SIZE` 28 → 32. OR use `#[repr(C, align(4))]`. OR replace `u64` with `[u32; 2]`. Architect decides. | ~10 |
| AP101B-013 | Task | **N3: Implement `execute_on_ferrite_core`** | Replace `current_alloc_snapshot()` stub with actual access to `TestAllocator` via global static or explicit parameter. | ~30 |
| AP101B-014 | Task | **B2: Remove `store(&self)` from safe API** | Delete `store` method from `impl<T: Copy + FerriteSafeRead>`. Keep only `unsafe fn write_volatile`. | ~5 |

### Definition of Done

- [x] `assert_no_padding!` passes miri (no UB)
- [x] `cargo check` succeeds (heapless available or removed)
- [x] `audit_exact_size::<NavigationState>()` passes
- [x] `execute_on_ferrite_core` detects real allocations

---

## Sprint P2: Fix Remaining Findings (Week 1, Day 4)

**Goal:** Resolve B4, B5, N2, N4–N8.

> **Thinking Effort:** `None` — cosmetic + low-risk fixes.

| ID | Type | Name | Fix | Lines |
|:---|:---|:---|:---|:---:|
| AP101B-020 | Task | **B4: Update rand API** | `rng.gen_range(0..N)` and `rng.gen()` per rand 0.8. | ~10 |
| AP101B-021 | Task | **B5: Document `as_bytes!` safety** | Add `SAFETY:` doc comment. Consider embedding `assert_no_padding!` check. | ~5 |
| AP101B-022 | Task | **N2: Fix `add_result` silent loss** | Replace `.ok()` with `debug_assert!` or `panic!` on overflow. Or increase capacity to 32. | ~5 |
| AP101B-024 | Task | **N5: Fix `store` bound** | Move `store` out of `FerriteSafeRead`-gated impl. Use `T: Copy` bound only. | ~5 |
| AP101B-025 | Task | **N8: Improve `assert_no_padding!` error** | Add `compile_error!` with struct name and expected/actual sizes. | ~10 |
| AP101B-026 | Task | **N4, N6, N7: Document** | Add SAFETY/design notes for allocator race, XorFold asymmetry, non-atomic RMW. | ~10 |

### Definition of Done

- [x] `cargo test` passes with rand 0.8 API
- [x] `as_bytes!` doc warns about padding
- [x] `add_result` overflow is loud (not silent)
- [x] All SAFETY comments present

---

## Sprint P3B: Architect Re-Review → Writer: AP-101B (Week 1, Day 5)

**Goal:** Architect confirms all fixes. Writer generates AP-101B `.rs` files from plan.

> **Thinking Effort:** `Max` (Architect) / `High` (Writer) — full design-to-code translation.

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101B-030 | Gate | **Architect re-review** | Architect reads updated plan, confirms B1–N8 resolved, emits GO/NOGO. |
| AP101B-031 | Story | **Write `ferrite-core/src/cell.rs`** | `FerriteCell<T>`, `FerriteSafeRead` trait. From plan. |
| AP101B-032 | Story | **Write `ferrite-core/src/audit.rs`** | `GeometryReport`, `audit_size_and_align`, `audit_exact_size`, `assert_no_padding!`. |
| AP101B-033 | Story | **Write `ferrite-core/src/checksum.rs`** | `Checksum` trait, `Crc16`, `Crc32`, `XorFold`, `as_bytes!`. |
| AP101B-034 | Story | **Write `ferrite-core/src/telemetry.rs`** | `IbmCrt` (behind `std` feature), `AuditResult`, `AuditReport`. |
| AP101B-035 | Story | **Write `ferrite-core/src/lib.rs`** | Module declarations, re-exports. `#![no_std]`. |
| AP101B-036 | Story | **Write `ferrite-testkit/`** | `heap.rs`, `strategy.rs`. |
| AP101B-037 | Story | **Write `examples/ap101b-core/`** | `Ap101bNavigationState`, `discipline.rs` test with AP101B-CORE- assertions. |

### Definition of Done

- [x] Architect GO signal
- [x] All `.rs` files generated and committed
- [x] `cargo check` succeeds for workspace
- [x] `cargo test -p ap101b-core` works

---

## Sprint P3S: Architect Design → Writer: AP-101S (Week 1, Day 5)

**Goal:** Architect designs AP-101S CMOS variant. Writer generates in parallel with B-model.

> **Thinking Effort:** `High` (Architect) / `High` (Writer) — new model, same primitives.

### AP-101S Design Brief

AP-101S replaces ferrite cores with CMOS SRAM (battery-backed) + DRAM with ECC.
Key differences from B-model:

| Aspect | AP-101B | AP-101S |
|:---|:---|:---|
| Memory | Ferrite core, ~256 KB usable | CMOS SRAM + DRAM/ECC, 1 MB |
| SEU model | Single-bit flip (cosmic ray) | Multi-bit burst (DRAM row upset) + single-bit (SRAM) |
| Error detection | CRC-32 only | CRC-32 + ECC syndrome check |
| Power loss | Ferrite non-volatile, destructive read | SRAM battery retention; DRAM volatile |
| Struct size | Compact (28–40 bytes) | Up to 128 bytes (ECC overhead) |
| Telemetry prefix | `AP101B-CORE-` | `AP101S-CMOS-` |
| Test filter | `cargo test -p ap101b-core` | `cargo test -p ap101s-cmos` |

### Tasks

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101S-001 | Spike | **Architect: AP-101S design** | Specify `Ap101sState` struct: ECC field, battery flag, multi-bit SEU injector. |
| AP101S-002 | Story | **Write `examples/ap101s-cmos/`** | `Ap101sNavigationState` with ECC checksum. Multi-bit fault injection (`inject_dram_burst`). |
| AP101S-003 | Story | **Write AP101S discipline test** | `ap101s()`: geometry, padding, determinism, multi-bit SEU, ECC detection, battery retention. |
| AP101S-004 | Story | **Extend `ferrite-testkit` if needed** | `inject_burst_error(data, start_byte, bit_count)` strategy. Only if proptest can't express multi-bit. |

### Definition of Done

- [x] `Ap101sState` passes `audit_exact_size`
- [x] `cargo test -p ap101s-cmos` runs AP101S-CMOS-01 through -06
- [x] Multi-bit SEU (2–8 adjacent bits) detected by CRC-32 + ECC
- [x] Battery retention: SRAM state survives `FerriteCell::into_inner` after power-loss simulation
- [x] AP101S-CMOS- prefixed telemetry output

---

## Sprint P4: Verification — B + S (Week 2, Days 1–2)

**Goal:** Full test suite for both models. Clippy, miri, SEU proof.

> **Thinking Effort:** `None` — verification only.

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101B-040 | Task | **Unit tests (B)** | `cargo test -p ap101b-core`. AP101B-CORE-01 through -05 all COMPLIANT. |
| AP101S-040 | Task | **Unit tests (S)** | `cargo test -p ap101s-cmos`. AP101S-CMOS-01 through -06 all COMPLIANT. |
| AP101B-041 | Task | **SEU prop-test (B)** | `prop_seu_detection`: 10K single-bit flips. Zero escapes. |
| AP101S-041 | Task | **SEU prop-test (S)** | `prop_burst_seu_detection`: 10K multi-bit bursts (2–8 bits). Zero escapes. |
| AP101B-042 | Task | **clippy** | `cargo clippy --all-targets -- -D warnings`. |
| AP101B-043 | Task | **miri** | `cargo miri test -p ferrite-core`. Zero UB. |
| AP101B-044 | Task | **`no_std` build** | `cargo build -p ferrite-core --no-default-features`. |

### Definition of Done

- [x] `cargo test -p ap101b-core` — COMPLIANT
- [x] `cargo test -p ap101s-cmos` — COMPLIANT
- [x] 10K single-bit: zero escapes
- [x] 10K multi-bit: zero escapes
- [x] clippy: 0 warnings
- [x] miri: 0 errors
- [x] `no_std` build: OK

---

## Sprint P5: Release Preparation (Week 2, Day 3)

**Goal:** Documentation for both models, SBOM, tag v0.1.0-alpha.

> **Thinking Effort:** `None` — docs.

| ID | Type | Name |
|:---|:---|:---|
| AP101B-050 | Docs | README final review — B+S extensibility, historical accuracy |
| AP101B-051 | Docs | API docs — `cargo doc --no-deps` |
| AP101B-052 | Setup | SBOM — `cargo cyclonedx` |
| AP101B-053 | Setup | Tag `v0.1.0-alpha` |

### Definition of Done

- [x] README covers B and S models
- [x] All doc comments IBM 370 English
- [x] SBOM generated
- [x] Tag pushed

---

## Sprint P6B 🔴 RELEASE: AP-101B Ferrite-Audit Clarus Core (Week 2, Day 4)

**Goal:** Run AP-101B Ferrite Discipline against Clarus Core. Prove production Rust
survives ferrite memory constraints.

> **Thinking Effort:** `Max` — cross-project audit, integration.

### Tasks

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101B-060 | Story | **Audit `ChunkRecord` geometry** | `size_of`, align, padding. 40 bytes expected. |
| AP101B-061 | Story | **Audit `DicomElement` geometry** | `vr: [u8; 2]` — zero heap. Check padding. |
| AP101B-062 | Story | **Audit `chunk_pixel_data` heap** | `execute_on_ferrite_core` — ZERO ALLOC. |
| AP101B-063 | Story | **Audit `extract_pixel_data` heap** | Zero `.to_vec()`. Returns `&[u8]`. |
| AP101B-064 | Story | **Audit `register_chunks` heap** | Exactly one `hex::encode` per INSERT. |
| AP101B-065 | Story | **SEU: `InstanceMeta`** | 1000 bit-flips. CRC-32 detects 100%. |
| AP101B-066 | Story | **Produce audit report** | IBM 3270 CRT. Archive in `Confidential/analysis/`. |
| AP101B-067 | Docs | **PR to Clarus README** | `AP101B-CORE: COMPLIANT` badge. |
| AP101B-068 | Docs | **Add real audit output to AP-101B README** | Capture actual `cargo test -p ap101b-core` output against Clarus. Add as `### Proof: Clarus Core` section with before/after table (ChunkRecord, DicomElement, chunk_pixel_data, extract_pixel_data). |
| AP101B-069 | Docs | **Add self-test output to AP-101B README** | Capture actual `cargo test --workspace` output — the discipline auditing itself. Add as `### Proof: Self-Test` section with full IBM PASS CRT telemetry block. Meta-circular proof: the harness passes its own checks. |

---

## Sprint P6S 🔴 RELEASE: AP-101S CMOS-Audit Clarus Core (Week 2, Day 4)

**Goal:** Run AP-101S CMOS Discipline against Clarus Core. Multi-bit errors, ECC logic.

> **Thinking Effort:** `Max` — same as P6B, different fault model.

### Tasks

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101S-060 | Story | **Audit Clarus structs (S-model)** | Same structs as P6B. Larger expected sizes (ECC overhead). |
| AP101S-061 | Story | **Multi-bit SEU: `InstanceMeta`** | 2–8 adjacent bit flips. CRC-32 + ECC detection. |
| AP101S-062 | Story | **DRAM burst: `ChunkRecord`** | Row-upset simulation on `[u8; 32]` hash. Detect corruption. |
| AP101S-063 | Story | **Produce S-model audit report** | IBM 3270 CRT. Archive alongside B-model report. |
| AP101S-064 | Docs | **PR to Clarus README** | `AP101S-CMOS: COMPLIANT` badge. |
| AP101S-065 | Docs | **Add real audit output to AP-101B README (S-model)** | Capture actual `cargo test -p ap101s-cmos` output against Clarus. Append to `### Proof` section with multi-bit SEU results and ECC detection stats. |
| AP101S-066 | Docs | **Add S-model self-test to README** | Capture `cargo test -p ap101s-cmos` self-test output (CMOS discipline auditing itself). Append to `### Proof: Self-Test` section. Shows ECC syndrome + battery retention self-check. |

---

## ID Taxonomy

| Prefix | Purpose |
|:---|:---|
| `AP101B-XXX` | Core project tasks |
| `INFRA-XXX` | Infrastructure (Git, CI, Docker, MCP) |

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|:---|:---|:---|:---|
| `assert_no_padding!` const-context UB not fixable without nightly | Medium | High | Accept nightly dependency for macro; use `compile_error!` fallback |
| `heapless` pulls transitive deps breaking zero-deps promise | Low | Medium | Replace with fixed arrays + manual counters |
| Clarus structs fail SEU detection (no checksum field) | Medium | Low | Add advisory recommendation; do not block release |
| Clarus hot paths trigger alloc in `tracing::warn!` macro | Medium | Low | Filter out `tracing` from heap audit; document |

---

## Resource Estimate

| Role | Sprints | Hours |
|:---|:---|:---|
| **Orchestrator** | P0–P6S | ~15h (prompts, reviews, Hard Gates) |
| **Architect** | P1, P3B, P3S, P6B, P6S | ~8h (re-review, S-model design, Writer prompts, audit reports) |
| **Writer (Pro)** | P1, P2, P3B, P3S, P6B, P6S | ~10h (fixes, B generation, S generation, audit harness) |
| **Verifier** | P4, P6B, P6S | ~3h (cargo test, clippy, miri, audit runs) |
| **Total** | 2 weeks | ~36 AI-hours |

**B-only baseline:** ~28h. **+S-model delta:** ~8h (+29%). The shared `ferrite-core`
and `ferrite-testkit` libraries absorb most of the cost; only examples differ.

---

## v0.2.0 Roadmap — «Industry Standard»

Based on external expert audit (2026-06-22). Prioritised by impact/effort ratio.

### Quick Wins (v0.1.1, ~4h)

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101B-100 | Story | **Criterion benchmarks** | `cargo bench` — before/after Clarus chunk dedup. `[u8; 32]` vs `String` hex: ns/op, latency tails (p50/p99/p999). |
| AP101B-101 | Story | **Latency tail report** | `GeometryReport` extended with cache-line analysis: false sharing detection, cross-cache-line splits. |

### Medium Term (v0.2.0, ~20h)

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101B-102 | Story | **HFT/GameDev feature profile** | `ferrite-core/features = ["hft"]` — disables SEU simulation, enables stricter cache locality checks. `features = ["gamedev"]` — frame-latency focus. |
| AP101B-103 | Story | **`FerriteDiscipline` derive macro** | `#[derive(FerriteDiscipline)]` — auto-generates `GeometryReport`, `assert_no_padding!`, `const CHECKSUM: u32`. One annotation instead of three manual steps. |
| AP101B-104 | Story | **CI lint plugin** | `cargo-ap101` subcommand: scans codebase for structs missing `#[repr(C)]`, detects `String`/`Vec` in `#[ferrite_hot]` functions. CI gate. |
| AP101B-105 | Story | **Spatial burst SEU (S-model)** | Multi-cell upset simulation: 2–4 adjacent bits with realistic charge deposition profile. Test Reed-Solomon resilience. |

### Long Term (v0.3.0+, ~40h)

| ID | Type | Name | Description |
|:---|:---|:---|:---|
| AP101B-106 | Story | **const fn CRC-32** | Compile-time checksum. Requires nightly `const fn` + `while`. Gate behind `nightly` feature. |
| AP101B-107 | Story | **`no-panics-whatsoever` pass** | Static analysis: prove zero panic paths in `#[ferrite_no_panic]` functions. Integrate with `no-panics-whatsoever` crate. |
| AP101B-108 | Story | **Hardware SEU injection** | FPGA-based fault injector for physical SEU testing on real silicon (Raspberry Pi, STM32). |
| AP101B-109 | Docs | **Technical article** | «How we removed 128 000 allocations from a medical server using 1981 Space Shuttle discipline.» Hacker News / Reddit / Habr. |

### Priority Matrix

```
                    IMPACT
              Low         High
        ┌───────────┬───────────┐
  Low   │           │ AP101B-100│ ← START HERE
E       │           │ AP101B-101│
F  ─────┼───────────┼───────────┤
F       │ AP101B-106│ AP101B-102│
O  High │ AP101B-108│ AP101B-103│
R       │           │ AP101B-104│
T       │           │ AP101B-105│
        └───────────┴───────────┘
```

**Recommendation:** AP101B-100 + AP101B-101 first. Criterion benchmarks give you the numbers for the article (AP101B-109). Then AP101B-103 (derive macro) — biggest UX improvement for smallest API surface change.

---

> *"256 KB and not one byte more. STOW 6 patients, not 3." — AP-101B discipline, applied to Clarus.*
