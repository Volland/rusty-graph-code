//! Ares — the running example binary for the book
//! "RDF, OWL and SHACL for Agentic Systems: A Hands-On Rust Course".
//!
//! The binary grows chapter by chapter. Each chapter adds one
//! function (`chapter_N`) and extends the `main` dispatcher. Run a
//! single chapter with:
//!
//! ```sh
//! cargo run --bin ares -- ch1
//! ```
//!
//! or run every chapter in order with `cargo run --bin ares -- all`.

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};

mod ch1;

fn data_dir() -> PathBuf {
    // The binary lives in `code/ares/`, the Turtle files live in
    // `code/data/`. We walk up from `CARGO_MANIFEST_DIR` so the
    // path is correct regardless of where `cargo run` is invoked.
    let manifest = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest).join("..").join("data")
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let which = args.get(1).map(String::as_str).unwrap_or("all");

    match which {
        "ch0" => chapter_0()?,
        "ch1" => ch1::run(&data_dir()).context("Chapter 1 failed")?,
        "all" => {
            chapter_0()?;
            ch1::run(&data_dir()).context("Chapter 1 failed")?;
        }
        other => {
            eprintln!("unknown chapter: {other}");
            eprintln!("usage: ares [ch0 | ch1 | all]");
            std::process::exit(2);
        }
    }
    Ok(())
}

fn chapter_0() -> Result<()> {
    println!("=== Chapter 0 ================================================");
    println!("Ares agent memory — Chapter 0 scaffold OK");
    let _store: graph::Store = graph::new_store()?;
    reasoner::placeholder();
    validator::placeholder();
    Ok(())
}
