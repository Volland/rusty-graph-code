//! Chapter 5 — SHACL: constraining the agent knowledge graph.
//!
//! We run SHACL validation twice:
//!   1. Against the good data from Chapter 1 — expected to conform.
//!   2. Against a deliberately broken file — expected to fail with
//!      a human-readable list of violations.

use std::path::Path;

use anyhow::Result;

pub fn run(data_dir: &Path) -> Result<()> {
    println!("=== Chapter 5 ================================================");

    let shapes = data_dir.join("shapes.ttl");

    // --- Clean run ----------------------------------------------------
    let good = data_dir.join("data.ttl");
    println!("-- validating the good data file --");
    let report = validator::validate_shacl(&good, &shapes)?;
    print_report(&report);

    // --- Intentionally broken run -------------------------------------
    let bad = data_dir.join("data-bad.ttl");
    println!("\n-- validating the deliberately broken file --");
    let bad_report = validator::validate_shacl(&bad, &shapes)?;
    print_report(&bad_report);

    // --- Emit the bad report as JSON for good measure -----------------
    let json = serde_json::to_string_pretty(&bad_report)?;
    println!("\n-- same report, as JSON --");
    println!("{json}");

    Ok(())
}

fn print_report(report: &validator::Report) {
    println!("  conforms    : {}", report.conforms);
    println!("  violations  : {}", report.violations.len());
    for (i, v) in report.violations.iter().enumerate() {
        println!(
            "    [{i}] {} — {}\n        focus: {}\n        component: {}",
            v.severity, v.message, v.focus_node, v.component
        );
    }
}
