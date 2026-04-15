//! Chapter 12 — Hybrid search: BM25, HNSW, and RRF fusion.
//!
//! This module builds a tiny paper corpus, attaches 3-dimensional
//! toy embeddings to each node, creates a text index and a vector
//! index, and then asks three questions: text-only, vector-only,
//! and hybrid. The dimensions are deliberately tiny so the sample
//! stays readable — everything scales the same at 384 or 1536 dims.

use anyhow::Result;
use grafeo::{GrafeoDB, Value};

fn embed(kind: &str) -> Vec<f32> {
    match kind {
        "compression" => vec![0.9, 0.1, 0.1],
        "memory" => vec![0.8, 0.2, 0.1],
        "graphs" => vec![0.1, 0.9, 0.2],
        "unrelated" => vec![0.1, 0.1, 0.9],
        _ => vec![0.0, 0.0, 0.0],
    }
}

pub fn run() -> Result<()> {
    println!("=== Chapter 12 ===============================================");
    let db = GrafeoDB::new_in_memory();

    let papers = [
        (
            "2401.00123",
            "Memory compression in transformers",
            "Techniques for shrinking attention caches in LLMs.",
            "compression",
        ),
        (
            "2402.00999",
            "Graph neural memory for agents",
            "A graph-based store for agent episodic memory.",
            "graphs",
        ),
        (
            "2403.00555",
            "Recurrent memory layers",
            "Lightweight RNN state reuse for long-context models.",
            "memory",
        ),
        (
            "2404.00042",
            "Yet another cat detector",
            "Unrelated vision work included as a negative example.",
            "unrelated",
        ),
    ];

    for (arxiv, title, abs, kind) in papers {
        db.create_node_with_props(
            &["Paper"],
            vec![
                ("arxivId", Value::String(arxiv.to_string())),
                ("title", Value::String(title.to_string())),
                ("abstract", Value::String(abs.to_string())),
                ("embedding", Value::Vector(embed(kind))),
            ],
        );
    }

    // BM25 text index on abstracts.
    db.create_text_index("Paper", "abstract")?;

    // HNSW vector index. Dimensions must match the embedding length.
    db.create_vector_index(
        "Paper",
        "embedding",
        Some(3),
        Some("cosine"),
        None,
        None,
        None,
    )?;

    let query_text = "memory compression";
    let query_vec = embed("compression");

    println!("\n-- BM25 text search --");
    let text_hits = db.text_search("Paper", "abstract", query_text, 5)?;
    for (id, score) in &text_hits {
        println!("  node={id:?}  score={score:.3}");
    }

    println!("\n-- HNSW vector search --");
    let vec_hits =
        db.vector_search("Paper", "embedding", &query_vec, 5, None, None)?;
    for (id, score) in &vec_hits {
        println!("  node={id:?}  score={score:.3}");
    }

    println!("\n-- Hybrid search (RRF default) --");
    let hybrid = db.hybrid_search(
        "Paper",
        "abstract",
        "embedding",
        query_text,
        Some(&query_vec),
        5,
        None,
    )?;
    for (id, score) in &hybrid {
        println!("  node={id:?}  score={score:.4}");
    }

    Ok(())
}
