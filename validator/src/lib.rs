//! `validator` — wraps `rudof_lib` for SHACL and ShEx validation.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use rudof_lib::{RDFFormat, ReaderMode, Rudof, RudofConfig, ShExFormat};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub focus_node: String,
    pub severity: String,
    pub message: String,
    pub component: String,
}

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
pub fn validate_shacl(data_ttl: &Path, shapes_ttl: &Path) -> Result<Report> {
    let config = RudofConfig::default_config()
        .context("loading rudof_lib default config")?;
    let mut rudof = Rudof::new(&config)
        .context("building a Rudof instance")?;

    let mut data_file = File::open(data_ttl)
        .with_context(|| format!("opening {}", data_ttl.display()))?;
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
        .with_context(|| format!("reading data {}", data_ttl.display()))?;

    let mut shapes_file = File::open(shapes_ttl)
        .with_context(|| format!("opening {}", shapes_ttl.display()))?;
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
        .with_context(|| format!("reading shapes {}", shapes_ttl.display()))?;

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

/// Parse a ShEx schema and report whether it loaded cleanly.
///
/// Chapter 8 uses this to show that `rudof_lib` parses ShEx
/// schemas out of the box. Full ShEx validation requires a
/// shapemap configuration; for this book we stop at parsing and
/// run cross-engine comparisons through SHACL.
pub fn parse_shex(shex_path: &Path) -> Result<()> {
    let config = RudofConfig::default_config()
        .context("loading rudof_lib default config")?;
    let mut rudof = Rudof::new(&config)
        .context("building a Rudof instance")?;
    let file = File::open(shex_path)
        .with_context(|| format!("opening {}", shex_path.display()))?;
    rudof
        .read_shex(
            BufReader::new(file),
            Some(&ShExFormat::ShExC),
            None,
            Some(&ReaderMode::Lax),
            shex_path.file_name().and_then(|s| s.to_str()),
        )
        .with_context(|| format!("parsing ShEx schema {}", shex_path.display()))?;
    Ok(())
}
