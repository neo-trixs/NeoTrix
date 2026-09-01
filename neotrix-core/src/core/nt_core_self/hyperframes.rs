use std::collections::HashMap;
use std::time::Instant;

/// HyperFrames — HTML→视频渲染管线
///
/// 参考: GitHub hyperframes/hyperframes
/// 核心思想: HTML/CSS/JS 作为创作层 + 确定性渲染,
/// 支持 agent 自动生成视频管线。

/// 视频场景
#[derive(Debug, Clone)]
pub struct VideoScene {
    pub id: String,
    pub name: String,
    pub html_content: String,
    pub css_styles: String,
    pub javascript: String,
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

/// 视频项目
#[derive(Debug, Clone)]
pub struct VideoProject {
    pub id: String,
    pub name: String,
    pub scenes: Vec<VideoScene>,
    pub total_duration_ms: u64,
    pub created_at: Instant,
    pub status: ProjectStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectStatus {
    Draft,
    Rendering,
    Completed,
    Failed,
}

/// 渲染配置
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub output_format: OutputFormat,
    pub quality: RenderQuality,
    pub codec: VideoCodec,
    pub bitrate: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Mp4,
    WebM,
    Gif,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoCodec {
    H264,
    H265,
    VP9,
    AV1,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Mp4,
            quality: RenderQuality::High,
            codec: VideoCodec::H264,
            bitrate: 8000,
        }
    }
}

/// 渲染结果
#[derive(Debug, Clone)]
pub struct RenderResult {
    pub project_id: String,
    pub output_path: String,
    pub file_size_bytes: u64,
    pub render_time_ms: u64,
    pub frames_rendered: u32,
    pub status: RenderStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderStatus {
    Success,
    PartialSuccess,
    Failed,
}

/// 模板系统
pub struct TemplateSystem {
    templates: Vec<VideoTemplate>,
}

#[derive(Debug, Clone)]
pub struct VideoTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub html_template: String,
    pub css_template: String,
    pub placeholders: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateCategory {
    ProductLaunch,
    Tutorial,
    Presentation,
    SocialMedia,
    Custom,
}

impl TemplateSystem {
    pub fn new() -> Self {
        Self { templates: Vec::new() }
    }

    pub fn register(&mut self, template: VideoTemplate) {
        self.templates.push(template);
    }

    pub fn find_by_category(&self, category: &TemplateCategory) -> Vec<&VideoTemplate> {
        self.templates.iter().filter(|t| t.category == *category).collect()
    }

    pub fn render(&self, template_id: &str, data: &HashMap<String, String>) -> Option<(String, String)> {
        let template = self.templates.iter().find(|t| t.id == template_id)?;

        let mut html = template.html_template.clone();
        let mut css = template.css_template.clone();

        for (key, value) in data {
            let placeholder = format!("{{{{{}}}}}", key);
            html = html.replace(&placeholder, value);
            css = css.replace(&placeholder, value);
        }

        Some((html, css))
    }
}

/// 渲染引擎
pub struct RenderEngine {
    _config: RenderConfig,
    templates: TemplateSystem,
    projects: Vec<VideoProject>,
}

impl RenderEngine {
    pub fn new(config: RenderConfig) -> Self {
        Self {
            _config: config,
            templates: TemplateSystem::new(),
            projects: Vec::new(),
        }
    }

    /// 创建新项目
    pub fn create_project(&mut self, name: &str) -> String {
        let id = format!("proj_{}", self.projects.len());
        let project = VideoProject {
            id: id.clone(),
            name: name.to_string(),
            scenes: Vec::new(),
            total_duration_ms: 0,
            created_at: Instant::now(),
            status: ProjectStatus::Draft,
        };
        self.projects.push(project);
        id
    }

    /// 添加场景
    pub fn add_scene(&mut self, project_id: &str, scene: VideoScene) -> bool {
        if let Some(project) = self.projects.iter_mut().find(|p| p.id == project_id) {
            project.total_duration_ms += scene.duration_ms;
            project.scenes.push(scene);
            true
        } else {
            false
        }
    }

    /// 渲染项目
    pub fn render(&self, project_id: &str) -> Option<RenderResult> {
        let project = self.projects.iter().find(|p| p.id == project_id)?;

        // 模拟渲染过程
        let total_frames: u32 = project.scenes.iter()
            .map(|s| (s.duration_ms as f64 / 1000.0 * s.fps as f64) as u32)
            .sum();

        let render_time = total_frames as u64 * 10; // 模拟每帧 10ms

        Some(RenderResult {
            project_id: project_id.to_string(),
            output_path: format!("output/{}.{}", project_id, "mp4"),
            file_size_bytes: total_frames as u64 * 10000, // 模拟
            render_time_ms: render_time,
            frames_rendered: total_frames,
            status: RenderStatus::Success,
        })
    }

    /// 从模板创建场景
    pub fn create_scene_from_template(
        &self,
        template_id: &str,
        data: &HashMap<String, String>,
        duration_ms: u64,
    ) -> Option<VideoScene> {
        let (html, css) = self.templates.render(template_id, data)?;

        Some(VideoScene {
            id: format!("scene_{}", Instant::now().elapsed().as_millis()),
            name: "Template Scene".to_string(),
            html_content: html,
            css_styles: css,
            javascript: String::new(),
            duration_ms,
            width: 1920,
            height: 1080,
            fps: 30,
        })
    }

    /// 获取项目列表
    pub fn get_projects(&self) -> &[VideoProject] {
        &self.projects
    }

    /// 获取模板系统
    pub fn get_templates(&self) -> &TemplateSystem {
        &self.templates
    }

    /// 获取模板系统 (可变)
    pub fn get_templates_mut(&mut self) -> &mut TemplateSystem {
        &mut self.templates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scene(id: &str) -> VideoScene {
        VideoScene {
            id: id.to_string(),
            name: format!("Scene {}", id),
            html_content: "<div>Hello</div>".to_string(),
            css_styles: "div { color: red; }".to_string(),
            javascript: String::new(),
            duration_ms: 5000,
            width: 1920,
            height: 1080,
            fps: 30,
        }
    }

    #[test]
    fn test_project_creation() {
        let mut engine = RenderEngine::new(RenderConfig::default());
        let project_id = engine.create_project("Test Video");
        assert!(!project_id.is_empty());
        assert_eq!(engine.get_projects().len(), 1);
    }

    #[test]
    fn test_add_scene() {
        let mut engine = RenderEngine::new(RenderConfig::default());
        let project_id = engine.create_project("Test Video");
        let added = engine.add_scene(&project_id, make_scene("s1"));
        assert!(added);
        assert_eq!(engine.get_projects()[0].scenes.len(), 1);
    }

    #[test]
    fn test_render() {
        let mut engine = RenderEngine::new(RenderConfig::default());
        let project_id = engine.create_project("Test Video");
        engine.add_scene(&project_id, make_scene("s1"));

        let result = engine.render(&project_id);
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, RenderStatus::Success);
    }

    #[test]
    fn test_template_rendering() {
        let mut system = TemplateSystem::new();
        system.register(VideoTemplate {
            id: "tpl1".to_string(),
            name: "Product Launch".to_string(),
            description: "Template for product launches".to_string(),
            category: TemplateCategory::ProductLaunch,
            html_template: "<div class='product'>{{name}}</div>".to_string(),
            css_template: ".product { color: {{color}}; }".to_string(),
            placeholders: vec!["name".to_string(), "color".to_string()],
        });

        let mut data = HashMap::new();
        data.insert("name".to_string(), "NeoTrix".to_string());
        data.insert("color".to_string(), "gold".to_string());

        let result = system.render("tpl1", &data);
        assert!(result.is_some());
        let (html, css) = result.unwrap();
        assert!(html.contains("NeoTrix"));
        assert!(css.contains("gold"));
    }
}
