//! Ares — the running example binary for the book
//! "RDF, OWL and SHACL for Agentic Systems: A Hands-On Rust Course".
//!
//! This file grows chapter by chapter. At Chapter 0 it does nothing
//! except prove the workspace compiles and link the three library
//! crates so their APIs are visible to the type checker.

use anyhow::Result;

fn main() -> Result<()> {
    println!("Ares agent memory — Chapter 0 scaffold OK");
    // Touch each library crate so the linker pulls them in.
    let _store: graph::Store = graph::Store::new()?;
    reasoner::placeholder();
    validator::placeholder();
    Ok(())
}
