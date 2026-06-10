use regex::Regex;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DesignViolation {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub description: String,
    pub location: String,
    pub suggestion: String,
}

pub trait AntiPatternDetector: Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, content: &str) -> Vec<DesignViolation>;
}

// ═══════════════════════════════════════════════════════
// COLOR CHECK: 10 detectors
// ═══════════════════════════════════════════════════════

pub struct ColorAntiPatternDetector;

impl AntiPatternDetector for ColorAntiPatternDetector {
    fn name(&self) -> &str {
        "color_anti_pattern"
    }

    fn detect(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        violations.extend(self.check_gray_text_on_colored_bg(content));
        violations.extend(self.check_pure_black_gray(content));
        violations.extend(self.check_purple_gradient(content));
        violations.extend(self.check_dark_glow(content));
        violations.extend(self.check_cyan_teal_default(content));
        violations.extend(self.check_saturated_card_bg(content));
        violations.extend(self.check_missing_dark_mode(content));
        violations.extend(self.check_low_contrast_text(content));
        violations.extend(self.check_brand_color_overuse(content));
        violations.extend(self.check_rainbow_hover(content));
        violations
    }
}

impl ColorAntiPatternDetector {
    fn check_gray_text_on_colored_bg(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(
            r"(?i)(background(?:-color)?\s*:\s*#[0-9a-f]{6})\s*;[^}]*?color\s*:\s*(?:#(?:808080|888|999|aaa|bbb)|gray|grey|rgb\s*\(\s*12[0-9])\s*[^}]*\}"
        ).unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "color-gray-text-on-bg".into(),
                name: "gray-text-on-colored-bg".into(),
                severity: Severity::High,
                description: "Gray text on a colored background reduces readability, especially for users with low vision.".into(),
                location: m.as_str()[..m.as_str().len().min(60)].to_string(),
                suggestion: "Use white (#FFFFFF) or near-white text on colored backgrounds.".into(),
            });
        }
        violations
    }

    fn check_pure_black_gray(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)color\s*:\s*(?:rgb\s*\(\s*0\s*,\s*0\s*,\s*0\s*\)|#000(?:000)?|black)\s*[;}]").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "color-pure-black".into(),
                name: "pure-black-gray".into(),
                severity: Severity::Medium,
                description: "Pure black (#000) text on white creates harsh contrast that can cause eye strain. Use dark gray (#1A1A1A or similar) instead.".into(),
                location: m.as_str()[..m.as_str().len().min(40)].to_string(),
                suggestion: "Replace pure black (#000) with a softer dark color like #1A1A1A or #2D2D2D.".into(),
            });
        }
        violations
    }

    fn check_purple_gradient(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)(linear-gradient|radial-gradient|conic-gradient)\s*\([^)]*(?:purple|#800080|#9b59b6|#8e44ad)[^)]*\)").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "color-purple-gradient".into(),
                name: "purple-gradient".into(),
                severity: Severity::Low,
                description: "Purple gradients are frequently overused in startup/MVPs and can make a design feel generic or dated.".into(),
                location: m.as_str()[..m.as_str().len().min(50)].to_string(),
                suggestion: "Consider a more distinctive color palette. If purple fits your brand, use it sparingly and with subtle saturation.".into(),
            });
        }
        violations
    }

    fn check_dark_glow(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)text-shadow\s*:\s*[^;]*rgba?\s*\(.*?0\s*,\s*0\s*,\s*0\s*,\s*0\.\d\)").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "color-dark-glow".into(),
                name: "dark-glow".into(),
                severity: Severity::Low,
                description: "Dark glow effects (text-shadow with dark semi-transparent color) often look muddy and reduce readability.".into(),
                location: m.as_str()[..m.as_str().len().min(50)].to_string(),
                suggestion: "Use a subtle box-shadow or remove the glow effect entirely. If needed, use a light-colored glow.".into(),
            });
        }
        violations
    }

    fn check_cyan_teal_default(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)(?:color|background(?:-color)?)\s*:\s*(?:#00bcd4|#009688|#00acc1|#26c6da|cyan|teal)\s*[;}]").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "color-cyan-teal-default".into(),
                name: "cyan-teal-default".into(),
                severity: Severity::Info,
                description: "Cyan and teal are common default accent colors in UI frameworks (Material Design). Using them can make your app look generic.".into(),
                location: m.as_str()[..m.as_str().len().min(40)].to_string(),
                suggestion: "Choose a more distinctive brand color that differentiates your product.".into(),
            });
        }
        violations
    }

    fn check_saturated_card_bg(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)\.(?:card|panel|box|tile)\s*\{[^}]*background(?:-color)?\s*:\s*(?:#[0-9a-f]{3,6}|rgba?\s*\([^)]+\))\s*;").unwrap();
        for m in re.find_iter(content) {
            let bg = m.as_str();
            let hex_re = Regex::new(r"#([0-9a-f]{6})").unwrap();
            if let Some(caps) = hex_re.captures(bg) {
                if let Ok(val) = u32::from_str_radix(&caps[1], 16) {
                    let r = (val >> 16) & 0xFF;
                    let g = (val >> 8) & 0xFF;
                    let b = val & 0xFF;
                    let max_component = r.max(g).max(b) as f64;
                    let min_component = r.min(g).min(b) as f64;
                    let saturation = if max_component > 0.0 {
                        (max_component - min_component) / max_component
                    } else {
                        0.0
                    };
                    if saturation > 0.7 {
                        violations.push(DesignViolation {
                            id: "color-saturated-card-bg".into(),
                            name: "saturated-card-bg".into(),
                            severity: Severity::Medium,
                            description: "Highly saturated background colors on cards cause visual fatigue and can distract from content.".into(),
                            location: bg[..bg.len().min(50)].to_string(),
                            suggestion: "Use a desaturated/lightened version of your brand color for card backgrounds (saturation < 30%).".into(),
                        });
                    }
                }
            }
        }
        violations
    }

    fn check_missing_dark_mode(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        if content.contains("prefers-color-scheme")
            && !content.contains("@media (prefers-color-scheme: dark)")
            && !content.contains("@media(prefers-color-scheme:dark)")
        {
            violations.push(DesignViolation {
                id: "color-missing-dark-mode".into(),
                name: "missing-dark-mode".into(),
                severity: Severity::Medium,
                description: "No dark mode support detected. Dark mode is now standard in modern UI design.".into(),
                location: "file-level".into(),
                suggestion: "Add a `@media (prefers-color-scheme: dark)` block with appropriate dark theme colors.".into(),
            });
        }
        violations
    }

    fn check_low_contrast_text(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)color\s*:\s*(?:#(?:999|aaa|bbb|ccc|ddd)|rgba?\s*\(\s*(?:\d{1,2})\s*,\s*(?:\d{1,2})\s*,\s*(?:\d{1,2})\s*,\s*0\.\d[25]\s*\))").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "color-low-contrast".into(),
                name: "low-contrast-text".into(),
                severity: Severity::High,
                description: "Light gray text (#999 or lighter) fails WCAG AA contrast requirements (4.5:1 for normal text).".into(),
                location: m.as_str()[..m.as_str().len().min(40)].to_string(),
                suggestion: "Use #595959 or darker for body text to meet WCAG AA contrast ratio of 4.5:1.".into(),
            });
        }
        violations
    }

    fn check_brand_color_overuse(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let brand_colors = ["#007bff", "#0052cc", "#6f42c1", "#fd7e14", "#e83e8c"];
        let mut count = 0;
        for color in &brand_colors {
            let escaped = regex::escape(color);
            let re = Regex::new(&format!("(?i){}", escaped)).unwrap();
            count += re.find_iter(content).count();
        }
        if count > 5 {
            violations.push(DesignViolation {
                id: "color-brand-overuse".into(),
                name: "brand-color-overuse".into(),
                severity: Severity::Low,
                description: format!("Brand color detected {} times in source. Overusing brand colors reduces their impact.", count),
                location: "file-level".into(),
                suggestion: "Use brand colors as accents (10-20% of UI). Let neutral colors carry most of the design weight.".into(),
            });
        }
        violations
    }

    fn check_rainbow_hover(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i):hover\s*\{[^}]*(?:linear-gradient|background-image|filter)\s*:[^}]*\}").unwrap();
        for m in re.find_iter(content) {
            if m.as_str().to_lowercase().contains("gradient") {
                violations.push(DesignViolation {
                    id: "color-rainbow-hover".into(),
                    name: "rainbow-hover".into(),
                    severity: Severity::Low,
                    description: "Rainbow/gradient hover effects on interactive elements can feel gimmicky and distract users.".into(),
                    location: m.as_str()[..m.as_str().len().min(50)].to_string(),
                    suggestion: "Use subtle transforms or color shifts on hover instead of gradient transitions.".into(),
                });
            }
        }
        violations
    }
}

// ═══════════════════════════════════════════════════════
// TYPOGRAPHY CHECK: 6 detectors
// ═══════════════════════════════════════════════════════

pub struct TypographyAntiPatternDetector;

impl AntiPatternDetector for TypographyAntiPatternDetector {
    fn name(&self) -> &str {
        "typography_anti_pattern"
    }

    fn detect(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        violations.extend(self.check_inter_only_font(content));
        violations.extend(self.check_font_stack_unreasonable(content));
        violations.extend(self.check_giant_heading(content));
        violations.extend(self.check_thin_light_font(content));
        violations.extend(self.check_uppercase_body(content));
        violations.extend(self.check_line_height_too_tight(content));
        violations
    }
}

impl TypographyAntiPatternDetector {
    fn check_inter_only_font(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r#"(?i)font-family\s*:\s*['"]?Inter['"]?\s*[;}]"#).unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "typo-inter-only".into(),
                name: "inter-only-font".into(),
                severity: Severity::Low,
                description: "Inter font used without fallback stack. If Inter fails to load, the browser uses its default font.".into(),
                location: m.as_str()[..m.as_str().len().min(40)].to_string(),
                suggestion: "Use a proper font stack: `font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;`".into(),
            });
        }
        violations
    }

    fn check_font_stack_unreasonable(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r#"(?i)font-family\s*:\s*['"]([^'"]+)['"]\s*[;}]"#).unwrap();
        for m in re.find_iter(content) {
            let family = m.as_str();
            if !family.contains(',') && !family.contains("sans-serif") && !family.contains("serif") && !family.contains("monospace") {
                violations.push(DesignViolation {
                    id: "typo-font-stack-unreasonable".into(),
                    name: "font-stack-unreasonable".into(),
                    severity: Severity::Medium,
                    description: "Font stack without generic fallback family. If the custom font fails, the browser has no sensible fallback.".into(),
                    location: family[..family.len().min(40)].to_string(),
                    suggestion: "Add a generic fallback: `font-family: 'YourFont', sans-serif;`".into(),
                });
            }
        }
        violations
    }

    fn check_giant_heading(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)font-size\s*:\s*(?:7[2-9]|[89]\d|\d{3,})\s*px").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "typo-giant-heading".into(),
                name: "giant-heading".into(),
                severity: Severity::Info,
                description: "Heading font size exceeds 72px. Very large text can break responsive layouts and overwhelm content.".into(),
                location: m.as_str()[..m.as_str().len().min(30)].to_string(),
                suggestion: "Use responsive font sizing (clamp() or viewport units) and let headings scale naturally.".into(),
            });
        }
        violations
    }

    fn check_thin_light_font(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)font-weight\s*:\s*(?:100|200|300)\s*[;}]").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "typo-thin-light-font".into(),
                name: "thin-light-font".into(),
                severity: Severity::Medium,
                description: "Font weight 300 or lighter reduces legibility, especially on non-retina displays and for low-vision users.".into(),
                location: m.as_str()[..m.as_str().len().min(30)].to_string(),
                suggestion: "Use font-weight: 400 (regular) for body text. Reserve 300/200 for large display headings only.".into(),
            });
        }
        violations
    }

    fn check_uppercase_body(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)(?:body|p|\.text)\s*\{[^}]*text-transform\s*:\s*uppercase").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "typo-uppercase-body".into(),
                name: "uppercase-body".into(),
                severity: Severity::High,
                description: "Body text in uppercase significantly reduces readability, especially for longer passages.".into(),
                location: m.as_str()[..m.as_str().len().min(40)].to_string(),
                suggestion: "Use uppercase sparingly for short labels or headings only. Body text should be sentence case.".into(),
            });
        }
        violations
    }

    fn check_line_height_too_tight(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)line-height\s*:\s*(?:1(?:\.0)?|1\.1[0-9]?)\s*[;}]").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "typo-line-height-tight".into(),
                name: "line-height-too-tight".into(),
                severity: Severity::Medium,
                description: "Line-height of 1.15 or less reduces readability by making lines of text feel cramped.".into(),
                location: m.as_str()[..m.as_str().len().min(30)].to_string(),
                suggestion: "Use line-height: 1.5-1.7 for body text and 1.2-1.4 for headings.".into(),
            });
        }
        violations
    }
}

// ═══════════════════════════════════════════════════════
// LAYOUT CHECK: 9 detectors
// ═══════════════════════════════════════════════════════

pub struct LayoutAntiPatternDetector;

impl AntiPatternDetector for LayoutAntiPatternDetector {
    fn name(&self) -> &str {
        "layout_anti_pattern"
    }

    fn detect(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        violations.extend(self.check_card_inception(content));
        violations.extend(self.check_tight_padding(content));
        violations.extend(self.check_touch_target_too_small(content));
        violations.extend(self.check_missing_grid(content));
        violations.extend(self.check_absolute_position_overuse(content));
        violations.extend(self.check_fixed_header_without_skip(content));
        violations.extend(self.check_missing_hierarchy(content));
        violations.extend(self.check_inconsistent_border_radius(content));
        violations.extend(self.check_overflow_hidden_content(content));
        violations
    }
}

impl LayoutAntiPatternDetector {
    fn check_card_inception(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let card_count = content.matches(".card").count() + content.matches("card-").count()
            + content.matches("card__").count() + content.matches("card_").count();
        let nested = Regex::new(r"(?i)\.card\s*\{[^}]*\.card").unwrap();
        let match_count = nested.find_iter(content).count();
        if match_count > 0 || card_count > 4 {
            violations.push(DesignViolation {
                id: "layout-card-inception".into(),
                name: "card-inception".into(),
                severity: Severity::Medium,
                description: format!("{} card-related CSS classes found. Excessive card nesting ('card inception') wastes space and adds visual noise.", card_count),
                location: "file-level".into(),
                suggestion: "Limit card depth to 1 level. Use simpler container patterns like plain sections or lists.".into(),
            });
        }
        violations
    }

    fn check_tight_padding(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)padding\s*:\s*[012]px").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "layout-tight-padding".into(),
                name: "tight-padding".into(),
                severity: Severity::Low,
                description: "Padding of 2px or less inside containers causes text to feel cramped against edges.".into(),
                location: m.as_str()[..m.as_str().len().min(30)].to_string(),
                suggestion: "Use minimum 8px padding (16px preferred) inside containers for comfortable spacing.".into(),
            });
        }
        violations
    }

    fn check_touch_target_too_small(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)\.(?:btn|button|link|icon|close|nav-item|menu-item)\s*\{[^}]*(?:width|height)\s*:\s*(?:2[0-9]|1[0-9]|[0-9])\s*px").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "layout-touch-target-small".into(),
                name: "touch-target-too-small".into(),
                severity: Severity::High,
                description: "Touch target smaller than 44px violates WCAG 2.5.8 (Target Size minimum) and makes mobile interaction difficult.".into(),
                location: m.as_str()[..m.as_str().len().min(50)].to_string(),
                suggestion: "Ensure all interactive elements are at least 44x44px touch targets. Use padding to increase hit area.".into(),
            });
        }
        violations
    }

    fn check_missing_grid(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let has_grid = content.contains("display: grid") || content.contains("display:grid")
            || content.contains("display: flex") || content.contains("display:flex");
        if !has_grid && content.len() > 200 {
            violations.push(DesignViolation {
                id: "layout-missing-grid".into(),
                name: "missing-grid".into(),
                severity: Severity::Medium,
                description: "No CSS Grid or Flexbox layout system detected. Using older layout methods (float/inline-block) leads to fragile designs.".into(),
                location: "file-level".into(),
                suggestion: "Use CSS Grid for 2D layouts and Flexbox for 1D layouts. These are more maintainable and responsive.".into(),
            });
        }
        violations
    }

    fn check_absolute_position_overuse(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let count = Regex::new(r"(?i)position\s*:\s*absolute").unwrap().find_iter(content).count();
        if count > 3 {
            violations.push(DesignViolation {
                id: "layout-absolute-overuse".into(),
                name: "absolute-position-overuse".into(),
                severity: Severity::Medium,
                description: format!("{} elements use position: absolute. Overuse of absolute positioning creates fragile layouts that break on content changes.", count),
                location: "file-level".into(),
                suggestion: "Use Flexbox or Grid alignment instead of absolute positioning for most layout needs.".into(),
            });
        }
        violations
    }

    fn check_fixed_header_without_skip(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let has_fixed_header = content.contains("position: fixed")
            || content.contains("position:fixed");
        let has_skip_link = content.contains("skip-link")
            || content.contains("skipnav")
            || content.contains("skip to content")
            || content.contains("skip-to-content");
        if has_fixed_header && !has_skip_link {
            violations.push(DesignViolation {
                id: "layout-fixed-header-no-skip".into(),
                name: "fixed-header-without-skip".into(),
                severity: Severity::High,
                description: "Fixed header detected without a skip-to-content link. Keyboard users must tab through the entire header on every page.".into(),
                location: "file-level".into(),
                suggestion: "Add a 'Skip to content' link as the first focusable element on the page.".into(),
            });
        }
        violations
    }

    fn check_missing_hierarchy(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let has_h1 = Regex::new(r"(?is)<h1[>\s]").unwrap().is_match(content);
        let has_h2 = Regex::new(r"(?is)<h2[>\s]").unwrap().is_match(content);
        if has_h2 && !has_h1 {
            violations.push(DesignViolation {
                id: "layout-missing-hierarchy".into(),
                name: "missing-hierarchy".into(),
                severity: Severity::High,
                description: "HTML has <h2> elements but no <h1>. This breaks document outline hierarchy and harms accessibility.".into(),
                location: "heading-levels".into(),
                suggestion: "Always start with an <h1> as the page title, then use <h2> for sections, <h3> for subsections, etc.".into(),
            });
        }
        violations
    }

    fn check_inconsistent_border_radius(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let radii: Vec<f64> = Regex::new(r"(?i)border-radius\s*:\s*(\d+)px")
            .unwrap()
            .captures_iter(content)
            .filter_map(|c| c[1].parse::<f64>().ok())
            .collect();
        if radii.len() >= 3 {
            let unique: std::collections::HashSet<u64> = radii.iter().map(|r| *r as u64).collect();
            if unique.len() as f64 > radii.len() as f64 * 0.6 {
                violations.push(DesignViolation {
                    id: "layout-inconsistent-radius".into(),
                    name: "inconsistent-border-radius".into(),
                    severity: Severity::Low,
                    description: format!("{} different border-radius values detected. Inconsistent rounding looks unpolished.", unique.len()),
                    location: "file-level".into(),
                    suggestion: "Define 2-3 standard border-radius values (e.g., --radius-sm: 4px, --radius-md: 8px, --radius-lg: 16px) and reuse them.".into(),
                });
            }
        }
        violations
    }

    fn check_overflow_hidden_content(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)(?:\.content|\.main|\.container|\.article|\.text)\s*\{[^}]*overflow\s*:\s*hidden").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "layout-overflow-hidden".into(),
                name: "overflow-hidden-content".into(),
                severity: Severity::High,
                description: "Content container with overflow: hidden may clip text or content when the user zooms in or when dynamic content overflows.".into(),
                location: m.as_str()[..m.as_str().len().min(50)].to_string(),
                suggestion: "Use `overflow: auto` or `overflow: clip` instead, and test behavior with zoomed content.".into(),
            });
        }
        violations
    }
}

// ═══════════════════════════════════════════════════════
// MOTION CHECK: 5 detectors
// ═══════════════════════════════════════════════════════

pub struct MotionAntiPatternDetector;

impl AntiPatternDetector for MotionAntiPatternDetector {
    fn name(&self) -> &str {
        "motion_anti_pattern"
    }

    fn detect(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        violations.extend(self.check_bounce_easing_only(content));
        violations.extend(self.check_no_reduced_motion(content));
        violations.extend(self.check_parallax_overuse(content));
        violations.extend(self.check_auto_play_video(content));
        violations.extend(self.check_infinite_animation(content));
        violations
    }
}

impl MotionAntiPatternDetector {
    fn check_bounce_easing_only(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let has_bounce = content.contains("cubic-bezier(0.68, -0.55,")
            || content.contains("cubic-bezier(0.175, 0.885,")
            || content.contains("cubic-bezier(.68,-0.55,");
        let has_standard = content.contains("cubic-bezier(0.4, 0, 0.2, 1)")
            || content.contains("cubic-bezier(0.0, 0, 0.2, 1)")
            || content.contains("ease-in-out")
            || content.contains("ease-out");
        if has_bounce && !has_standard && content.len() > 100 {
            violations.push(DesignViolation {
                id: "motion-bounce-easing-only".into(),
                name: "bounce-easing-only".into(),
                severity: Severity::Medium,
                description: "Only bounce-easing (overshoot) curves found. Overusing bounce animations can feel distracting and unprofessional.".into(),
                location: "file-level".into(),
                suggestion: "Use standard easing (ease-in-out, ease-out) for most transitions. Reserve bounce easing for playful accent animations.".into(),
            });
        }
        violations
    }

    fn check_no_reduced_motion(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        if content.contains("animation") || content.contains("transition") {
            if !content.contains("prefers-reduced-motion") {
                violations.push(DesignViolation {
                    id: "motion-no-reduced-motion".into(),
                    name: "no-reduced-motion".into(),
                    severity: Severity::High,
                    description: "Animations detected without a `prefers-reduced-motion` media query. Users with vestibular disorders may experience discomfort.".into(),
                    location: "file-level".into(),
                    suggestion: "Add `@media (prefers-reduced-motion: reduce) { * { animation-duration: 0.01ms !important; } }` to respect user preferences.".into(),
                });
            }
        }
        violations
    }

    fn check_parallax_overuse(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)transform\s*:\s*[^}]*translate[Z3]?[^}]*\d+[^}]*\)").unwrap();
        let parallax_hints = ["parallax", "scroll-effect", "data-speed", "data-depth", "will-change: transform"];
        let has_parallax = parallax_hints.iter().any(|h| content.contains(h));
        let z_transforms = re.find_iter(content).count();
        if has_parallax || z_transforms > 2 {
            violations.push(DesignViolation {
                id: "motion-parallax-overuse".into(),
                name: "parallax-overuse".into(),
                severity: Severity::Medium,
                description: "Parallax scrolling effects detected. These can cause motion sickness, hurt performance, and break scroll-based interactions.".into(),
                location: "file-level".into(),
                suggestion: "Use parallax sparingly as a subtle accent. Always test with prefers-reduced-motion and ensure content is fully accessible without it.".into(),
            });
        }
        violations
    }

    fn check_auto_play_video(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?is)<video[^>]*autoplay[^>]*>").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "motion-auto-play-video".into(),
                name: "auto-play-video".into(),
                severity: Severity::High,
                description: "Auto-playing video can distract users, consume data, and cause accessibility issues.".into(),
                location: m.as_str()[..m.as_str().len().min(60)].to_string(),
                suggestion: "Use a click-to-play placeholder. If autoplay is necessary, ensure it has no audio and provide a pause control.".into(),
            });
        }
        violations
    }

    fn check_infinite_animation(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i)animation(?:-iteration-count)?\s*:\s*infinite").unwrap();
        for m in re.find_iter(content) {
            let before = &content[..m.start()];
            let is_loading = before.contains("spinner") || before.contains("loading")
                || before.contains("skeleton") || before.contains("pulse");
            if !is_loading {
                violations.push(DesignViolation {
                    id: "motion-infinite-animation".into(),
                    name: "infinite-animation".into(),
                    severity: Severity::Medium,
                    description: "Infinite (looping) non-loading animation detected. Endless motion is distracting and can cause discomfort.".into(),
                    location: m.as_str()[..m.as_str().len().min(40)].to_string(),
                    suggestion: "Limit animation loops to 2-3 iterations or remove infinite animation for non-loading elements.".into(),
                });
            }
        }
        violations
    }
}

// ═══════════════════════════════════════════════════════
// ACCESSIBILITY CHECK: 11 detectors
// ═══════════════════════════════════════════════════════

pub struct AccessibilityAntiPatternDetector;

impl AntiPatternDetector for AccessibilityAntiPatternDetector {
    fn name(&self) -> &str {
        "accessibility_anti_pattern"
    }

    fn detect(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        violations.extend(self.check_skipped_heading_level(content));
        violations.extend(self.check_missing_alt_text(content));
        violations.extend(self.check_button_without_aria(content));
        violations.extend(self.check_focus_visible_removed(content));
        violations.extend(self.check_color_only_indicator(content));
        violations.extend(self.check_missing_skip_link(content));
        violations.extend(self.check_small_tap_target(content));
        violations.extend(self.check_auto_play_audio(content));
        violations.extend(self.check_missing_lang_attr(content));
        violations.extend(self.check_form_without_label(content));
        violations.extend(self.check_keyboard_trap(content));
        violations
    }
}

impl AccessibilityAntiPatternDetector {
    fn check_skipped_heading_level(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?is)<(h[1-6])[>\s]").unwrap();
        let levels: Vec<u32> = re.captures_iter(content)
            .filter_map(|c| c[1][1..].parse::<u32>().ok())
            .collect();
        for i in 1..levels.len() {
            if levels[i] > levels[i-1] + 1 {
                violations.push(DesignViolation {
                    id: "a11y-skipped-heading".into(),
                    name: "skipped-heading-level".into(),
                    severity: Severity::High,
                    description: format!("Heading level jumps from h{} to h{}, skipping h{}. This breaks screen reader navigation.", levels[i-1], levels[i], levels[i-1]+1),
                    location: format!("h{} → h{}", levels[i-1], levels[i]),
                    suggestion: "Never skip heading levels. If you need a visually smaller heading, use CSS instead of jumping to a lower level.".into(),
                });
                break;
            }
        }
        violations
    }

    fn check_missing_alt_text(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r#"(?is)<img[^>]*src\s*=\s*['"][^'"]+['"][^>]*>"#).unwrap();
        for m in re.find_iter(content) {
            let tag = m.as_str();
            if !tag.contains("alt=") {
                violations.push(DesignViolation {
                    id: "a11y-missing-alt".into(),
                    name: "missing-alt-text".into(),
                    severity: Severity::Critical,
                    description: "Image without alt attribute. Screen readers cannot convey the image content to visually impaired users.".into(),
                    location: tag[..tag.len().min(60)].to_string(),
                    suggestion: "Add an alt attribute describing the image content, or `alt=\"\"` for decorative images.".into(),
                });
            }
        }
        violations
    }

    fn check_button_without_aria(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?is)<button[^>]*>[\s]*(?:<[^>]*>[\s]*)*</button>").unwrap();
        for m in re.find_iter(content) {
            let tag = m.as_str();
            let has_label = tag.contains("aria-label") || tag.contains("aria-labelledby");
            let inner_text = Regex::new(r"(?is)>([^<]+)<").unwrap();
            let has_text = inner_text.captures(tag).map_or(false, |c| {
                let t = c[1].trim();
                !t.is_empty()
            });
            if !has_label && !has_text {
                violations.push(DesignViolation {
                    id: "a11y-button-no-aria".into(),
                    name: "button-without-aria".into(),
                    severity: Severity::High,
                    description: "Icon-only button without aria-label. Screen readers cannot determine the button's purpose.".into(),
                    location: tag[..tag.len().min(60)].to_string(),
                    suggestion: "Add `aria-label=\"Describe action\"` to icon-only buttons.".into(),
                });
            }
        }
        violations
    }

    fn check_focus_visible_removed(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?i):focus\s*\{[^}]*outline\s*:\s*none").unwrap();
        for m in re.find_iter(content) {
            let has_focus_visible = content.contains(":focus-visible");
            if !has_focus_visible {
                violations.push(DesignViolation {
                    id: "a11y-focus-visible-removed".into(),
                    name: "focus-visible-removed".into(),
                    severity: Severity::Critical,
                    description: "`:focus { outline: none }` without `:focus-visible` fallback. Keyboard users lose visual focus indication.".into(),
                    location: m.as_str()[..m.as_str().len().min(50)].to_string(),
                    suggestion: "Replace with `:focus-visible { outline: 2px solid blue; outline-offset: 2px; }` and `:focus { outline: none; }` only alongside it.".into(),
                });
            }
        }
        violations
    }

    fn check_color_only_indicator(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let patterns = [
            r"(?i)(?:error|success|warning|info|valid|invalid)\s*\{[^}]*(?:color|background(?:-color)?)\s*:\s*[^;]+;",
            r"(?i)\.(?:status|badge|tag|pill|indicator|dot)\s*\{[^}]*(?:color|background(?:-color)?)\s*:\s*[^;]+;(?![^}]*content)",
        ];
        for pat in &patterns {
            let re = Regex::new(pat).unwrap();
            for m in re.find_iter(content) {
                violations.push(DesignViolation {
                    id: "a11y-color-only".into(),
                    name: "color-only-indicator".into(),
                    severity: Severity::High,
                    description: "Status indicator using color alone. Color-blind users cannot distinguish the state.".into(),
                    location: m.as_str()[..m.as_str().len().min(60)].to_string(),
                    suggestion: "Add an icon, text label, or pattern alongside the color to convey the state to all users.".into(),
                });
            }
        }
        violations
    }

    fn check_missing_skip_link(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let has_skip = content.contains("skip")
            && (content.contains("skip-link") || content.contains("skip_to_content")
                || content.contains("skip-to-content") || content.contains("skipnav")
                || content.contains("skipNav"));
        let has_nav = content.contains("<nav") || content.contains("role=\"navigation\"")
            || content.contains("role='navigation'");
        if has_nav && !has_skip {
            violations.push(DesignViolation {
                id: "a11y-missing-skip-link".into(),
                name: "missing-skip-link".into(),
                severity: Severity::High,
                description: "Navigation present without a skip-to-content link. Keyboard users must tab through all navigation items.".into(),
                location: "file-level".into(),
                suggestion: "Add a skip link as the first focusable element: `<a href=\"#main\" class=\"skip-link\">Skip to content</a>`".into(),
            });
        }
        violations
    }

    fn check_small_tap_target(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r#"(?is)<(?:button|a)[^>]*style\s*=\s*['"][^'"]*(?:width|height)\s*:\s*(?:2[0-9]|1[0-9]|[0-9])\s*px"#).unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "a11y-small-tap-target".into(),
                name: "small-tap-target".into(),
                severity: Severity::High,
                description: "Interactive element with inline style dimension < 30px. Fails WCAG 2.5.8 target size requirements.".into(),
                location: m.as_str()[..m.as_str().len().min(60)].to_string(),
                suggestion: "Ensure touch targets are at least 44x44px with accessible spacing between them.".into(),
            });
        }
        violations
    }

    fn check_auto_play_audio(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?is)<audio[^>]*autoplay[^>]*>").unwrap();
        for m in re.find_iter(content) {
            violations.push(DesignViolation {
                id: "a11y-auto-play-audio".into(),
                name: "auto-play-audio".into(),
                severity: Severity::Critical,
                description: "Auto-playing audio can be disorienting for screen reader users, as it masks the screen reader output.".into(),
                location: m.as_str()[..m.as_str().len().min(60)].to_string(),
                suggestion: "Never autoplay audio. Use a play button that users must explicitly click.".into(),
            });
        }
        violations
    }

    fn check_missing_lang_attr(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let re = Regex::new(r"(?is)<html[^>]*>").unwrap();
        for m in re.find_iter(content) {
            if !m.as_str().contains("lang=") {
                violations.push(DesignViolation {
                    id: "a11y-missing-lang".into(),
                    name: "missing-lang-attr".into(),
                    severity: Severity::Critical,
                    description: "<html> element missing lang attribute. Screen readers cannot determine the page language.".into(),
                    location: m.as_str()[..m.as_str().len().min(40)].to_string(),
                    suggestion: "Add `lang=\"en\"` (or appropriate language code) to the <html> tag.".into(),
                });
            }
        }
        violations
    }

    fn check_form_without_label(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let input_re = Regex::new(r"(?is)<(?:input|textarea|select)[^>]*>").unwrap();
        for m in input_re.find_iter(content) {
            let tag = m.as_str();
            if tag.contains("type=\"hidden\"") || tag.contains("type='hidden'") {
                continue;
            }
            let has_aria = tag.contains("aria-label") || tag.contains("aria-labelledby");
            let id_re = Regex::new(r#"id\s*=\s*['"]([^'"]+)['"]"#).unwrap();
            let has_label_for = if let Some(caps) = id_re.captures(tag) {
                let id = &caps[1];
                let label_re = Regex::new(&format!("(?is)<label[^>]*for\\s*=\\s*['\"]{}['\"][^>]*>", regex::escape(id))).unwrap();
                label_re.is_match(content)
            } else {
                false
            };
            let has_wrapping = Regex::new(r"(?is)<label[^>]*>[\s\S]*?<(?:input|textarea|select)").unwrap().is_match(content);
            if !has_aria && !has_label_for && !has_wrapping {
                violations.push(DesignViolation {
                    id: "a11y-form-no-label".into(),
                    name: "form-without-label".into(),
                    severity: Severity::Critical,
                    description: "Form control without an associated label. Screen reader users cannot identify the input's purpose.".into(),
                    location: tag[..tag.len().min(60)].to_string(),
                    suggestion: "Wrap the input in a `<label>` element or use `for=\"id\"` matching the input's `id` attribute.".into(),
                });
            }
        }
        violations
    }

    fn check_keyboard_trap(&self, content: &str) -> Vec<DesignViolation> {
        let mut violations = Vec::new();
        let trap_indicators = [
            "onblur=\"focus", "onblur='focus",
            "onkeydown=\"return false", "onkeydown='return false",
            "onkeydown=\"event.preventDefault", "onkeydown='event.preventDefault",
        ];
        let has_trap = trap_indicators.iter().any(|t| content.contains(t));
        if has_trap {
            violations.push(DesignViolation {
                id: "a11y-keyboard-trap".into(),
                name: "keyboard-trap".into(),
                severity: Severity::Critical,
                description: "Keyboard trap detected — script prevents leaving an element via keyboard. This makes navigation impossible for keyboard-only users.".into(),
                location: "at event handler".into(),
                suggestion: "Ensure users can navigate away from any element using Tab/Escape. Use focus-trap libraries with Escape-to-close patterns.".into(),
            });
        }
        violations
    }
}

// ═══════════════════════════════════════════════════════
// INNER CRITIC — orchestrates all detectors
// ═══════════════════════════════════════════════════════

pub struct InnerCritic {
    detectors: Vec<Box<dyn AntiPatternDetector>>,
}

impl Default for InnerCritic {
    fn default() -> Self {
        Self::new()
    }
}

impl InnerCritic {
    pub fn new() -> Self {
        let detectors: Vec<Box<dyn AntiPatternDetector>> = vec![
            Box::new(ColorAntiPatternDetector),
            Box::new(TypographyAntiPatternDetector),
            Box::new(LayoutAntiPatternDetector),
            Box::new(MotionAntiPatternDetector),
            Box::new(AccessibilityAntiPatternDetector),
        ];
        Self { detectors }
    }

    pub fn audit(&self, content: &str) -> Vec<DesignViolation> {
        let mut all = Vec::new();
        for detector in &self.detectors {
            all.extend(detector.detect(content));
        }
        all.sort_by(|a, b| b.severity.cmp(&a.severity));
        all
    }

    pub fn detector_count(&self) -> usize {
        self.detectors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_detectors_loaded() {
        let critic = InnerCritic::new();
        assert_eq!(critic.detector_count(), 5);
    }

    #[test]
    fn test_empty_content_no_violations() {
        let critic = InnerCritic::new();
        let violations = critic.audit("");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_gray_text_on_colored_bg() {
        let critic = InnerCritic::new();
        let content = r#".header { background-color: #1a73e8; color: #999; }"#;
        let violations = critic.audit(content);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.name == "gray-text-on-colored-bg"));
    }

    #[test]
    fn test_pure_black_detected() {
        let critic = InnerCritic::new();
        let content = r#"body { color: #000000; font-size: 16px; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "pure-black-gray"));
    }

    #[test]
    fn test_giant_heading_detected() {
        let critic = InnerCritic::new();
        let content = r#".hero-title { font-size: 96px; font-weight: 700; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "giant-heading"));
    }

    #[test]
    fn test_thin_font_detected() {
        let critic = InnerCritic::new();
        let content = r#"p { font-weight: 200; color: #333; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "thin-light-font"));
    }

    #[test]
    fn test_missing_alt_text_detected() {
        let critic = InnerCritic::new();
        let content = r#"<img src="photo.jpg">"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "missing-alt-text"));
    }

    #[test]
    fn test_image_with_alt_passes() {
        let critic = InnerCritic::new();
        let content = r#"<img src="photo.jpg" alt="A sunny landscape">"#;
        let violations = critic.audit(content);
        assert!(violations.iter().all(|v| v.name != "missing-alt-text"));
    }

    #[test]
    fn test_missing_lang_attr_detected() {
        let critic = InnerCritic::new();
        let content = r#"<html><head><title>Test</title></head><body></body></html>"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "missing-lang-attr"));
    }

    #[test]
    fn test_auto_play_video_detected() {
        let critic = InnerCritic::new();
        let content = r#"<video autoplay muted><source src="vid.mp4"></video>"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "auto-play-video"));
    }

    #[test]
    fn test_skipped_heading_level_detected() {
        let critic = InnerCritic::new();
        let content = r#"<h1>Title</h1><h3>Section</h3>"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "skipped-heading-level"));
    }

    #[test]
    fn test_fixed_header_without_skip_detected() {
        let critic = InnerCritic::new();
        let content = r#"
            .nav { position: fixed; top: 0; width: 100%; }
            .nav a { padding: 10px; }
        "#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "fixed-header-without-skip"));
    }

    #[test]
    fn test_inter_only_font_detected() {
        let critic = InnerCritic::new();
        let content = r#"body { font-family: 'Inter'; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "inter-only-font"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Info), "info");
        assert_eq!(format!("{}", Severity::Critical), "critical");
    }

    #[test]
    fn test_design_violation_clone() {
        let v = DesignViolation {
            id: "test".into(),
            name: "test-name".into(),
            severity: Severity::Medium,
            description: "desc".into(),
            location: "loc".into(),
            suggestion: "sugg".into(),
        };
        let v2 = v.clone();
        assert_eq!(v.id, v2.id);
        assert_eq!(v.severity, v2.severity);
    }

    #[test]
    fn test_uppercase_body_detected() {
        let critic = InnerCritic::new();
        let content = r#"body { text-transform: uppercase; font-size: 14px; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "uppercase-body"));
    }

    #[test]
    fn test_focus_visible_removed_detected() {
        let critic = InnerCritic::new();
        let content = r#"button:focus { outline: none; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "focus-visible-removed"));
    }

    #[test]
    fn test_form_without_label_detected() {
        let critic = InnerCritic::new();
        let content = r#"<input type="text" name="email">"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "form-without-label"));
    }

    #[test]
    fn test_missing_grid_detected() {
        let critic = InnerCritic::new();
        let content = "a very long CSS file with no grid or flexbox mentioned whatsoever ".repeat(50);
        let violations = critic.audit(&content);
        assert!(violations.iter().any(|v| v.name == "missing-grid"));
    }

    #[test]
    fn test_no_false_positives_on_clean_html() {
        let critic = InnerCritic::new();
        let content = r##"<!DOCTYPE html><html lang="en"><head><title>Clean</title></head><body><h1>Title</h1><p>Clean text</p><img src="a.jpg" alt="A"><a href="#">link</a></body></html>"##;
        let violations = critic.audit(content);
        let critical: Vec<&DesignViolation> = violations.iter().filter(|v| v.severity >= Severity::High).collect();
        assert!(critical.is_empty(), "Clean HTML should not produce high/critical violations: {:?}", critical);
    }

    #[test]
    fn test_audit_sorts_by_severity() {
        let critic = InnerCritic::new();
        let content = concat!(
            r#"<img src="x.jpg">"#,
            r#".card { border-radius: 4px; }"#,
            r#"<html><title>T</title></html>"#,
        );
        let violations = critic.audit(content);
        for i in 1..violations.len() {
            assert!(violations[i-1].severity >= violations[i].severity,
                "Violations should be sorted by severity descending");
        }
    }
}
