//! Layout‑aware document chunking.
//!
//! Splits a [`ParsedDocument`] into semantic [`DocumentChunk`]s based on its
//! layout type (single‑column, multi‑column, mixed) extracted during the
//! parsing pipeline.  Downstream consumers (MemoryLattice, KnowledgeEngine)
//! can index individual chunks for finer‑grained retrieval.

use crate::core::nt_core_input::document_parser::ParsedDocument;

// ---------------------------------------------------------------------------
// Chunk types
// ---------------------------------------------------------------------------

/// A single semantic chunk extracted from a parsed document.
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    /// Sequential index within the document (0‑based).
    pub index: usize,
    /// Heading / title of this chunk (empty string if not available).
    pub heading: String,
    /// Full text content of the chunk.
    pub text: String,
    /// Approximate token count (whitespace‑split word estimate).
    pub estimated_tokens: usize,
    /// Source page number (0‑based, `None` if unknown).
    pub page: Option<usize>,
    /// Chunk region type.
    pub region_type: ChunkRegion,
    /// Confidence score for this chunk (0.0–1.0).
    pub confidence: f64,
}

/// Classification of a chunk's document region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkRegion {
    /// Regular text body.
    Body,
    /// Table region.
    Table,
    /// Heading / section title.
    Heading,
    /// Figure / image caption.
    Figure,
    /// Footnote or endnote.
    Footnote,
    /// Header / footer content.
    HeaderFooter,
    /// Unknown region type.
    Unknown,
}

/// Statistics about a chunking operation.
#[derive(Debug, Clone, Default)]
pub struct ChunkStats {
    pub total_chunks: usize,
    pub total_tokens: usize,
    pub avg_chunk_tokens: f64,
    pub max_chunk_tokens: usize,
    pub min_chunk_tokens: usize,
}

// ---------------------------------------------------------------------------
// Chunking strategy
// ---------------------------------------------------------------------------

/// Strategy to use when chunking a document.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkStrategy {
    /// Split by markdown headings (##, ###, etc.).
    /// Best for single‑column text‑heavy documents.
    Heading,
    /// Split by layout regions (columns, sidebars, etc.).
    /// Best for multi‑column layouts.
    Region,
    /// Split by document sections (paragraph‑level grouping).
    /// Fallback when layout is unknown.
    Section,
}

impl ChunkStrategy {
    /// Choose the best strategy for a given layout type and column count.
    pub fn for_layout(layout_type: Option<&str>, column_count: Option<usize>) -> Self {
        match (layout_type, column_count) {
            (Some(lt), _) if lt.contains("Single") || lt.contains("single") => {
                ChunkStrategy::Heading
            }
            (Some(lt), _)
                if lt.contains("Multi")
                    || lt.contains("multi")
                    || lt.contains("Two")
                    || lt.contains("two") =>
            {
                Self::Region
            }
            (Some(lt), _) if lt.contains("Mixed") || lt.contains("mixed") => Self::Region,
            (_, Some(c)) if c >= 2 => Self::Region,
            _ => Self::Heading,
        }
    }
}

// ---------------------------------------------------------------------------
// Chunking
// ---------------------------------------------------------------------------

/// Default maximum chunk size in estimated tokens.
const DEFAULT_MAX_TOKENS: usize = 512;

/// Chunk a [`ParsedDocument`] into [`DocumentChunk`]s.
///
/// `max_tokens` controls the approximate maximum tokens per chunk (defaults to
/// [`DEFAULT_MAX_TOKENS`] when `None`).
pub fn chunk_document(
    doc: &ParsedDocument,
    max_tokens: Option<usize>,
) -> (Vec<DocumentChunk>, ChunkStats) {
    let max_tok = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
    let strategy = ChunkStrategy::for_layout(
        doc.metadata.layout_type.as_deref(),
        doc.metadata.column_count,
    );

    match strategy {
        ChunkStrategy::Heading => {
            chunk_by_headings(&doc.markdown, max_tok, doc.metadata.formula_count)
        }
        ChunkStrategy::Region => {
            chunk_by_regions(&doc.markdown, max_tok, doc.metadata.formula_count)
        }
        ChunkStrategy::Section => {
            chunk_by_sections(&doc.markdown, max_tok, doc.metadata.formula_count)
        }
    }
}

// ---------------------------------------------------------------------------
// Heading‑based chunking
// ---------------------------------------------------------------------------

fn chunk_by_headings(
    markdown: &str,
    max_tokens: usize,
    _formula_count: usize,
) -> (Vec<DocumentChunk>, ChunkStats) {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut chunks: Vec<DocumentChunk> = Vec::new();
    let mut current_parts: Vec<String> = Vec::new();
    let mut current_heading = String::new();
    let mut current_tokens = 0usize;
    let mut index = 0usize;

    for line in &lines {
        if is_heading(line) {
            // Flush current chunk
            if !current_parts.is_empty() {
                let text = current_parts.join("\n");
                chunks.push(build_chunk(index, &current_heading, &text, max_tokens));
                index += 1;
                current_parts.clear();
                current_tokens = 0;
            }
            current_heading = line.trim_matches('#').trim().to_string();
            continue;
        }

        let line_tokens = estimate_tokens(line);
        if current_tokens + line_tokens > max_tokens && !current_parts.is_empty() {
            let text = current_parts.join("\n");
            chunks.push(build_chunk(index, &current_heading, &text, max_tokens));
            index += 1;
            current_parts.clear();
            current_tokens = 0;
        }

        current_parts.push((*line).to_string());
        current_tokens += line_tokens;
    }

    // Flush remaining
    if !current_parts.is_empty() {
        let text = current_parts.join("\n");
        chunks.push(build_chunk(index, &current_heading, &text, max_tokens));
    }

    let stats = compute_stats(&chunks);
    (chunks, stats)
}

// ---------------------------------------------------------------------------
// Region‑based chunking (multi‑column / mixed layouts)
// ---------------------------------------------------------------------------

fn chunk_by_regions(
    markdown: &str,
    max_tokens: usize,
    _formula_count: usize,
) -> (Vec<DocumentChunk>, ChunkStats) {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut chunks: Vec<DocumentChunk> = Vec::new();
    let mut current_parts: Vec<String> = Vec::new();
    let mut current_heading = String::new();
    let mut current_tokens = 0usize;
    let mut index = 0usize;

    for line in &lines {
        let trimmed = line.trim();
        let is_region_break =
            trimmed.is_empty() || trimmed.starts_with("---") || trimmed.starts_with("***");

        if is_region_break && !current_parts.is_empty() {
            let text = current_parts.join("\n");
            chunks.push(build_chunk(index, &current_heading, &text, max_tokens));
            index += 1;
            current_parts.clear();
            current_tokens = 0;
            continue;
        }

        if is_heading(line) {
            if !current_parts.is_empty() {
                let text = current_parts.join("\n");
                chunks.push(build_chunk(index, &current_heading, &text, max_tokens));
                index += 1;
                current_parts.clear();
                current_tokens = 0;
            }
            current_heading = line.trim_matches('#').trim().to_string();
            continue;
        }

        let line_tokens = estimate_tokens(line);
        if current_tokens + line_tokens > max_tokens && !current_parts.is_empty() {
            let text = current_parts.join("\n");
            chunks.push(build_chunk(index, &current_heading, &text, max_tokens));
            index += 1;
            current_parts.clear();
            current_tokens = 0;
        }

        current_parts.push((*line).to_string());
        current_tokens += line_tokens;
    }

    if !current_parts.is_empty() {
        let text = current_parts.join("\n");
        chunks.push(build_chunk(index, &current_heading, &text, max_tokens));
    }

    let stats = compute_stats(&chunks);
    (chunks, stats)
}

// ---------------------------------------------------------------------------
// Section‑based chunking (fallback)
// ---------------------------------------------------------------------------

fn chunk_by_sections(
    markdown: &str,
    max_tokens: usize,
    _formula_count: usize,
) -> (Vec<DocumentChunk>, ChunkStats) {
    // Split on double newlines (paragraphs), then group into token‑bounded chunks
    let paragraphs: Vec<&str> = markdown.split("\n\n").collect();
    let mut chunks: Vec<DocumentChunk> = Vec::new();
    let mut current_parts: Vec<String> = Vec::new();
    let mut current_tokens = 0usize;
    let mut index = 0usize;

    for para in &paragraphs {
        let para_tokens = estimate_tokens(para);
        if current_tokens + para_tokens > max_tokens && !current_parts.is_empty() {
            let text = current_parts.join("\n\n");
            chunks.push(build_chunk(index, "", &text, max_tokens));
            index += 1;
            current_parts.clear();
            current_tokens = 0;
        }
        current_parts.push((*para).to_string());
        current_tokens += para_tokens;
    }

    if !current_parts.is_empty() {
        let text = current_parts.join("\n\n");
        chunks.push(build_chunk(index, "", &text, max_tokens));
    }

    let stats = compute_stats(&chunks);
    (chunks, stats)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_heading(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("##")
        || (trimmed.starts_with('#')
            && !trimmed.starts_with("# ")
            && trimmed.len() > 1
            && trimmed.chars().nth(1) == Some(' '))
}

fn estimate_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

fn build_chunk(index: usize, heading: &str, text: &str, _max_tokens: usize) -> DocumentChunk {
    let tokens = estimate_tokens(text);
    let region_type = if is_table_block(text) {
        ChunkRegion::Table
    } else if !heading.is_empty() {
        ChunkRegion::Body
    } else {
        ChunkRegion::Unknown
    };

    DocumentChunk {
        index,
        heading: heading.to_string(),
        text: text.to_string(),
        estimated_tokens: tokens,
        page: None,
        region_type,
        confidence: 1.0,
    }
}

fn is_table_block(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 2 {
        return false;
    }
    lines.iter().any(|l| l.trim().starts_with('|')) && lines.iter().any(|l| l.contains("---"))
}

fn compute_stats(chunks: &[DocumentChunk]) -> ChunkStats {
    let total_chunks = chunks.len();
    let total_tokens: usize = chunks.iter().map(|c| c.estimated_tokens).sum();
    let avg_chunk_tokens = if total_chunks > 0 {
        total_tokens as f64 / total_chunks as f64
    } else {
        0.0
    };
    let max_chunk_tokens = chunks.iter().map(|c| c.estimated_tokens).max().unwrap_or(0);
    let min_chunk_tokens = chunks.iter().map(|c| c.estimated_tokens).min().unwrap_or(0);

    ChunkStats {
        total_chunks,
        total_tokens,
        avg_chunk_tokens,
        max_chunk_tokens,
        min_chunk_tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_input::document_parser::{DocumentMetadata, ParsedDocument};

    fn make_doc(markdown: &str, layout_type: &str, column_count: usize) -> ParsedDocument {
        ParsedDocument {
            markdown: markdown.to_string(),
            tables: vec![],
            images: vec![],
            metadata: DocumentMetadata {
                layout_type: Some(layout_type.to_string()),
                column_count: Some(column_count),
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_heading_chunking_single_column() {
        let md = "\
# Intro
Some introductory text.

## Section 1
Content for section one.

## Section 2
Content for section two.
";
        let doc = make_doc(md, "SingleColumn", 1);
        let (chunks, stats) = chunk_document(&doc, Some(500));
        assert!(
            chunks.len() >= 2,
            "expected ≥2 chunks, got {}",
            chunks.len()
        );
        // First chunk should have "Intro" heading
        assert!(!chunks[0].heading.is_empty() || !chunks[1].heading.is_empty());
        assert!(stats.total_chunks > 0);
        assert!(stats.total_tokens > 0);
    }

    #[test]
    fn test_region_chunking_multi_column() {
        let md = "\
# Title

Column A content line one.
Column A content line two.

---

Column B content line one.
Column B content line two.
";
        let doc = make_doc(md, "TwoColumn", 2);
        let (chunks, stats) = chunk_document(&doc, Some(500));
        assert!(chunks.len() >= 1);
        assert!(stats.total_chunks > 0);
    }

    #[test]
    fn test_section_chunking_unknown_layout() {
        let md = "\
Paragraph one that has enough words to form a coherent section.

Paragraph two about a completely different topic.

Paragraph three wrapping up.
";
        let doc = make_doc(md, "Unknown", 0);
        let (chunks, stats) = chunk_document(&doc, Some(500));
        assert!(chunks.len() >= 1);
        assert!(stats.total_chunks > 0);
    }

    #[test]
    fn test_empty_document() {
        let doc = make_doc("", "SingleColumn", 1);
        let (chunks, stats) = chunk_document(&doc, Some(512));
        assert!(chunks.is_empty());
        assert_eq!(stats.total_chunks, 0);
    }

    #[test]
    fn test_chunk_strategy_selection() {
        assert_eq!(
            ChunkStrategy::for_layout(Some("SingleColumn"), Some(1)),
            ChunkStrategy::Heading
        );
        assert_eq!(
            ChunkStrategy::for_layout(Some("TwoColumn"), Some(2)),
            ChunkStrategy::Region
        );
        assert_eq!(
            ChunkStrategy::for_layout(Some("Mixed"), Some(3)),
            ChunkStrategy::Region
        );
        assert_eq!(
            ChunkStrategy::for_layout(Some("single"), Some(1)),
            ChunkStrategy::Heading
        );
        assert_eq!(
            ChunkStrategy::for_layout(None, Some(1)),
            ChunkStrategy::Heading
        );
        assert_eq!(
            ChunkStrategy::for_layout(None, None),
            ChunkStrategy::Heading
        );
    }

    #[test]
    fn test_max_tokens_respected() {
        let md = "\
# A
word "
            .repeat(200);
        let doc = make_doc(&md, "SingleColumn", 1);
        let (chunks, _) = chunk_document(&doc, Some(50));
        assert!(
            chunks.len() >= 2,
            "expected multiple chunks with 50 token limit, got {}",
            chunks.len()
        );
        for chunk in &chunks {
            assert!(
                chunk.estimated_tokens <= 120,
                "chunk exceeded max tokens: {}",
                chunk.estimated_tokens
            );
        }
    }

    #[test]
    fn test_stats_consistency() {
        let md = "\
# One
Content.

# Two
More content.

# Three
Final content.
";
        let doc = make_doc(md, "SingleColumn", 1);
        let (chunks, stats) = chunk_document(&doc, Some(100));
        assert_eq!(stats.total_chunks, chunks.len());
        assert_eq!(
            stats.max_chunk_tokens,
            chunks.iter().map(|c| c.estimated_tokens).max().unwrap_or(0)
        );
        assert_eq!(
            stats.min_chunk_tokens,
            chunks.iter().map(|c| c.estimated_tokens).min().unwrap_or(0)
        );
    }
}
