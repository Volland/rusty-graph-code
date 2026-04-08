//! `reasoner` — wraps the `reasonable` OWL 2 RL reasoner.
//!
//! The crate gives the rest of the workspace a single function,
//! [`closure_as_ntriples`], that takes a list of Turtle input files
//! and returns the full inferred closure as a single N-Triples string.
//! The caller then loads that string into oxigraph. We go through
//! strings on purpose: `reasonable` re-exports `oxrdf::Triple` from
//! its own oxrdf version, and funnelling everything through the
//! N-Triples line format avoids any version mismatch between the two
//! crates.

use std::path::Path;

use anyhow::{Context, Result};
use reasonable::reasoner::Reasoner;

/// Run OWL 2 RL forward-chaining over every Turtle file in `inputs`
/// and return the full closure as an N-Triples document (one triple
/// per line, terminated with ` .\n`).
///
/// The closure includes both the base triples and the newly inferred
/// ones.
pub fn closure_as_ntriples<P: AsRef<Path>>(inputs: &[P]) -> Result<String> {
    let mut r = Reasoner::new();
    for path in inputs {
        let p = path.as_ref();
        let s = p.to_str().with_context(|| {
            format!("reasonable needs a UTF-8 path, got {}", p.display())
        })?;
        r.load_file(s)
            .with_context(|| format!("reasonable failed to load {s}"))?;
    }
    r.reason();

    let mut out = String::new();
    for (s, p, o) in r.get_triples_string() {
        // reasonable's `to_string()` emits each part in the same
        // syntax that oxrdf uses: `<iri>` for named nodes,
        // `_:name` for blank nodes, and quoted + datatyped literals
        // for literals. That is exactly N-Triples, so we can glue
        // the three with spaces and a trailing dot.
        out.push_str(&s);
        out.push(' ');
        out.push_str(&p);
        out.push(' ');
        out.push_str(&o);
        out.push_str(" .\n");
    }
    Ok(out)
}

/// Count how many lines the closure contains. Useful for "triples
/// before / triples after" diffs in examples.
pub fn count_lines(ntriples: &str) -> usize {
    ntriples.lines().filter(|l| !l.trim().is_empty()).count()
}
