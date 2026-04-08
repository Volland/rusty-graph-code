//! Chapter 1 — RDF foundations via Ares's first observation.
//!
//! This module is the concrete companion to Chapter 1 of the book.
//! It:
//!
//!   1. Loads `data/data.ttl` into an in-memory oxigraph store.
//!   2. Runs a SPARQL SELECT that asks "what did Ares observe?".
//!   3. Asks the same question via the Rust `quads_for_pattern` API.
//!   4. Loads Calliope's observation into a named graph.
//!   5. Queries across both named graphs and the default graph.
//!   6. Serializes the full dataset back to NQuads.

use std::path::Path;

use anyhow::{Context, Result};
use graph::model::{GraphNameRef, NamedNodeRef};

/// Entry point used by `main.rs`.
pub fn run(data_dir: &Path) -> Result<()> {
    println!("=== Chapter 1 ================================================");
    let store = graph::new_store()?;

    // --- Step 1: load the default graph ---------------------------------
    let data_ttl = data_dir.join("data.ttl");
    graph::load_turtle_file(&store, &data_ttl)
        .with_context(|| format!("loading {}", data_ttl.display()))?;
    println!("loaded {} ({} quads)", data_ttl.display(), store_len(&store)?);

    // --- Step 2: SPARQL SELECT ------------------------------------------
    let q = r#"
        PREFIX ex:   <http://example.org/agent#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?paper ?title ?confidence ?when
        WHERE {
            ?obs a             ex:Observation ;
                 ex:assertedBy ex:Ares ;
                 ex:hasSubject ?paper ;
                 ex:confidence ?confidence ;
                 ex:observedAt ?when .
            ?paper rdfs:label  ?title .
        }
    "#;

    println!("\n-- SPARQL: what has Ares observed? --");
    let rows = graph::select_rows(&store, q)?;
    if rows.is_empty() {
        println!("(no results)");
    }
    for row in rows {
        for (var, val) in row {
            println!("  {var:<11} = {val}");
        }
        println!("  ---");
    }

    // --- Step 3: the same question via quads_for_pattern ----------------
    //
    // SPARQL is convenient; the quad-pattern API is what you reach
    // for when you want zero query parsing overhead or when you
    // are inside a tight inner loop. Both go through the same
    // underlying store indexes.

    println!("-- quads_for_pattern: Ares's observation nodes --");
    let asserted_by_iri = format!("{}assertedBy", graph::EX);
    let ares_iri = format!("{}Ares", graph::EX);
    let asserted_by = NamedNodeRef::new(&asserted_by_iri)?;
    let ares = NamedNodeRef::new(&ares_iri)?;
    for quad in store.quads_for_pattern(
        None,
        Some(asserted_by),
        Some(ares.into()),
        Some(GraphNameRef::DefaultGraph),
    ) {
        let quad = quad.context("iterating assertedBy=Ares quads")?;
        println!("  {} {} {}", quad.subject, quad.predicate, quad.object);
    }

    // --- Step 4: load Calliope's observation into a named graph ---------
    let calliope_ttl = data_dir.join("observations-calliope.ttl");
    let calliope_graph = format!("{}observations/Calliope", graph::EX);
    graph::load_turtle_into_named_graph(&store, &calliope_ttl, &calliope_graph)
        .with_context(|| format!("loading {}", calliope_ttl.display()))?;
    println!(
        "\nloaded {} into named graph <{}>",
        calliope_ttl.display(),
        calliope_graph
    );

    // --- Step 5: SPARQL across default + named graphs -------------------
    //
    // `GRAPH ?g { ... }` matches inside any named graph and binds
    // `?g` to its IRI. The default graph is *not* matched by a
    // bare `GRAPH` block, so Ares's observation (which lives in
    // the default graph) comes from the outer BGP and Calliope's
    // observation comes from the inner GRAPH block.

    let q2 = r#"
        PREFIX ex:   <http://example.org/agent#>

        SELECT ?agent ?obs ?confidence ?g
        WHERE {
            {
                ?obs ex:assertedBy ?agent ;
                     ex:confidence ?confidence .
                BIND(<urn:default> AS ?g)
            } UNION {
                GRAPH ?g {
                    ?obs ex:assertedBy ?agent ;
                         ex:confidence ?confidence .
                }
            }
        }
        ORDER BY ?agent
    "#;

    println!("\n-- SPARQL across every graph --");
    for row in graph::select_rows(&store, q2)? {
        for (var, val) in row {
            println!("  {var:<11} = {val}");
        }
        println!("  ---");
    }

    // --- Step 6: serialize the dataset back to NQuads -------------------
    println!("\n-- NQuads dump (first 600 chars) --");
    let nq = graph::dump_nquads(&store)?;
    let preview: String = nq.chars().take(600).collect();
    println!("{preview}");
    if nq.len() > 600 {
        println!("... ({} bytes total)", nq.len());
    }

    Ok(())
}

fn store_len(store: &graph::Store) -> Result<usize> {
    let mut n = 0;
    for q in store.iter() {
        let _ = q.context("counting quads")?;
        n += 1;
    }
    Ok(n)
}
