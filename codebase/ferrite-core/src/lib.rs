//--------------------------------------------------------------------
// MODULE:        ferrite-core/src/lib.rs
// PURPOSE:       LIBRARY ROOT. NO_STD WHEN STD FEATURE DISABLED.
//                FOUR MODULES: cell, audit, checksum, telemetry.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   256 KB USABLE CORE. EVERY BYTE COUNTS.
//--------------------------------------------------------------------

#![cfg_attr(not(feature = "std"), no_std)]

// FERRITE CELL — CONTROLLED-ACCESS MEMORY WITH UNSAFE CONTRACT.
pub mod cell;

// STRUCTURAL GEOMETRY AUDIT — SIZE, ALIGN, ZERO PADDING.
pub mod audit;

// CHECKSUM TRAIT — CRC-16, CRC-32, XOR-FOLD.
pub mod checksum;

// IBM PASS CRT TELEMETRY — DECORATIVE OUTPUT (STD FEATURE).
pub mod telemetry;

// RE-EXPORT KEY SYMBOLS FOR ERGONOMIC USE.
pub use audit::{audit_exact_size, audit_size_and_align, GeometryReport};
pub use cell::FerriteCell;
pub use checksum::{Checksum, Crc16, Crc32, XorFold};
