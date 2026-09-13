//! RAG Pipeline — 检索增强生成
//!
//! 吸收 KB 经验:
//! - 文档分块 (Chunking)
//! - 向量检索
//! - 重排序 (Re-ranking)
//! - 上下文压缩
//! - 引用追溯

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// RAG 管线
pub struct _RAGPipeline {
    chunker: _DocumentChunker,
    retriever: _VectorRetriever,
    reranker: Reranker,
    compressor: ContextCompressor,
    config: _RAGConfig,
}

/// RAG 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RAGConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub top_k: usize,
    pub similarity_threshold: f64,
    pub max_context_tokens: usize,
    pub enable_reranking: bool,
    pub enable_compression: bool,
}

impl Default for _RAGConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 50,
            top_k: 10,
            similarity_threshold: 0.7,
            max_context_tokens: 4000,
            enable_reranking: true,
            enable_compression: true,
        }
    }
}

/// 文档分块器
pub struct _DocumentChunker {
    chunk_size: usize,
    overlap: usize,
}

/// 文档块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub metadata: _ChunkMetadata,
    pub embedding: Option<Vec<f32>>,
}

/// 块元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ChunkMetadata {
    pub start_pos: usize,
    pub end_pos: usize,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub source: Option<String>,
    pub page: Option<u32>,
}

/// 向量检索器
pub struct _VectorRetriever {
    index: HashMap<String, Vec<f32>>,
    chunks: HashMap<String, _DocumentChunk>,
}

/// 重排序器
pub struct Reranker {
    model: String,
}

/// 上下文压缩器
pub struct ContextCompressor {
    max_tokens: usize,
}

/// RAG 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RAGResult {
    pub query: String,
    pub chunks: Vec<_DocumentChunk>,
    pub context: String,
    pub citations: Vec<Citation>,
    pub relevance_score: f64,
}

/// 引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub chunk_id: String,
    pub document_id: String,
    pub source: Option<String>,
    pub page: Option<u32>,
    pub relevance: f64,
}

/// 检索结果
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub chunk: _DocumentChunk,
    pub score: f64,
}

impl _DocumentChunker {
    /// 创建新的分块器
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self { chunk_size, overlap }
    }

    /// 分块文档
    pub(crate) fn _chunk_document(&self, document_id: &str, content: &str, source: Option<String>) -> Vec<_DocumentChunk> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let total_chars = chars.len();
        let mut start = 0;
        let mut chunk_index = 0;

        while start < total_chars {
            let end = (start + self.chunk_size).min(total_chars);
            let chunk_content: String = chars[start..end].iter().collect();

            chunks.push(_DocumentChunk {
                id: format!("{}:{}", document_id, chunk_index),
                document_id: document_id.to_string(),
                content: chunk_content,
                metadata: _ChunkMetadata {
                    start_pos: start,
                    end_pos: end,
                    chunk_index,
                    total_chunks: 0, // 会稍后更新
                    source: source.clone(),
                    page: None,
                },
                embedding: None,
            });

            start += self.chunk_size - self.overlap;
            chunk_index += 1;
        }

        // 更新 total_chunks
        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.metadata.total_chunks = total;
        }

        chunks
    }

    /// 按段落分块
    pub(crate) fn _chunk_by_paragraph(&self, document_id: &str, content: &str, source: Option<String>) -> Vec<_DocumentChunk> {
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        let mut chunks = Vec::new();

        for (i, paragraph) in paragraphs.iter().enumerate() {
            if !paragraph.trim().is_empty() {
                chunks.push(_DocumentChunk {
                    id: format!("{}:para:{}", document_id, i),
                    document_id: document_id.to_string(),
                    content: paragraph.to_string(),
                    metadata: _ChunkMetadata {
                        start_pos: 0,
                        end_pos: paragraph.len(),
                        chunk_index: i,
                        total_chunks: paragraphs.len(),
                        source: source.clone(),
                        page: None,
                    },
                    embedding: None,
                });
            }
        }

        chunks
    }
}

impl _VectorRetriever {
    /// 创建新的检索器
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            chunks: HashMap::new(),
        }
    }

    /// 索引文档块
    pub(crate) fn _index_chunks(&mut self, chunks: Vec<_DocumentChunk>) {
        for chunk in chunks {
            if let Some(embedding) = &chunk.embedding {
                self.index.insert(chunk.id.clone(), embedding.clone());
            }
            self.chunks.insert(chunk.id.clone(), chunk);
        }
    }

    /// 检索相似块
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<RetrievalResult> {
        let mut results: Vec<RetrievalResult> = self.index.iter()
            .map(|(id, embedding)| {
                let score = cosine_similarity(query_embedding, embedding);
                RetrievalResult {
                    chunk: self.chunks.get(id).unwrap().clone(),
                    score,
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.into_iter().take(top_k).collect()
    }
}

impl Reranker {
    /// 创建新的重排序器
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
        }
    }

    /// 重排序结果
    pub fn rerank(&self, query: &str, results: Vec<RetrievalResult>) -> Vec<RetrievalResult> {
        // 简化版: 基于关键词匹配重排序
        let query_words: Vec<&str> = query.split_whitespace().collect();

        let mut reranked: Vec<RetrievalResult> = results.into_iter()
            .map(|mut result| {
                let content_words: Vec<&str> = result.chunk.content.split_whitespace().collect();
                let overlap = query_words.iter()
                    .filter(|q| content_words.iter().any(|c| c.eq_ignore_ascii_case(q)))
                    .count();
                result.score += overlap as f64 * 0.1;
                result
            })
            .collect();

        reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        reranked
    }
}

impl ContextCompressor {
    /// 创建新的压缩器
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }

    /// 压缩上下文
    pub fn compress(&self, chunks: Vec<_DocumentChunk>) -> (String, Vec<Citation>) {
        let mut context = String::new();
        let mut citations = Vec::new();
        let mut current_tokens = 0;

        for chunk in chunks {
            // 粗略估计 token 数 (1 token ≈ 4 chars)
            let estimated_tokens = chunk.content.len() / 4;

            if current_tokens + estimated_tokens > self.max_tokens {
                break;
            }

            context.push_str(&chunk.content);
            context.push_str("\n\n");

            citations.push(Citation {
                chunk_id: chunk.id.clone(),
                document_id: chunk.document_id.clone(),
                source: chunk.metadata.source.clone(),
                page: chunk.metadata.page,
                relevance: 0.0,
            });

            current_tokens += estimated_tokens;
        }

        (context, citations)
    }
}

impl _RAGPipeline {
    /// 创建新的 RAG 管线
    pub fn new(config: _RAGConfig) -> Self {
        Self {
            chunker: _DocumentChunker::new(config.chunk_size, config.chunk_overlap),
            retriever: _VectorRetriever::new(),
            reranker: Reranker::new("default"),
            compressor: ContextCompressor::new(config.max_context_tokens),
            config,
        }
    }

    /// 索引文档
    pub(crate) fn _index_document(&mut self, document_id: &str, content: &str, source: Option<String>) {
        let chunks = self.chunker._chunk_document(document_id, content, source);
        self.retriever._index_chunks(chunks);
    }

    /// 查询
    pub fn query(&self, query: &str, query_embedding: &[f32]) -> _RAGResult {
        // 检索
        let mut results = self.retriever.search(query_embedding, self.config.top_k);

        // 重排序
        if self.config.enable_reranking {
            results = self.reranker.rerank(query, results);
        }

        // 过滤低相关性结果
        let filtered_results: Vec<RetrievalResult> = results.into_iter()
            .filter(|r| r.score >= self.config.similarity_threshold)
            .collect();

        let chunks: Vec<_DocumentChunk> = filtered_results.iter().map(|r| r.chunk.clone()).collect();
        let len = chunks.len();

        // 压缩上下文
        let (context, citations) = if self.config.enable_compression {
            self.compressor.compress(chunks.clone())
        } else {
            let ctx = chunks.iter().map(|c| c.content.as_str()).collect::<Vec<_>>().join("\n\n");
            let cits = chunks.iter().map(|c| Citation {
                chunk_id: c.id.clone(),
                document_id: c.document_id.clone(),
                source: c.metadata.source.clone(),
                page: c.metadata.page,
                relevance: 0.0,
            }).collect();
            (ctx, cits)
        };

        let relevance_score = if len > 0 {
            filtered_results.iter().map(|r| r.score).sum::<f64>() / len as f64
        } else {
            0.0
        };

        _RAGResult {
            query: query.to_string(),
            chunks,
            context,
            citations,
            relevance_score,
        }
    }
}

use crate::core::nt_core_math::cosine_similarity_f32 as cosine_similarity;
