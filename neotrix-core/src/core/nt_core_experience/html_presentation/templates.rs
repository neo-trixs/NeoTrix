use super::types::{
    HtmlPresentation, PresentationBuilder, Slide, SlideContent, SlideLayout, StatBlock,
    TimelineEntry,
};

#[derive(Debug, Clone)]
pub struct PresentationTemplate;

impl PresentationTemplate {
    /// 6-slide pitch deck: cover, problem, solution, market, traction, cta.
    pub fn pitch_deck(theme: &str) -> HtmlPresentation {
        PresentationBuilder::new("Pitch Deck")
            .theme(theme)
            .add_slide(Slide::new(
                0,
                SlideLayout::Cover,
                "Our Vision",
                SlideContent::Text(String::new()),
            ))
            .add_slide(Slide::new(
                1,
                SlideLayout::Bullets,
                "The Problem",
                SlideContent::BulletList(vec![
                    "Current solutions are slow and expensive".into(),
                    "Legacy systems cannot scale".into(),
                    "Users demand real-time intelligence".into(),
                ]),
            ))
            .add_slide(Slide::new(
                2,
                SlideLayout::TwoColumn,
                "Our Solution",
                SlideContent::TwoColumnText(
                    "**Before:** Months of manual analysis, human error, high cost.".into(),
                    "**After:** AI-powered real-time insights, 10x faster, 99% accuracy.".into(),
                ),
            ))
            .add_slide(Slide::new(
                3,
                SlideLayout::StatHighlight,
                "Market Opportunity",
                SlideContent::StatBlock(StatBlock {
                    value: "$50B".into(),
                    label: "TAM by 2028".into(),
                    trend: Some("↑ 24% CAGR".into()),
                }),
            ))
            .add_slide(Slide::new(
                4,
                SlideLayout::Timeline,
                "Our Traction",
                SlideContent::Timeline(vec![
                    TimelineEntry {
                        date: "Q1 2025".into(),
                        title: "MVP Launch".into(),
                        description: "First 100 enterprise users".into(),
                    },
                    TimelineEntry {
                        date: "Q3 2025".into(),
                        title: "Series A".into(),
                        description: "$12M raised".into(),
                    },
                    TimelineEntry {
                        date: "Q1 2026".into(),
                        title: "Scale".into(),
                        description: "2M users, 40% MoM growth".into(),
                    },
                ]),
            ))
            .add_slide(Slide::new(
                5,
                SlideLayout::Cta,
                "Join Us",
                SlideContent::Text("Let's build the future together. → contact@example.com".into()),
            ))
            .build()
    }

    /// 8-slide tech sharing: cover, agenda, background, arch diagram (code), deep dive, demo, results, q&a.
    pub fn tech_sharing(theme: &str) -> HtmlPresentation {
        PresentationBuilder::new("Tech Sharing")
            .theme(theme)
            .add_slide(Slide::new(
                0,
                SlideLayout::Cover,
                "Tech Sharing",
                SlideContent::Text(String::new()),
            ))
            .add_slide(
                Slide::new(1, SlideLayout::TableOfContents, "Agenda", SlideContent::BulletList(vec![
                    "Background & Motivation".into(),
                    "System Architecture".into(),
                    "Deep Dive: Core Algorithm".into(),
                    "Demo".into(),
                    "Benchmark Results".into(),
                    "Q&A".into(),
                ])),
            )
            .add_slide(Slide::new(2, SlideLayout::Bullets, "Background", SlideContent::BulletList(vec![
                "Existing approaches rely on heuristic rules".into(),
                "We propose a hyperdimensional reasoning engine".into(),
                "Key insight: hyperdimensional computation".into(),
            ])))
            .add_slide(Slide::new(3, SlideLayout::Code, "Architecture", SlideContent::CodeBlock {
                language: "rust".into(),
                code: "struct Engine {\n    vsa: HyperVector<4096>,\n    memory: KnowledgeGraph,\n}\n\nimpl Engine {\n    fn reason(&self, input: &str) -> Result<Vec<Fact>> {\n        // VSA encoding + graph traversal\n    }\n}".into(),
            }))
            .add_slide(Slide::new(4, SlideLayout::ProcessSteps, "Deep Dive", SlideContent::ProcessSteps(vec![
                "Encode input → 4096-bit hyperdimensional vector".into(),
                "Activate related concepts via spreading activation".into(),
                "Infer new facts through compositional reasoning".into(),
                "Return ranked results with confidence scores".into(),
            ])))
            .add_slide(Slide::new(5, SlideLayout::KpiGrid, "Benchmark Results", SlideContent::KpiGrid(vec![
                StatBlock { value: "94%".into(), label: "Accuracy".into(), trend: Some("+12% vs SOTA".into()) },
                StatBlock { value: "2.3ms".into(), label: "Avg Latency".into(), trend: Some("10x faster".into()) },
                StatBlock { value: "56K".into(), label: "Knowledge Nodes".into(), trend: None },
            ])))
            .add_slide(Slide::new(6, SlideLayout::BigQuote, "Demo", SlideContent::Quote {
                text: "The best way to predict the future is to invent it.".into(),
                attribution: "Alan Kay".into(),
            }))
            .add_slide(Slide::new(7, SlideLayout::Cta, "Q&A", SlideContent::Text("Questions? Let's discuss!".into())))
            .build()
    }

    /// 5-slide weekly report: cover, kpi summary, achievements, risks, next week.
    pub fn weekly_report(theme: &str) -> HtmlPresentation {
        PresentationBuilder::new("Weekly Report")
            .theme(theme)
            .add_slide(Slide::new(
                0,
                SlideLayout::Cover,
                "Weekly Engineering Report",
                SlideContent::Text(String::new()),
            ))
            .add_slide(Slide::new(
                1,
                SlideLayout::KpiGrid,
                "Key Metrics",
                SlideContent::KpiGrid(vec![
                    StatBlock {
                        value: "12".into(),
                        label: "PRs Merged".into(),
                        trend: Some("↑ 3 vs last week".into()),
                    },
                    StatBlock {
                        value: "98%".into(),
                        label: "Test Pass Rate".into(),
                        trend: None,
                    },
                    StatBlock {
                        value: "0".into(),
                        label: "P0 Incidents".into(),
                        trend: Some("↓ 2".into()),
                    },
                    StatBlock {
                        value: "4.2h".into(),
                        label: "Avg Review Time".into(),
                        trend: Some("↓ 1.1h".into()),
                    },
                ]),
            ))
            .add_slide(Slide::new(
                2,
                SlideLayout::Bullets,
                "Achievements",
                SlideContent::BulletList(vec![
                    "Shipped hypergraph RAG module".into(),
                    "Zero-compile-warning milestone achieved".into(),
                    "BFT consensus layer passing all tests".into(),
                ]),
            ))
            .add_slide(Slide::new(
                3,
                SlideLayout::Comparison,
                "Risks & Mitigations",
                SlideContent::Comparison {
                    left_label: "Risk".into(),
                    left_items: vec![
                        "Hypervector dimension mismatch".into(),
                        "API rate limiting".into(),
                    ],
                    right_label: "Mitigation".into(),
                    right_items: vec![
                        "Auto-detect at startup".into(),
                        "Circuit breaker + backoff".into(),
                    ],
                },
            ))
            .add_slide(Slide::new(
                4,
                SlideLayout::Bullets,
                "Next Week Plan",
                SlideContent::BulletList(vec![
                    "Implement multi-head resonator".into(),
                    "Begin self-improvement edit safety net".into(),
                    "Release v0.8 to staging".into(),
                ]),
            ))
            .build()
    }
}
