//! Document Exporter — markdown-viewer/skills 多格式导出吸收
//! 
//! Markdown Viewer 渲染 PlantUML，支持 Chrome/Edge/Firefox/VS Code
//! 一键 Word 导出，PDF 导出

use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    Pdf,
    Word,      // .docx
    Html,
    Markdown,
    Png,
    Svg,
}

/// 导出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub output_path: PathBuf,
    pub title: Option<String>,
    pub include_toc: bool,
    pub page_size: PageSize,
    pub margins: Margins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSize {
    pub width_mm: f64,
    pub height_mm: f64,
}

impl Default for PageSize {
    fn default() -> Self {
        Self { width_mm: 210.0, height_mm: 297.0 } // A4
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top_mm: f64,
    pub bottom_mm: f64,
    pub left_mm: f64,
    pub right_mm: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self { top_mm: 25.0, bottom_mm: 25.0, left_mm: 25.0, right_mm: 25.0 }
    }
}

/// 导出结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub output_path: PathBuf,
    pub format: ExportFormat,
    pub file_size_bytes: u64,
    pub error: Option<String>,
}

/// 文档导出器
#[derive(Debug)]
pub struct DocumentExporter {
    plantuml_jar: Option<PathBuf>,
    pandoc_path: Option<PathBuf>,
}

impl DocumentExporter {
    pub fn new() -> Self {
        Self {
            plantuml_jar: Self::find_plantuml_jar(),
            pandoc_path: Self::find_pandoc(),
        }
    }

    fn find_plantuml_jar() -> Option<PathBuf> {
        // Common locations
        let paths = [
            "/usr/local/bin/plantuml.jar",
            "/opt/plantuml/plantuml.jar",
            "./plantuml.jar",
        ];
        for p in &paths {
            if std::path::Path::new(p).exists() {
                return Some(PathBuf::from(p));
            }
        }
        None
    }

    fn find_pandoc() -> Option<PathBuf> {
        let paths = ["/usr/bin/pandoc", "/usr/local/bin/pandoc", "/opt/homebrew/bin/pandoc"];
        for p in &paths {
            if std::path::Path::new(p).exists() {
                return Some(PathBuf::from(p));
            }
        }
        None
    }

    /// 导出 PlantUML 到指定格式
    pub fn export_plantuml(&self, plantuml: &str, config: ExportConfig) -> ExportResult {
        match config.format {
            ExportFormat::Pdf => self.export_to_pdf(plantuml, &config),
            ExportFormat::Word => self.export_to_word(plantuml, &config),
            ExportFormat::Html => self.export_to_html(plantuml, &config),
            ExportFormat::Markdown => self.export_to_markdown(plantuml, &config),
            ExportFormat::Png => self.export_to_png(plantuml, &config),
            ExportFormat::Svg => self.export_to_svg(plantuml, &config),
        }
    }

    fn export_to_pdf(&self, plantuml: &str, config: &ExportConfig) -> ExportResult {
        // Method 1: plantuml.jar -> SVG -> pandoc -> PDF
        if let Some(ref jar) = self.plantuml_jar {
            let temp_svg = config.output_path.with_extension("svg");
            let output = Command::new("java")
                .args(["-jar", jar.to_str().unwrap(), "-tsvg", "-pipe"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    child.stdin.as_mut().unwrap().write_all(plantuml.as_bytes()).ok();
                    child.wait_with_output()
                });
            
            if let Ok(out) = output {
                if out.status.success() {
                    std::fs::write(&temp_svg, &out.stdout).ok();
                    return self.svg_to_pdf(&temp_svg, config);
                }
            }
        }
        
        // Fallback: Use pandoc directly if it supports plantuml filter
        self.pandoc_export(plantuml, config, "pdf")
    }

    fn export_to_word(&self, plantuml: &str, config: &ExportConfig) -> ExportResult {
        // PlantUML -> SVG -> pandoc -> docx
        let temp_svg = config.output_path.with_extension("svg");
        
        if let Some(ref jar) = self.plantuml_jar {
            let output = Command::new("java")
                .args(["-jar", jar.to_str().unwrap(), "-tsvg", "-pipe"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    child.stdin.as_mut().unwrap().write_all(plantuml.as_bytes()).ok();
                    child.wait_with_output()
                });
            
            if let Ok(out) = output {
                if out.status.success() {
                    std::fs::write(&temp_svg, &out.stdout).ok();
                    return self.svg_to_docx(&temp_svg, config);
                }
            }
        }
        
        self.pandoc_export(plantuml, config, "docx")
    }

    fn export_to_html(&self, plantuml: &str, config: &ExportConfig) -> ExportResult {
        // PlantUML -> SVG -> embed in HTML
        if let Some(ref jar) = self.plantuml_jar {
            let output = Command::new("java")
                .args(["-jar", jar.to_str().unwrap(), "-tsvg", "-pipe"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    child.stdin.as_mut().unwrap().write_all(plantuml.as_bytes()).ok();
                    child.wait_with_output()
                });
            
            if let Ok(out) = output {
                if out.status.success() {
                    let svg = String::from_utf8_lossy(&out.stdout);
                    let html = Self::embed_svg_in_html(&svg, config.title.as_deref());
                    std::fs::write(&config.output_path, html).ok();
                    return ExportResult {
                        success: true,
                        output_path: config.output_path.clone(),
                        format: ExportFormat::Html,
                        file_size_bytes: std::fs::metadata(&config.output_path).map(|m| m.len()).unwrap_or(0),
                        error: None,
                    };
                }
            }
        }
        
        ExportResult { success: false, output_path: config.output_path.clone(), format: ExportFormat::Html, file_size_bytes: 0, error: Some("No plantuml.jar found".to_string()) }
    }

    fn export_to_markdown(&self, plantuml: &str, config: &ExportConfig) -> ExportResult {
        let mut md = String::new();
        if let Some(title) = &config.title {
            md.push_str(&format!("# {}\n\n", title));
        }
        if config.include_toc {
            md.push_str("## Table of Contents\n\n");
        }
        md.push_str("```plantuml\n");
        md.push_str(plantuml);
        md.push_str("\n```\n");
        
        std::fs::write(&config.output_path, md).ok();
        ExportResult {
            success: true,
            output_path: config.output_path.clone(),
            format: ExportFormat::Markdown,
            file_size_bytes: std::fs::metadata(&config.output_path).map(|m| m.len()).unwrap_or(0),
            error: None,
        }
    }

    fn export_to_png(&self, plantuml: &str, config: &ExportConfig) -> ExportResult {
        if let Some(ref jar) = self.plantuml_jar {
            let output = Command::new("java")
                .args(["-jar", jar.to_str().unwrap(), "-tpng", "-pipe"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    child.stdin.as_mut().unwrap().write_all(plantuml.as_bytes()).ok();
                    child.wait_with_output()
                });
            
            if let Ok(out) = output {
                if out.status.success() {
                    std::fs::write(&config.output_path, &out.stdout).ok();
                    return ExportResult {
                        success: true,
                        output_path: config.output_path.clone(),
                        format: ExportFormat::Png,
                        file_size_bytes: std::fs::metadata(&config.output_path).map(|m| m.len()).unwrap_or(0),
                        error: None,
                    };
                }
            }
        }
        
        ExportResult { success: false, output_path: config.output_path.clone(), format: ExportFormat::Png, file_size_bytes: 0, error: Some("plantuml.jar not found".to_string()) }
    }

    fn export_to_svg(&self, plantuml: &str, config: &ExportConfig) -> ExportResult {
        if let Some(ref jar) = self.plantuml_jar {
            let output = Command::new("java")
                .args(["-jar", jar.to_str().unwrap(), "-tsvg", "-pipe"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    child.stdin.as_mut().unwrap().write_all(plantuml.as_bytes()).ok();
                    child.wait_with_output()
                });
            
            if let Ok(out) = output {
                if out.status.success() {
                    std::fs::write(&config.output_path, &out.stdout).ok();
                    return ExportResult {
                        success: true,
                        output_path: config.output_path.clone(),
                        format: ExportFormat::Svg,
                        file_size_bytes: std::fs::metadata(&config.output_path).map(|m| m.len()).unwrap_or(0),
                        error: None,
                    };
                }
            }
        }
        
        ExportResult { success: false, output_path: config.output_path.clone(), format: ExportFormat::Svg, file_size_bytes: 0, error: Some("plantuml.jar not found".to_string()) }
    }

    fn svg_to_pdf(&self, svg_path: &PathBuf, config: &ExportConfig) -> ExportResult {
        if let Some(ref pandoc) = self.pandoc_path {
            let output = Command::new(pandoc)
                .args([
                    svg_path.to_str().unwrap(),
                    "-o", config.output_path.to_str().unwrap(),
                    "--pdf-engine=weasyprint",
                ])
                .output();
            
            if let Ok(out) = output {
                if out.status.success() {
                    return ExportResult {
                        success: true,
                        output_path: config.output_path.clone(),
                        format: ExportFormat::Pdf,
                        file_size_bytes: std::fs::metadata(&config.output_path).map(|m| m.len()).unwrap_or(0),
                        error: None,
                    };
                }
            }
        }
        
        ExportResult { success: false, output_path: config.output_path.clone(), format: ExportFormat::Pdf, file_size_bytes: 0, error: Some("pandoc or weasyprint not available".to_string()) }
    }

    fn svg_to_docx(&self, svg_path: &PathBuf, config: &ExportConfig) -> ExportResult {
        if let Some(ref pandoc) = self.pandoc_path {
            let output = Command::new(pandoc)
                .args([
                    svg_path.to_str().unwrap(),
                    "-o", config.output_path.to_str().unwrap(),
                    "-f", "html",
                    "-t", "docx",
                ])
                .output();
            
            if let Ok(out) = output {
                if out.status.success() {
                    return ExportResult {
                        success: true,
                        output_path: config.output_path.clone(),
                        format: ExportFormat::Word,
                        file_size_bytes: std::fs::metadata(&config.output_path).map(|m| m.len()).unwrap_or(0),
                        error: None,
                    };
                }
            }
        }
        
        ExportResult { success: false, output_path: config.output_path.clone(), format: ExportFormat::Word, file_size_bytes: 0, error: Some("pandoc not available".to_string()) }
    }

    fn pandoc_export(&self, plantuml: &str, config: &ExportConfig, format: &str) -> ExportResult {
        // Write plantuml to temp file, use pandoc with plantuml filter
        // Simplified: just note the limitation
        ExportResult {
            success: false,
            output_path: config.output_path.clone(),
            format: match format {
                "pdf" => ExportFormat::Pdf,
                "docx" => ExportFormat::Word,
                _ => ExportFormat::Html,
            },
            file_size_bytes: 0,
            error: Some("Requires plantuml.jar + pandoc for full export".to_string()),
        }
    }

    fn embed_svg_in_html(svg: &str, title: Option<&str>) -> String {
        let title = title.unwrap_or("PlantUML Diagram");
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .diagram {{ max-width: 100%; height: auto; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="diagram">{}</div>
</body>
</html>"#, title, title, svg)
    }

    /// 批量导出
    pub fn batch_export(&self, diagrams: Vec<(String, String)>, config: ExportConfig) -> Vec<ExportResult> {
        diagrams.into_iter()
            .map(|(name, plantuml)| {
                let mut cfg = config.clone();
                cfg.output_path = cfg.output_path.with_file_name(format!("{}.{}", name, config.format.to_string().to_lowercase()));
                self.export_plantuml(&plantuml, cfg)
            })
            .collect()
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let exporter = DocumentExporter::new();
        
        // Test 1: Markdown export (no external deps)
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let output = temp_dir.path().join("test.md");
        let config = ExportConfig {
            format: ExportFormat::Markdown,
            output_path: output.clone(),
            title: Some("Test".to_string()),
            include_toc: false,
            page_size: PageSize::default(),
            margins: Margins::default(),
        };
        
        let result = exporter.export_plantuml("@startuml\na --> b\n@enduml", config);
        assert!(result.success);
        assert!(output.exists());
        let content = std::fs::read_to_string(&output).map_err(|e| e.to_string())?;
        assert!(content.contains("```plantuml"));
        assert!(content.contains("a --> b"));
        
        // Test 2: Config serialization
        let config = ExportConfig {
            format: ExportFormat::Pdf,
            output_path: PathBuf::from("test.pdf"),
            title: Some("Test".to_string()),
            include_toc: true,
            page_size: PageSize::default(),
            margins: Margins::default(),
        };
        let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
        assert!(json.contains("Pdf"));
        
        Ok(())
    }
}

impl Default for DocumentExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporter_creation() {
        let exporter = DocumentExporter::new();
        // May or may not find plantuml/pandoc
    }

    #[test]
    fn test_markdown_export() {
        let exporter = DocumentExporter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let output = temp_dir.path().join("test.md");
        let config = ExportConfig {
            format: ExportFormat::Markdown,
            output_path: output.clone(),
            title: Some("Test".to_string()),
            include_toc: false,
            page_size: PageSize::default(),
            margins: Margins::default(),
        };
        let result = exporter.export_plantuml("@startuml\na --> b\n@enduml", config);
        assert!(result.success);
        assert!(output.exists());
    }

    #[test]
    fn test_self_test_passes() {
        assert!(DocumentExporter::self_test().is_ok());
    }
}