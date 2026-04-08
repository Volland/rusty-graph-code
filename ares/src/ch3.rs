//! Chapter 3 — OWL reasoning with `reasonable`.
//!
//! Steps:
//!   1. Count the base triples by loading schema + data into a
//!      fresh oxigraph store.
//!   2. Run `reasonable` over the same files to produce an
//!      N-Triples closure.
//!   3. Load the closure into a second store.
//!   4. Ask a few questions that only have answers after reasoning:
//!        - is obs_0001 a ConfidentObservation?
//!        - whom does Ares transitively trust?
//!        - who has observed the DOI IRI of the paper?

use std::path::Path;

use anyhow::{Context, Result};
use graph::io::RdfFormat;

pub fn run(data_dir: &Path) -> Result<()> {
    println!("=== Chapter 3 ================================================");

    // --- Base store (no reasoning) --------------------------------------
    let base_store = graph::new_store()?;
    graph::load_turtle_file(&base_store, data_dir.join("schema.ttl"))?;
    graph::load_turtle_file(&base_store, data_dir.join("data.ttl"))?;
    let base_count = store_len(&base_store)?;
    println!("base store: {base_count} quads (no reasoning)");

    // --- Reasoner over schema + data ------------------------------------
    let inputs = [
        data_dir.join("schema.ttl"),
        data_dir.join("data.ttl"),
    ];
    let closure_nt = reasoner::closure_as_ntriples(&inputs)
        .context("running reasonable over schema + data")?;
    let closure_count = reasoner::count_lines(&closure_nt);
    println!(
        "closure:   {closure_count} triples (delta: +{})",
        closure_count as i64 - base_count as i64
    );

    // --- Load the closure into a fresh store ----------------------------
    let inferred_store = graph::new_store()?;
    inferred_store
        .load_from_slice(RdfFormat::NTriples, closure_nt.as_bytes())
        .context("loading closure into a fresh oxigraph store")?;

    // --- Query 1: is obs_0001 a ConfidentObservation? -------------------
    let q1 = r#"
        PREFIX ex: <http://example.org/agent#>
        ASK { ex:obs_0001 a ex:ConfidentObservation . }
    "#;
    println!(
        "\n-- is obs_0001 a ConfidentObservation after reasoning? {}",
        graph::ask(&inferred_store, q1)?
    );
    // Same question on the base store should be false.
    println!(
        "-- same question on the base store:                    {}",
        graph::ask(&base_store, q1)?
    );

    // --- Query 2: transitive trust --------------------------------------
    let q2 = r#"
        PREFIX ex: <http://example.org/agent#>
        SELECT ?other
        WHERE { ex:Ares ex:trustsAgent ?other . }
        ORDER BY ?other
    "#;
    println!("\n-- after reasoning, Ares transitively trusts: --");
    print_rows(graph::select_rows(&inferred_store, q2)?);

    // --- Query 3: owl:sameAs unifies DOI and arXiv IRIs -----------------
    let q3 = r#"
        PREFIX ex:   <http://example.org/agent#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?paper ?asserter
        WHERE {
            ?paper rdfs:label ?label .
            ?obs ex:hasSubject ?paper ;
                 ex:assertedBy ?asserter .
        }
        ORDER BY ?paper
    "#;
    println!("-- observed papers (owl:sameAs should fan out): --");
    print_rows(graph::select_rows(&inferred_store, q3)?);

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
