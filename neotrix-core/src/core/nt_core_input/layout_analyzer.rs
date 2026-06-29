use super::column_layout;

/// Configuration for layout analysis post-processing.
#[derive(Debug, Clone)]
pub struct LayoutAnalyzerConfig {
    /// Enable layout analysis (default: true).
    pub enabled: bool,
    /// Correct multi-column read order (default: true).
    pub correct_read_order: bool,
    /// Detect semantic regions (headings, tables, formulas, etc.) (default: true).
    pub detect_regions: bool,
    /// If true, replaces original markdown with layout-corrected text (default: true).
    pub apply_corrections: bool,
}

impl Default for LayoutAnalyzerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            correct_read_order: true,
            detect_regions: true,
            apply_corrections: true,
        }
    }
}

/// Result of layout analysis on a document.
#[derive(Debug, Clone)]
pub struct LayoutAnalysis {
    /// Layout-corrected text (multi-column read order fixed).
    pub corrected_text: String,
    /// Number of columns detected.
    pub column_count: usize,
    /// Layout type description.
    pub layout_type: String,
    /// Detected semantic regions with their types and confidence.
    pub regions: Vec<RegionInfo>,
    /// Human-readable report of the analysis.
    pub report: String,
}

/// A detected region with its semantic type.
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub region_type: String,
    pub confidence: f32,
    pub text_snippet: String,
}

/// Post-processing layer for document layout analysis.
///
/// Wraps the heuristic layout analysis from `column_layout` and integrates
/// it into the document parsing pipeline. Corrects multi-column read order,
/// detects semantic regions (headings, tables, formulas, list items),
/// and enriches document metadata with layout information.
pub struct LayoutAnalyzer {
    config: LayoutAnalyzerConfig,
}

impl LayoutAnalyzer {
    pub fn new(config: LayoutAnalyzerConfig) -> Self {
        Self { config }
    }

    /// Analyze the layout of a document given its markdown text.
    ///
    /// Returns a `LayoutAnalysis` containing corrected text, detected regions,
    /// and a human-readable report. When `apply_corrections` is enabled, the
    /// `corrected_text` field contains the read-order-fixed version of the input.
    pub fn analyze(&self, text: &str) -> LayoutAnalysis {
        if !self.config.enabled {
            return LayoutAnalysis {
                corrected_text: text.to_string(),
                column_count: 1,
                layout_type: "unknown".to_string(),
                regions: vec![],
                report: "layout analysis disabled".to_string(),
            };
        }

        let page_num = 0;

        if self.config.correct_read_order {
            let (corrected_text, column_report) = column_layout::correct_read_order(text, page_num);

            if self.config.detect_regions {
                let segments = column_layout::segments_from_text(&corrected_text, page_num);
                let page_layout = column_layout::analyze_page_layout(&segments);
                let region_report = column_layout::layout_report(&page_layout);

                let regions: Vec<RegionInfo> = page_layout
                    .regions
                    .iter()
                    .map(|r| RegionInfo {
                        region_type: r.region_type.name().to_string(),
                        confidence: r.confidence,
                        text_snippet: r.text.chars().take(60).collect(),
                    })
                    .collect();

                let combined_report = format!("{}\n{}", column_report, region_report);

                let final_text = if self.config.apply_corrections {
                    corrected_text
                } else {
                    text.to_string()
                };

                return LayoutAnalysis {
                    corrected_text: final_text,
                    column_count: page_layout.columns.len(),
                    layout_type: format!("{:?}", page_layout.layout_type),
                    regions,
                    report: combined_report,
                };
            }

            let final_text = if self.config.apply_corrections {
                corrected_text
            } else {
                text.to_string()
            };

            LayoutAnalysis {
                corrected_text: final_text,
                column_count: column_layout::detect_columns(&column_layout::segments_from_text(
                    text, page_num,
                ))
                .column_count,
                layout_type: column_layout::detect_columns(&column_layout::segments_from_text(
                    text, page_num,
                ))
                .layout_type_name(),
                regions: vec![],
                report: column_report,
            }
        } else if self.config.detect_regions {
            let segments = column_layout::segments_from_text(text, page_num);
            let page_layout = column_layout::analyze_page_layout(&segments);
            let report = column_layout::layout_report(&page_layout);

            let regions: Vec<RegionInfo> = page_layout
                .regions
                .iter()
                .map(|r| RegionInfo {
                    region_type: r.region_type.name().to_string(),
                    confidence: r.confidence,
                    text_snippet: r.text.chars().take(60).collect(),
                })
                .collect();

            LayoutAnalysis {
                corrected_text: text.to_string(),
                column_count: page_layout.columns.len(),
                layout_type: format!("{:?}", page_layout.layout_type),
                regions,
                report,
            }
        } else {
            LayoutAnalysis {
                corrected_text: text.to_string(),
                column_count: 1,
                layout_type: "unknown".to_string(),
                regions: vec![],
                report: "layout analysis: all detectors disabled".to_string(),
            }
        }
    }

    /// Convenience: analyze and log the layout report.
    pub fn analyze_and_log(&self, text: &str, doc_label: &str) -> LayoutAnalysis {
        let result = self.analyze(text);
        log::info!(
            "[LAYOUT] {}: {} cols, {} regions, {}",
            doc_label,
            result.column_count,
            result.regions.len(),
            result.layout_type,
        );
        if !result.regions.is_empty() {
            log::debug!("[LAYOUT] Report for {}:\n{}", doc_label, result.report);
        }
        result
    }
}

impl Default for LayoutAnalyzer {
    fn default() -> Self {
        Self::new(LayoutAnalyzerConfig::default())
    }
}

// ---------------------------------------------------------------------------
// Helper: extend column_layout types with minor missing pieces
// ---------------------------------------------------------------------------

/// Extension: provide a display name for LayoutType.
trait LayoutTypeDisplay {
    fn layout_type_name(&self) -> String;
}

impl LayoutTypeDisplay for column_layout::ColumnLayout {
    fn layout_type_name(&self) -> String {
        format!("{:?}", self.layout_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_analyzer() {
        let config = LayoutAnalyzerConfig {
            enabled: false,
            ..Default::default()
        };
        let analyzer = LayoutAnalyzer::new(config);
        let result = analyzer.analyze("Hello world");
        assert_eq!(result.corrected_text, "Hello world");
        assert_eq!(result.column_count, 1);
    }

    #[test]
    fn test_single_column_passthrough() {
        let analyzer = LayoutAnalyzer::default();
        let result = analyzer.analyze("Line 1\nLine 2\nLine 3");
        assert!(result.corrected_text.contains("Line 1"));
        assert_eq!(result.column_count, 1);
    }

    #[test]
    fn test_empty_text() {
        let analyzer = LayoutAnalyzer::default();
        let result = analyzer.analyze("");
        assert_eq!(result.corrected_text, "");
        assert_eq!(result.regions.len(), 0);
    }

    #[test]
    fn test_no_corrections_mode() {
        let config = LayoutAnalyzerConfig {
            apply_corrections: false,
            ..Default::default()
        };
        let analyzer = LayoutAnalyzer::new(config);
        let input = "Some text";
        let result = analyzer.analyze(input);
        // Text unchanged in no-corrections mode
        assert_eq!(result.corrected_text, input);
        // But layout analysis still ran
        assert_eq!(result.column_count, 1);
    }

    #[test]
    fn test_region_detection_only() {
        let config = LayoutAnalyzerConfig {
            correct_read_order: false,
            detect_regions: true,
            ..Default::default()
        };
        let analyzer = LayoutAnalyzer::new(config);
        let result = analyzer.analyze("Hello world");
        assert_eq!(result.corrected_text, "Hello world");
        // Should have at least basic region detection
        assert!(result.column_count >= 1);
    }

    #[test]
    fn test_report_content() {
        let analyzer = LayoutAnalyzer::default();
        let result = analyzer.analyze("Hello\nWorld\nTest");
        assert!(!result.report.is_empty());
        assert!(result.report.contains("columns") || result.report.contains("disabled"));
    }
}
