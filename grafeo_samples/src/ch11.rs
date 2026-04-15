//! Chapter 11 — The Ares scenario as a labelled property graph.
//!
//! Same three nodes, same two edges, but now the timestamp and
//! confidence live on the `ASSERTED` edge itself rather than on a
//! reified observation. The query shapes that fall out of this model
//! are the traversal-heavy ones Chapter 11 walks through.

use anyhow::Result;
use grafeo::{GrafeoDB, Value};

pub fn run() -> Result<()> {
    println!("=== Chapter 11 ===============================================");
    let db = GrafeoDB::new_in_memory();

    let ares = db.create_node_with_props(
        &["Agent"],
        vec![
            ("id", Value::String("ares".to_string())),
            ("trust", Value::Float64(0.87)),
        ],
    );
    let gus = db.create_node_with_props(
        &["Agent"],
        vec![
            ("id", Value::String("gus".to_string())),
            ("trust", Value::Float64(0.75)),
        ],
    );
    let calliope = db.create_node_with_props(
        &["Agent"],
        vec![
            ("id", Value::String("calliope".to_string())),
            ("trust", Value::Float64(0.80)),
        ],
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

    for (agent, at, conf) in [
        (ares, "2025-01-15T09:00:00Z", 0.92),
        (gus, "2025-01-15T09:15:00Z", 0.88),
        (calliope, "2025-01-15T09:30:00Z", 0.81),
    ] {
        let obs = db.create_node(&["Observation"]);
        db.create_edge_with_props(
            agent,
            obs,
            "ASSERTED",
            vec![
                ("at", Value::String(at.to_string())),
                ("confidence", Value::Float64(conf)),
            ],
        );
        db.create_edge(obs, paper, "HAS_SUBJECT");
    }

    // The Chapter-1 question, in GQL: what has Ares observed?
    println!("\n-- GQL: what has Ares observed? --");
    let gql = r#"
        MATCH (a:Agent {id: 'ares'})
              -[r:ASSERTED]->(o:Observation)
              -[:HAS_SUBJECT]->(p:Paper)
        RETURN p.arxivId, r.at, r.confidence
        ORDER BY r.at ASC
    "#;
    let result = db.execute(gql)?;
    for row in result.rows() {
        println!("  {row:?}");
    }

    // The multi-hop trust question from the chapter, trimmed to what
    // our three-row fixture can answer: for every paper Ares observed,
    // how many *other* agents corroborated it?
    println!("\n-- GQL: corroborators per paper --");
    let corroborators = r#"
        MATCH (a:Agent {id: 'ares'})-[:ASSERTED]->(:Observation)
              -[:HAS_SUBJECT]->(p:Paper)
              <-[:HAS_SUBJECT]-(:Observation)
              <-[:ASSERTED]-(other:Agent)
        WHERE other.id <> 'ares'
        RETURN p.arxivId, count(DISTINCT other) AS n
    "#;
    let result = db.execute(corroborators)?;
    for row in result.rows() {
        println!("  {row:?}");
    }

    // openCypher dialect on the same engine — demonstrates that the
    // query language is per-call, not per-database.
    println!("\n-- Cypher: all agents --");
    let cypher =
        db.execute_language("MATCH (a:Agent) RETURN a.id", "cypher", None)?;
    for row in cypher.rows() {
        println!("  {row:?}");
    }

    Ok(())
}
