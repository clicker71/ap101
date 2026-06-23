//--------------------------------------------------------------------
// MODULE:        ap101s-cmos/src/lib.rs
// PURPOSE:       AP-101S CMOS NAVIGATION STATE.
//                EXTENDS B-MODEL WITH ECC SYNDROME AND BATTERY FLAG.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101S (HONORARY)
// CONSTRAINTS:   1 MB CMOS SRAM + DRAM/ECC. NO HEAP WITHOUT CAUSE.
//--------------------------------------------------------------------

use ferrite_core::as_bytes;
use ferrite_core::checksum::{Checksum, Crc32};

/// AP-101S CMOS NAVIGATION SYSTEM STATE.
///
/// EXTENDS B-MODEL WITH:
/// - ECC SYNDROME FIELD (DRAM ROW PROTECTION).
/// - BATTERY FLAG (SRAM RETENTION INDICATOR, 0x5A = OK).
///
/// ## LAYOUT
///
/// FIELD              TYPE    SIZE    OFFSET
/// timestamp          u64     8       0
/// velocity_x         f32     4       8
/// velocity_y         f32     4       12
/// velocity_z         f32     4       16
/// status_flags       u32     4       20
/// ecc_syndrome       u32     4       24
/// battery_flag       u8      1       28
/// \[padding\]          —       3       29  (u32 alignment for checksum)
/// checksum           u32     4       32
/// \[tail padding\]     —       4       36  (struct align 8)
///
/// TOTAL: 40 bytes (33 data + 3 internal padding + 4 tail padding).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Ap101sNavigationState {
    pub timestamp: u64,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub status_flags: u32,
    pub ecc_syndrome: u32, // ECC SYNDROME FOR DRAM ROW PROTECTION
    pub battery_flag: u8,  // SRAM BATTERY STATUS (0x5A = OK, 0x00 = LOST)
    pub checksum: u32,     // CRC-32/ISO-HDLC OVER ALL FIELDS
}

// COMPILE-TIME ZERO-PADDING CHECK
ferrite_core::assert_no_padding!(
    Ap101sNavigationState,
    timestamp: u64,
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
    status_flags: u32,
    ecc_syndrome: u32,
    battery_flag: u8,
    checksum: u32
);

impl Default for Ap101sNavigationState {
    fn default() -> Self {
        let mut state = Self {
            timestamp: 0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            status_flags: 0,
            ecc_syndrome: 0,
            battery_flag: 0x5A, // BATTERY OK
            checksum: 0,
        };
        state.checksum = state.compute_checksum();
        state
    }
}

impl Ap101sNavigationState {
    /// COMPUTE CRC-32 CHECKSUM OF ALL FIELDS EXCEPT checksum.
    pub fn compute_checksum(&self) -> u32 {
        let mut raw = *self;
        raw.checksum = 0;
        Crc32::compute(&as_bytes!(raw))
    }

    /// UPDATE CHECKSUM.
    pub fn update_checksum(&mut self) {
        self.checksum = self.compute_checksum();
    }

    /// VERIFY STATE INTEGRITY VIA CRC-32.
    pub fn verify_integrity(&self) -> bool {
        self.checksum == self.compute_checksum()
    }

    /// VERIFY ECC SYNDROME FOR DRAM ROW.
    ///
    /// SIMPLIFIED ECC: XOR OF ALL 32-BIT WORDS IN THE STRUCT
    /// (EXCLUDING ecc_syndrome AND checksum FIELDS).
    /// PRODUCTION WOULD USE HAMMING(7,4) OR REED-SOLOMON.
    pub fn verify_ecc(&self) -> bool {
        let mut raw = *self;
        raw.ecc_syndrome = 0;
        raw.checksum = 0;
        let bytes = as_bytes!(raw);
        let mut ecc: u32 = 0;
        for chunk in bytes.chunks(4) {
            let mut word = 0u32;
            for (i, &b) in chunk.iter().enumerate() {
                word |= (b as u32) << (i * 8);
            }
            ecc ^= word;
        }
        ecc == self.ecc_syndrome
    }

    /// COMPUTE AND STORE ECC SYNDROME.
    pub fn update_ecc(&mut self) {
        let mut raw = *self;
        raw.ecc_syndrome = 0;
        raw.checksum = 0;
        let bytes = as_bytes!(raw);
        let mut ecc: u32 = 0;
        for chunk in bytes.chunks(4) {
            let mut word = 0u32;
            for (i, &b) in chunk.iter().enumerate() {
                word |= (b as u32) << (i * 8);
            }
            ecc ^= word;
        }
        self.ecc_syndrome = ecc;
    }

    /// CHECK SRAM BATTERY RETENTION.
    ///
    /// RETURNS TRUE IF battery_flag == 0x5A (BATTERY OK).
    /// 0x00 INDICATES POWER LOSS — SRAM CONTENT MAY BE CORRUPT.
    pub fn battery_ok(&self) -> bool {
        self.battery_flag == 0x5A
    }

    /// COMPUTE VELOCITY VECTOR MAGNITUDE.
    pub fn velocity_magnitude(&self) -> f32 {
        (self.velocity_x.powi(2) + self.velocity_y.powi(2) + self.velocity_z.powi(2)).sqrt()
    }
}
