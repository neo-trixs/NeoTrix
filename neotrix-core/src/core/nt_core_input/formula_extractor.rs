#![allow(dead_code)]

//! Formula extraction for the VLM Document Parsing Evolution (Phase 3.2).
//!
//! Detects LaTeX‑encoded math (inline and display) and MathML blocks
//! within document text.  Designed as a post‑processing step on
//! [`ParsedDocument`] after VLM or PDF extraction.

use regex::Regex;

use super::document_parser::{BBox, ParsedDocument};

// ---------------------------------------------------------------------------
// FormulaElement
// ---------------------------------------------------------------------------

/// A mathematical formula element extracted from a document.
#[derive(Debug, Clone, PartialEq)]
pub enum FormulaElement {
    /// Inline formula embedded in a paragraph.
    Inline { latex: String, bbox: Option<BBox> },
    /// Display (block‑level) formula set apart from the text flow.
    Display {
        latex: String,
        bbox: Option<BBox>,
        /// Equation number, extracted from `\tag{…}`, `\label{…}` or
        /// environment numbering if present.
        equation_number: Option<String>,
    },
    /// MathML‑encoded formula.
    MathML {
        mathml: String,
        /// LaTeX equivalent, if the MathML was converted inline.
        latex: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// FormulaExtractor
// ---------------------------------------------------------------------------

/// Extracts mathematical formulas from document text.
///
/// Detection is purely heuristic — it scans for well‑known delimiters
/// (`$`, `$$`, `\(` … `\)`, `\[` … `\]`, `\begin{env}…\end{env}`)
/// and `<math>…</math>` tags.  It does **not** validate the syntax of the
/// enclosed content.
#[derive(Clone)]
pub struct FormulaExtractor {
    max_formulas: usize,
    enable_latex_detection: bool,
    enable_mathml_detection: bool,
}

impl Default for FormulaExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl FormulaExtractor {
    /// Creates a new extractor with all detection enabled and a cap of
    /// 1024 formulas per call.
    pub fn new() -> Self {
        FormulaExtractor {
            max_formulas: 1024,
            enable_latex_detection: true,
            enable_mathml_detection: true,
        }
    }

    /// Creates an extractor with a custom per‑call limit.
    pub fn with_max(max_formulas: usize) -> Self {
        FormulaExtractor {
            max_formulas,
            ..Self::new()
        }
    }

    // ---------------------------------------------------------------
    // Public API
    // ---------------------------------------------------------------

    /// Extract formulas from raw document text.
    ///
    /// All detection strategies are attempted independently; results are
    /// deduplicated by the captured string and type.
    pub fn extract_from_text(&self, text: &str) -> Vec<FormulaElement> {
        let mut results: Vec<FormulaElement> = Vec::new();

        if self.enable_latex_detection {
            self.extract_dollar_display(text, &mut results);
            self.extract_dollar_inline(text, &mut results);
            self.extract_paren_inline(text, &mut results);
            self.extract_bracket_display(text, &mut results);
            self.extract_environment_display(text, &mut results);
        }

        if self.enable_mathml_detection {
            self.extract_mathml(text, &mut results);
        }

        // Deduplicate — same (type, content) pair is counted once.
        results.sort_by(|a, b| {
            let key_a = Self::dedup_key(a);
            let key_b = Self::dedup_key(b);
            key_a.cmp(&key_b)
        });
        results.dedup_by(|a, b| Self::dedup_key(a) == Self::dedup_key(b));

        if results.len() > self.max_formulas {
            results.truncate(self.max_formulas);
        }

        results
    }

    /// Extract formulas from a [`ParsedDocument`] — scans both its
    /// `markdown` body and the metadata fields that may contain math.
    pub fn extract_from_document(&self, doc: &ParsedDocument) -> Vec<FormulaElement> {
        let mut results = self.extract_from_text(&doc.markdown);

        // Title and author often contain no math, but check anyway.
        if let Some(ref title) = doc.metadata.title {
            results.extend(self.extract_from_text(title));
        }
        if let Some(ref author) = doc.metadata.author {
            results.extend(self.extract_from_text(author));
        }

        results.sort_by(|a, b| {
            let key_a = Self::dedup_key(a);
            let key_b = Self::dedup_key(b);
            key_a.cmp(&key_b)
        });
        results.dedup_by(|a, b| Self::dedup_key(a) == Self::dedup_key(b));

        results
    }

    /// Count formulas found in text (convenience, no alloc).
    pub fn count_formulas(text: &str) -> usize {
        let extractor = FormulaExtractor::new();
        extractor.extract_from_text(text).len()
    }

    /// Quick check whether text contains any math content.
    pub fn contains_math(text: &str) -> bool {
        Self::count_formulas(text) > 0
    }

    // ---------------------------------------------------------------
    // Internal — LaTeX
    // ---------------------------------------------------------------

    /// Display math with `$$…$$`.
    fn extract_dollar_display(&self, text: &str, out: &mut Vec<FormulaElement>) {
        let re = Regex::new(r"(?s)\$\$(.+?)\$\$").unwrap();
        for cap in re.captures_iter(text) {
            out.push(FormulaElement::Display {
                latex: cap[1].trim().to_string(),
                bbox: None,
                equation_number: Self::find_equation_tag(&cap[1]),
            });
        }
    }

    /// Inline math with `$…$` — runs **after** `$$…$$` ranges have been
    /// blanked out so that a bare `$` that belongs to a `$$` pair is not
    /// misinterpreted.
    fn extract_dollar_inline(&self, text: &str, out: &mut Vec<FormulaElement>) {
        // Blank $$…$$ ranges so $…$ regex does not see the interior.
        let re_display = Regex::new(r"(?s)\$\$(.+?)\$\$").unwrap();
        let cleaned = re_display.replace_all(text, |_: &regex::Captures| {
            " ".repeat(4) // $$ + content + $$
        });

        let re_inline = Regex::new(r"(?s)\$(.+?)\$").unwrap();
        for cap in re_inline.captures_iter(&cleaned) {
            let content = cap[1].trim();
            if content.is_empty() {
                continue;
            }
            out.push(FormulaElement::Inline {
                latex: content.to_string(),
                bbox: None,
            });
        }
    }

    /// Inline math with `\(…\)`.
    fn extract_paren_inline(&self, text: &str, out: &mut Vec<FormulaElement>) {
        let re = Regex::new(r"(?s)\\\((.+?)\\\)").unwrap();
        for cap in re.captures_iter(text) {
            out.push(FormulaElement::Inline {
                latex: cap[1].trim().to_string(),
                bbox: None,
            });
        }
    }

    /// Display math with `\[…\]`.
    fn extract_bracket_display(&self, text: &str, out: &mut Vec<FormulaElement>) {
        let re = Regex::new(r"(?s)\\\[(.+?)\\\]").unwrap();
        for cap in re.captures_iter(text) {
            out.push(FormulaElement::Display {
                latex: cap[1].trim().to_string(),
                bbox: None,
                equation_number: Self::find_equation_tag(&cap[1]),
            });
        }
    }

    /// Display math for `\begin{env}…\end{env}` environments.
    fn extract_environment_display(&self, text: &str, out: &mut Vec<FormulaElement>) {
        let envs = ["equation", "equation*", "align", "align*", "matrix"];
        for env in envs {
            let pattern = format!(r"(?s)\\begin\{{{env}\}}(.+?)\\end\{{{env}\}}");
            // Safe because env names are known‑good ASCII strings.
            let re = Regex::new(&pattern).unwrap();
            for cap in re.captures_iter(text) {
                out.push(FormulaElement::Display {
                    latex: cap[1].trim().to_string(),
                    bbox: None,
                    equation_number: Self::find_equation_tag(&cap[1]),
                });
            }
        }
    }

    /// Try to extract `\tag{…}` or `\label{…}` as an equation number.
    fn find_equation_tag(content: &str) -> Option<String> {
        // \tag{…}
        if let Some(cap) = Regex::new(r"\\tag\{([^}]*)\}")
            .ok()
            .and_then(|re| re.captures(content))
        {
            let tag = cap[1].trim().to_string();
            if !tag.is_empty() {
                return Some(tag);
            }
        }
        // \label{…}
        if let Some(cap) = Regex::new(r"\\label\{([^}]*)\}")
            .ok()
            .and_then(|re| re.captures(content))
        {
            let label = cap[1].trim().to_string();
            if !label.is_empty() {
                return Some(label);
            }
        }
        None
    }

    // ---------------------------------------------------------------
    // Internal — MathML
    // ---------------------------------------------------------------

    /// Detect `<math>…</math>` blocks.
    fn extract_mathml(&self, text: &str, out: &mut Vec<FormulaElement>) {
        let re = Regex::new(r"(?s)<math>(.+?)</math>").unwrap();
        for cap in re.captures_iter(text) {
            out.push(FormulaElement::MathML {
                mathml: cap[1].trim().to_string(),
                latex: None,
            });
        }

        // Also handle namespaced / annotated MathML.
        let re_ns = Regex::new(r"(?s)<math\s[^>]*>(.+?)</math>").unwrap();
        for cap in re_ns.captures_iter(text) {
            // Only add if the tag has attributes (the bare case was
            // handled above).
            let mathml = cap[1].trim().to_string();
            let already = out.iter().any(|e| match e {
                FormulaElement::MathML { mathml: m, .. } => m == &mathml,
                _ => false,
            });
            if !already {
                out.push(FormulaElement::MathML {
                    mathml,
                    latex: None,
                });
            }
        }
    }

    // ---------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------

    fn dedup_key(elem: &FormulaElement) -> (u8, String) {
        match elem {
            FormulaElement::Inline { latex, .. } => (0, latex.clone()),
            FormulaElement::Display { latex, .. } => (1, latex.clone()),
            FormulaElement::MathML { mathml, .. } => (2, mathml.clone()),
        }
    }
}

// ---------------------------------------------------------------------------
// Integration helper
// ---------------------------------------------------------------------------

/// Post‑process a [`ParsedDocument`] to enrich it with formula metadata.
///
/// Extracts all formulas from the document and records the count in
/// `metadata`.
///
/// **Prerequisite:** `DocumentMetadata` must have a `pub formula_count: usize`
/// field.  Add it in `document_parser.rs` alongside the existing metadata
/// fields if it is not already present.
pub fn enrich_document_with_formulas(
    extractor: &FormulaExtractor,
    doc: &mut ParsedDocument,
) -> Vec<FormulaElement> {
    let formulas = extractor.extract_from_document(doc);
    doc.metadata.formula_count = formulas.len();
    formulas
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Inline detection ------------------------------------------------

    #[test]
    fn test_single_dollar_inline() {
        let ext = FormulaExtractor::new();
        let results = ext.extract_from_text("Consider $f(x) = ax^2 + bx + c$.");
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::Inline { latex, bbox } => {
                assert_eq!(latex, "f(x) = ax^2 + bx + c");
                assert!(bbox.is_none());
            }
            other => panic!("expected Inline, got {other:?}"),
        }
    }

    #[test]
    fn test_double_dollar_display() {
        let ext = FormulaExtractor::new();
        let results = ext.extract_from_text(
            "We have $$\n\\int_{-\\infty}^{\\infty} e^{-x^2}\\,dx = \\sqrt{\\pi}\n$$",
        );
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::Display { latex, .. } => {
                assert!(latex.contains(r"\int_{-\infty}^{\infty}"));
                assert!(latex.contains(r"e^{-x^2}"));
            }
            other => panic!("expected Display, got {other:?}"),
        }
    }

    #[test]
    fn test_paren_inline() {
        let ext = FormulaExtractor::new();
        let results = ext.extract_from_text("The energy is \\(E = mc^2\\), which is famous.");
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::Inline { latex, .. } => {
                assert_eq!(latex, "E = mc^2");
            }
            other => panic!("expected Inline, got {other:?}"),
        }
    }

    #[test]
    fn test_bracket_display() {
        let ext = FormulaExtractor::new();
        let results =
            ext.extract_from_text("We solve \\[\n\\frac{d^2x}{dt^2} + \\omega^2 x = 0\n\\]");
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::Display { latex, .. } => {
                assert!(latex.contains(r"\frac{d^2x}{dt^2}"));
                assert!(latex.contains(r"\omega^2 x = 0"));
            }
            other => panic!("expected Display, got {other:?}"),
        }
    }

    #[test]
    fn test_begin_end_equation() {
        let ext = FormulaExtractor::new();
        let results = ext.extract_from_text(
            r"\begin{equation}
    E = h\nu
    \label{eq:planck}
\end{equation}",
        );
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::Display {
                latex,
                equation_number,
                ..
            } => {
                assert!(latex.contains("E = h\\nu") || latex.contains("E = h\\nu"));
                assert_eq!(equation_number.as_deref(), Some("eq:planck"));
            }
            other => panic!("expected Display, got {other:?}"),
        }
    }

    #[test]
    fn test_begin_end_align() {
        let ext = FormulaExtractor::new();
        let results = ext.extract_from_text(
            r"\begin{align}
    a &= b + c \\
    d &= e + f
\end{align}",
        );
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::Display { latex, .. } => {
                assert!(latex.contains("a &= b + c"));
                assert!(latex.contains("d &= e + f"));
            }
            other => panic!("expected Display, got {other:?}"),
        }
    }

    // ---- Multi & mixed detection -----------------------------------------

    #[test]
    fn test_multiple_formulas_in_one_text() {
        let ext = FormulaExtractor::new();
        let text = r"Let \(x\) be a real number.
\[
    x^2 \ge 0
\]
Consider the quadratic $ax^2 + bx + c = 0$.";
        let results = ext.extract_from_text(text);
        // \(x\), \[x^2 ≥ 0\], $ax^2 + bx + c = 0$  →  3 formulas
        assert!(results.len() >= 3, "expected ≥3, got {}", results.len());
    }

    // ---- MathML ----------------------------------------------------------

    #[test]
    fn test_mathml_detection() {
        let ext = FormulaExtractor::new();
        let results = ext.extract_from_text(
            r"Some text <math><msup><mi>x</mi><mn>2</mn></msup></math> more text.",
        );
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::MathML { mathml, .. } => {
                assert!(mathml.contains("<msup>"));
                assert!(mathml.contains("<mi>x</mi>"));
            }
            other => panic!("expected MathML, got {other:?}"),
        }
    }

    // ---- Plain text / edge cases -----------------------------------------

    #[test]
    fn test_no_math_in_plain_text() {
        let ext = FormulaExtractor::new();
        let results = ext.extract_from_text(
            "This is plain English text with no mathematical content whatsoever.",
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_contains_math_check() {
        assert!(FormulaExtractor::contains_math("$x+1$"));
        assert!(!FormulaExtractor::contains_math("just words"));
        assert!(FormulaExtractor::contains_math(r"\[E=mc^2\]"));
    }

    #[test]
    fn test_count_formulas() {
        let text = r"\(a\) and \(b\) and \[c\]";
        // Two \(…\) + one \[…\] = 3
        assert_eq!(FormulaExtractor::count_formulas(text), 3);
    }

    #[test]
    fn test_enrich_document_with_formulas() {
        let ext = FormulaExtractor::new();
        let mut doc = ParsedDocument {
            markdown: "The equation $E=mc^2$ is famous.".into(),
            tables: vec![],
            images: vec![],
            metadata: Default::default(),
        };
        let formulas = enrich_document_with_formulas(&ext, &mut doc);
        assert_eq!(formulas.len(), 1);
        // Requires `pub formula_count: usize` on DocumentMetadata.
        assert_eq!(doc.metadata.formula_count, 1);
        match &formulas[0] {
            FormulaElement::Inline { latex, .. } => {
                assert_eq!(latex, "E=mc^2");
            }
            other => panic!("expected Inline, got {other:?}"),
        }
    }

    // ---- Edge case: nested $ inside $$ -----------------------------------

    #[test]
    fn test_nested_dollars() {
        let ext = FormulaExtractor::new();
        // A display formula that contains a literal $ inside (rare but possible).
        let text = "We have $$\\text{cost} = \\$5.00$$ and also $x=1$.";
        let results = ext.extract_from_text(text);
        // One display + one inline = 2
        assert_eq!(results.len(), 2);
        // The display one should be the $$…$$
        let has_display = results
            .iter()
            .any(|e| matches!(e, FormulaElement::Display { .. }));
        assert!(has_display);
        let has_inline = results
            .iter()
            .any(|e| matches!(e, FormulaElement::Inline { latex, .. } if latex == "x=1"));
        assert!(has_inline);
    }

    // ---- Edge case: empty / degenerate content ---------------------------

    #[test]
    fn test_empty_math_content_is_skipped() {
        let ext = FormulaExtractor::new();
        let results = ext.extract_from_text("$$$$");
        // The inner content is empty → should it still emit?  Our regex
        // matches `.` zero times with `.+?`, so it should **not** match.
        // Actually `.+?` requires at least one character, so `$$$$` (two
        // empty $$ pairs) won't match.  Let's verify.
        assert!(results.is_empty());
    }

    #[test]
    fn test_detects_tag_as_equation_number() {
        let ext = FormulaExtractor::new();
        let text = r"\begin{equation}
    \int_0^1 x\,dx = \frac12
    \tag{1}
\end{equation}";
        let results = ext.extract_from_text(text);
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::Display {
                equation_number, ..
            } => {
                assert_eq!(equation_number.as_deref(), Some("1"));
            }
            other => panic!("expected Display, got {other:?}"),
        }
    }

    #[test]
    fn test_extract_from_document_includes_title() {
        let ext = FormulaExtractor::new();
        let mut doc = ParsedDocument {
            markdown: "Some body text.".into(),
            tables: vec![],
            images: vec![],
            metadata: Default::default(),
        };
        doc.metadata.title = Some("The $\\alpha$-decay model".into());

        let results = ext.extract_from_document(&doc);
        // One inline from title: $\alpha$
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormulaElement::Inline { latex, .. } => {
                assert_eq!(latex, r"\alpha");
            }
            other => panic!("expected Inline, got {other:?}"),
        }
    }
}
