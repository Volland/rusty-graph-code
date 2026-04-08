//! Chapter 7 — the full pipeline.
//!
//! Steps:
//!   1. Load schema.ttl + data.ttl into an oxigraph store (base).
//!   2. Run reasonable over the same files to get the closure.
//!   3. Load the closure into the base store in a named graph
//!      `ex:inferred`.
//!   4. Write a combined "schema + data + closure" Turtle file
//!      to a temporary path so rudof_lib can validate the
//!      enriched graph. (rudof_lib loads files, not stores.)
//!   5. Run rudof_lib SHACL validation against that temp file.
//!   6. Emit a structured JSON report.
//!   7. If `strict` is true, return Err on any violation so
//!      main() can exit with a non-zero code.

use std::path::Path;

use anyhow::{Context, Result};
use graph::io::RdfFormat;
use graph::model::NamedNodeRef;
use serde::Serialize;

#[derive(Serialize)]
pub struct PipelineReport {
    pub base_triples: usize,
    pub closure_triples: usize,
    pub inferred_delta: i64,
    pub shacl: validator::Report,
    pub sample_queries: Vec<QueryResult>,
}

#[derive(Serialize)]
pub struct QueryResult {
    pub name: String,
    pub rows: Vec<Vec<(String, String)>>,
}

/// Run the full pipeline. Returns the structured report.
///
/// `strict` does not affect the return value; callers read
/// `report.shacl.conforms` to decide whether to exit non-zero.
pub fn run_pipeline(data_dir: &Path) -> Result<PipelineReport> {
    let schema = data_dir.join("schema.ttl");
    let data = data_dir.join("data.ttl");
    let shapes = data_dir.join("shapes.ttl");

    // --- 1. Base store -------------------------------------------------
    let store = graph::new_store()?;
    graph::load_turtle_file(&store, &schema)?;
    graph::load_turtle_file(&store, &data)?;
    let base_triples = count_all(&store)?;

    // --- 2. Run reasonable --------------------------------------------
    let inputs = [schema.clone(), data.clone()];
    let closure = reasoner::closure_as_ntriples(&inputs)
        .context("reasonable failed")?;
    let closure_triples = reasoner::count_lines(&closure);

    // --- 3. Load closure into ex:inferred -----------------------------
    let inferred_graph = format!("{}inferred", graph::EX);
    let graph_iri = NamedNodeRef::new(&inferred_graph)?;
    let parser = graph::io::RdfParser::from_format(RdfFormat::NTriples)
        .without_named_graphs()
        .with_default_graph(graph_iri);
    store
        .load_from_slice(parser, closure.as_bytes())
        .context("loading closure into ex:inferred")?;

    // --- 4. Write a combined file for rudof_lib -----------------------
    //
    // rudof_lib reads Turtle from a file handle. We dump the default
    // graph (schema + data) and append the inferred closure so
    // SHACL sees everything — including materialized subclass
    // triples that let sh:class work over supertypes.
    let tmp = std::env::temp_dir().join("ares-ch7-enriched.ttl");
    let mut enriched: String = std::fs::read_to_string(&data)
        .context("reading data.ttl for enriched build")?;
    enriched.push('\n');
    enriched.push_str(&std::fs::read_to_string(&schema)?);
    enriched.push('\n');
    // The closure is N-Triples; that is a legal subset of Turtle,
    // so we can concatenate it directly.
    enriched.push_str(&closure);
    std::fs::write(&tmp, &enriched)
        .with_context(|| format!("writing {}", tmp.display()))?;

    // --- 5. SHACL validation ------------------------------------------
    let shacl = validator::validate_shacl(&tmp, &shapes)?;

    // --- 6. Collect a few interesting query results -------------------
    let mut sample_queries = Vec::new();

    sample_queries.push(QueryResult {
        name: "agents-by-role".to_string(),
        rows: graph::select_rows(
            &store,
            r#"
            PREFIX rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            PREFIX ex:   <http://example.org/agent#>
            SELECT ?agent ?role
            WHERE {
                GRAPH ex:inferred {
                    ?agent rdf:type ?role .
                    ?role rdfs:subClassOf* ex:Agent .
                }
            }
            ORDER BY ?agent
            "#,
        )?,
    });

    sample_queries.push(QueryResult {
        name: "confident-observations".to_string(),
        rows: graph::select_rows(
            &store,
            r#"
            PREFIX ex: <http://example.org/agent#>
            SELECT ?obs
            WHERE {
                GRAPH ex:inferred { ?obs a ex:ConfidentObservation . }
            }
            "#,
        )?,
    });

    Ok(PipelineReport {
        base_triples,
        closure_triples,
        inferred_delta: closure_triples as i64 - base_triples as i64,
        shacl,
        sample_queries,
    })
}

/// Entry point used by `main.rs`.
pub fn run(data_dir: &Path, strict: bool) -> Result<()> {
    println!("=== Chapter 7 ================================================");
    let report = run_pipeline(data_dir)?;
    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");

    if strict && !report.shacl.conforms {
        anyhow::bail!(
            "pipeline: SHACL validation failed with {} violations",
            report.shacl.violations.len()
        );
    }
    Ok(())
}

fn count_all(store: &graph::Store) -> Result<usize> {
    let mut n = 0;
    for q in store.iter() {
        let _ = q?;
        n += 1;
    }
    Ok(n)
}
