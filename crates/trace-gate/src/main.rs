//! `trace-gate` — CI coverage gate binary.
//!
//! Scans source for FR annotations, checks against a manifest, exits 0 on
//! full coverage or 1 if any FR is missing.
//!
//! ```text
//! USAGE:
//!     trace-gate [OPTIONS]
//!
//! OPTIONS:
//!     --manifest <FILE>   Path to trace-gate.toml  [default: trace-gate.toml]
//!     --src <DIR>         Source directory to scan  [default: src]
//!     --push <URL>        Push results to Tracera ingest (best-effort, no exit-code effect)
//!     --json              Emit JSON summary to stdout
//! ```

mod coverage;
mod manifest;
mod push;

use std::path::PathBuf;
use std::process;

use clap::Parser;
use traceability_core::CoverageState;
use traceability_decorators::{patterns::Patterns, scan_dir};

use coverage::CoverageSummary;
use manifest::Manifest;

#[derive(Parser, Debug)]
#[command(
    name = "trace-gate",
    version,
    about = "CI gate: verify FR coverage via source annotations"
)]
struct Args {
    /// Path to the trace-gate.toml manifest.
    #[arg(long, default_value = "trace-gate.toml")]
    manifest: String,

    /// Source directory to scan for annotations.
    #[arg(long, default_value = "src")]
    src: String,

    /// POST coverage set to this Tracera ingest URL (best-effort, does not
    /// affect exit code).
    #[arg(long)]
    push: Option<String>,

    /// Emit full JSON summary to stdout in addition to human-readable output.
    #[arg(long)]
    json: bool,
}

// FR: FR-GATE-001
fn main() {
    let args = Args::parse();

    // ── Load manifest ────────────────────────────────────────────────────────
    let manifest = match Manifest::load(&args.manifest) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("trace-gate: {e}");
            process::exit(2);
        }
    };

    if manifest.requirement.is_empty() {
        eprintln!("trace-gate: manifest has no [[requirement]] entries — nothing to check.");
        process::exit(0);
    }

    // ── Scan source ──────────────────────────────────────────────────────────
    let patterns = Patterns::new();
    let src_path = PathBuf::from(&args.src);
    let scan_links = match scan_dir(&src_path, &patterns) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("trace-gate: scan error: {e}");
            process::exit(2);
        }
    };

    // ── Build coverage summary ───────────────────────────────────────────────
    let summary = CoverageSummary::build(&manifest.requirement, &scan_links);

    // ── Human-readable report ────────────────────────────────────────────────
    println!(
        "\ntrace-gate: {} FR(s) checked — {} covered, {} missing\n",
        manifest.requirement.len(),
        summary.covered_count,
        summary.missing_count,
    );

    for req in &summary.requirements {
        match req.state {
            CoverageState::Covered => {
                println!(
                    "  ✓  {} — covered at {} location(s)",
                    req.fr_id,
                    req.found_at.len()
                );
                for loc in &req.found_at {
                    let sym = if loc.symbol.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", loc.symbol)
                    };
                    println!("       {}:{}{}", loc.file, loc.line, sym);
                }
            }
            _ => {
                println!("  ✗  {} — MISSING ({})", req.fr_id, req.description);
            }
        }
    }
    println!();

    // ── Optional JSON output ─────────────────────────────────────────────────
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&summary).unwrap_or_default()
        );
    }

    // ── Optional push ────────────────────────────────────────────────────────
    if let Some(url) = &args.push {
        push::push_to_tracera(url, &summary);
    }

    // ── Exit code ────────────────────────────────────────────────────────────
    if summary.all_covered {
        process::exit(0);
    } else {
        eprintln!(
            "trace-gate: FAIL — {} FR(s) not covered in source",
            summary.missing_count
        );
        process::exit(1);
    }
}
