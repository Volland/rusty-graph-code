//! Chapter 9 — GrafeoDB as a second RDF engine.
//!
//! This module is the practical companion to Chapter 9. It boots an
//! in-memory Grafeo database and drives it through the same
//! "what has Ares observed?" question we asked Oxigraph in Chapter 1.
//!
//! Grafeo's SPARQL front end is exposed via `execute_language` with
//! the language name `"sparql"`. On builds that were compiled without
//! the `sparql` feature the call returns an error; we catch that and
//! fall back to the native LPG path so the sample still runs.

use anyhow::Result;
use grafeo::{GrafeoDB, Value};

pub fn run() -> Result<()> {
    println!("=== Chapter 9 ================================================");
    let db = GrafeoDB::new_in_memory();

    // We seed the same three facts Chapter 1's data.ttl holds: Ares
    // observed an arXiv paper at 09:00 with 0.92 confidence. The LPG
    // form is the shape Grafeo can always answer; the RDF form is the
    // optional one we try afterwards.
    let ares = db.create_node_with_props(
        &["Agent"],
        vec![("id", Value::String("ares".to_string()))],
    );
    let paper = db.create_node_with_props(
        &["Paper"],
        vec![
            ("arxivId", Value::String("2401.00123".to_string())),
            (
                "title",
                Value::String("Agent memory as graphs".to_string()),
            ),
        ],
    );
    let obs = db.create_node_with_props(
        &["Observation"],
        vec![("id", Value::String("obs_0001".to_string()))],
    );

    db.create_edge_with_props(
        ares,
        obs,
        "ASSERTED",
        vec![
            (
                "at",
                Value::String("2025-01-15T09:00:00Z".to_string()),
            ),
            ("confidence", Value::Float64(0.92)),
        ],
    );
    db.create_edge(obs, paper, "HAS_SUBJECT");

    println!(
        "seeded {} nodes and {} edges",
        db.node_count(),
        db.edge_count()
    );

    // Ask Grafeo the Chapter-1 question through GQL.
    let gql = r#"
        MATCH (a:Agent {id: 'ares'})
              -[r:ASSERTED]->(o:Observation)
              -[:HAS_SUBJECT]->(p:Paper)
        RETURN p.arxivId AS arxiv_id,
               r.at       AS observed_at,
               r.confidence AS confidence
        ORDER BY r.at ASC
    "#;

    println!("\n-- GQL: what has Ares observed? --");
    let result = db.execute(gql)?;
    for row in result.rows() {
        println!("{row:?}");
    }

    // Attempt the same question over SPARQL 1.1. This exercises
    // Grafeo's execute_language dispatcher. If the build does not
    // include the SPARQL front end, we surface that as a note rather
    // than a hard failure — the sample is still useful.
    println!("\n-- SPARQL (optional): what has Ares observed? --");
    let sparql = r#"
        PREFIX ex: <http://example.org/agent#>

        SELECT ?obs WHERE {
            ?obs a ex:Observation .
        }
    "#;
    match db.execute_language(sparql, "sparql", None) {
        Ok(r) => println!("sparql returned {} row(s)", r.row_count()),
        Err(e) => println!("sparql unavailable on this build: {e}"),
    }

    Ok(())
}
