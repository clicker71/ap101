# Architectural Decisions & Boundaries

**AP-101B Ferrite Discipline Standard.** This document outlines the core
architectural choices, boundaries, and trade-offs of the `ap101` test
harness. We believe in high-utility engineering and radical transparency.

---

## D-1: Scope of Application & Tool Boundaries

**Decision:** `ap101` is an automated CI gate and validation harness. It is
**not** a standalone static analyzer, a formal verification tool, or a
replacement for target-hardware testing.

### Explicit Non-Goals (What It DOES NOT Do)

- **Does not guarantee panic-free code.** It prevents hidden heap allocations
  and padding, but logical panics (bounds checking, arithmetic overflow) must
  be caught by standard tools.
- **Does not replace on-target hardware testing.** Host-side simulation
  isolates ~90% of architectural issues early, but final verification
  belongs on physical silicon.
- **Does not perform formal mathematical proof.** It uses property-based
  testing and deterministic constraints, not formal methods (like `Kani`
  or `Creusot`).

---

## D-2: Three-Tier Safety & Verification Model

To systematically audit data structures against the physical silicon,
`ap101` enforces a three-tier compliance ladder:

```
[ TIER 3: CONTRACTUAL ] → Safety contracts, formal unsafe invariants, Miri validation.
       ▲
[ TIER 2: BEHAVIOURAL ] → SEU bit-flip injection, multi-bit burst detection, ULP determinism.
       ▲
[ TIER 1: STRUCTURAL   ] → Compile-time geometry, zero hidden padding, alignment contracts.
```

1. **Tier 1 (Structural):** Compile-time checks (`assert_no_padding!`,
   `GeometryReport`). Enforces cache locality and hardware-bus matching.
2. **Tier 2 (Behavioural):** Runtime host-side simulation (`FerriteCell`,
   `proptest` SEU injection, ULP delta tracking). Checks algorithmic
   resilience against faulty silicon.
3. **Tier 3 (Contractual):** Explicit `# Safety` documentation, formal
   invariants, and strict `Miri` execution to ensure `unsafe` blocks are
   free of Undefined Behavior.

---

## D-3: Known Consequences & Trade-Offs

### The Positive

- **Zero-Heap Enforcement.** Catching unexpected allocation storms (like
  `String` conversions in processing pipelines) before they reach embedded
  devices.
- **Deterministic Alignment.** Forcing developers to think in CPU cache
  lines and structural packaging.

### The Negative & Blind Spots

- **The Stack Overflow Trap.** `ferrite-core` successfully eliminates
  heap-OOM, but it cannot natively stop a developer from blowing up the
  stack with massive structures.
- **Host-Target Disconnect.** Running SEU and runtime verification on a
  host system (`x86_64`) introduces risks due to subtle architecture-specific
  differences in assembly emission and memory layouts compared to target
  microcontrollers (e.g. `Cortex-M`).

---

## D-4: Roadmap Alignment

We resolve these trade-offs through an explicit, version-controlled
execution plan:

| Mitigation Strategy | Target | Mitigated Risk |
|:---|:---|:---|
| **Stack Depth Analysis** | `v0.2.0` | Integrates `cargo-call-stack` into CI to prevent Stack Overflows. |
| **Safety Contracts Audit** | `v0.2.0` | Full audit of `FerriteCell` `unsafe` paths with Miri reporting. |
| **Cross-Platform Assertions** | `v0.2.0` | Cross-compiling Tier 1 checks under explicit hardware target flags. |
| **QEMU Target Testing** | `v0.3.0` | Executes SEU runtime tests inside target-emulated environments (ARM/RISC-V). |

---

*Definitum semel.* — Decided once, verified continuously.
