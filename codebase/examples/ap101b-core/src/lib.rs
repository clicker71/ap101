//--------------------------------------------------------------------
// MODULE:        ap101b-core/src/lib.rs
// PURPOSE:       AP-101B FERRITE CORE NAVIGATION STATE.
//                CRC-32 PROTECTED. COMPILE-TIME PADDING CHECK.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   256 KB USABLE CORE. NO HEAP WITHOUT CAUSE.
//--------------------------------------------------------------------

use ferrite_core::checksum::{Checksum, Crc32};
use ferrite_core::as_bytes;

/// NAVIGATION SYSTEM STATE.
///
/// #[repr(C)] GUARANTEES PREDICTABLE FIELD LAYOUT.
/// CRC-32/ISO-HDLC PROTECTS AGAINST COSMIC-RAY SEU.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NavigationState {
    pub timestamp: u64,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub status_flags: u32,
    pub checksum: u32,  // CRC-32/ISO-HDLC
}

// COMPILE-TIME ZERO-PADDING CHECK
// EXPECTED_SIZE = u64(8) + f32*3(12) + u32(4) + u32(4) = 28 + 4 TAIL PADDING = 32
ferrite_core::assert_no_padding!(
    NavigationState,
    timestamp: u64,
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
    status_flags: u32,
    checksum: u32
);

impl Default for NavigationState {
    fn default() -> Self {
        let mut state = Self {
            timestamp: 0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            status_flags: 0,
            checksum: 0,
        };
        state.checksum = state.compute_checksum();
        state
    }
}

impl NavigationState {
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

    /// COMPUTE VELOCITY VECTOR MAGNITUDE.
    pub fn velocity_magnitude(&self) -> f32 {
        (self.velocity_x.powi(2) + self.velocity_y.powi(2) + self.velocity_z.powi(2)).sqrt()
    }
}
