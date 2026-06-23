//--------------------------------------------------------------------
// MODULE:        ferrite-core/src/telemetry.rs
// PURPOSE:       IBM PASS CRT-STYLE DECORATIVE TELEMETRY OUTPUT.
//                IbmCrt (STD FEATURE), AuditResult, AuditReport (NO_STD).
// AUTHOR:        Daniil Solgalov <clicker71@github>
// DATE:          2026-06-22
// MACHINE:       IBM AP-101B (HONORARY)
// CONSTRAINTS:   NO_STD AuditReport USES heapless. STD IbmCrt USES println.
//--------------------------------------------------------------------

/// IBM AP-101 CRT-STYLE DECORATIVE TELEMETRY.
///
/// AVAILABLE ONLY WITH FEATURE "std".
#[cfg(feature = "std")]
pub struct IbmCrt;

#[cfg(feature = "std")]
impl IbmCrt {
    pub fn print_header(system: &str, subsystem: &str) {
        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║ IBM {:<58}║", system);
        println!("║ TARGET: {:<54}║", subsystem);
        println!("╚════════════════════════════════════════════════════════════════╝");
    }

    pub fn print_row(prefix: &str, id: &str, verification: &str, passed: bool, detail: &str) {
        let status = if passed { " COMPLIANT " } else { "ANOMALY DET" };
        println!("[{status}] {prefix}{id} | {verification:<40} | {detail}");
    }

    pub fn print_footer(all_clear: bool) {
        println!("╔════════════════════════════════════════════════════════════════╗");
        if all_clear {
            println!("║ MISSION STATUS: GO FOR LAUNCH.                               ║");
        } else {
            println!("║ MISSION STATUS: ABORT. VIOLATION DETECTED.                   ║");
        }
        println!("╚════════════════════════════════════════════════════════════════╝\n");
    }
}

/// NO_STD TELEMETRY: RETURNS STRUCTS FOR MACHINE PROCESSING.
#[derive(Debug)]
pub struct AuditResult {
    pub id: &'static str,
    pub verification: &'static str,
    pub passed: bool,
    pub detail: heapless::String<256>,
}

pub struct AuditReport {
    pub subsystem: heapless::String<64>,
    pub results: heapless::Vec<AuditResult, 16>,
    pub all_clear: bool,
}

impl AuditReport {
    pub fn new(subsystem: &str) -> Self {
        let mut sub = heapless::String::new();
        let _ = sub.push_str(subsystem);
        Self {
            subsystem: sub,
            results: heapless::Vec::new(),
            all_clear: true,
        }
    }

    /// ADD AUDIT RESULT.
    ///
    /// ## PANICS
    ///
    /// PANICS IF RESULTS CAPACITY (16) EXCEEDED.
    /// SILENT LOSS IS A DATA INTEGRITY VIOLATION —
    /// WE FAIL LOUDLY ON OVERFLOW.
    pub fn add_result(
        &mut self,
        id: &'static str,
        verification: &'static str,
        passed: bool,
        detail: &str,
    ) {
        let mut det = heapless::String::new();
        let _ = det.push_str(detail);
        self.results
            .push(AuditResult {
                id,
                verification,
                passed,
                detail: det,
            })
            .expect("AUDIT REPORT OVERFLOW: CAPACITY 16 EXCEEDED");
        self.all_clear &= passed;
    }

    /// EXPORT REPORT AS JSON STRING (REQUIRES STD).
    #[cfg(feature = "std")]
    pub fn to_json(&self) -> String {
        // PRODUCTION: USE SERDE. SIMPLIFIED FOR BREVITY.
        format!("{{\"all_clear\": {}}}", self.all_clear)
    }
}
