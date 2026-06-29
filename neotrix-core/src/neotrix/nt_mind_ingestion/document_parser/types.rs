use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentFormat {
    Pdf,
    Docx,
    Html,
    Markdown,
    PlainText,
    Rtf,
}

impl DocumentFormat {
    pub fn name(&self) -> &str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Html => "html",
            Self::Markdown => "markdown",
            Self::PlainText => "plain_text",
            Self::Rtf => "rtf",
        }
    }

    pub fn from_extension(path: &std::path::Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "html" | "htm" => Some(Self::Html),
            "md" | "markdown" => Some(Self::Markdown),
            "txt" => Some(Self::PlainText),
            "rtf" => Some(Self::Rtf),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub heading: Option<String>,
    pub level: u8,
    pub content: String,
    pub bounding_box: Option<Rect>,
    pub subsections: Vec<Section>,
}

impl Section {
    pub fn flatten(&self) -> Vec<&Section> {
        let mut result = vec![self];
        for sub in &self.subsections {
            result.extend(sub.flatten());
        }
        result
    }

    pub fn total_words(&self) -> usize {
        let mut count = self.content.split_whitespace().count();
        for sub in &self.subsections {
            count += sub.total_words();
        }
        count
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub format: DocumentFormat,
    pub title: Option<String>,
    pub sections: Vec<Section>,
    pub metadata: HashMap<String, String>,
    pub raw_text: String,
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub document: Document,
    pub vsa_vectors: Vec<Vec<u8>>,
    pub combined_vector: Vec<u8>,
    pub section_count: usize,
    pub estimated_reading_time: f64,
}

#[derive(Debug, Clone)]
pub struct KnowledgeNode {
    pub id: String,
    pub title: String,
    pub content: String,
    pub node_type: String,
    pub vector: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_format_name() {
        assert_eq!(DocumentFormat::Pdf.name(), "pdf");
        assert_eq!(DocumentFormat::Html.name(), "html");
        assert_eq!(DocumentFormat::Markdown.name(), "markdown");
    }

    #[test]
    fn test_document_format_from_extension() {
        assert_eq!(
            DocumentFormat::from_extension(std::path::Path::new("file.pdf")),
            Some(DocumentFormat::Pdf)
        );
        assert_eq!(
            DocumentFormat::from_extension(std::path::Path::new("doc.html")),
            Some(DocumentFormat::Html)
        );
        assert_eq!(
            DocumentFormat::from_extension(std::path::Path::new("readme.md")),
            Some(DocumentFormat::Markdown)
        );
        assert_eq!(
            DocumentFormat::from_extension(std::path::Path::new("notes.txt")),
            Some(DocumentFormat::PlainText)
        );
        assert_eq!(
            DocumentFormat::from_extension(std::path::Path::new("unknown.xyz")),
            None
        );
        assert_eq!(
            DocumentFormat::from_extension(std::path::Path::new("no_ext")),
            None
        );
    }

    #[test]
    fn test_section_flatten() {
        let sub = Section {
            heading: Some("Sub".into()),
            level: 2,
            content: "sub content".into(),
            bounding_box: None,
            subsections: vec![],
        };
        let parent = Section {
            heading: Some("Parent".into()),
            level: 1,
            content: "parent content".into(),
            bounding_box: None,
            subsections: vec![sub],
        };
        let flat = parent.flatten();
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].heading.as_deref(), Some("Parent"));
        assert_eq!(flat[1].heading.as_deref(), Some("Sub"));
    }

    #[test]
    fn test_section_total_words() {
        let sub = Section {
            heading: Some("Sub".into()),
            level: 2,
            content: "two words".into(),
            bounding_box: None,
            subsections: vec![],
        };
        let parent = Section {
            heading: Some("Parent".into()),
            level: 1,
            content: "three little words".into(),
            bounding_box: None,
            subsections: vec![sub],
        };
        assert_eq!(parent.total_words(), 5);
    }

    #[test]
    fn test_rect_construction() {
        let r = Rect {
            x: 1.0,
            y: 2.0,
            width: 100.0,
            height: 200.0,
        };
        assert_eq!(r.x, 1.0);
        assert_eq!(r.height, 200.0);
    }
}
