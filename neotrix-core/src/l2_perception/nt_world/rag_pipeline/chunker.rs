use super::document::{Chunk, Document};

/// Split a document's content into overlapping character-level chunks.
///
/// Each chunk is at most `chunk_size` characters long. Consecutive chunks
/// overlap by `overlap` characters to preserve context across boundaries.
/// The resulting `Document` retains the original `id` and `metadata`; its
/// `chunks` field is populated with the generated `Chunk` values (offsets
/// are in **bytes**).
///
/// # Panics
/// Panics if `chunk_size == 0` or `overlap >= chunk_size`.
pub fn chunk_document(doc: &Document, chunk_size: usize, overlap: usize) -> Document {
    assert!(chunk_size > 0, "chunk_size must be > 0");
    assert!(overlap < chunk_size, "overlap must be < chunk_size");

    let content_bytes = doc.content.as_bytes();
    let len = content_bytes.len();

    let mut chunks = Vec::new();
    let mut start = 0usize;

    while start < len {
        let end = (start + chunk_size).min(len);
        let slice = &content_bytes[start..end];

        // Find a valid UTF-8 boundary (avoid splitting a char).
        let text = if end < len {
            let mut safe_end = end;
            while safe_end > start && !std::str::from_utf8(&content_bytes[start..safe_end]).is_ok()
            {
                safe_end -= 1;
            }
            std::str::from_utf8(&content_bytes[start..safe_end])
                .unwrap_or("")
                .to_string()
        } else {
            std::str::from_utf8(slice).unwrap_or("").to_string()
        };

        let actual_end = start + text.len();

        chunks.push(Chunk {
            text,
            start_offset: start,
            end_offset: actual_end,
            embedding: None,
        });

        if actual_end >= len {
            break;
        }
        start = actual_end - overlap;
    }

    Document {
        id: doc.id.clone(),
        content: doc.content.clone(),
        metadata: doc.metadata.clone(),
        chunks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_doc(content: &str) -> Document {
        Document {
            id: "test".into(),
            content: content.into(),
            metadata: HashMap::new(),
            chunks: vec![],
        }
    }

    #[test]
    fn basic_chunking() {
        let doc = make_doc("abcdefghij");
        let result = chunk_document(&doc, 4, 1);
        assert_eq!(result.chunks.len(), 3);
        assert_eq!(result.chunks[0].text, "abcd");
        assert_eq!(result.chunks[0].start_offset, 0);
        assert_eq!(result.chunks[1].text, "defg");
        assert_eq!(result.chunks[1].start_offset, 3);
    }

    #[test]
    fn no_overlap() {
        let doc = make_doc("abcdef");
        let result = chunk_document(&doc, 3, 0);
        assert_eq!(result.chunks.len(), 2);
        assert_eq!(result.chunks[0].text, "abc");
        assert_eq!(result.chunks[1].text, "def");
    }

    #[test]
    fn content_shorter_than_chunk_size() {
        let doc = make_doc("hi");
        let result = chunk_document(&doc, 10, 2);
        assert_eq!(result.chunks.len(), 1);
        assert_eq!(result.chunks[0].text, "hi");
        assert_eq!(result.chunks[0].end_offset, 2);
    }

    #[test]
    fn utf8_preserved() {
        let doc = make_doc("日本語テスト");
        let result = chunk_document(&doc, 6, 1);
        for c in &result.chunks {
            assert!(std::str::from_utf8(c.text.as_bytes()).is_ok());
        }
    }

    #[test]
    #[should_panic]
    fn zero_chunk_size_panics() {
        let doc = make_doc("x");
        chunk_document(&doc, 0, 0);
    }

    #[test]
    #[should_panic]
    fn overlap_gte_chunk_size_panics() {
        let doc = make_doc("x");
        chunk_document(&doc, 3, 3);
    }
}
