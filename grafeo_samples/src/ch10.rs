//! Chapter 10 — Integrity validation on a Grafeo store.
//!
//! The book chapter discusses in-engine SHACL Core validation. The
//! public `grafeo` 0.5 crate exposes a store-level integrity checker
//! (`GrafeoDB::validate`) that catches the class of errors SHACL
//! `sh:minCount`, `sh:nodeKind`, and dangling-reference constraints
//! would catch. Full SHACL shapes still travel through the Chapter 5
//! `validator` crate — this module shows the in-engine half.

use anyhow::Result;
use grafeo::{GrafeoDB, Value};

pub fn run() -> Result<()> {
    println!("=== Chapter 10 ===============================================");
    let db = GrafeoDB::new_in_memory();

    // A correctly-shaped observation: agent, observation, paper,
    // both edges wired up, confidence in [0.0, 1.0].
    let ares = db.create_node_with_props(
        &["Agent"],
        vec![
            ("id", Value::String("ares".to_string())),
            ("trust", Value::Float64(0.87)),
        ],
    );
    let paper = db.create_node_with_props(
        &["Paper"],
        vec![("arxivId", Value::String("2401.00123".to_string()))],
    );
    let obs = db.create_node_with_props(
        &["Observation"],
        vec![
            ("id", Value::String("obs_0001".to_string())),
            ("confidence", Value::Float64(0.92)),
        ],
    );
    db.create_edge(ares, obs, "ASSERTED");
    db.create_edge(obs, paper, "HAS_SUBJECT");

    let report = db.validate();
    println!(
        "integrity check: {} error(s), {} warning(s)",
        report.errors.len(),
        report.warnings.len()
    );

    // Local-invariant check: every Observation node should have
    // confidence in [0, 1]. This is exactly the shape `sh:minInclusive`
    // / `sh:maxInclusive` express, and it is cheap to run as a GQL
    // filter at validation time.
    let bad = db.execute(
        "MATCH (o:Observation)
         WHERE o.confidence < 0.0 OR o.confidence > 1.0
         RETURN o.id AS id",
    )?;
    if bad.row_count() == 0 {
        println!("confidence range: OK");
    } else {
        println!(
            "confidence range: {} violation(s)",
            bad.row_count()
        );
    }

    // Cardinality check: every Observation should have exactly one
    // ASSERTED edge incoming. Chapter 5's `sh:minCount`/`sh:maxCount`
    // in one GQL query.
    let asserters = db.execute(
        "MATCH (o:Observation)
         OPTIONAL MATCH (a:Agent)-[:ASSERTED]->(o)
         WITH o, count(a) AS n
         WHERE n <> 1
         RETURN o.id AS id, n",
    )?;
    println!(
        "ASSERTED cardinality: {} violation(s)",
        asserters.row_count()
    );

    Ok(())
}
