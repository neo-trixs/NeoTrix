use std::collections::HashMap;

/// A text chunk within a document.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The chunk text content.
    pub text: String,
    /// Byte offset where this chunk starts in the original document content.
    pub start_offset: usize,
    /// Byte offset where this chunk ends (exclusive) in the original document content.
    pub end_offset: usize,
    /// Optional embedding vector for semantic search.
    pub embedding: Option<Vec<f32>>,
}

/// A document to be ingested into the RAG pipeline.
#[derive(Debug, Clone)]
pub struct Document {
    /// Unique document identifier.
    pub id: String,
    /// Raw text content of the document.
    pub content: String,
    /// Arbitrary metadata key-value pairs (e.g. source URL, timestamp).
    pub metadata: HashMap<String, String>,
    /// Chunks produced by the chunker. Empty until chunked.
    pub chunks: Vec<Chunk>,
}

/// A single search result returned by the vector store.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The document ID the chunk belongs to.
    pub doc_id: String,
    /// The matched chunk text.
    pub chunk_text: String,
    /// Cosine similarity score (0.0 – 1.0).
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_defaults() {
        let c = Chunk {
            text: "hello".into(),
            start_offset: 0,
            end_offset: 5,
            embedding: None,
        };
        assert_eq!(c.text, "hello");
        assert!(c.embedding.is_none());
    }

    #[test]
    fn document_defaults() {
        let doc = Document {
            id: "d1".into(),
            content: "content".into(),
            metadata: HashMap::new(),
            chunks: vec![],
        };
        assert_eq!(doc.id, "d1");
        assert!(doc.chunks.is_empty());
    }
}
