//! `graph` — thin wrappers over oxigraph used throughout the book.
//!
//! At this stage (Chapter 0) the crate only re-exports the few types we
//! need so that the binary in `ares/` compiles. Later chapters will add
//! helpers here for loading Turtle files, running SPARQL, and writing
//! NQuads.

pub use oxigraph::store::Store;
