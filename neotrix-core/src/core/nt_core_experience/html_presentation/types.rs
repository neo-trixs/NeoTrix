use std::collections::HashMap;

use super::design_tokens::tokens_by_name;
use super::markdown::markdown_to_html;
use super::themes::{
    ACADEMIC_PAPER_CSS, CORPORATE_CLEAN_CSS, CYBERPUNK_NEON_CSS, MINIMAL_WHITE_CSS, SOFT_PASTEL_CSS,
};

// ── Enums ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SlideLayout {
    Cover,
    TableOfContents,
    SectionDivider,
    Bullets,
    TwoColumn,
    ThreeColumn,
    BigQuote,
    StatHighlight,
    KpiGrid,
    Code,
    ImageHero,
    ImageGrid,
    Timeline,
    Comparison,
    ProcessSteps,
    Cta,
    Thanks,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AnimationEffect {
    FadeIn,
    RiseIn,
    ZoomPop,
    BlurIn,
    GlitchIn,
    Typewriter,
    NeonGlow,
    None,
}

impl AnimationEffect {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::FadeIn => "anim-fade-in",
            Self::RiseIn => "anim-rise-in",
            Self::ZoomPop => "anim-zoom-pop",
            Self::BlurIn => "anim-blur-in",
            Self::GlitchIn => "anim-glitch-in",
            Self::Typewriter => "anim-typewriter",
            Self::NeonGlow => "anim-neon-glow",
            Self::None => "",
        }
    }
}

// ── Supporting structs ──

#[derive(Debug, Clone)]
pub struct StatBlock {
    pub value: String,
    pub label: String,
    pub trend: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub date: String,
    pub title: String,
    pub description: String,
}

// ── SlideContent ──

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SlideContent {
    Text(String),
    BulletList(Vec<String>),
    TwoColumnText(String, String),
    CodeBlock {
        language: String,
        code: String,
    },
    StatBlock(StatBlock),
    KpiGrid(Vec<StatBlock>),
    Timeline(Vec<TimelineEntry>),
    Comparison {
        left_label: String,
        left_items: Vec<String>,
        right_label: String,
        right_items: Vec<String>,
    },
    Image {
        url: String,
        alt: String,
        caption: Option<String>,
    },
    Quote {
        text: String,
        attribution: String,
    },
    ProcessSteps(Vec<String>),
}

// ── Slide ──

#[derive(Debug, Clone)]
pub struct Slide {
    pub id: usize,
    pub layout: SlideLayout,
    pub title: String,
    pub content: SlideContent,
    pub notes: Option<String>,
    pub animation: Option<AnimationEffect>,
}

impl Slide {
    pub fn new(id: usize, layout: SlideLayout, title: &str, content: SlideContent) -> Self {
        Self {
            id,
            layout,
            title: title.to_string(),
            content,
            notes: None,
            animation: None,
        }
    }

    pub fn with_notes(mut self, notes: &str) -> Self {
        self.notes = Some(notes.to_string());
        self
    }

    pub fn with_animation(mut self, anim: AnimationEffect) -> Self {
        self.animation = Some(anim);
        self
    }

    fn layout_name(&self) -> &'static str {
        match self.layout {
            SlideLayout::Cover => "cover",
            SlideLayout::TableOfContents => "toc",
            SlideLayout::SectionDivider => "section",
            SlideLayout::Bullets => "bullets",
            SlideLayout::TwoColumn => "two-col",
            SlideLayout::ThreeColumn => "three-col",
            SlideLayout::BigQuote => "quote",
            SlideLayout::StatHighlight => "stat",
            SlideLayout::KpiGrid => "kpi",
            SlideLayout::Code => "code",
            SlideLayout::ImageHero => "image-hero",
            SlideLayout::ImageGrid => "image-grid",
            SlideLayout::Timeline => "timeline",
            SlideLayout::Comparison => "comparison",
            SlideLayout::ProcessSteps => "steps",
            SlideLayout::Cta => "cta",
            SlideLayout::Thanks => "thanks",
        }
    }
}

// ── HtmlPresentation ──

#[derive(Debug, Clone)]
pub struct HtmlPresentation {
    pub slides: Vec<Slide>,
    pub theme: String,
    pub title: String,
    pub author: Option<String>,
}

impl HtmlPresentation {
    pub fn new(title: &str) -> Self {
        Self {
            slides: Vec::new(),
            theme: "minimal-white".to_string(),
            title: title.to_string(),
            author: None,
        }
    }

    pub fn default_themes() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("minimal-white".to_string(), MINIMAL_WHITE_CSS.to_string());
        m.insert("cyberpunk-neon".to_string(), CYBERPUNK_NEON_CSS.to_string());
        m.insert("soft-pastel".to_string(), SOFT_PASTEL_CSS.to_string());
        m.insert(
            "corporate-clean".to_string(),
            CORPORATE_CLEAN_CSS.to_string(),
        );
        m.insert("academic-paper".to_string(), ACADEMIC_PAPER_CSS.to_string());
        m
    }

    /// Build a presentation from a title and list of section (heading, body_text) pairs.
    pub fn from_outline(title: &str, sections: &[(&str, &str)], theme: &str) -> Self {
        let mut slides = Vec::new();
        slides.push(Slide::new(
            0,
            SlideLayout::Cover,
            title,
            SlideContent::Text(String::new()),
        ));
        for (i, &(heading, body)) in sections.iter().enumerate() {
            slides.push(Slide::new(
                i + 1,
                if i == 0 {
                    SlideLayout::TableOfContents
                } else {
                    SlideLayout::Bullets
                },
                heading,
                SlideContent::Text(body.to_string()),
            ));
        }
        slides.push(Slide::new(
            sections.len() + 1,
            SlideLayout::Thanks,
            "Thank You",
            SlideContent::Text(String::new()),
        ));
        Self {
            slides,
            theme: theme.to_string(),
            title: title.to_string(),
            author: None,
        }
    }

    /// Render a theme's CSS by looking it up from `default_themes()`, or fallback to minimal-white.
    pub fn theme_css(name: &str) -> String {
        let themes = Self::default_themes();
        themes
            .get(name)
            .cloned()
            .unwrap_or_else(|| themes.get("minimal-white").cloned().unwrap_or_default())
    }

    /// Generate a complete, self-contained HTML5 document.
    pub fn render_html(&self) -> String {
        let theme_css = Self::theme_css(&self.theme);
        let (nav_arrows, fullscreen_btn) = self.build_controls();
        let slides_html: String = self
            .slides
            .iter()
            .map(|s| self.render_slide(s, &theme_css))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
html, body {{ width: 100%; height: 100%; overflow: hidden; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; }}
{theme_css}
.slide-container {{ position: relative; width: 100%; height: 100%; }}
.slide {{
  position: absolute; top: 0; left: 0; width: 100%; height: 100%;
  display: flex; flex-direction: column; justify-content: center; align-items: center;
  padding: 5%; opacity: 0; transition: opacity 0.4s ease; pointer-events: none;
}}
.slide.active {{ opacity: 1; pointer-events: auto; position: relative; }}
.slide-notes {{ display: none; }}
.controls {{
  position: fixed; bottom: 24px; left: 50%; transform: translateX(-50%);
  display: flex; gap: 16px; z-index: 1000;
}}
.controls button {{
  background: rgba(0,0,0,0.6); color: #fff; border: none; border-radius: 50%;
  width: 44px; height: 44px; font-size: 20px; cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.2s;
}}
.controls button:hover {{ background: rgba(0,0,0,0.85); }}
.slide-counter {{
  position: fixed; bottom: 28px; right: 28px; font-size: 14px;
  color: rgba(255,255,255,0.5); z-index: 1000; font-variant-numeric: tabular-nums;
}}
/* ── Animation keyframes ── */
@keyframes fadeIn {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
@keyframes riseIn {{ from {{ opacity: 0; transform: translateY(30px); }} to {{ opacity: 1; transform: translateY(0); }} }}
@keyframes zoomPop {{ from {{ opacity: 0; transform: scale(0.8); }} to {{ opacity: 1; transform: scale(1); }} }}
@keyframes blurIn {{ from {{ opacity: 0; filter: blur(8px); }} to {{ opacity: 1; filter: blur(0); }} }}
@keyframes glitchIn {{
  0% {{ opacity: 0; transform: translate(0); }}
  20% {{ opacity: 1; transform: translate(-3px, 2px); }}
  40% {{ transform: translate(3px, -1px); }}
  60% {{ transform: translate(-2px, 1px); }}
  80% {{ transform: translate(1px, -2px); }}
  100% {{ transform: translate(0); }}
}}
@keyframes typewriter {{ from {{ width: 0; }} to {{ width: 100%; }} }}
@keyframes neonPulse {{
  0%, 100% {{ text-shadow: 0 0 7px #fff, 0 0 10px currentColor, 0 0 21px currentColor; }}
  50% {{ text-shadow: 0 0 14px #fff, 0 0 20px currentColor, 0 0 40px currentColor; }}
}}
.anim-fade-in.active {{ animation: fadeIn 0.6s ease forwards; }}
.anim-rise-in.active {{ animation: riseIn 0.6s ease forwards; }}
.anim-zoom-pop.active {{ animation: zoomPop 0.5s ease forwards; }}
.anim-blur-in.active {{ animation: blurIn 0.6s ease forwards; }}
.anim-glitch-in.active {{ animation: glitchIn 0.4s ease forwards; }}
.anim-typewriter .slide-title {{ display: inline-block; overflow: hidden; white-space: nowrap; animation: typewriter 1s steps(30) forwards; }}
.anim-neon-glow.active {{ animation: neonPulse 1.5s ease-in-out infinite; }}
.fullscreen-btn {{
  position: fixed; top: 16px; right: 16px; z-index: 1000;
  background: rgba(0,0,0,0.4); color: #fff; border: none; border-radius: 6px;
  padding: 8px 12px; font-size: 14px; cursor: pointer;
}}
</style>
</head>
<body>
<div class="slide-container">
{slides_html}
</div>
<div class="controls">
{nav_arrows}
</div>
<div class="slide-counter" id="slideCounter">1 / {total}</div>
{fullscreen_btn}
<script>
(function() {{
  const slides = document.querySelectorAll('.slide');
  let current = 0;
  function show(idx) {{
    if (idx < 0 || idx >= slides.length) return;
    slides.forEach(s => s.classList.remove('active'));
    slides[idx].classList.add('active');
    document.getElementById('slideCounter').textContent = (idx + 1) + ' / ' + slides.length;
    current = idx;
  }}
  document.addEventListener('keydown', function(e) {{
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' ') {{ e.preventDefault(); show(Math.min(current + 1, slides.length - 1)); }}
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {{ e.preventDefault(); show(Math.max(current - 1, 0)); }}
  }});
  document.querySelectorAll('.controls button').forEach(function(btn) {{
    btn.addEventListener('click', function() {{
      var d = parseInt(this.getAttribute('data-dir') || '0');
      show(Math.max(0, Math.min(current + d, slides.length - 1)));
    }});
  }});
  show(0);
}})();
function toggleFullscreen() {{
  if (!document.fullscreenElement) {{ document.documentElement.requestFullscreen(); }}
  else {{ document.exitFullscreen(); }}
}}
</script>
</body>
</html>"#,
            title = self.title,
            theme_css = theme_css,
            slides_html = slides_html,
            total = self.slides.len(),
            nav_arrows = nav_arrows,
            fullscreen_btn = fullscreen_btn,
        )
    }

    /// Render with design-token CSS variables instead of raw theme CSS.
    /// Falls back to `render_html()` if the theme name is not registered in the token system.
    pub fn render_with_design_tokens(&self) -> String {
        let tokens = match tokens_by_name(&self.theme) {
            Some(t) => t,
            None => return self.render_html(),
        };
        let token_css = tokens.serialize_css_vars();
        let (nav_arrows, fullscreen_btn) = self.build_controls();
        let slides_html: String = self
            .slides
            .iter()
            .map(|s| self.render_slide_with_tokens(s))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
html, body {{ width: 100%; height: 100%; overflow: hidden; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; }}
{token_css}
.slide-container {{ position: relative; width: 100%; height: 100%; }}
.slide {{
  position: absolute; top: 0; left: 0; width: 100%; height: 100%;
  display: flex; flex-direction: column; justify-content: center; align-items: center;
  padding: 5%; opacity: 0; transition: opacity 0.4s ease; pointer-events: none;
}}
.slide.active {{ opacity: 1; pointer-events: auto; position: relative; }}
.slide-notes {{ display: none; }}
.controls {{
  position: fixed; bottom: 24px; left: 50%; transform: translateX(-50%);
  display: flex; gap: 16px; z-index: 1000;
}}
.controls button {{
  background: rgba(0,0,0,0.6); color: #fff; border: none; border-radius: 50%;
  width: 44px; height: 44px; font-size: 20px; cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.2s;
}}
.controls button:hover {{ background: rgba(0,0,0,0.85); }}
.slide-counter {{
  position: fixed; bottom: 28px; right: 28px; font-size: 14px;
  color: rgba(255,255,255,0.5); z-index: 1000; font-variant-numeric: tabular-nums;
}}
/* ── Animation keyframes ── */
@keyframes fadeIn {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
@keyframes riseIn {{ from {{ opacity: 0; transform: translateY(30px); }} to {{ opacity: 1; transform: translateY(0); }} }}
@keyframes zoomPop {{ from {{ opacity: 0; transform: scale(0.8); }} to {{ opacity: 1; transform: scale(1); }} }}
@keyframes blurIn {{ from {{ opacity: 0; filter: blur(8px); }} to {{ opacity: 1; filter: blur(0); }} }}
@keyframes glitchIn {{
  0% {{ opacity: 0; transform: translate(0); }}
  20% {{ opacity: 1; transform: translate(-3px, 2px); }}
  40% {{ transform: translate(3px, -1px); }}
  60% {{ transform: translate(-2px, 1px); }}
  80% {{ transform: translate(1px, -2px); }}
  100% {{ transform: translate(0); }}
}}
@keyframes typewriter {{ from {{ width: 0; }} to {{ width: 100%; }} }}
@keyframes neonPulse {{
  0%, 100% {{ text-shadow: 0 0 7px #fff, 0 0 10px currentColor, 0 0 21px currentColor; }}
  50% {{ text-shadow: 0 0 14px #fff, 0 0 20px currentColor, 0 0 40px currentColor; }}
}}
.anim-fade-in.active {{ animation: fadeIn 0.6s ease forwards; }}
.anim-rise-in.active {{ animation: riseIn 0.6s ease forwards; }}
.anim-zoom-pop.active {{ animation: zoomPop 0.5s ease forwards; }}
.anim-blur-in.active {{ animation: blurIn 0.6s ease forwards; }}
.anim-glitch-in.active {{ animation: glitchIn 0.4s ease forwards; }}
.anim-typewriter .slide-title {{ display: inline-block; overflow: hidden; white-space: nowrap; animation: typewriter 1s steps(30) forwards; }}
.anim-neon-glow.active {{ animation: neonPulse 1.5s ease-in-out infinite; }}
.fullscreen-btn {{
  position: fixed; top: 16px; right: 16px; z-index: 1000;
  background: rgba(0,0,0,0.4); color: #fff; border: none; border-radius: 6px;
  padding: 8px 12px; font-size: 14px; cursor: pointer;
}}
</style>
</head>
<body>
<div class="slide-container">
{slides_html}
</div>
<div class="controls">
{nav_arrows}
</div>
<div class="slide-counter" id="slideCounter">1 / {total}</div>
{fullscreen_btn}
<script>
(function() {{
  const slides = document.querySelectorAll('.slide');
  let current = 0;
  function show(idx) {{
    if (idx < 0 || idx >= slides.length) return;
    slides.forEach(s => s.classList.remove('active'));
    slides[idx].classList.add('active');
    document.getElementById('slideCounter').textContent = (idx + 1) + ' / ' + slides.length;
    current = idx;
  }}
  document.addEventListener('keydown', function(e) {{
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' ') {{ e.preventDefault(); show(Math.min(current + 1, slides.length - 1)); }}
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {{ e.preventDefault(); show(Math.max(current - 1, 0)); }}
  }});
  document.querySelectorAll('.controls button').forEach(function(btn) {{
    btn.addEventListener('click', function() {{
      var d = parseInt(this.getAttribute('data-dir') || '0');
      show(Math.max(0, Math.min(current + d, slides.length - 1)));
    }});
  }});
  show(0);
}})();
function toggleFullscreen() {{
  if (!document.fullscreenElement) {{ document.documentElement.requestFullscreen(); }}
  else {{ document.exitFullscreen(); }}
}}
</script>
</body>
</html>"#,
            title = self.title,
            token_css = token_css,
            slides_html = slides_html,
            total = self.slides.len(),
            nav_arrows = nav_arrows,
            fullscreen_btn = fullscreen_btn,
        )
    }

    /// Render a single slide with design tokens (delegates to render_slide since the
    /// token CSS is embedded at the document level, not per-slide).
    pub fn render_slide_with_tokens(&self, slide: &Slide) -> String {
        self.render_slide(slide, "")
    }

    fn build_controls(&self) -> (String, String) {
        let arrows = format!(
            r#"<button data-dir="-1" title="Previous">&larr;</button>
<button data-dir="1" title="Next">&rarr;</button>"#
        );
        let fs =
            r#"<button class="fullscreen-btn" onclick="toggleFullscreen()">⛶ Fullscreen</button>"#
                .to_string();
        (arrows, fs)
    }

    pub fn render_slide(&self, slide: &Slide, _theme_css: &str) -> String {
        let anim_class = slide
            .animation
            .as_ref()
            .map(|a| format!(" {}", a.css_class()))
            .unwrap_or_default();
        let content_html = self.render_content(&slide.content);
        let notes_html = slide
            .notes
            .as_ref()
            .map(|n| format!("<div class=\"slide-notes\">{}</div>", Self::escape_html(n)))
            .unwrap_or_default();

        format!(
            r#"<div class="slide{anim}" id="slide-{id}" data-layout="{layout}">
  <div class="slide-header"><h1 class="slide-title">{title}</h1></div>
  <div class="slide-body">{content}</div>
  {notes}
</div>"#,
            anim = anim_class,
            id = slide.id,
            layout = slide.layout_name(),
            title = Self::escape_html(&slide.title),
            content = content_html,
            notes = notes_html,
        )
    }

    fn render_content(&self, content: &SlideContent) -> String {
        match content {
            SlideContent::Text(t) => {
                let html = markdown_to_html(t);
                format!("<div class=\"text-content\">{}</div>", html)
            }
            SlideContent::BulletList(items) => {
                let lis: String = items
                    .iter()
                    .map(|i| format!("<li>{}</li>", Self::escape_html(i)))
                    .collect();
                format!("<ul class=\"bullet-list\">{}</ul>", lis)
            }
            SlideContent::TwoColumnText(l, r) => {
                format!(
                    r#"<div class="two-column">
  <div class="col">{}</div>
  <div class="col">{}</div>
</div>"#,
                    markdown_to_html(l),
                    markdown_to_html(r),
                )
            }
            SlideContent::CodeBlock { language, code } => {
                format!(
                    r#"<div class="code-block">
  <div class="code-lang">{}</div>
  <pre><code>{}</code></pre>
</div>"#,
                    Self::escape_html(language),
                    Self::escape_html(code),
                )
            }
            SlideContent::StatBlock(sb) => {
                let trend_html = sb
                    .trend
                    .as_ref()
                    .map(|t| format!("<span class=\"stat-trend\">{}</span>", Self::escape_html(t)))
                    .unwrap_or_default();
                format!(
                    r#"<div class="stat-block">
  <div class="stat-value">{}</div>
  <div class="stat-label">{}</div>
  {}
</div>"#,
                    Self::escape_html(&sb.value),
                    Self::escape_html(&sb.label),
                    trend_html,
                )
            }
            SlideContent::KpiGrid(stats) => {
                let cards: String = stats
                    .iter()
                    .map(|s| self.render_content(&SlideContent::StatBlock(s.clone())))
                    .collect();
                format!("<div class=\"kpi-grid\">{}</div>", cards)
            }
            SlideContent::Timeline(entries) => {
                let items: String = entries
                    .iter()
                    .map(|e| {
                        format!(
                            r#"<div class="timeline-item">
  <div class="tl-date">{}</div>
  <div class="tl-content">
    <h3>{}</h3>
    <p>{}</p>
  </div>
</div>"#,
                            Self::escape_html(&e.date),
                            Self::escape_html(&e.title),
                            Self::escape_html(&e.description),
                        )
                    })
                    .collect();
                format!("<div class=\"timeline\">{}</div>", items)
            }
            SlideContent::Comparison {
                left_label,
                left_items,
                right_label,
                right_items,
            } => {
                let li: String = left_items
                    .iter()
                    .map(|i| format!("<li>{}</li>", Self::escape_html(i)))
                    .collect();
                let ri: String = right_items
                    .iter()
                    .map(|i| format!("<li>{}</li>", Self::escape_html(i)))
                    .collect();
                format!(
                    r#"<div class="comparison">
  <div class="comp-col">
    <h3>{}</h3>
    <ul>{}</ul>
  </div>
  <div class="comp-col">
    <h3>{}</h3>
    <ul>{}</ul>
  </div>
</div>"#,
                    Self::escape_html(left_label),
                    li,
                    Self::escape_html(right_label),
                    ri,
                )
            }
            SlideContent::Image { url, alt, caption } => {
                let cap = caption
                    .as_ref()
                    .map(|c| format!("<p class=\"img-caption\">{}</p>", Self::escape_html(c)))
                    .unwrap_or_default();
                format!(
                    r#"<div class="image-container">
  <img src="{}" alt="{}" />
  {}
</div>"#,
                    Self::escape_html(url),
                    Self::escape_html(alt),
                    cap,
                )
            }
            SlideContent::Quote { text, attribution } => {
                format!(
                    r#"<blockquote class="big-quote">
  <p>"{}"</p>
  <cite>— {}</cite>
</blockquote>"#,
                    Self::escape_html(text),
                    Self::escape_html(attribution),
                )
            }
            SlideContent::ProcessSteps(steps) => {
                let items: String = steps
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        format!(
                            r#"<div class="step">
  <span class="step-num">{}</span>
  <span class="step-text">{}</span>
</div>"#,
                            i + 1,
                            Self::escape_html(s),
                        )
                    })
                    .collect();
                format!("<div class=\"process-steps\">{}</div>", items)
            }
        }
    }

    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    pub fn to_file(&self, path: &str) -> std::io::Result<()> {
        let p = std::path::Path::new(path);
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, self.render_html())?;
        std::fs::rename(&tmp, p)
    }
}

// ── PresentationBuilder ──

#[derive(Debug, Clone)]
pub struct PresentationBuilder {
    slides: Vec<Slide>,
    theme: String,
    title: String,
    author: Option<String>,
}

impl PresentationBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            slides: Vec::new(),
            theme: "minimal-white".to_string(),
            title: title.to_string(),
            author: None,
        }
    }

    pub fn theme(mut self, name: &str) -> Self {
        self.theme = name.to_string();
        self
    }

    pub fn author(mut self, name: &str) -> Self {
        self.author = Some(name.to_string());
        self
    }

    pub fn add_slide(mut self, slide: Slide) -> Self {
        self.slides.push(slide);
        self
    }

    pub fn build(self) -> HtmlPresentation {
        HtmlPresentation {
            slides: self.slides,
            theme: self.theme,
            title: self.title,
            author: self.author,
        }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_presentation() -> HtmlPresentation {
        let mut p = HtmlPresentation::new("Test Deck");
        p.slides.push(Slide::new(
            0,
            SlideLayout::Cover,
            "Hello",
            SlideContent::Text("Welcome".to_string()),
        ));
        p.slides.push(Slide::new(
            1,
            SlideLayout::Bullets,
            "Agenda",
            SlideContent::BulletList(vec!["Item A".to_string(), "Item B".to_string()]),
        ));
        p
    }

    #[test]
    fn test_render_with_tokens_matches_legacy() {
        let p = sample_presentation();
        let legacy = p.render_html();
        let token = p.render_with_design_tokens();

        // Both should produce valid HTML containing the slides
        assert!(legacy.contains("Hello"));
        assert!(token.contains("Hello"));
        assert!(legacy.contains("Item A"));
        assert!(token.contains("Item A"));
        assert!(legacy.contains("Test Deck"));
        assert!(token.contains("Test Deck"));

        // The token version should contain CSS variable syntax
        assert!(token.contains("--color-primary"));
        assert!(token.contains("--font-body"));

        // Both should have the same total slide count
        assert!(legacy.contains("2 / 2"));
        assert!(token.contains("2 / 2"));
    }

    #[test]
    fn test_render_with_tokens_fallback() {
        let mut p = HtmlPresentation::new("Fallback Test");
        p.theme = "nonexistent-theme".to_string();
        p.slides.push(Slide::new(
            0,
            SlideLayout::Cover,
            "Fallback",
            SlideContent::Text("Should use legacy".to_string()),
        ));

        let result = p.render_with_design_tokens();
        assert!(result.contains("Fallback"));
        assert!(result.contains("Fallback Test"));

        // When theme is unknown, render_with_design_tokens falls back to render_html
        // so the output should be identical
        let legacy = p.render_html();
        assert_eq!(result, legacy);
    }
}
