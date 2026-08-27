//--------------------------------------------------------------------
// MODULE:        jpegls-driver/src/main.rs
// PURPOSE:       AP-101 codec-parity DRIVER (stage 1 reference build).
//                Reads ONE DICOM file with JPEG-LS (1.2.840.10008.1.2.80)
//                encapsulated PixelData, decodes EVERY component via
//                upstream pure_jpegls, writes raw pixels to stdout.
//
//                STDOUT FORMAT (byte-exact, machine-independent):
//                  W=<cols> H=<rows> SPP=<n> BITS=<b>\n
//                  <u16 LE pixel bytes, component 0>
//                  <u16 LE pixel bytes, component 1>
//                  ...
//
//                Exit codes: 0 = decoded, 1 = usage, 2 = decode error.
//                ALL diagnostics go to STDERR; stdout carries ONLY
//                the header line + raw pixels (parity compares stdout).
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-08-27
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   NO dependency on Clarus code. Pure, self-contained
//                Explicit-VR-Little-Endian DICOM walk.
//--------------------------------------------------------------------

use std::io::Write;
use std::process::ExitCode;

// ---------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------

const TAG_PIXEL_DATA: u32 = 0x7FE0_0010;
const TAG_ROWS: u32 = 0x0028_0010;
const TAG_COLUMNS: u32 = 0x0028_0011;
const TAG_SAMPLES_PER_PIXEL: u32 = 0x0028_0002;
const TAG_BITS_ALLOCATED: u32 = 0x0028_0100;

/// VRs that use 4-byte lengths (plus 2 reserved bytes) in Explicit VR LE.
fn is_long_vr(vr: &str) -> bool {
    matches!(
        vr,
        "OB" | "OD" | "OF" | "OL" | "OV" | "OW" | "SQ" | "SV" | "UC" | "UN" | "UR" | "UT" | "UV"
    )
}

/// VRs that may legitimately carry undefined length (encapsulation).
fn is_capsule_vr(vr: &str) -> bool {
    matches!(vr, "OB" | "OW" | "OD" | "OF" | "OL" | "OV" | "UN" | "SQ")
}

// ---------------------------------------------------------------------
// Metadata collection
// ---------------------------------------------------------------------

#[derive(Default, Debug, Clone)]
struct Meta {
    rows: u32,
    columns: u32,
    samples_per_pixel: u32,
    bits_allocated: u32,
}

/// One walk over the dataset: collect Rows/Columns/SPP/BitsAllocated and
/// the encapsulated PixelData fragments (first capsule level only).
///
/// Handles nested SQ/items via a depth counter so that (7FE0,0010) found
/// inside a sequence is not mistaken for the top-level PixelData.
fn walk_dataset(data: &[u8]) -> Result<(Meta, Vec<Vec<u8>>), String> {
    let mut pos = dicom_start(data).ok_or("file too small for DICOM")?;
    let end = data.len();
    let mut meta = Meta {
        rows: 0,
        columns: 0,
        samples_per_pixel: 1,
        bits_allocated: 8,
    };
    let mut fragments: Vec<Vec<u8>> = Vec::new();

    // Depth of open undefined-length sequences above the current position.
    let mut sq_depth: u32 = 0;

    while pos + 8 <= end {
        let group = u16::from_le_bytes([data[pos], data[pos + 1]]);
        let elem = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
        pos += 4;

        // ---- Item / delimitation tags (group 0xFFFE) ----
        if group == 0xFFFE {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
            match elem {
                0xE000 => {
                    // Item. Undefined length: skip to matching E00D.
                    if len == 0xFFFF_FFFF {
                        pos = skip_to_item_delim(data, pos).ok_or("unterminated item")?;
                    } else {
                        if pos + len > end {
                            return Err("item extends past data".into());
                        }
                        pos += len;
                    }
                    continue;
                }
                0xE0DD => {
                    if sq_depth == 0 {
                        // PixelData capsule closed at top level: done.
                        break;
                    }
                    sq_depth -= 1;
                    continue;
                }
                0xE00D => continue, // item delimiter (handled by skip above)
                _ => {
                    // Unexpected tag inside capsule walk: stop.
                    break;
                }
            }
        }

        let vr = std::str::from_utf8(&data[pos..pos + 2]).unwrap_or("UN");
        pos += 2;

        let (len, undefined) = if is_long_vr(vr) {
            if pos + 6 > end {
                return Err("truncated at long VR length".into());
            }
            pos += 2; // reserved
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
            (len, len == 0xFFFF_FFFF)
        } else {
            if pos + 2 > end {
                return Err("truncated at short VR length".into());
            }
            let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            (len, len == 0xFFFF)
        };

        let tag = (group as u32) << 16 | elem as u32;

        if undefined {
            if !is_capsule_vr(vr) {
                return Err(format!("undefined length for non-capsule VR {vr}"));
            }
            if vr == "SQ" {
                sq_depth += 1;
                continue;
            }
            // Encapsulated pixel-data-like element.
            if tag == TAG_PIXEL_DATA && sq_depth == 0 {
                fragments = collect_fragments(data, pos)?;
                break; // PixelData terminates the interesting walk
            } else {
                // Skip the whole capsule without collecting.
                pos = skip_to_sequence_delim(data, pos).ok_or("unterminated capsule")?;
                continue;
            }
        }

        // Defined length element.
        match tag {
            TAG_ROWS if len == 2 => {
                meta.rows = u16::from_le_bytes([data[pos], data[pos + 1]]) as u32
            }
            TAG_COLUMNS if len == 2 => {
                meta.columns = u16::from_le_bytes([data[pos], data[pos + 1]]) as u32
            }
            TAG_SAMPLES_PER_PIXEL if len == 2 => {
                meta.samples_per_pixel = u16::from_le_bytes([data[pos], data[pos + 1]]) as u32
            }
            TAG_BITS_ALLOCATED if len == 2 => {
                meta.bits_allocated = u16::from_le_bytes([data[pos], data[pos + 1]]) as u32
            }
            TAG_PIXEL_DATA if sq_depth == 0 => {
                // Defined-length PixelData (uncompressed) is not JPEG-LS.
                return Err("PixelData has defined length (not encapsulated)".into());
            }
            _ => {}
        }
        pos += len;
    }

    if meta.rows == 0 || meta.columns == 0 {
        return Err("missing Rows/Columns metadata".into());
    }
    if fragments.is_empty() {
        return Err("no encapsulated PixelData fragments found".into());
    }
    Ok((meta, fragments))
}

/// Collect item fragments until the sequence delimiter closes the capsule.
fn collect_fragments(data: &[u8], mut pos: usize) -> Result<Vec<Vec<u8>>, String> {
    let end = data.len();
    let mut out = Vec::new();
    while pos + 8 <= end {
        let group = u16::from_le_bytes([data[pos], data[pos + 1]]);
        let elem = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
        pos += 4;
        let len =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        if group != 0xFFFE {
            return Err("non-item tag inside pixel data capsule".into());
        }
        match elem {
            0xE000 => {
                if len == 0 {
                    continue; // Basic Offset Table (empty) or empty item
                }
                if len == 0xFFFF_FFFF {
                    return Err("undefined-length item inside pixel data".into());
                }
                if pos + len > end {
                    return Err("fragment extends past data".into());
                }
                out.push(data[pos..pos + len].to_vec());
                pos += len;
            }
            0xE0DD => break,
            0xE00D => break,
            _ => break,
        }
    }
    Ok(out)
}

/// Skip bytes up to and including an item delimiter (FFFE,E00D,00000000).
fn skip_to_item_delim(data: &[u8], mut pos: usize) -> Option<usize> {
    let end = data.len();
    while pos + 8 <= end {
        let tag = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let len = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            as usize;
        if tag == 0xFFFE_E00D && len == 0 {
            return Some(pos + 8);
        }
        pos += 1;
    }
    None
}

/// Skip a capsule to just after its sequence delimiter (FFFE,E0DD,00000000).
fn skip_to_sequence_delim(data: &[u8], mut pos: usize) -> Option<usize> {
    let end = data.len();
    while pos + 8 <= end {
        let tag = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let len = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            as usize;
        if tag == 0xFFFE_E0DD && len == 0 {
            return Some(pos + 8);
        }
        pos += 1;
    }
    None
}

fn dicom_start(data: &[u8]) -> Option<usize> {
    if data.len() < 132 {
        return None;
    }
    if &data[128..132] == b"DICM" {
        Some(132)
    } else {
        Some(0)
    }
}

// ---------------------------------------------------------------------
// Decode
// ---------------------------------------------------------------------

fn decode_file(path: &str) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("read: {e}"))?;
    let (meta, fragments) = walk_dataset(&data)?;

    let spp = meta.samples_per_pixel.max(1) as usize;
    if fragments.len() < spp {
        return Err(format!(
            "only {} fragment(s) for SPP={spp}",
            fragments.len()
        ));
    }

    let mut stdout = std::io::stdout().lock();
    writeln!(
        stdout,
        "W={} H={} SPP={} BITS={}",
        meta.columns, meta.rows, spp, meta.bits_allocated
    )
    .map_err(|e| format!("stdout: {e}"))?;

    for comp in 0..spp {
        let frag = &fragments[comp];
        let (pixels, w, h) = jpegls::decode(frag, meta.columns, meta.rows)
            .map_err(|e| format!("decode component {comp}: {e}"))?;
        if w != meta.columns || h != meta.rows {
            return Err(format!(
                "component {comp}: decoded {w}x{h}, expected {}x{}",
                meta.columns, meta.rows
            ));
        }
        for px in &pixels {
            stdout
                .write_all(&px.to_le_bytes())
                .map_err(|e| format!("stdout: {e}"))?;
        }
    }
    stdout.flush().map_err(|e| format!("stdout: {e}"))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: jpegls-driver <dicom-file>");
        return ExitCode::from(1);
    }
    match decode_file(&args[1]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("jpegls-driver: error: {e}");
            ExitCode::from(2)
        }
    }
}
