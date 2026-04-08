//! `graph` — thin wrappers over `oxigraph` used throughout the book.
//!
//! The crate has two jobs: (1) hide the most verbose bits of the
//! oxigraph 0.5 API behind a few small functions, and (2) give every
//! chapter a single place to look when it wants to load Turtle,
//! run SPARQL, or serialize a store.

use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{GraphNameRef, NamedNodeRef};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};

pub use oxigraph::io;
pub use oxigraph::model;
pub use oxigraph::sparql;
pub use oxigraph::store::Store;

/// The canonical namespace used in every example in the book.
pub const EX: &str = "http://example.org/agent#";

/// Build a new empty in-memory oxigraph store.
///
/// Chapter 0 used `Store::new()` directly; from Chapter 1 on we
/// funnel every instantiation through this helper so that if we
/// ever want to switch to a RocksDB-backed store the change is
/// one line.
pub fn new_store() -> Result<Store> {
    Store::new().context("failed to create in-memory oxigraph store")
}

/// Load a Turtle file from disk into the store's default graph.
pub fn load_turtle_file(store: &Store, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading Turtle file {}", path.display()))?;
    store
        .load_from_slice(RdfFormat::Turtle, &bytes)
        .with_context(|| format!("parsing Turtle file {}", path.display()))?;
    Ok(())
}

/// Load a Turtle document from a `Read` source into the default graph.
pub fn load_turtle_reader(store: &Store, reader: impl Read) -> Result<()> {
    store
        .load_from_reader(RdfParser::from_format(RdfFormat::Turtle), reader)
        .context("parsing Turtle from reader")?;
    Ok(())
}

/// Load a Turtle document into a specific named graph.
///
/// The Turtle file itself must contain only default-graph triples
/// (no `GRAPH` blocks); the caller decides the graph name.
pub fn load_turtle_into_named_graph(
    store: &Store,
    path: impl AsRef<Path>,
    graph_iri: &str,
) -> Result<()> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading Turtle file {}", path.display()))?;
    let graph = NamedNodeRef::new(graph_iri)
        .with_context(|| format!("invalid graph IRI {graph_iri}"))?;
    let parser = RdfParser::from_format(RdfFormat::Turtle)
        .without_named_graphs()
        .with_default_graph(graph);
    store
        .load_from_slice(parser, &bytes)
        .with_context(|| format!("loading {} into graph {graph_iri}", path.display()))?;
    Ok(())
}

/// Run a SPARQL `SELECT` query and return its solutions as rows of
/// `(variable, n-triples value)` pairs. Handy for printing.
pub fn select_rows(store: &Store, query: &str) -> Result<Vec<Vec<(String, String)>>> {
    let results = SparqlEvaluator::new()
        .parse_query(query)
        .with_context(|| format!("parsing SPARQL query:\n{query}"))?
        .on_store(store)
        .execute()
        .with_context(|| format!("executing SPARQL query:\n{query}"))?;

    let mut out = Vec::new();
    if let QueryResults::Solutions(solutions) = results {
        let variables: Vec<String> = solutions
            .variables()
            .iter()
            .map(|v| v.as_str().to_owned())
            .collect();
        for solution in solutions {
            let solution = solution.context("reading SPARQL solution row")?;
            let mut row = Vec::with_capacity(variables.len());
            for (i, var) in variables.iter().enumerate() {
                let value = solution
                    .get(i)
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "UNBOUND".to_owned());
                row.push((var.clone(), value));
            }
            out.push(row);
        }
    }
    Ok(out)
}

/// Run a SPARQL `ASK` query.
pub fn ask(store: &Store, query: &str) -> Result<bool> {
    let results = SparqlEvaluator::new()
        .parse_query(query)
        .with_context(|| format!("parsing ASK query:\n{query}"))?
        .on_store(store)
        .execute()
        .with_context(|| format!("executing ASK query:\n{query}"))?;
    match results {
        QueryResults::Boolean(b) => Ok(b),
        _ => anyhow::bail!("query was not an ASK"),
    }
}

/// Count quads matching a subject-predicate-object-graph pattern.
/// Any of the four may be `None` for a wildcard.
pub fn count_quads(
    store: &Store,
    subject: Option<model::NamedOrBlankNodeRef<'_>>,
    predicate: Option<NamedNodeRef<'_>>,
    object: Option<model::TermRef<'_>>,
    graph: Option<GraphNameRef<'_>>,
) -> Result<usize> {
    let mut n = 0usize;
    for quad in store.quads_for_pattern(subject, predicate, object, graph) {
        let _ = quad.context("iterating quads")?;
        n += 1;
    }
    Ok(n)
}

/// Serialize the entire dataset as NQuads into an owned `String`.
pub fn dump_nquads(store: &Store) -> Result<String> {
    let buf: Vec<u8> = store
        .dump_to_writer(RdfFormat::NQuads, Vec::new())
        .context("dumping store as NQuads")?;
    Ok(String::from_utf8(buf).context("NQuads output was not UTF-8")?)
}
