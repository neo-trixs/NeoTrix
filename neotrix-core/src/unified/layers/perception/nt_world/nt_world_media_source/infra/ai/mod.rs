pub mod summarizer;
pub mod sentiment;
pub mod ner;
pub mod embedding_search;
pub mod rag_search;
pub mod multimodal;

pub struct ContentSummarizer;
impl ContentSummarizer {
    pub fn new() -> Self {
        Self
    }
}

pub struct SentimentAnalyzer;
impl SentimentAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

pub struct NamedEntityRecognizer;
impl NamedEntityRecognizer {
    pub fn new() -> Self {
        Self
    }
}

pub struct EmbeddingSearch;
impl EmbeddingSearch {
    pub fn new() -> Self {
        Self
    }
}

pub struct RagSearch;
impl RagSearch {
    pub fn new() -> Self {
        Self
    }
}

pub struct MultimodalUnderstanding;
impl MultimodalUnderstanding {
    pub fn new() -> Self {
        Self
    }
}
