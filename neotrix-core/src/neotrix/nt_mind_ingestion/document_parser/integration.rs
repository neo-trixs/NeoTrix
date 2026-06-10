use std::collections::HashMap;
use super::types::*;

/// Trait for integrating parsed documents into the knowledge ingestion pipeline.
pub trait DocumentIngestion {
    fn ingest_document(&mut self, doc: &ParsedDocument) -> Result<Vec<String>, String>;
    fn extract_knowledge_nodes(&self, doc: &ParsedDocument) -> Vec<KnowledgeNode>;
}

/// Default implementation that extracts knowledge nodes from a parsed document.
pub struct DefaultDocumentIngestion;

impl DefaultDocumentIngestion {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentIngestion for DefaultDocumentIngestion {
    fn ingest_document(&mut self, doc: &ParsedDocument) -> Result<Vec<String>, String> {
        let nodes = self.extract_knowledge_nodes(doc);
        let ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
        Ok(ids)
    }

    fn extract_knowledge_nodes(&self, doc: &ParsedDocument) -> Vec<KnowledgeNode> {
        let mut nodes = Vec::new();

        let sections: Vec<&Section> = doc.document.sections.iter()
            .flat_map(|s| s.flatten())
            .collect();

        for (i, section) in sections.iter().enumerate() {
            let title = section.heading.clone()
                .unwrap_or_else(|| format!("section_{}", i));
            let node = KnowledgeNode {
                id: format!("{}_{}", doc.document.format.name(), i),
                title,
                content: section.content.clone(),
                node_type: "section".to_string(),
                vector: doc.vsa_vectors.get(i).cloned().unwrap_or_else(|| vec![0; 4096]),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("section_level".to_string(), section.level.to_string());
                    if let Some(ref heading) = section.heading {
                        m.insert("heading".to_string(), heading.clone());
                    }
                    m.insert("format".to_string(), doc.document.format.name().to_string());
                    m
                },
            };
            nodes.push(node);
        }

        if !sections.is_empty() {
            let doc_node = KnowledgeNode {
                id: format!("{}_document", doc.document.format.name()),
                title: doc.document.title.clone()
                    .unwrap_or_else(|| "untitled".to_string()),
                content: doc.document.raw_text.clone(),
                node_type: "document".to_string(),
                vector: doc.combined_vector.clone(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("format".to_string(), doc.document.format.name().to_string());
                    m.insert("section_count".to_string(), doc.section_count.to_string());
                    m.insert("estimated_reading_time".to_string(), doc.estimated_reading_time.to_string());
                    m
                },
            };
            nodes.push(doc_node);
        }

        nodes
    }
}

/// Filter knowledge nodes by type.
pub fn filter_by_type<'a>(nodes: &'a [KnowledgeNode], node_type: &str) -> Vec<&'a KnowledgeNode> {
    nodes.iter().filter(|n| n.node_type == node_type).collect()
}

/// Find the most similar knowledge node to a query vector using Hamming similarity.
pub fn find_most_similar<'a>(
    query: &[u8],
    nodes: &'a [KnowledgeNode],
) -> Option<(&'a KnowledgeNode, f64)> {
    nodes.iter()
        .map(|n| {
            let sim = super::engine::hamming_similarity(query, &n.vector);
            (n, sim)
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn make_test_document() -> ParsedDocument {
        let section = Section {
            heading: Some("Test Section".into()),
            level: 1,
            content: "This is test content for the document parser.".into(),
            bounding_box: None,
            subsections: vec![],
        };
        let document = Document {
            format: DocumentFormat::PlainText,
            title: Some("Test Doc".into()),
            sections: vec![section],
            metadata: HashMap::new(),
            raw_text: "This is test content for the document parser.".into(),
        };
        let vsa = vec![1u8; VSA_DIM];
        ParsedDocument {
            document,
            vsa_vectors: vec![vsa.clone()],
            combined_vector: vsa,
            section_count: 1,
            estimated_reading_time: 3.0,
        }
    }

    #[test]
    fn test_extract_knowledge_nodes() {
        let ingestion = DefaultDocumentIngestion::new();
        let doc = make_test_document();
        let nodes = ingestion.extract_knowledge_nodes(&doc);
        assert_eq!(nodes.len(), 2);

        let section_node = &nodes[0];
        assert_eq!(section_node.node_type, "section");
        assert_eq!(section_node.title, "Test Section");

        let doc_node = &nodes[1];
        assert_eq!(doc_node.node_type, "document");
        assert_eq!(doc_node.title, "Test Doc");
    }

    #[test]
    fn test_ingest_document_returns_ids() {
        let mut ingestion = DefaultDocumentIngestion::new();
        let doc = make_test_document();
        let ids = ingestion.ingest_document(&doc).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids[0].contains("plain_text_"));
    }

    #[test]
    fn test_filter_by_type() {
        let ingestion = DefaultDocumentIngestion::new();
        let doc = make_test_document();
        let nodes = ingestion.extract_knowledge_nodes(&doc);

        let sections = filter_by_type(&nodes, "section");
        assert_eq!(sections.len(), 1);

        let docs = filter_by_type(&nodes, "document");
        assert_eq!(docs.len(), 1);

        let none = filter_by_type(&nodes, "concept");
        assert!(none.is_empty());
    }

    #[test]
    fn test_find_most_similar() {
        let ingestion = DefaultDocumentIngestion::new();
        let doc = make_test_document();
        let nodes = ingestion.extract_knowledge_nodes(&doc);

        let query = vec![1u8; VSA_DIM];
        let (best, sim) = find_most_similar(&query, &nodes).unwrap();
        assert_eq!(best.title, "Test Section");
        assert!((sim - 1.0).abs() < 1e-10);
    }
}
