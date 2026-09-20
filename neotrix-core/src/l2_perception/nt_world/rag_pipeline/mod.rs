//! Local RAG (Retrieval-Augmented Generation) pipeline.
//!
//! Provides document ingestion, chunking, hash-based embedding, and
//! cosine-similarity retrieval — all without external dependencies.

pub mod chunker;
pub mod document;
pub mod rag_engine;
pub mod vector_store;

pub use chunker::chunk_document;
pub use document::{Chunk, Document, SearchResult};
pub use rag_engine::RagEngine;
pub use vector_store::{DiskVectorStore, VectorStore};
