//! Chapter 4 — named graphs and quad stores for provenance.
//!
//! Layout we build in this module:
//!
//!   default graph            : schema + base data
//!   ex:inferred              : the OWL closure from Chapter 3
//!   ex:observations/Ares     : Ares's observations (already default)
//!   ex:observations/Calliope : Calliope's observations
//!   ex:promises              : promises from one agent to another
//!   ex:meta                  : a meta-graph that timestamps updates
//!
//! Chapter 4 walks through loading every graph, querying across
//! them with `GRAPH ?g { ... }`, and running a tiny SPARQL UPDATE
//! inside an oxigraph transaction.

use std::path::Path;

use anyhow::{Context, Result};
use graph::io::RdfFormat;
use graph::model::{GraphNameRef, NamedNodeRef};
use graph::sparql::SparqlEvaluator;

pub fn run(data_dir: &Path) -> Result<()> {
    println!("=== Chapter 4 ================================================");

    let store = graph::new_store()?;

    // --- Default graph: schema + data ---------------------------------
    graph::load_turtle_file(&store, data_dir.join("schema.ttl"))?;
    graph::load_turtle_file(&store, data_dir.join("data.ttl"))?;
    println!("default graph loaded ({} quads)", count_all(&store)?);

    // --- ex:inferred: the OWL 2 RL closure ----------------------------
    let inputs = [
        data_dir.join("schema.ttl"),
        data_dir.join("data.ttl"),
    ];
    let closure = reasoner::closure_as_ntriples(&inputs)?;
    let inferred_graph = format!("{}inferred", graph::EX);
    load_ntriples_into_named_graph(&store, &closure, &inferred_graph)?;
    println!("ex:inferred loaded  ({} triples)", reasoner::count_lines(&closure));

    // --- ex:observations/Calliope -------------------------------------
    graph::load_turtle_into_named_graph(
        &store,
        data_dir.join("observations-calliope.ttl"),
        &format!("{}observations/Calliope", graph::EX),
    )?;

    // --- Record a promise via the Rust helper -------------------------
    //
    // Ares promises to summarize the paper. We write the promise
    // into ex:promises inside a transaction so that if anything in
    // the middle fails we don't half-commit.
    record_promise(
        &store,
        "Ares",
        "Hermes",
        "SummarizeArxiv2401",
        "2025-01-15T12:00:00Z",
    )?;
    println!("\npromise recorded in ex:promises");

    // --- Query across every graph --------------------------------------
    let q = r#"
        PREFIX ex:   <http://example.org/agent#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?g ?obs ?agent ?confidence
        WHERE {
            GRAPH ?g {
                ?obs a ex:Observation ;
                     ex:assertedBy ?agent ;
                     ex:confidence ?confidence .
            }
        }
        ORDER BY ?g ?agent
    "#;
    println!("\n-- observations, bucketed by named graph --");
    print_rows(graph::select_rows(&store, q)?);

    // --- Query: what has Ares promised, when? --------------------------
    let qp = r#"
        PREFIX ex: <http://example.org/agent#>
        SELECT ?promise ?to ?task ?at
        WHERE {
            GRAPH ex:promises {
                ?promise ex:promisedBy   ex:Ares ;
                         ex:promisedTo   ?to ;
                         ex:promiseContent ?task ;
                         ex:promisedAt   ?at .
            }
        }
    "#;
    println!("-- Ares's outgoing promises --");
    print_rows(graph::select_rows(&store, qp)?);

    // --- Retract a fulfilled promise via SPARQL UPDATE -----------------
    //
    // When Hermes has actually summarized the paper, Ares marks
    // the promise as fulfilled by removing the quad from
    // ex:promises. DELETE WHERE is the right SPARQL Update verb.
    let delete = r#"
        PREFIX ex: <http://example.org/agent#>
        DELETE WHERE {
            GRAPH ex:promises {
                ?p ex:promisedBy ex:Ares ;
                   ex:promiseContent "SummarizeArxiv2401" ;
                   ?k ?v .
            }
        }
    "#;
    SparqlEvaluator::new()
        .parse_update(delete)
        .context("parsing DELETE WHERE")?
        .on_store(&store)
        .execute()
        .context("executing DELETE WHERE")?;
    println!("\npromise retracted; ex:promises now contains:");
    print_rows(graph::select_rows(&store, qp)?);

    Ok(())
}

/// Load an N-Triples blob into a named graph by routing every
/// triple through a Turtle parser with `without_named_graphs()`.
fn load_ntriples_into_named_graph(
    store: &graph::Store,
    ntriples: &str,
    graph_iri: &str,
) -> Result<()> {
    let graph = NamedNodeRef::new(graph_iri)
        .with_context(|| format!("invalid graph IRI {graph_iri}"))?;
    let parser = graph::io::RdfParser::from_format(RdfFormat::NTriples)
        .without_named_graphs()
        .with_default_graph(graph);
    store
        .load_from_slice(parser, ntriples.as_bytes())
        .with_context(|| format!("loading N-Triples into {graph_iri}"))?;
    Ok(())
}

/// Write a promise quad into `ex:promises`. The four arguments
/// are the local names of the agents, the task's local name, and
/// the ISO-8601 timestamp at which the promise was made.
fn record_promise(
    store: &graph::Store,
    from: &str,
    to: &str,
    task: &str,
    when: &str,
) -> Result<()> {
    let update = format!(
        r#"
        PREFIX ex:  <http://example.org/agent#>
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
        INSERT DATA {{
            GRAPH ex:promises {{
                ex:promise_{task} a ex:Promise ;
                    ex:promisedBy    ex:{from} ;
                    ex:promisedTo    ex:{to} ;
                    ex:promiseContent "{task}" ;
                    ex:promisedAt    "{when}"^^xsd:dateTime .
            }}
        }}
        "#
    );
    SparqlEvaluator::new()
        .parse_update(&update)
        .with_context(|| "parsing INSERT DATA for promise")?
        .on_store(store)
        .execute()
        .with_context(|| "executing INSERT DATA for promise")?;
    Ok(())
}

fn count_all(store: &graph::Store) -> Result<usize> {
    let mut n = 0;
    for q in store.quads_for_pattern(None, None, None, Some(GraphNameRef::DefaultGraph)) {
        let _ = q?;
        n += 1;
    }
    Ok(n)
}

fn print_rows(rows: Vec<Vec<(String, String)>>) {
    if rows.is_empty() {
        println!("  (no results)");
        return;
    }
    for row in rows {
        for (var, val) in row {
            println!("  {var:<11} = {val}");
        }
        println!("  ---");
    }
}
