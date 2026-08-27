//--------------------------------------------------------------------
// MODULE:        jpegls-parity/src/main.rs
// PURPOSE:       AP-101 codec-parity HARNESS (stage 1).
//
//                CLI:
//                  jpegls-parity codec-parity <dir> <ref-cmd> <test-cmd>
//                                   [--max N] [--ext E]
//
//                For every file in <dir> (sorted by path):
//                  1. run "<ref-cmd> <file>"  -> capture stdout bytes,
//                     stderr, exit code, wall time;
//                  2. run "<test-cmd> <file>" -> same;
//                  3. byte-compare stdout of both runs.
//
//                stdout comparison is TOTAL: header line + every pixel
//                byte. A single differing byte = mismatch. This is the
//                strongest check (parity byte-diff), deliberately chosen
//                over CRC/SEU machinery.
//
//                Output: an IBM-CRT-style ASCII table + verdict line.
//                Exit code: 0 = ALL files matched; 1 = any mismatch
//                or command failure.
//
//                NOTE: the harness measures wall time of the WHOLE
//                process (spawn..exit). For a proper hot-loop benchmark
//                see the stage-2 profile anchors; parity timing here is
//                indicative only.
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-08-27
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   ZERO dependencies beyond std.
//--------------------------------------------------------------------

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output};
use std::time::Instant;

// ---------------------------------------------------------------------
// Run helpers
// ---------------------------------------------------------------------

#[derive(Debug)]
struct Run {
    exit_ok: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    micros: u128,
}

fn run_cmd(cmd: &str, file: &Path) -> Run {
    let mut parts = cmd.split_whitespace();
    let program = match parts.next() {
        Some(p) => p,
        None => {
            return Run {
                exit_ok: false,
                stdout: Vec::new(),
                stderr: b"empty command".to_vec(),
                micros: 0,
            }
        }
    };
    let args: Vec<&str> = parts.collect();
    let t0 = Instant::now();
    let result: std::io::Result<Output> = Command::new(program).args(&args).arg(file).output();
    let micros = t0.elapsed().as_micros();
    match result {
        Ok(out) => Run {
            exit_ok: out.status.success(),
            stdout: out.stdout,
            stderr: out.stderr,
            micros,
        },
        Err(e) => Run {
            exit_ok: false,
            stdout: Vec::new(),
            stderr: format!("spawn failed: {e}").into_bytes(),
            micros,
        },
    }
}

// ---------------------------------------------------------------------
// Table formatting
// ---------------------------------------------------------------------

const RULE: &str = "----------------------------------------------------------------------";

fn print_header() {
    println!("{RULE}");
    println!("AP-101 CODEC-PARITY  |  JPEG-LS DECODE BYTE-DIFF AUDIT");
    println!("{RULE}");
    println!(
        "{:<34} {:>10} {:>8} {:>8} {:>8}  {}",
        "FILE", "DIMS", "REF-us", "TEST-us", "MATCH", "NOTE"
    );
    println!("{RULE}");
}

fn trunc_mid(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(3) / 2;
    format!("{}...{}", &s[..keep], &s[s.len() - keep..])
}

// ---------------------------------------------------------------------
// Corpus scan
// ---------------------------------------------------------------------

fn collect_files(dir: &Path, ext: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir).map_err(|e| format!("read_dir {dir:?}: {e}"))?;
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        if let Some(e) = p.extension().and_then(|e| e.to_str()) {
            if e.eq_ignore_ascii_case(ext) || ext == "*" {
                out.push(p);
            }
        } else if ext == "*" {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

// ---------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 || args[1] != "codec-parity" {
        eprintln!(
            "usage: jpegls-parity codec-parity <dir> <ref-cmd> <test-cmd> [--max N] [--ext E]"
        );
        return ExitCode::from(1);
    }

    let dir = PathBuf::from(&args[2]);
    let ref_cmd = &args[3];
    let test_cmd = &args[4];

    let mut max: Option<usize> = None;
    let mut ext: String = "dcm".into();
    let mut i = 5;
    while i < args.len() {
        match args[i].as_str() {
            "--max" => {
                i += 1;
                if i < args.len() {
                    max = args[i].parse().ok();
                }
            }
            "--ext" => {
                i += 1;
                if i < args.len() {
                    ext = args[i].clone();
                }
            }
            other => {
                eprintln!("unknown option: {other}");
                return ExitCode::from(1);
            }
        }
        i += 1;
    }

    let files = match collect_files(&dir, &ext) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("corpus error: {e}");
            return ExitCode::from(1);
        }
    };
    if files.is_empty() {
        eprintln!("no files in {dir:?} (ext .{ext})");
        return ExitCode::from(1);
    }

    let mut files: Vec<PathBuf> = if let Some(n) = max {
        files.into_iter().take(n).collect()
    } else {
        files
    };
    files.sort();

    print_header();

    let mut matched = 0usize;
    let mut failed = 0usize;
    let mut total_ref_us: u128 = 0;
    let mut total_test_us: u128 = 0;

    for f in &files {
        let name = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let r_ref = run_cmd(ref_cmd, f);
        let r_test = run_cmd(test_cmd, f);

        total_ref_us += r_ref.micros;
        total_test_us += r_test.micros;

        // Dimension header = first line of ref stdout.
        let dims = dims_of(&r_ref.stdout);

        let (verdict, note): (&str, String) = if !r_ref.exit_ok {
            ("FAIL", "ref cmd failed".into())
        } else if !r_test.exit_ok {
            ("FAIL", "test cmd failed".into())
        } else if r_ref.stdout != r_test.stdout {
            (
                "FAIL",
                format!(
                    "stdout diff at {}",
                    first_diff(&r_ref.stdout, &r_test.stdout)
                ),
            )
        } else {
            ("OK", String::new())
        };

        if verdict == "OK" {
            matched += 1;
        } else {
            failed += 1;
            if !r_ref.stderr.is_empty() {
                eprintln!(
                    "  [ref-stderr {}] {}",
                    name,
                    String::from_utf8_lossy(&r_ref.stderr).trim()
                );
            }
            if !r_test.stderr.is_empty() {
                eprintln!(
                    "  [test-stderr {}] {}",
                    name,
                    String::from_utf8_lossy(&r_test.stderr).trim()
                );
            }
        }

        println!(
            "{:<34} {:>10} {:>8} {:>8} {:>8}  {}",
            trunc_mid(&name, 34),
            dims,
            r_ref.micros,
            r_test.micros,
            verdict,
            note
        );
    }

    println!("{RULE}");
    println!(
        "TOTAL: {}/{} MATCH  |  ref wall {:.2} s  test wall {:.2} s",
        matched,
        files.len(),
        total_ref_us as f64 / 1e6,
        total_test_us as f64 / 1e6
    );
    println!("{RULE}");

    if failed > 0 {
        eprintln!("codec-parity: FAILED on {failed} file(s)");
        return ExitCode::from(1);
    }
    let _ = std::io::stdout().flush();
    ExitCode::SUCCESS
}

fn dims_of(stdout: &[u8]) -> String {
    if let Some(nl) = stdout.iter().position(|&b| b == b'\n') {
        String::from_utf8_lossy(&stdout[..nl]).to_string()
    } else {
        "-".into()
    }
}

fn first_diff(a: &[u8], b: &[u8]) -> String {
    let n = a.len().min(b.len());
    for i in 0..n {
        if a[i] != b[i] {
            return format!("byte {i}: {:#04x} vs {:#04x}", a[i], b[i]);
        }
    }
    format!("length {} vs {}", a.len(), b.len())
}
