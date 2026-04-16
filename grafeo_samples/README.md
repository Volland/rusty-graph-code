# grafeo_samples

Companion code for Chapters 9–12 of the book, built against the
[`grafeo`](https://crates.io/crates/grafeo) crate.

## Layout

- `src/ch9.rs`  — Grafeo as a second RDF/LPG engine; the Chapter-1 "what has Ares observed?" question in GQL, with an optional SPARQL path via `execute_language`.
- `src/ch10.rs` — store-level integrity validation (`GrafeoDB::validate`) plus GQL expressions of the SHACL Core invariants from Chapter 5 (range, cardinality).
- `src/ch11.rs` — the Ares scenario remodelled as a labelled property graph: nodes, typed edges with properties, GQL and Cypher queries.
- `src/ch12.rs` — BM25 text index, HNSW vector index, and hybrid search (RRF fusion).

## Running

This crate is **not** part of the workspace's `default-members`, so
`cargo build` at the workspace root skips it. Opt in explicitly:

```sh
cargo run -p grafeo_samples -- ch9
cargo run -p grafeo_samples -- all
```

## Toolchain note

`grafeo` 0.5.38 requires **rustc 1.91.1** or newer (the root
`rust-toolchain.toml` pins 1.90 for Chapters 1–8). To build this
crate, install the newer toolchain and opt in per-command:

```sh
rustup install 1.91.1
cargo +1.91.1 run -p grafeo_samples -- ch9
```

The toolchain pin for the rest of the workspace is intentionally left
at 1.90 so the Chapter 1–8 reference implementation keeps building on
the same rustc the book was written against.
