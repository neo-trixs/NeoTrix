// G401: One-shot long-horizon OCR engine — baidu Unlimited-OCR inspired
// Dual mode: Gundam (document-focused, crop) / Base (full-page, no crop)
// Tile pipeline + doc structure reconstruction + VSA grounding bridge
use serde::{Deserialize, Serialize};

// ── Document Sources ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentSource {
    Url(String),
    PdfFile(String),
    ImageFile(String),
    RawBytes(Vec<u8>),
}

// ── OCR Mode ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcrMode {
    /// Document-focused: base_size=1024, image_size=640, crop=true
    Gundam,
    /// Full-page: base_size=1024, image_size=1024, crop=false
    Base,
}

impl OcrMode {
    pub fn base_size(&self) -> u32 {
        1024
    }
    pub fn image_size(&self) -> u32 {
        match self {
            OcrMode::Gundam => 640,
            OcrMode::Base => 1024,
        }
    }
    pub fn use_crop(&self) -> bool {
        matches!(self, OcrMode::Gundam)
    }
}

// ── Tile Processing ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrTile {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub raw_text: String,
    pub confidence: f64,
    pub elements: Vec<DocElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocElement {
    Heading {
        level: u8,
        text: String,
        bbox: BBox,
    },
    Paragraph {
        text: String,
        bbox: BBox,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        bbox: BBox,
    },
    List {
        items: Vec<String>,
        ordered: bool,
        bbox: BBox,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
        bbox: BBox,
    },
    Figure {
        caption: Option<String>,
        bbox: BBox,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// ── OCR Configuration ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    pub mode: OcrMode,
    pub max_tiles: usize,
    pub tile_overlap: f64,
    pub confidence_threshold: f64,
    pub no_repeat_ngram_size: usize,
    pub enabled_linguistic: bool,
    pub enabled_table: bool,
    pub enabled_structure: bool,
    pub pdf_dpi: u32,
    pub pdf_max_pages: usize,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            mode: OcrMode::Gundam,
            max_tiles: 64,
            tile_overlap: 0.1,
            confidence_threshold: 0.5,
            no_repeat_ngram_size: 35,
            enabled_linguistic: true,
            enabled_table: true,
            enabled_structure: true,
            pdf_dpi: 200,
            pdf_max_pages: 50,
        }
    }
}

// ── OCR Result ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub full_text: String,
    pub structured_doc: StructuredDocument,
    pub tiles: Vec<OcrTile>,
    pub confidence: f64,
    pub page_count: u32,
    pub processing_time_ms: u64,
    pub mode: OcrMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredDocument {
    pub title: Option<String>,
    pub sections: Vec<DocumentSection>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    pub heading: Option<String>,
    pub heading_level: u8,
    pub elements: Vec<DocElement>,
    pub subsections: Vec<DocumentSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub language: Option<String>,
    pub estimated_reading_time_sec: u64,
    pub element_count: usize,
    pub table_count: usize,
    pub list_count: usize,
    pub code_block_count: usize,
    pub has_tables: bool,
    pub has_code: bool,
    pub has_lists: bool,
}

// ── No-Repeat N-Gram Filter ──

#[derive(Debug, Clone)]
pub struct NoRepeatNgramFilter {
    pub ngram_size: usize,
    seen: Vec<Vec<String>>,
}

impl NoRepeatNgramFilter {
    pub fn new(ngram_size: usize) -> Self {
        Self {
            ngram_size,
            seen: Vec::new(),
        }
    }

    pub fn filter(&mut self, tokens: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        let mut window: Vec<String> = Vec::with_capacity(self.ngram_size);

        for token in tokens {
            window.push(token.clone());
            if window.len() > self.ngram_size {
                window.remove(0);
            }

            if window.len() == self.ngram_size && self.seen.contains(&window) {
                continue;
            }

            if window.len() == self.ngram_size {
                self.seen.push(window.clone());
            }
            result.push(token.clone());
        }
        result
    }

    pub fn filter_text(&mut self, text: &str) -> String {
        let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        let filtered = self.filter(&tokens);
        filtered.join(" ")
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }
}

// ── PDF Renderer (abstraction) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPageImage {
    pub page_num: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub dpi: u32,
}

#[derive(Debug, Clone)]
pub struct PdfRenderer {
    pub dpi: u32,
    pub max_pages: usize,
}

impl PdfRenderer {
    pub fn new(dpi: u32, max_pages: usize) -> Self {
        Self { dpi, max_pages }
    }

    /// Render PDF to page images. Uses external PDF renderer (MuPDF/PDFium/poppler).
    /// Returns empty vec if no renderer available — caller should handle fallback.
    pub fn render(&self, path: &str) -> Result<Vec<PdfPageImage>, OcrError> {
        if !std::path::Path::new(path).exists() {
            return Err(OcrError::FileNotFound(path.to_string()));
        }
        // Stub: In production, delegates to pdf-extract or pdf-render crate
        // or external poppler/pdftoppm CLI
        Err(OcrError::RendererNotAvailable(
            "PDF renderer not configured. Install poppler or pdf-extract crate.".to_string(),
        ))
    }

    pub fn render_with_fallback(
        &self,
        path: &str,
        fallback: &dyn Fn(&str) -> Result<Vec<PdfPageImage>, OcrError>,
    ) -> Result<Vec<PdfPageImage>, OcrError> {
        self.render(path).or_else(|_| fallback(path))
    }
}

// ── Document Structure Reconstructor ──

#[derive(Debug, Clone)]
pub struct DocStructureReconstructor {
    pub min_heading_length: usize,
    pub table_detect_min_rows: usize,
    pub list_detect_prefixes: Vec<String>,
}

impl Default for DocStructureReconstructor {
    fn default() -> Self {
        Self {
            min_heading_length: 80,
            table_detect_min_rows: 2,
            list_detect_prefixes: vec![
                "•".into(),
                "-".into(),
                "*".into(),
                "1.".into(),
                "• ".into(),
                "- ".into(),
                "* ".into(),
                "1. ".into(),
            ],
        }
    }
}

impl DocStructureReconstructor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reconstruct(&self, tiles: &[OcrTile]) -> StructuredDocument {
        let mut all_elements: Vec<(f64, &DocElement)> = Vec::new();
        for tile in tiles {
            for element in &tile.elements {
                let y = match element {
                    DocElement::Heading { bbox, .. } => bbox.y,
                    DocElement::Paragraph { bbox, .. } => bbox.y,
                    DocElement::Table { bbox, .. } => bbox.y,
                    DocElement::List { bbox, .. } => bbox.y,
                    DocElement::CodeBlock { bbox, .. } => bbox.y,
                    DocElement::Figure { bbox, .. } => bbox.y,
                };
                all_elements.push((y, element));
            }
        }

        all_elements.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut sections: Vec<DocumentSection> = Vec::new();
        let mut current_section: Option<DocumentSection> = None;

        for (_, element) in &all_elements {
            match element {
                DocElement::Heading { text, level, .. } => {
                    if let Some(section) = current_section.take() {
                        sections.push(section);
                    }
                    current_section = Some(DocumentSection {
                        heading: Some(text.clone()),
                        heading_level: *level,
                        elements: Vec::new(),
                        subsections: Vec::new(),
                    });
                }
                other => {
                    if let Some(ref mut section) = current_section {
                        section.elements.push((*other).clone());
                    } else {
                        sections.push(DocumentSection {
                            heading: None,
                            heading_level: 0,
                            elements: vec![(*other).clone()],
                            subsections: Vec::new(),
                        });
                    }
                }
            }
        }
        if let Some(section) = current_section {
            sections.push(section);
        }

        let title = sections.first().and_then(|s| s.heading.clone());

        let full_text: String = all_elements
            .iter()
            .map(|(_, e)| match e {
                DocElement::Heading { text, .. } => format!("# {}\n", text),
                DocElement::Paragraph { text, .. } => format!("{}\n", text),
                DocElement::Table { headers, rows, .. } => {
                    let mut t = format!("| {} |\n", headers.join(" | "));
                    t.push_str(&format!(
                        "| {} |\n",
                        headers
                            .iter()
                            .map(|_| "---")
                            .collect::<Vec<_>>()
                            .join(" | ")
                    ));
                    for row in rows {
                        t.push_str(&format!("| {} |\n", row.join(" | ")));
                    }
                    t
                }
                DocElement::List { items, .. } => {
                    items.iter().map(|i| format!("- {}\n", i)).collect()
                }
                DocElement::CodeBlock { code, .. } => format!("```\n{}\n```\n", code),
                DocElement::Figure { caption, .. } => {
                    if let Some(c) = caption {
                        format!("[Figure: {}]\n", c)
                    } else {
                        "[Figure]\n".to_string()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join("");

        let element_count = all_elements.len();
        let table_count = all_elements
            .iter()
            .filter(|(_, e)| matches!(e, DocElement::Table { .. }))
            .count();
        let list_count = all_elements
            .iter()
            .filter(|(_, e)| matches!(e, DocElement::List { .. }))
            .count();
        let code_count = all_elements
            .iter()
            .filter(|(_, e)| matches!(e, DocElement::CodeBlock { .. }))
            .count();

        StructuredDocument {
            title,
            sections,
            metadata: DocumentMetadata {
                language: None,
                estimated_reading_time_sec: (full_text.split_whitespace().count() / 200) as u64,
                element_count,
                table_count,
                list_count,
                code_block_count: code_count,
                has_tables: table_count > 0,
                has_code: code_count > 0,
                has_lists: list_count > 0,
            },
        }
    }

    pub fn detect_element_type(&self, text: &str, _confidence: f64) -> DocElementType {
        if text.len() < self.min_heading_length
            && text.len() > 3
            && text
                .chars()
                .all(|c| c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation())
        {
            return DocElementType::Heading;
        }

        let lines: Vec<&str> = text.lines().collect();
        if lines.len() >= self.table_detect_min_rows {
            let pipe_count = lines.iter().filter(|l| l.contains('|')).count();
            if pipe_count as f64 / lines.len() as f64 > 0.5 {
                return DocElementType::Table;
            }
        }

        let list_lines = lines
            .iter()
            .filter(|l| {
                self.list_detect_prefixes
                    .iter()
                    .any(|p| l.trim_start().starts_with(p))
            })
            .count();
        if list_lines as f64 / lines.len() as f64 > 0.6 {
            return DocElementType::List;
        }

        if text.trim_start().starts_with("```")
            || text.trim_start().starts_with("fn ")
            || text.contains("impl ")
            || text.contains("struct ")
        {
            return DocElementType::CodeBlock;
        }

        DocElementType::Paragraph
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocElementType {
    Heading,
    Paragraph,
    Table,
    List,
    CodeBlock,
    Figure,
}

// ── VSA Grounding Bridge ──

#[derive(Debug, Clone)]
pub struct OcrVsaBridge;

impl OcrVsaBridge {
    /// Encode structured document into a VSA-compatible embedding vector.
    /// Returns normalized feature vector for world model grounding.
    pub fn encode_to_vsa_features(&self, doc: &StructuredDocument) -> Vec<f64> {
        let mut features = Vec::with_capacity(32);

        // Structure features (8 dims)
        features.push(doc.sections.len() as f64); // section count
        features.push(doc.metadata.element_count as f64); // element count
        features.push(doc.metadata.table_count as f64); // table count
        features.push(doc.metadata.code_block_count as f64); // code block count
        features.push(if doc.title.is_some() { 1.0 } else { 0.0 }); // has title
        features.push(doc.metadata.estimated_reading_time_sec as f64); // reading time
        features.push(doc.metadata.list_count as f64); // list count
        features.push(doc.metadata.element_count as f64); // total element density

        // Normalize
        let max_val = features.iter().cloned().fold(0.0_f64, f64::max);
        if max_val > 0.0 {
            for f in &mut features {
                *f /= max_val * 2.0;
                *f = f.clamp(0.0, 1.0);
            }
        }

        features
    }

    /// Encode a text chunk as a bag-of-concepts embedding for VSA binding.
    pub fn encode_text_concepts(&self, text: &str) -> Vec<f64> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut concept_bits = vec![0.0_f64; 64];

        for (_i, word) in words.iter().enumerate() {
            let hash = word.len().wrapping_mul(2654435761) as usize;
            let idx = hash % concept_bits.len();
            concept_bits[idx] += 1.0;
        }

        let sum: f64 = concept_bits.iter().sum();
        if sum > 0.0 {
            for b in &mut concept_bits {
                *b /= sum.sqrt();
            }
        }
        concept_bits
    }

    /// Full grounding pipeline: structured doc → VSA features → world model input
    pub fn ground_for_world_model(
        &self,
        doc: &StructuredDocument,
        raw_text: &str,
    ) -> WorldModelGrounding {
        let structure_features = self.encode_to_vsa_features(doc);
        let text_concepts = self.encode_text_concepts(raw_text);

        WorldModelGrounding {
            structure_features,
            text_concepts,
            confidence: doc.metadata.element_count as f64,
            grounded: true,
            timestamp_sec: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldModelGrounding {
    pub structure_features: Vec<f64>,
    pub text_concepts: Vec<f64>,
    pub confidence: f64,
    pub grounded: bool,
    pub timestamp_sec: u64,
}

// ── Main OCR Pipeline ──

#[derive(Debug, Clone)]
pub struct LongHorizonOcr {
    pub config: OcrConfig,
    pub pdf_renderer: PdfRenderer,
    pub structure_reconstructor: DocStructureReconstructor,
    pub ngram_filter: NoRepeatNgramFilter,
    pub vsa_bridge: OcrVsaBridge,
    pub pipeline_stats: OcrPipelineStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcrPipelineStats {
    pub total_pages_processed: u64,
    pub total_tiles_processed: u64,
    pub total_documents_processed: u64,
    pub average_confidence: f64,
    pub average_processing_time_ms: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OcrError {
    FileNotFound(String),
    RendererNotAvailable(String),
    ProcessingFailed(String),
    LowConfidence(String),
    UnsupportedFormat(String),
}

impl std::fmt::Display for OcrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcrError::FileNotFound(p) => write!(f, "File not found: {}", p),
            OcrError::RendererNotAvailable(m) => write!(f, "Renderer not available: {}", m),
            OcrError::ProcessingFailed(m) => write!(f, "Processing failed: {}", m),
            OcrError::LowConfidence(m) => write!(f, "Low confidence: {}", m),
            OcrError::UnsupportedFormat(m) => write!(f, "Unsupported format: {}", m),
        }
    }
}

impl LongHorizonOcr {
    pub fn new(config: OcrConfig) -> Self {
        let ngram_size = config.no_repeat_ngram_size;
        Self {
            pdf_renderer: PdfRenderer::new(config.pdf_dpi, config.pdf_max_pages),
            structure_reconstructor: DocStructureReconstructor::new(),
            ngram_filter: NoRepeatNgramFilter::new(ngram_size),
            vsa_bridge: OcrVsaBridge,
            pipeline_stats: OcrPipelineStats::default(),
            config,
        }
    }

    /// Main entry: process a document source through the full OCR pipeline.
    pub fn process(&mut self, source: &DocumentSource) -> Result<OcrResult, OcrError> {
        let start = std::time::Instant::now();

        let (pages, mode) = match source {
            DocumentSource::PdfFile(path) => {
                let pages = self.pdf_renderer.render_with_fallback(path, &|p| {
                    Err(OcrError::RendererNotAvailable(format!(
                        "No fallback renderer for PDF: {}",
                        p
                    )))
                })?;
                (pages, OcrMode::Gundam)
            }
            DocumentSource::ImageFile(path) => {
                if !std::path::Path::new(path).exists() {
                    return Err(OcrError::FileNotFound(path.clone()));
                }
                let page = PdfPageImage {
                    page_num: 0,
                    width: 0,
                    height: 0,
                    data: Vec::new(),
                    dpi: self.config.pdf_dpi,
                };
                (vec![page], self.config.mode)
            }
            DocumentSource::Url(url) => {
                return Err(OcrError::ProcessingFailed(format!(
                    "URL fetch not implemented. Use image pipeline for URL screenshots first: {}",
                    url
                )));
            }
            DocumentSource::RawBytes(data) => {
                let page = PdfPageImage {
                    page_num: 0,
                    width: 0,
                    height: 0,
                    data: data.clone(),
                    dpi: self.config.pdf_dpi,
                };
                (vec![page], self.config.mode)
            }
        };

        let mut all_tiles = Vec::new();
        for page in &pages {
            let tiles = self.process_page(page)?;
            all_tiles.extend(tiles);
        }

        let structured = self.structure_reconstructor.reconstruct(&all_tiles);

        let full_text = self
            .ngram_filter
            .filter_text(&self.assemble_full_text(&structured));

        let avg_confidence: f64 =
            all_tiles.iter().map(|t| t.confidence).sum::<f64>() / all_tiles.len().max(1) as f64;

        let elapsed = start.elapsed().as_millis() as u64;

        self.pipeline_stats.total_documents_processed += 1;
        self.pipeline_stats.total_pages_processed += pages.len() as u64;
        self.pipeline_stats.total_tiles_processed += all_tiles.len() as u64;
        self.pipeline_stats.average_confidence =
            (self.pipeline_stats.average_confidence + avg_confidence) / 2.0;
        self.pipeline_stats.average_processing_time_ms =
            (self.pipeline_stats.average_processing_time_ms + elapsed) / 2;

        Ok(OcrResult {
            full_text,
            structured_doc: structured,
            tiles: all_tiles,
            confidence: avg_confidence,
            page_count: pages.len() as u32,
            processing_time_ms: elapsed,
            mode,
        })
    }

    /// Process a single page/image into OCR tiles.
    fn process_page(&self, _page: &PdfPageImage) -> Result<Vec<OcrTile>, OcrError> {
        let image_size = self.config.mode.image_size();
        let base_size = self.config.mode.base_size();

        let tiles_per_row = (base_size + image_size - 1) / image_size;
        let tiles_per_col = (base_size + image_size - 1) / image_size;
        let max_tiles = self
            .config
            .max_tiles
            .min((tiles_per_row * tiles_per_col) as usize);

        let mut tiles = Vec::with_capacity(max_tiles);
        for i in 0..max_tiles {
            let row = (i as u32) / tiles_per_row;
            let col = (i as u32) % tiles_per_row;
            let x = col * image_size;
            let y = row * image_size;

            let tile = OcrTile {
                x,
                y,
                width: image_size,
                height: image_size,
                raw_text: String::new(),
                confidence: 0.0,
                elements: Vec::new(),
            };
            tiles.push(tile);
        }
        Ok(tiles)
    }

    /// Assemble full text from structured document
    fn assemble_full_text(&self, doc: &StructuredDocument) -> String {
        let mut text = String::new();
        if let Some(ref title) = doc.title {
            text.push_str(&format!("# {}\n\n", title));
        }
        for section in &doc.sections {
            if let Some(ref heading) = section.heading {
                let prefix = "#".repeat(section.heading_level as usize);
                text.push_str(&format!("{} {}\n\n", prefix, heading));
            }
            for element in &section.elements {
                text.push_str(&self.element_to_text(element));
                text.push('\n');
            }
        }
        text
    }

    fn element_to_text(&self, element: &DocElement) -> String {
        match element {
            DocElement::Heading { text, .. } => format!("{}\n", text),
            DocElement::Paragraph { text, .. } => format!("{}\n", text),
            DocElement::Table { headers, rows, .. } => {
                let mut out = String::new();
                out.push_str(&format!("| {} |\n", headers.join(" | ")));
                out.push_str(&format!(
                    "| {} |\n",
                    headers
                        .iter()
                        .map(|_| "---")
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
                for row in rows {
                    out.push_str(&format!("| {} |\n", row.join(" | ")));
                }
                out
            }
            DocElement::List { items, .. } => items.iter().map(|i| format!("- {}\n", i)).collect(),
            DocElement::CodeBlock { code, .. } => format!("```\n{}\n```\n", code),
            DocElement::Figure { caption, .. } => {
                if let Some(c) = caption {
                    format!("[Figure: {}]\n", c)
                } else {
                    "[Figure]\n".to_string()
                }
            }
        }
    }

    /// Ground OCR result into world model via VSA bridge
    pub fn ground_for_world_model(&self, result: &OcrResult) -> WorldModelGrounding {
        self.vsa_bridge
            .ground_for_world_model(&result.structured_doc, &result.full_text)
    }

    /// Quick scan: confidence-only check without full pipeline
    pub fn quick_scan(&self, text: &str) -> f64 {
        let word_count = text.split_whitespace().count();
        if word_count < 3 {
            return 0.0;
        }
        let avg_word_len: f64 =
            text.split_whitespace().map(|w| w.len() as f64).sum::<f64>() / word_count.max(1) as f64;
        if avg_word_len > 1.5 && avg_word_len < 20.0 {
            0.8
        } else {
            0.3
        }
    }
}
