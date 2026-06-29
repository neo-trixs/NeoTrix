#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StyleMode {
    Card,
    Infographic,
    SocialPost,
    Slide,
    Illustration,
    Poster,
    Magazine,
    Deck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentType {
    Tutorial,
    Story,
    Analysis,
    Promotion,
    Report,
    Portfolio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LayoutDirection {
    SingleColumn,
    TwoColumn,
    ThreeColumn,
    Grid,
    Hero,
    Split,
}

#[derive(Debug, Clone)]
pub struct StyleProfile {
    pub mode: StyleMode,
    pub palette: [&'static str; 5],
    pub font_headline: &'static str,
    pub font_body: &'static str,
    pub border_radius: f64,
    pub shadow: &'static str,
}

impl StyleProfile {
    pub fn for_mode(mode: StyleMode, palette_seed: &str) -> Self {
        let palette = match palette_seed {
            "professional" => ["#1e3a5f", "#3b82f6", "#94a3b8", "#f1f5f9", "#0f172a"],
            "creative" => ["#7c3aed", "#ec4899", "#f59e0b", "#faf5ff", "#1e1b4b"],
            "warm" => ["#b91c1c", "#f97316", "#fde047", "#fff7ed", "#431407"],
            "tech" => ["#0891b2", "#06b6d4", "#22d3ee", "#ecfeff", "#164e63"],
            "minimal" => ["#334155", "#64748b", "#cbd5e1", "#f8fafc", "#020617"],
            _ => ["#6366f1", "#818cf8", "#a5b4fc", "#eef2ff", "#1e1b4b"],
        };
        let (headline, body) = match mode {
            StyleMode::Card | StyleMode::SocialPost => {
                ("'Inter', sans-serif", "'Inter', sans-serif")
            }
            StyleMode::Infographic | StyleMode::Magazine => {
                ("'Playfair Display', serif", "'Source Sans Pro', sans-serif")
            }
            StyleMode::Slide | StyleMode::Deck => {
                ("'Plus Jakarta Sans', sans-serif", "'Inter', sans-serif")
            }
            StyleMode::Illustration => ("'Space Grotesk', sans-serif", "'Inter', sans-serif"),
            StyleMode::Poster => ("'Clash Display', sans-serif", "'Inter', sans-serif"),
        };
        Self {
            mode,
            palette,
            font_headline: headline,
            font_body: body,
            border_radius: if matches!(mode, StyleMode::SocialPost | StyleMode::Card) {
                16.0
            } else {
                8.0
            },
            shadow: if matches!(mode, StyleMode::SocialPost | StyleMode::Deck) {
                "0 4px 24px rgba(0,0,0,0.12)"
            } else {
                "0 1px 3px rgba(0,0,0,0.08)"
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShotBlock {
    pub index: usize,
    pub scene_description: String,
    pub suggested_style: StyleMode,
    pub layout: LayoutDirection,
    pub key_elements: Vec<String>,
    pub color_accent: &'static str,
    pub aspect_ratio: &'static str,
}

#[derive(Debug, Clone)]
pub struct VisualPlan {
    pub title: String,
    pub content_type: ContentType,
    pub profile: StyleProfile,
    pub shots: Vec<ShotBlock>,
    pub export_format: &'static str,
}

#[derive(Debug, Clone)]
pub struct VisualPlannerStats {
    pub total_plans: u64,
    pub shots_generated: u64,
    pub per_style: HashMap<String, u64>,
    pub avg_shots_per_plan: f64,
}

impl Default for VisualPlannerStats {
    fn default() -> Self {
        Self {
            total_plans: 0,
            shots_generated: 0,
            per_style: HashMap::new(),
            avg_shots_per_plan: 0.0,
        }
    }
}

pub struct VisualPlanner {
    stats: VisualPlannerStats,
}

impl VisualPlanner {
    pub fn new() -> Self {
        Self {
            stats: VisualPlannerStats::default(),
        }
    }

    pub fn plan(
        &mut self,
        title: &str,
        content_type: ContentType,
        style: StyleMode,
        palette_seed: &str,
    ) -> VisualPlan {
        self.stats.total_plans += 1;
        let profile = StyleProfile::for_mode(style, palette_seed);
        *self
            .stats
            .per_style
            .entry(format!("{:?}", style))
            .or_insert(0) += 1;

        let (n_shots, aspect) = match style {
            StyleMode::Card | StyleMode::SocialPost => (3, "1:1"),
            StyleMode::Infographic => (6, "9:16"),
            StyleMode::Slide | StyleMode::Deck => (8, "16:9"),
            StyleMode::Poster => (1, "9:16"),
            StyleMode::Illustration => (1, "4:3"),
            StyleMode::Magazine => (4, "4:3"),
        };

        let mut shots = Vec::new();
        for i in 0..n_shots {
            let (scene, elements) = self.generate_shot_scene(title, content_type, i, n_shots);
            let layout = match i % 4 {
                0 => LayoutDirection::Hero,
                1 => LayoutDirection::TwoColumn,
                2 => LayoutDirection::SingleColumn,
                _ => LayoutDirection::Grid,
            };
            let color_idx = i % 4;
            shots.push(ShotBlock {
                index: i,
                scene_description: scene,
                suggested_style: style,
                layout,
                key_elements: elements,
                color_accent: profile.palette[color_idx + 1],
                aspect_ratio: aspect,
            });
        }

        self.stats.shots_generated += shots.len() as u64;
        self.stats.avg_shots_per_plan =
            self.stats.shots_generated as f64 / self.stats.total_plans as f64;

        VisualPlan {
            title: title.to_string(),
            content_type,
            profile,
            shots,
            export_format: match style {
                StyleMode::Slide | StyleMode::Deck => "html",
                StyleMode::Card | StyleMode::SocialPost => "png",
                _ => "svg",
            },
        }
    }

    fn generate_shot_scene(
        &self,
        title: &str,
        content_type: ContentType,
        idx: usize,
        _total: usize,
    ) -> (String, Vec<String>) {
        let scenes = match content_type {
            ContentType::Tutorial => vec![
                format!("Opening: {} — hook with the problem", title),
                "Step 1: Core concept breakdown".into(),
                "Step 2: Practical demonstration".into(),
                format!("Closing: {} — key takeaway + CTA", title),
            ],
            ContentType::Story => vec![
                format!("Cover: {} — atmospheric establishing shot", title),
                "Character / conflict introduction".into(),
                "Climax — emotional peak".into(),
                "Resolution — reflective closing".into(),
            ],
            ContentType::Analysis => vec![
                format!("Title card: {} + key stat", title),
                "Context and background data".into(),
                "Deep analysis with comparison".into(),
                format!("Insight summary for {}", title),
            ],
            ContentType::Promotion => vec![
                format!("Hero: {} — bold headline + visual", title),
                "Value proposition breakdown".into(),
                "Social proof / testimonial".into(),
                "Call to action".into(),
            ],
            ContentType::Report => vec![
                format!("Cover: {}", title),
                "Executive summary with KPIs".into(),
                "Detailed findings and charts".into(),
                "Recommendations and next steps".into(),
            ],
            ContentType::Portfolio => vec![
                format!("Title: {}", title),
                "Project overview and role".into(),
                "Key achievements with metrics".into(),
                "Reflection and contact".into(),
            ],
        };
        let idx = idx.min(scenes.len() - 1);
        let scene = scenes[idx].clone();
        let elements = vec![
            format!("element_{}_title", idx),
            format!("element_{}_visual", idx),
        ];
        (scene, elements)
    }

    pub fn render_html_preview(&self, plan: &VisualPlan) -> String {
        let mut html = format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>{}</title><style>
body {{ font-family: {}; background: {}; color: {}; padding: 2rem; }}
.shot {{ background: {}; border-radius: {:.0}px; box-shadow: {}; padding: 1.5rem; margin: 1rem 0; }}
.shot h3 {{ font-family: {}; }}
.palette {{ display: flex; gap: 0.5rem; }}
.swatch {{ width: 40px; height: 40px; border-radius: 8px; }}
</style></head><body>
<h1>{}</h1>
<div class="palette">"#,
            plan.title,
            plan.profile.font_body,
            plan.profile.palette[3],
            plan.profile.palette[4],
            plan.profile.palette[3],
            plan.profile.border_radius,
            plan.profile.shadow,
            plan.profile.font_headline,
            plan.title
        );

        for color in &plan.profile.palette {
            html.push_str(&format!(
                r#"<div class="swatch" style="background:{}"></div>"#,
                color
            ));
        }
        html.push_str("</div>");
        for shot in &plan.shots {
            html.push_str(&format!(
                r#"<div class="shot"><h3>Scene {}: {}</h3><p>Layout: {:?} | Ratio: {} | Accent: {}</p><p>Elements: {}</p></div>"#,
                shot.index + 1,
                shot.scene_description,
                shot.layout,
                shot.aspect_ratio,
                shot.color_accent,
                shot.key_elements.join(", "),
            ));
        }
        html.push_str("</body></html>");
        html
    }

    pub fn stats(&self) -> &VisualPlannerStats {
        &self.stats
    }

    pub fn tick(&mut self, input: Option<(&str, ContentType, StyleMode, &str)>) -> String {
        match input {
            Some((title, ct, style, palette)) => {
                let plan = self.plan(title, ct, style, palette);
                format!(
                    "visual_planner:tick=planned_shots={}_style={:?}_preview={}b",
                    plan.shots.len(),
                    style,
                    self.render_html_preview(&plan).len()
                )
            }
            None => {
                format!(
                    "visual_planner:tick=idle_plans={}_shots={}_avg={:.1}",
                    self.stats.total_plans,
                    self.stats.shots_generated,
                    self.stats.avg_shots_per_plan
                )
            }
        }
    }
}

impl Default for VisualPlanner {
    fn default() -> Self {
        Self::new()
    }
}
