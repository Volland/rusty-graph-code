//! Grafeo samples — the running companion binary for Chapters 9–12
//! of the book. Each chapter is one module with a `run()` entry point.
//!
//! ```sh
//! cargo run -p grafeo_samples -- ch9
//! cargo run -p grafeo_samples -- ch11
//! cargo run -p grafeo_samples -- all
//! ```

use std::env;

use anyhow::{Context, Result};

mod ch10;
mod ch11;
mod ch12;
mod ch9;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let which = args.get(1).map(String::as_str).unwrap_or("all");

    match which {
        "ch9" => ch9::run().context("Chapter 9 failed")?,
        "ch10" => ch10::run().context("Chapter 10 failed")?,
        "ch11" => ch11::run().context("Chapter 11 failed")?,
        "ch12" => ch12::run().context("Chapter 12 failed")?,
        "all" => {
            ch9::run().context("Chapter 9 failed")?;
            ch10::run().context("Chapter 10 failed")?;
            ch11::run().context("Chapter 11 failed")?;
            ch12::run().context("Chapter 12 failed")?;
        }
        other => {
            eprintln!("unknown chapter: {other}");
            eprintln!("usage: grafeo_samples [ch9|ch10|ch11|ch12|all]");
            std::process::exit(2);
        }
    }
    Ok(())
}
