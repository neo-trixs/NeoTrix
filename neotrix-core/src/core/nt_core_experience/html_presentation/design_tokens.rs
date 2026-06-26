#![forbid(unsafe_code)]

// ── Core token types ──

pub struct ColorPalette {
    pub primary: &'static str,
    pub primary_light: &'static str,
    pub primary_dark: &'static str,
    pub secondary: &'static str,
    pub accent: &'static str,
    pub surface: &'static str,
    pub background: &'static str,
    pub text_primary: &'static str,
    pub text_body: &'static str,
    pub text_muted: &'static str,
    pub border: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub danger: &'static str,
}

pub struct TypographyScale {
    pub font_display: &'static str,
    pub font_body: &'static str,
    pub font_code: &'static str,
    pub size_display: &'static str,
    pub size_heading: &'static str,
    pub size_body: &'static str,
    pub size_caption: &'static str,
    pub size_code: &'static str,
}

pub struct SpacingScale {
    pub xs: &'static str,
    pub sm: &'static str,
    pub md: &'static str,
    pub lg: &'static str,
    pub xl: &'static str,
    pub xxl: &'static str,
    pub xxxl: &'static str,
}

pub struct RadiusScale {
    pub sm: &'static str,
    pub md: &'static str,
    pub lg: &'static str,
    pub full: &'static str,
}

pub struct ShadowScale {
    pub sm: &'static str,
    pub md: &'static str,
    pub lg: &'static str,
}

pub struct AnimationTokens {
    pub duration_fast: &'static str,
    pub duration_normal: &'static str,
    pub duration_slow: &'static str,
    pub easing_in: &'static str,
    pub easing_out: &'static str,
    pub easing_in_out: &'static str,
}

pub struct DesignTokens {
    pub name: &'static str,
    pub palette: ColorPalette,
    pub typography: TypographyScale,
    pub spacing: SpacingScale,
    pub radius: RadiusScale,
    pub shadows: ShadowScale,
    pub animation: AnimationTokens,
}

impl DesignTokens {
    pub fn serialize_css_vars(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str(":root {\n");

        // Colors
        let p = &self.palette;
        out.push_str("  --color-primary: ");
        out.push_str(p.primary);
        out.push_str(";\n");
        out.push_str("  --color-primary-light: ");
        out.push_str(p.primary_light);
        out.push_str(";\n");
        out.push_str("  --color-primary-dark: ");
        out.push_str(p.primary_dark);
        out.push_str(";\n");
        out.push_str("  --color-secondary: ");
        out.push_str(p.secondary);
        out.push_str(";\n");
        out.push_str("  --color-accent: ");
        out.push_str(p.accent);
        out.push_str(";\n");
        out.push_str("  --color-surface: ");
        out.push_str(p.surface);
        out.push_str(";\n");
        out.push_str("  --color-background: ");
        out.push_str(p.background);
        out.push_str(";\n");
        out.push_str("  --color-text-primary: ");
        out.push_str(p.text_primary);
        out.push_str(";\n");
        out.push_str("  --color-text-body: ");
        out.push_str(p.text_body);
        out.push_str(";\n");
        out.push_str("  --color-text-muted: ");
        out.push_str(p.text_muted);
        out.push_str(";\n");
        out.push_str("  --color-border: ");
        out.push_str(p.border);
        out.push_str(";\n");
        out.push_str("  --color-success: ");
        out.push_str(p.success);
        out.push_str(";\n");
        out.push_str("  --color-warning: ");
        out.push_str(p.warning);
        out.push_str(";\n");
        out.push_str("  --color-danger: ");
        out.push_str(p.danger);
        out.push_str(";\n");

        // Typography
        let t = &self.typography;
        out.push_str("  --font-display: ");
        out.push_str(t.font_display);
        out.push_str(";\n");
        out.push_str("  --font-body: ");
        out.push_str(t.font_body);
        out.push_str(";\n");
        out.push_str("  --font-code: ");
        out.push_str(t.font_code);
        out.push_str(";\n");
        out.push_str("  --font-size-display: ");
        out.push_str(t.size_display);
        out.push_str(";\n");
        out.push_str("  --font-size-heading: ");
        out.push_str(t.size_heading);
        out.push_str(";\n");
        out.push_str("  --font-size-body: ");
        out.push_str(t.size_body);
        out.push_str(";\n");
        out.push_str("  --font-size-caption: ");
        out.push_str(t.size_caption);
        out.push_str(";\n");
        out.push_str("  --font-size-code: ");
        out.push_str(t.size_code);
        out.push_str(";\n");

        // Spacing
        let s = &self.spacing;
        out.push_str("  --space-xs: ");
        out.push_str(s.xs);
        out.push_str(";\n");
        out.push_str("  --space-sm: ");
        out.push_str(s.sm);
        out.push_str(";\n");
        out.push_str("  --space-md: ");
        out.push_str(s.md);
        out.push_str(";\n");
        out.push_str("  --space-lg: ");
        out.push_str(s.lg);
        out.push_str(";\n");
        out.push_str("  --space-xl: ");
        out.push_str(s.xl);
        out.push_str(";\n");
        out.push_str("  --space-xxl: ");
        out.push_str(s.xxl);
        out.push_str(";\n");
        out.push_str("  --space-xxxl: ");
        out.push_str(s.xxxl);
        out.push_str(";\n");

        // Radius
        let r = &self.radius;
        out.push_str("  --radius-sm: ");
        out.push_str(r.sm);
        out.push_str(";\n");
        out.push_str("  --radius-md: ");
        out.push_str(r.md);
        out.push_str(";\n");
        out.push_str("  --radius-lg: ");
        out.push_str(r.lg);
        out.push_str(";\n");
        out.push_str("  --radius-full: ");
        out.push_str(r.full);
        out.push_str(";\n");

        // Shadows
        let sh = &self.shadows;
        out.push_str("  --shadow-sm: ");
        out.push_str(sh.sm);
        out.push_str(";\n");
        out.push_str("  --shadow-md: ");
        out.push_str(sh.md);
        out.push_str(";\n");
        out.push_str("  --shadow-lg: ");
        out.push_str(sh.lg);
        out.push_str(";\n");

        // Animation
        let a = &self.animation;
        out.push_str("  --duration-fast: ");
        out.push_str(a.duration_fast);
        out.push_str(";\n");
        out.push_str("  --duration-normal: ");
        out.push_str(a.duration_normal);
        out.push_str(";\n");
        out.push_str("  --duration-slow: ");
        out.push_str(a.duration_slow);
        out.push_str(";\n");
        out.push_str("  --easing-in: ");
        out.push_str(a.easing_in);
        out.push_str(";\n");
        out.push_str("  --easing-out: ");
        out.push_str(a.easing_out);
        out.push_str(";\n");
        out.push_str("  --easing-in-out: ");
        out.push_str(a.easing_in_out);
        out.push_str(";\n");

        out.push_str("}\n");
        out
    }
}

// ── Shared defaults for spacing, radius, shadows, animation ──

const SPACING: SpacingScale = SpacingScale {
    xs: "0.25rem",
    sm: "0.5rem",
    md: "1rem",
    lg: "1.5rem",
    xl: "2rem",
    xxl: "3rem",
    xxxl: "5rem",
};

const RADIUS: RadiusScale = RadiusScale {
    sm: "4px",
    md: "8px",
    lg: "16px",
    full: "50%",
};

const SHADOWS: ShadowScale = ShadowScale {
    sm: "0 1px 3px rgba(0,0,0,0.08)",
    md: "0 4px 16px rgba(0,0,0,0.1)",
    lg: "0 8px 32px rgba(0,0,0,0.12)",
};

const ANIMATION: AnimationTokens = AnimationTokens {
    duration_fast: "150ms",
    duration_normal: "300ms",
    duration_slow: "500ms",
    easing_in: "cubic-bezier(0.4, 0, 1, 1)",
    easing_out: "cubic-bezier(0, 0, 0.2, 1)",
    easing_in_out: "cubic-bezier(0.4, 0, 0.2, 1)",
};

// ── 5 token presets ──

pub static MINIMAL_WHITE_TOKENS: DesignTokens = DesignTokens {
    name: "minimal-white",
    palette: ColorPalette {
        primary: "#1a1a1a",
        primary_light: "#555555",
        primary_dark: "#000000",
        secondary: "#333333",
        accent: "#1a1a1a",
        surface: "#ffffff",
        background: "#ffffff",
        text_primary: "#1a1a1a",
        text_body: "#333333",
        text_muted: "#666666",
        border: "#dddddd",
        success: "#22c55e",
        warning: "#e0a800",
        danger: "#dc3545",
    },
    typography: TypographyScale {
        font_display: "'Inter', 'SF Pro Display', -apple-system, sans-serif",
        font_body: "'Inter', 'SF Pro Text', -apple-system, sans-serif",
        font_code: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
        size_display: "3.5rem",
        size_heading: "2.8rem",
        size_body: "1.3rem",
        size_caption: "0.9rem",
        size_code: "0.95rem",
    },
    spacing: SPACING,
    radius: RADIUS,
    shadows: SHADOWS,
    animation: ANIMATION,
};

pub static CYBERPUNK_NEON_TOKENS: DesignTokens = DesignTokens {
    name: "cyberpunk-neon",
    palette: ColorPalette {
        primary: "#00ffe1",
        primary_light: "#66ffe8",
        primary_dark: "#00ccb4",
        secondary: "#ff00e4",
        accent: "#ff00e4",
        surface: "#0d0d2b",
        background: "#0a0a1a",
        text_primary: "#00ffe1",
        text_body: "#c0c0f0",
        text_muted: "#8888cc",
        border: "#2a2a5a",
        success: "#00ff88",
        warning: "#ffcc00",
        danger: "#ff3355",
    },
    typography: TypographyScale {
        font_display: "'Orbitron', 'Rajdhani', 'SF Pro Display', sans-serif",
        font_body: "'Rajdhani', 'SF Pro Text', sans-serif",
        font_code: "'JetBrains Mono', 'Fira Code', monospace",
        size_display: "3.5rem",
        size_heading: "2.8rem",
        size_body: "1.3rem",
        size_caption: "0.9rem",
        size_code: "0.95rem",
    },
    spacing: SPACING,
    radius: RADIUS,
    shadows: SHADOWS,
    animation: ANIMATION,
};

pub static SOFT_PASTEL_TOKENS: DesignTokens = DesignTokens {
    name: "soft-pastel",
    palette: ColorPalette {
        primary: "#7c6f9e",
        primary_light: "#9b8abf",
        primary_dark: "#5d5080",
        secondary: "#e8a0c0",
        accent: "#e8a0c0",
        surface: "#fef9f0",
        background: "#f8f0fe",
        text_primary: "#7c6f9e",
        text_body: "#6a6a7a",
        text_muted: "#9a9aaa",
        border: "#d4c4e8",
        success: "#a0d8a0",
        warning: "#d4a050",
        danger: "#e07080",
    },
    typography: TypographyScale {
        font_display: "'Quicksand', 'Nunito', -apple-system, sans-serif",
        font_body: "'Nunito', -apple-system, sans-serif",
        font_code: "'JetBrains Mono', monospace",
        size_display: "3.5rem",
        size_heading: "2.8rem",
        size_body: "1.3rem",
        size_caption: "0.9rem",
        size_code: "0.95rem",
    },
    spacing: SPACING,
    radius: RadiusScale {
        sm: "6px",
        md: "12px",
        lg: "16px",
        full: "50%",
    },
    shadows: SHADOWS,
    animation: ANIMATION,
};

pub static CORPORATE_CLEAN_TOKENS: DesignTokens = DesignTokens {
    name: "corporate-clean",
    palette: ColorPalette {
        primary: "#3182ce",
        primary_light: "#63b3ed",
        primary_dark: "#1a365d",
        secondary: "#2c3e50",
        accent: "#3182ce",
        surface: "#ffffff",
        background: "#f8f9fc",
        text_primary: "#1a365d",
        text_body: "#4a5568",
        text_muted: "#718096",
        border: "#e2e8f0",
        success: "#38a169",
        warning: "#d69e2e",
        danger: "#e53e3e",
    },
    typography: TypographyScale {
        font_display: "'Plus Jakarta Sans', 'SF Pro Display', -apple-system, sans-serif",
        font_body: "'Inter', 'SF Pro Text', -apple-system, sans-serif",
        font_code: "'JetBrains Mono', 'Fira Code', monospace",
        size_display: "3rem",
        size_heading: "2.4rem",
        size_body: "1.2rem",
        size_caption: "0.85rem",
        size_code: "0.9rem",
    },
    spacing: SPACING,
    radius: RADIUS,
    shadows: SHADOWS,
    animation: ANIMATION,
};

pub static ACADEMIC_PAPER_TOKENS: DesignTokens = DesignTokens {
    name: "academic-paper",
    palette: ColorPalette {
        primary: "#8b4513",
        primary_light: "#a06030",
        primary_dark: "#5a2d0a",
        secondary: "#2d2d2d",
        accent: "#8b4513",
        surface: "#fafaf8",
        background: "#fafaf8",
        text_primary: "#1a1a1a",
        text_body: "#333333",
        text_muted: "#5a4a3a",
        border: "#d4c9b8",
        success: "#2d6a2d",
        warning: "#8b5e34",
        danger: "#8b2020",
    },
    typography: TypographyScale {
        font_display: "'Georgia', 'Times New Roman', serif",
        font_body: "'Georgia', 'Times New Roman', serif",
        font_code: "'IBM Plex Mono', 'JetBrains Mono', monospace",
        size_display: "2.8rem",
        size_heading: "2.2rem",
        size_body: "1.15rem",
        size_caption: "0.85rem",
        size_code: "0.85rem",
    },
    spacing: SPACING,
    radius: RadiusScale {
        sm: "2px",
        md: "4px",
        lg: "8px",
        full: "50%",
    },
    shadows: SHADOWS,
    animation: ANIMATION,
};

static ALL_TOKENS: &[&DesignTokens] = &[
    &MINIMAL_WHITE_TOKENS,
    &CYBERPUNK_NEON_TOKENS,
    &SOFT_PASTEL_TOKENS,
    &CORPORATE_CLEAN_TOKENS,
    &ACADEMIC_PAPER_TOKENS,
];

pub fn tokens_by_name(name: &str) -> Option<&'static DesignTokens> {
    ALL_TOKENS.iter().copied().find(|t| t.name == name)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tokens_resolve() {
        let names = [
            "minimal-white",
            "cyberpunk-neon",
            "soft-pastel",
            "corporate-clean",
            "academic-paper",
        ];
        for name in &names {
            let t = tokens_by_name(name).unwrap_or_else(|| panic!("token not found: {name}"));
            assert_eq!(t.name, *name);
        }
    }

    #[test]
    fn test_names_are_distinct() {
        let names: std::collections::HashSet<&str> = ALL_TOKENS.iter().map(|t| t.name).collect();
        assert_eq!(names.len(), ALL_TOKENS.len());
    }

    #[test]
    fn test_unknown_name_returns_none() {
        assert!(tokens_by_name("nonexistent-theme").is_none());
    }

    #[test]
    fn test_css_vars_contains_all_categories() {
        let css = MINIMAL_WHITE_TOKENS.serialize_css_vars();
        assert!(css.contains("--color-primary"));
        assert!(css.contains("--font-display"));
        assert!(css.contains("--space-md"));
        assert!(css.contains("--radius-md"));
        assert!(css.contains("--shadow-md"));
        assert!(css.contains("--duration-normal"));
        assert!(css.contains("--easing-out"));
    }

    #[test]
    fn test_css_vars_starts_and_ends_correctly() {
        let css = CYBERPUNK_NEON_TOKENS.serialize_css_vars();
        assert!(css.starts_with(":root {"));
        assert!(css.trim_end().ends_with('}'));
    }

    #[test]
    fn test_css_vars_all_presets_contain_their_name_colors() {
        for t in ALL_TOKENS {
            let css = t.serialize_css_vars();
            assert!(
                css.contains(t.palette.primary),
                "{} should contain primary color",
                t.name
            );
        }
    }

    #[test]
    fn test_all_tokens_roundtrip() {
        for t in ALL_TOKENS {
            let css = t.serialize_css_vars();
            assert!(css.contains("--color-primary"));
            assert!(css.contains("--font-size-body"));
            assert!(css.contains("--space-xl"));
            assert!(css.contains("--radius-lg"));
            assert!(css.contains("--shadow-lg"));
            assert!(css.contains("--duration-slow"));
        }
    }
}
