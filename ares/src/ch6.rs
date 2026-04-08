//! Chapter 6 — SHACL-SPARQL constraints.
//!
//! `rudof_lib` 0.2.8's native SHACL engine does not yet execute
//! `sh:sparql` constraint components. Rather than pretend it does,
//! this chapter shows two things:
//!
//!   1. The SHACL-SPARQL shape the way you *would* write it, so
//!      readers know what the spec looks like.
//!   2. A pragmatic fallback: run the same SPARQL ASK/SELECT
//!      queries directly against an oxigraph store and collect
//!      the violations by hand. This is how you close the gap
//!      today.

use std::path::Path;

use anyhow::{Context, Result};

pub fn run(data_dir: &Path) -> Result<()> {
    println!("=== Chapter 6 ================================================");

    // --- Build a store containing the bad data ---------------------
    let store = graph::new_store()?;
    graph::load_turtle_file(&store, data_dir.join("schema.ttl"))?;
    graph::load_turtle_file(&store, data_dir.join("data-bad-promise.ttl"))?;
    println!("loaded schema + data-bad-promise.ttl");

    // --- Constraint 1: a Promise's promiser must differ from -------
    //     its recipient.
    //
    // A SELECT query that returns one row per violating promise,
    // with the focus node bound to ?focus.
    let q_self_promise = r#"
        PREFIX ex: <http://example.org/agent#>
        SELECT ?focus ?agent
        WHERE {
            ?focus a ex:Promise ;
                   ex:promisedBy ?agent ;
                   ex:promisedTo ?agent .
        }
    "#;

    println!("\n-- constraint: promiser != recipient --");
    let rows = graph::select_rows(&store, q_self_promise)?;
    if rows.is_empty() {
        println!("  (no violations)");
    } else {
        for row in rows {
            let mut iter = row.into_iter();
            let (_, focus) = iter.next().unwrap();
            let (_, agent) = iter.next().unwrap();
            println!(
                "  VIOLATION: {focus} promises itself ({agent} = {agent})"
            );
        }
    }

    // --- Constraint 2: no trust cycles ----------------------------
    //
    // A trust edge cycle is "an agent transitively trusts itself".
    // SPARQL property paths let us express the transitive closure
    // directly, which is exactly the kind of query SHACL Core
    // cannot match on its own.
    //
    // To exercise the constraint we insert a deliberate cycle first.
    graph::sparql::SparqlEvaluator::new()
        .parse_update(
            r#"
            PREFIX ex: <http://example.org/agent#>
            INSERT DATA {
                ex:Loopy a ex:ResearchAgent ; ex:agentId "agent-0099" .
                ex:Loopy ex:trustsAgent ex:Loopy .
            }
            "#,
        )
        .context("parsing cycle insert")?
        .on_store(&store)
        .execute()
        .context("executing cycle insert")?;

    let q_cycle = r#"
        PREFIX ex: <http://example.org/agent#>
        SELECT ?agent
        WHERE { ?agent ex:trustsAgent+ ?agent . }
    "#;
    println!("\n-- constraint: no trust cycles --");
    let rows = graph::select_rows(&store, q_cycle)?;
    if rows.is_empty() {
        println!("  (no violations)");
    } else {
        for row in rows {
            let (_, agent) = row.into_iter().next().unwrap();
            println!("  VIOLATION: {agent} is inside a trust cycle");
        }
    }

    Ok(())
}
