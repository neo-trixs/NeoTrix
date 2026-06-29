#![forbid(unsafe_code)]

pub mod design_tokens;
pub mod markdown;
pub mod templates;
pub mod themes;
pub mod types;

pub use design_tokens::DesignTokens;
pub use templates::PresentationTemplate;
pub use types::{
    AnimationEffect, HtmlPresentation, PresentationBuilder, Slide, SlideContent, SlideLayout,
    StatBlock, TimelineEntry,
};

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_creation() {
        let s = Slide::new(
            0,
            SlideLayout::Cover,
            "Hello",
            SlideContent::Text("world".into()),
        );
        assert_eq!(s.id, 0);
        assert_eq!(s.title, "Hello");
        assert!(s.notes.is_none());
        assert!(s.animation.is_none());
    }

    #[test]
    fn test_all_layout_types_render_without_panic() {
        let layouts = vec![
            (SlideLayout::Cover, SlideContent::Text("Cover".into())),
            (
                SlideLayout::TableOfContents,
                SlideContent::BulletList(vec!["Item".into()]),
            ),
            (
                SlideLayout::SectionDivider,
                SlideContent::Text("Section".into()),
            ),
            (
                SlideLayout::Bullets,
                SlideContent::BulletList(vec!["A".into(), "B".into()]),
            ),
            (
                SlideLayout::TwoColumn,
                SlideContent::TwoColumnText("L".into(), "R".into()),
            ),
            (SlideLayout::ThreeColumn, SlideContent::Text("3col".into())),
            (
                SlideLayout::BigQuote,
                SlideContent::Quote {
                    text: "Q".into(),
                    attribution: "A".into(),
                },
            ),
            (
                SlideLayout::StatHighlight,
                SlideContent::StatBlock(StatBlock {
                    value: "50".into(),
                    label: "pct".into(),
                    trend: None,
                }),
            ),
            (
                SlideLayout::KpiGrid,
                SlideContent::KpiGrid(vec![StatBlock {
                    value: "1".into(),
                    label: "a".into(),
                    trend: None,
                }]),
            ),
            (
                SlideLayout::Code,
                SlideContent::CodeBlock {
                    language: "rs".into(),
                    code: "fn main() {}".into(),
                },
            ),
            (
                SlideLayout::ImageHero,
                SlideContent::Image {
                    url: "img.png".into(),
                    alt: "img".into(),
                    caption: None,
                },
            ),
            (
                SlideLayout::ImageGrid,
                SlideContent::Image {
                    url: "img2.png".into(),
                    alt: "img".into(),
                    caption: Some("cap".into()),
                },
            ),
            (
                SlideLayout::Timeline,
                SlideContent::Timeline(vec![TimelineEntry {
                    date: "2025".into(),
                    title: "Event".into(),
                    description: "desc".into(),
                }]),
            ),
            (
                SlideLayout::Comparison,
                SlideContent::Comparison {
                    left_label: "L".into(),
                    left_items: vec!["a".into()],
                    right_label: "R".into(),
                    right_items: vec!["b".into()],
                },
            ),
            (
                SlideLayout::ProcessSteps,
                SlideContent::ProcessSteps(vec!["Step 1".into()]),
            ),
            (SlideLayout::Cta, SlideContent::Text("CTA".into())),
            (SlideLayout::Thanks, SlideContent::Text("Thanks".into())),
        ];
        let pres = HtmlPresentation::new("Test");
        let css = HtmlPresentation::theme_css("minimal-white");
        for (layout, content) in &layouts {
            let slide = Slide::new(0, *layout, "Test", content.clone());
            let html = pres.render_slide(&slide, &css);
            assert!(!html.is_empty(), "Layout {:?} produced empty HTML", layout);
        }
    }

    #[test]
    fn test_render_full_html() {
        let mut pres = HtmlPresentation::new("Full Test");
        pres.slides.push(Slide::new(
            0,
            SlideLayout::Cover,
            "Cover",
            SlideContent::Text("Welcome".into()),
        ));
        pres.slides.push(Slide::new(
            1,
            SlideLayout::Bullets,
            "Bullets",
            SlideContent::BulletList(vec!["One".into(), "Two".into()]),
        ));
        let html = pres.render_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Full Test"));
        assert!(html.contains("Cover"));
        assert!(html.contains("Bullets"));
        assert!(html.contains("One"));
        assert!(html.contains("slide-counter"));
        assert!(html.contains("data-dir"));
    }

    #[test]
    fn test_theme_application() {
        let css = HtmlPresentation::theme_css("cyberpunk-neon");
        assert!(!css.is_empty());
        let fallback = HtmlPresentation::theme_css("nonexistent-theme");
        assert!(!fallback.is_empty());
    }

    #[test]
    fn test_builder_pattern() {
        let pres = PresentationBuilder::new("Builder Test")
            .theme("academic-paper")
            .author("Alice")
            .add_slide(Slide::new(
                0,
                SlideLayout::Cover,
                "Title",
                SlideContent::Text("Body".into()),
            ))
            .build();
        assert_eq!(pres.title, "Builder Test");
        assert_eq!(pres.theme, "academic-paper");
        assert_eq!(pres.author, Some("Alice".into()));
        assert_eq!(pres.slides.len(), 1);
    }

    #[test]
    fn test_outline_creation() {
        let pres = HtmlPresentation::from_outline(
            "Outline",
            &[
                ("Intro", "This is the intro"),
                ("Body", "Main content here"),
            ],
            "corporate-clean",
        );
        assert_eq!(pres.title, "Outline");
        assert_eq!(pres.theme, "corporate-clean");
        assert_eq!(pres.slides.len(), 4);
        assert_eq!(pres.slides[0].layout, SlideLayout::Cover);
        assert_eq!(pres.slides[1].layout, SlideLayout::TableOfContents);
        assert_eq!(pres.slides[3].layout, SlideLayout::Thanks);
    }

    #[test]
    fn test_file_output() {
        let pres = PresentationBuilder::new("File Test")
            .add_slide(Slide::new(
                0,
                SlideLayout::Cover,
                "Output Test",
                SlideContent::Text("Body".into()),
            ))
            .build();
        let path = std::env::temp_dir().join("test_presentation_output.html");
        let path_str = path.to_str().unwrap().to_string();
        assert!(pres.to_file(&path_str).is_ok());
        let contents =
            std::fs::read_to_string(&path).expect("failed to read presentation output file");
        assert!(contents.contains("Output Test"));
        std::fs::remove_file(&path).expect("failed to remove presentation output file");
    }

    #[test]
    fn test_template_instantiation() {
        let pitch = PresentationTemplate::pitch_deck("minimal-white");
        assert_eq!(pitch.slides.len(), 6);
        assert_eq!(pitch.slides[0].layout, SlideLayout::Cover);

        let tech = PresentationTemplate::tech_sharing("cyberpunk-neon");
        assert_eq!(tech.slides.len(), 8);
        assert_eq!(tech.slides[1].layout, SlideLayout::TableOfContents);

        let weekly = PresentationTemplate::weekly_report("corporate-clean");
        assert_eq!(weekly.slides.len(), 5);
        assert_eq!(weekly.slides[1].layout, SlideLayout::KpiGrid);
    }

    #[test]
    fn test_animation_effect_assignment() {
        let s = Slide::new(
            0,
            SlideLayout::Cover,
            "Animated",
            SlideContent::Text("".into()),
        )
        .with_animation(AnimationEffect::FadeIn);
        assert!(s.animation.is_some());
        assert_eq!(s.animation.unwrap().css_class(), "anim-fade-in");

        let s2 = Slide::new(
            1,
            SlideLayout::Bullets,
            "Static",
            SlideContent::Text("".into()),
        )
        .with_animation(AnimationEffect::None);
        assert_eq!(s2.animation.unwrap().css_class(), "");
    }

    #[test]
    fn test_empty_deck() {
        let pres = HtmlPresentation::new("Empty");
        let html = pres.render_html();
        assert!(html.contains("Empty"));
        assert_eq!(pres.slides.len(), 0);
    }

    #[test]
    fn test_slide_with_notes_and_animation() {
        let s = Slide::new(
            0,
            SlideLayout::Cover,
            "Notes Demo",
            SlideContent::Text("Hello".into()),
        )
        .with_notes("Speaker note here")
        .with_animation(AnimationEffect::RiseIn);
        assert_eq!(s.notes, Some("Speaker note here".into()));
        let pres = HtmlPresentation::new("Notes");
        let css = HtmlPresentation::theme_css("minimal-white");
        let html = pres.render_slide(&s, &css);
        assert!(html.contains("Speaker note here"));
        assert!(html.contains("anim-rise-in"));
    }

    #[test]
    fn test_markdown_conversion() {
        let html = super::markdown::markdown_to_html("**bold** *italic* `code`");
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_stat_block_with_trend() {
        let sb = StatBlock {
            value: "99.9%".into(),
            label: "Uptime".into(),
            trend: Some("↑ 0.5%".into()),
        };
        let pres = HtmlPresentation::new("Stats");
        let css = HtmlPresentation::theme_css("minimal-white");
        let slide = Slide::new(
            0,
            SlideLayout::StatHighlight,
            "Reliability",
            SlideContent::StatBlock(sb),
        );
        let html = pres.render_slide(&slide, &css);
        assert!(html.contains("99.9%"));
        assert!(html.contains("Uptime"));
        assert!(html.contains("0.5%"));
    }

    #[test]
    fn test_default_themes_count() {
        let themes = HtmlPresentation::default_themes();
        assert!(themes.len() >= 5);
        assert!(themes.contains_key("minimal-white"));
        assert!(themes.contains_key("cyberpunk-neon"));
        assert!(themes.contains_key("soft-pastel"));
        assert!(themes.contains_key("corporate-clean"));
        assert!(themes.contains_key("academic-paper"));
    }
}
