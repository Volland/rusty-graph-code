# rusty-graph-code

Companion code for **"RDF, OWL and SHACL for Agentic Systems: A Hands-On Rust Course"**.

## Quick start

```sh
cargo build
cargo run --bin ares -- all        # run every chapter in order
cargo run --bin ares -- ch3        # run a single chapter
cargo run --bin ares -- ch7 --strict  # pipeline with CI gate
```

Requires Rust 1.90+ (pinned in `rust-toolchain.toml`).

## Workspace layout

```
code/
├── Cargo.toml           # workspace root
├── rust-toolchain.toml  # pins the compiler
├── data/                # Turtle, SHACL, ShEx files
├── graph/               # library crate: oxigraph wrappers
├── reasoner/            # library crate: reasonable (OWL 2 RL)
├── validator/           # library crate: rudof_lib (SHACL/ShEx)
├── ares/                # binary crate: the running example
└── grafeo_samples/      # optional: GrafeoDB examples (Ch 9-12)
```

## The three acts

The book — and this code — is structured in three acts.

### Act I — Build the Memory (Chapters 0-3)

Set up the workspace, model Ares's first observation as RDF triples,
type the agent world with RDFS, and run an OWL 2 RL reasoner to infer
facts the store cannot see on its own.

| Chapter | File | What it does |
|---------|------|--------------|
| 0 | `ares/src/main.rs` | Scaffold — proves the workspace compiles |
| 1 | `ares/src/ch1.rs` | Load Turtle, SPARQL SELECT, named graphs |
| 2 | `ares/src/ch2.rs` | RDFS subclass hierarchy, property paths |
| 3 | `ares/src/ch3.rs` | OWL 2 RL reasoning with `reasonable` |

### Act II — Defend the Memory (Chapters 4-7)

Partition the store into named graphs for provenance, write SHACL shapes
that reject malformed data, build SPARQL-level constraint checks, and
wire everything into a CI-ready pipeline.

| Chapter | File | What it does |
|---------|------|--------------|
| 4 | `ares/src/ch4.rs` | Named graphs, SPARQL UPDATE, promises |
| 5 | `ares/src/ch5.rs` | SHACL Core validation via `rudof_lib` |
| 6 | `ares/src/ch6.rs` | SHACL-SPARQL constraints via raw SPARQL |
| 7 | `ares/src/ch7.rs` | Full pipeline: load, reason, validate, report |

### Act III — Outgrow the Memory (Chapters 8-12)

Compare shape languages, introduce a second graph engine (GrafeoDB),
remodel data as a labelled property graph, and build hybrid retrieval.

| Chapter | File | What it does |
|---------|------|--------------|
| 8 | `ares/src/ch8.rs` | ShEx vs SHACL — open vs closed shapes |
| 9 | `grafeo_samples/src/ch9.rs` | GrafeoDB RDF + SPARQL |
| 10 | `grafeo_samples/src/ch10.rs` | SHACL validation inside GrafeoDB |
| 11 | `grafeo_samples/src/ch11.rs` | Labelled property graph + GQL |
| 12 | `grafeo_samples/src/ch12.rs` | Hybrid vector + text + graph search |

## Data files

| File | Purpose |
|------|---------|
| `data/schema.ttl` | Agent ontology (RDFS + OWL axioms) |
| `data/data.ttl` | Base facts: agents, paper, observation, trust |
| `data/observations-calliope.ttl` | Calliope's observation (named graph) |
| `data/shapes.ttl` | SHACL Core shapes (Ch 5) |
| `data/shapes-closed.ttl` | Closed SHACL variant (Ch 8) |
| `data/shapes-sparql.ttl` | SHACL-SPARQL shapes (reference) |
| `data/observations.shex` | ShEx version of shapes (Ch 8) |
| `data/data-bad.ttl` | Deliberately broken observation (Ch 5) |
| `data/data-bad-promise.ttl` | Self-promise fixture (Ch 6) |
| `data/data-extra.ttl` | Extra-property fixture (Ch 8) |

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `oxigraph` | 0.5 | RDF quad store + SPARQL |
| `reasonable` | 0.3.3-a4 | OWL 2 RL forward-chaining reasoner |
| `rudof_lib` | 0.2 | SHACL + ShEx validation |
| `anyhow` | 1 | Error handling |
| `serde` + `serde_json` | 1 | JSON report serialization |

## License

MIT OR Apache-2.0
