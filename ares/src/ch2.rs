//! Chapter 2 — RDFS: typing the agent world.
//!
//! We load `schema.ttl` and `data.ttl` together into one store,
//! then run a few SPARQL queries that exploit the class hierarchy
//! using property paths. The last query deliberately fails to do
//! what a reader might expect, which motivates Chapter 3.

use std::path::Path;

use anyhow::{Context, Result};

pub fn run(data_dir: &Path) -> Result<()> {
    println!("=== Chapter 2 ================================================");

    let store = graph::new_store()?;
    graph::load_turtle_file(&store, data_dir.join("schema.ttl"))
        .context("loading schema.ttl")?;
    graph::load_turtle_file(&store, data_dir.join("data.ttl"))
        .context("loading data.ttl")?;
    println!("loaded schema + data ({} quads total)", store_len(&store)?);

    // --- SPARQL 1: find every agent, regardless of specific role ----
    //
    // `rdfs:subClassOf*` is a zero-or-more property path. It walks
    // the subclass chain. An agent of type ex:ResearchAgent matches
    // because ex:ResearchAgent rdfs:subClassOf ex:Agent.
    let q1 = r#"
        PREFIX rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX ex:   <http://example.org/agent#>

        SELECT ?agent ?role
        WHERE {
            ?agent rdf:type ?role .
            ?role  rdfs:subClassOf* ex:Agent .
        }
        ORDER BY ?agent
    "#;
    println!("\n-- every agent, via rdfs:subClassOf* --");
    print_rows(graph::select_rows(&store, q1)?);

    // --- SPARQL 2: confidence distribution per agent ----------------
    let q2 = r#"
        PREFIX ex: <http://example.org/agent#>

        SELECT ?agent ?confidence
        WHERE {
            ?obs ex:assertedBy ?agent ;
                 ex:confidence ?confidence .
        }
        ORDER BY DESC(?confidence)
    "#;
    println!("-- observation confidence per agent --");
    print_rows(graph::select_rows(&store, q2)?);

    // --- SPARQL 3: the limitation that motivates Chapter 3 ----------
    //
    // We want: "every observation with confidence > 0.8 should count
    // as a ConfidentObservation". SPARQL *can* filter on a numeric
    // threshold, but it cannot *type* new instances — the store
    // still contains zero triples of the form
    //     ?obs a ex:ConfidentObservation .
    // A reasoner is the only way to turn that implication into
    // concrete triples. We will do exactly that in Chapter 3.
    let q3_filter = r#"
        PREFIX ex: <http://example.org/agent#>
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

        SELECT ?obs ?confidence
        WHERE {
            ?obs a ex:Observation ;
                 ex:confidence ?confidence .
            FILTER (?confidence > "0.8"^^xsd:decimal)
        }
    "#;
    println!("-- SPARQL FILTER picks confident observations --");
    print_rows(graph::select_rows(&store, q3_filter)?);

    let q3_type = r#"
        PREFIX ex: <http://example.org/agent#>
        SELECT ?obs
        WHERE { ?obs a ex:ConfidentObservation . }
    "#;
    println!("-- but no triple types them as ConfidentObservation --");
    let rows = graph::select_rows(&store, q3_type)?;
    if rows.is_empty() {
        println!("  (zero rows — exactly the Chapter 3 problem)");
    } else {
        print_rows(rows);
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
