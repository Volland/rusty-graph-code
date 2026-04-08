//! `validator` — wraps `rudof_lib` for SHACL and ShEx validation.
//!
//! Chapter 5 uses [`validate_shacl`] to run SHACL Core validation
//! from pure Rust. Chapter 8 will add a sibling `validate_shex`.

use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use rudof_lib::{RDFFormat, ReaderMode, Rudof, RudofConfig};
use serde::Serialize;

/// One line of a SHACL validation report, flattened to the fields
/// we care about for the book's examples.
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub focus_node: String,
    pub severity: String,
    pub message: String,
    pub component: String,
}

/// The full result of a SHACL validation run.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub conforms: bool,
    pub violations: Vec<Violation>,
}

impl Report {
    pub fn is_empty(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Run SHACL Core validation of `data_ttl` against `shapes_ttl`.
///
/// Both arguments are paths to Turtle files. Any of them can
/// contain multiple shapes or graphs; `rudof_lib` parses them
/// directly.
pub fn validate_shacl(data_ttl: &Path, shapes_ttl: &Path) -> Result<Report> {
    let config = RudofConfig::default_config()
        .context("loading rudof_lib default config")?;
    let mut rudof = Rudof::new(&config)
        .context("building a Rudof instance")?;

    // --- Load the data graph -----------------------------------------
    let mut data_file = File::open(data_ttl)
        .with_context(|| format!("opening data file {}", data_ttl.display()))?;
    rudof
        .read_data(
            &mut data_file,
            data_ttl
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("data.ttl"),
            Some(&RDFFormat::Turtle),
            None,
            Some(&ReaderMode::Lax),
            Some(false),
        )
        .with_context(|| format!("rudof failed to read data {}", data_ttl.display()))?;

    // --- Load the shapes graph ---------------------------------------
    let mut shapes_file = File::open(shapes_ttl)
        .with_context(|| format!("opening shapes file {}", shapes_ttl.display()))?;
    rudof
        .read_shacl(
            &mut shapes_file,
            shapes_ttl
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("shapes.ttl"),
            None,
            None,
            Some(&ReaderMode::Lax),
        )
        .with_context(|| format!("rudof failed to read shapes {}", shapes_ttl.display()))?;

    // --- Run validation -----------------------------------------------
    let report = rudof
        .validate_shacl(None, None)
        .context("rudof_lib SHACL validation crashed")?;

    let mut violations = Vec::new();
    for result in report.results() {
        violations.push(Violation {
            focus_node: result.focus_node().to_string(),
            severity: format!("{:?}", result.severity()),
            message: result
                .message()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            component: result.component().to_string(),
        });
    }

    Ok(Report {
        conforms: report.conforms(),
        violations,
    })
}
