//--------------------------------------------------------------------
// MODULE:        clarus-audit/src/lib.rs
// PURPOSE:       AUDITED CLARUS CORE STRUCTS — COPIED FROM PRODUCTION.
//                ChunkRecord, DicomElement, InstanceMeta.
//                CRC-32 checksum logic for InstanceMeta.
//                Compile-time assert_no_padding! for all three.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   Do NOT modify struct layouts. These are Clarus production.
//--------------------------------------------------------------------

use ferrite_core::checksum::{Checksum, Crc32};
use ferrite_core::as_bytes;

//--------------------------------------------------------------------
// CHUNKRECORD — from clarus-adapters/src/storage/chunk_store.rs
//--------------------------------------------------------------------

/// Record stored in chunk registry. 40 bytes, #[repr(C)].
/// ONE heap allocation: hex::encode deferred to SQL bind point.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ChunkRecord {
    pub hash: [u8; 32],      // SHA-256 digest — ZERO heap
    pub block_index: usize,  // 8 bytes on 64-bit
}

//--------------------------------------------------------------------
// DICOMELEMENT — from clarus-adapters/src/dicom_rs/parser.rs
//--------------------------------------------------------------------

/// Parsed DICOM element. #[repr(C)].
/// vr: [u8; 2] — ZERO heap (was Option<String> before ferrite audit)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DicomElement {
    pub tag: u32,
    pub vr: [u8; 2],
    pub len: u32,
}

//--------------------------------------------------------------------
// INSTANCEMETA — from clarus-adapters/src/dicom_rs/parser.rs
//--------------------------------------------------------------------

/// DICOM instance metadata. CRC-32 protected.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InstanceMeta {
    pub study_uid: [u8; 64],
    pub series_uid: [u8; 64],
    pub instance_uid: [u8; 64],
    pub modality: [u8; 16],
    pub checksum: u32,  // CRC-32/ISO-HDLC
}

impl Default for InstanceMeta {
    fn default() -> Self {
        let mut meta = Self {
            study_uid: [0u8; 64],
            series_uid: [0u8; 64],
            instance_uid: [0u8; 64],
            modality: [0u8; 16],
            checksum: 0,
        };
        meta.checksum = meta.compute_checksum();
        meta
    }
}

impl InstanceMeta {
    /// Compute CRC-32 checksum of all fields except checksum.
    pub fn compute_checksum(&self) -> u32 {
        let mut raw = *self;
        raw.checksum = 0;
        Crc32::compute(&as_bytes!(raw))
    }

    /// Update checksum field to current value.
    pub fn update_checksum(&mut self) {
        self.checksum = self.compute_checksum();
    }

    /// Verify data integrity via CRC-32.
    pub fn verify_integrity(&self) -> bool {
        self.checksum == self.compute_checksum()
    }
}

//--------------------------------------------------------------------
// COMPILE-TIME ZERO HIDDEN PADDING CHECKS
//--------------------------------------------------------------------

ferrite_core::assert_no_padding!(
    ChunkRecord,
    hash: [u8; 32],
    block_index: usize
);

ferrite_core::assert_no_padding!(
    DicomElement,
    tag: u32,
    vr: [u8; 2],
    len: u32
);

ferrite_core::assert_no_padding!(
    InstanceMeta,
    study_uid: [u8; 64],
    series_uid: [u8; 64],
    instance_uid: [u8; 64],
    modality: [u8; 16],
    checksum: u32
);
