//--------------------------------------------------------------------
// MODULE:        ferrite-core/src/checksum.rs
// PURPOSE:       CHECKSUM TRAIT FOR STRUCTURAL INTEGRITY.
//                Checksum TRAIT, Crc16, Crc32, XorFold, as_bytes! MACRO.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   STACK-ONLY. O(n) IN STRUCT SIZE. ZERO ALLOCATIONS.
//--------------------------------------------------------------------

/// CHECKSUM TRAIT FOR STRUCTURAL INTEGRITY.
///
/// ## IMPLEMENTATION REQUIREMENTS
///
/// 1. DETERMINISTIC: SAME DATA → SAME CHECKSUM.
/// 2. AVALANCHE: ANY SINGLE BIT FLIP CHANGES CHECKSUM.
/// 3. PERFORMANCE: O(n) IN STRUCT SIZE.
/// 4. STACK-ONLY: ZERO HEAP ALLOCATIONS.
///
/// ## STANDARD IMPLEMENTATIONS
///
/// - `Crc16` — CRC-16/XMODEM (2 BYTES, GOOD BIT-ERROR DETECTION).
/// - `Crc32` — CRC-32/ISO-HDLC (4 BYTES, EXCELLENT DETECTION).
/// - `XorFold` — XOR WITH ROTATION (FASTER, WEAKER, TEST-ONLY).
pub trait Checksum {
    type Output: PartialEq + Copy;

    fn compute<T: AsRef<[u8]>>(data: &T) -> Self::Output;
    fn verify<T: AsRef<[u8]>>(data: &T, expected: Self::Output) -> bool {
        Self::compute(data) == expected
    }
}

/// CRC-16/XMODEM — GOOD SIZE/RELIABILITY BALANCE.
pub struct Crc16;

impl Checksum for Crc16 {
    type Output = u16;

    fn compute<T: AsRef<[u8]>>(data: &T) -> u16 {
        let mut crc: u16 = 0x0000;
        for &byte in data.as_ref() {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }
}

/// CRC-32/ISO-HDLC — MAX PROTECTION, REASONABLE OVERHEAD.
pub struct Crc32;

impl Checksum for Crc32 {
    type Output = u32;

    fn compute<T: AsRef<[u8]>>(data: &T) -> u32 {
        let mut crc: u32 = 0xFFFF_FFFF;
        for &byte in data.as_ref() {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB8_8320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }
}

/// FAST XOR-FOLD WITH ROTATION (NOT FOR PRODUCTION SEU PROTECTION!).
///
/// TEST AND DEBUG ONLY.
/// DO NOT USE WHERE DATA INTEGRITY MATTERS.
///
/// NOTE: MAIN LOOP PROCESSES 4-BYTE CHUNKS LE.
/// REMAINDER BYTES PROCESSED IN REVERSE ORDER (MSB-FIRST).
/// THIS ASYMMETRY IS BY DESIGN — ACCEPTABLE FOR TEST-ONLY USE.
pub struct XorFold;

impl Checksum for XorFold {
    type Output = u32;

    fn compute<T: AsRef<[u8]>>(data: &T) -> u32 {
        let bytes = data.as_ref();
        let mut hash: u32 = 0xDEAD_BEEF;
        let mut i = 0;
        while i + 4 <= bytes.len() {
            let chunk = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
            hash = hash.rotate_left(7) ^ chunk;
            i += 4;
        }
        // REMAINDER BYTES
        let mut remainder: u32 = 0;
        for j in (0..bytes.len() - i).rev() {
            remainder = (remainder << 8) | bytes[i + j] as u32;
        }
        hash ^ remainder
    }
}

/// OBTAIN RAW BYTE REPRESENTATION OF VALUE (NO TRANSMUTE).
///
/// ## SAFETY
///
/// CALLER MUST ENSURE:
/// - `$val` HAS NO INVALID BIT PATTERNS (E.G. `bool` MUST BE 0 OR 1).
/// - `$val` HAS NO PADDING BYTES (USE `assert_no_padding!` TO VERIFY).
///   UNINITIALIZED PADDING BYTES CAUSE NON-DETERMINISTIC CHECKSUMS.
///
/// ## EXAMPLE
///
/// ```ignore
/// let state = NavigationState { ... };
/// let expected = Crc32::compute(&as_bytes!(state));
/// ```
#[macro_export]
macro_rules! as_bytes {
    ($val:expr) => {{
        let val = &$val;
        let ptr = (val as *const _) as *const u8;
        let len = core::mem::size_of_val(val);
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }};
}
