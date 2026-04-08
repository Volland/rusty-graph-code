//! Chapter 8 — ShEx vs SHACL.
//!
//! Demonstrates the open-vs-closed-world distinction:
//!   - data-extra.ttl has a valid observation plus an undeclared
//!     ex:mood property.
//!   - Running it against the default (open) ObservationShape
//!     from shapes.ttl: passes.
//!   - Running it against shapes-closed.ttl with sh:closed true:
//!     fails — the extra property is rejected.
//!   - Parses the equivalent ShEx schema to show rudof_lib can
//!     load ShEx directly.

use std::path::Path;

use anyhow::Result;

pub fn run(data_dir: &Path) -> Result<()> {
    println!("=== Chapter 8 ================================================");

    let data_extra = data_dir.join("data-extra.ttl");

    // --- Open SHACL: extra property is fine ---------------------------
    let open_shapes = data_dir.join("shapes.ttl");
    println!("-- SHACL (open by default) against data-extra.ttl --");
    let open_report = validator::validate_shacl(&data_extra, &open_shapes)?;
    println!("  conforms: {}", open_report.conforms);
    println!("  violations: {}", open_report.violations.len());

    // --- Closed SHACL: extra property is rejected ---------------------
    let closed_shapes = data_dir.join("shapes-closed.ttl");
    println!("\n-- SHACL with sh:closed true against data-extra.ttl --");
    let closed_report = validator::validate_shacl(&data_extra, &closed_shapes)?;
    println!("  conforms: {}", closed_report.conforms);
    println!("  violations: {}", closed_report.violations.len());
    for (i, v) in closed_report.violations.iter().enumerate() {
        println!(
            "    [{i}] {} — {}\n        focus: {}\n        component: {}",
            v.severity, v.message, v.focus_node, v.component
        );
    }

    // --- ShEx: parse the equivalent schema ---------------------------
    let shex_schema = data_dir.join("observations.shex");
    println!("\n-- parsing the ShEx version of the same schema --");
    validator::parse_shex(&shex_schema)?;
    println!("  parsed {} successfully", shex_schema.display());
    println!(
        "  (ShEx shapes are closed by default, so adding ex:mood would\n   fail validation the same way SHACL-closed does above.)"
    );

    Ok(())
}
