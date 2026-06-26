use crate::neotrix::nt_world_exploration::content::{
    Engagement, ExplorationSourceType, SourceContent,
};
use crate::neotrix::nt_world_exploration::source_trait::ExplorationSource;

/// A node in the accessibility tree
#[derive(Debug, Clone)]
pub struct UiaNode {
    pub id: u64,
    pub role: String,
    pub label: String,
    pub value: String,
    pub description: String,
    pub enabled: bool,
    pub visible: bool,
    pub bounds: Option<[f64; 4]>,
    pub children: Vec<UiaNode>,
}

impl UiaNode {
    pub fn new(id: u64, role: &str, label: &str) -> Self {
        Self {
            id,
            role: role.to_string(),
            label: label.to_string(),
            value: String::new(),
            description: String::new(),
            enabled: true,
            visible: true,
            bounds: None,
            children: Vec::new(),
        }
    }

    pub fn flatten(&self) -> Vec<&UiaNode> {
        let mut nodes = vec![self];
        for child in &self.children {
            nodes.extend(child.flatten());
        }
        nodes
    }

    pub fn text_content(&self) -> String {
        let mut text = String::new();
        if !self.label.is_empty() {
            text.push_str(&self.label);
            text.push(' ');
        }
        if !self.value.is_empty() {
            text.push_str(&self.value);
            text.push(' ');
        }
        for child in &self.children {
            text.push_str(&child.text_content());
        }
        text
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TreeProvider {
    AccessKit,
    OsScript,
    AXUIElement,
    Stub,
}

pub struct UiaTreeSource {
    name: &'static str,
    provider: TreeProvider,
    poll_interval_ms: u64,
    last_poll_ns: u64,
    last_tree: Option<UiaNode>,
    node_count: usize,
    total_scans: u64,
}

impl UiaTreeSource {
    pub fn new() -> Self {
        let provider = if cfg!(target_os = "macos") {
            TreeProvider::OsScript
        } else if cfg!(target_os = "windows") {
            TreeProvider::AccessKit
        } else {
            TreeProvider::Stub
        };
        Self {
            name: "uia_tree",
            provider,
            poll_interval_ms: 5000,
            last_poll_ns: 0,
            last_tree: None,
            node_count: 0,
            total_scans: 0,
        }
    }

    pub fn with_poll_interval(mut self, ms: u64) -> Self {
        self.poll_interval_ms = ms;
        self
    }

    pub fn current_tree(&self) -> Option<&UiaNode> {
        self.last_tree.as_ref()
    }

    pub fn scan(&mut self) -> Vec<UiaNode> {
        self.total_scans += 1;
        self.last_poll_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        match self.provider {
            TreeProvider::OsScript => self.scan_macos(),
            TreeProvider::AXUIElement => self.scan_axuielement(),
            TreeProvider::AccessKit => self.scan_accesskit(),
            TreeProvider::Stub => self.scan_stub(),
        }
    }

    fn sanitize_applescript_string(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
            .collect()
    }

    // TODO(platform): macOS-only (osascript), needs cfg(target_os = "macos") guard + Linux fallback
    fn scan_macos(&self) -> Vec<UiaNode> {
        let mut roots = Vec::new();
        if let Ok(output) = std::process::Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "System Events" to get name of every process whose visible is true"#)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for (i, app_name) in stdout.trim().split(", ").enumerate() {
                if app_name.is_empty() {
                    continue;
                }
                let safe_name = Self::sanitize_applescript_string(app_name);
                let mut win = UiaNode::new(i as u64, "window", &safe_name);
                if let Ok(win_output) = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(format!(
                        r#"tell application "System Events" to get name of first window of process "{}""#,
                        safe_name
                    ))
                    .output()
                {
                    let win_name = String::from_utf8_lossy(&win_output.stdout).trim().to_string();
                    if !win_name.is_empty() {
                        win.value = win_name;
                    }
                }
                if let Ok(ui_output) = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(format!(
                        r#"tell application "System Events" to get description of every UI element of first window of process "{}""#,
                        safe_name
                    ))
                    .output()
                {
                    let elements = String::from_utf8_lossy(&ui_output.stdout);
                    for (j, elem) in elements.trim().split(", ").enumerate() {
                        if elem.is_empty() || elem == "missing value" {
                            continue;
                        }
                        win.children.push(UiaNode::new((i * 1000 + j) as u64, "ui_element", elem));
                    }
                }
                roots.push(win);
            }
        }
        roots
    }

    #[allow(dead_code)]
    fn scan_axuielement(&self) -> Vec<UiaNode> {
        // axuielement crate was removed — this always falls back to stub.
        // Re-add the crate and this implementation if native AX API scanning is needed.
        self.scan_stub()
    }

    fn scan_stub(&self) -> Vec<UiaNode> {
        vec![UiaNode {
            id: 1,
            role: "window".into(),
            label: "NeoTrix".into(),
            value: "Consciousness Dashboard".into(),
            description: String::new(),
            enabled: true,
            visible: true,
            bounds: None,
            children: vec![
                UiaNode::new(2, "button", "Evolve"),
                UiaNode::new(3, "text", "Status: Running"),
                UiaNode::new(4, "list", "Active Threads: 7"),
            ],
        }]
    }

    fn scan_accesskit(&self) -> Vec<UiaNode> {
        self.scan_stub()
    }

    pub fn to_source_contents(&self, roots: &[UiaNode]) -> Vec<SourceContent> {
        roots
            .iter()
            .map(|root| {
                let text = root.text_content();
                let id = format!("uia_{}", root.id);
                let title = root.label.clone();
                SourceContent::new(id, text.clone(), ExplorationSourceType::System)
                    .with_title(title)
                    .with_meta("uia_role", root.role.clone())
                    .with_meta("uia_node_count", root.flatten().len().to_string())
                    .with_meta("uia_provider", format!("{:?}", self.provider))
                    .with_engagement(Engagement {
                        likes: 0,
                        shares: 0,
                        replies: 0,
                        views: None,
                    })
            })
            .collect()
    }
}

impl ExplorationSource for UiaTreeSource {
    fn name(&self) -> &'static str {
        self.name
    }

    fn confidence(&self) -> f64 {
        0.35
    }

    fn explore(&mut self) -> Result<Vec<SourceContent>, String> {
        let trees = self.scan();
        self.node_count = trees.iter().map(|t| t.flatten().len()).sum();
        let contents = self.to_source_contents(&trees);
        Ok(contents)
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn pending_count(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_building() {
        let tree = UiaNode::new(1, "window", "TestApp");
        assert_eq!(tree.role, "window");
        assert_eq!(tree.label, "TestApp");
        assert!(tree.enabled);
        assert!(tree.visible);
    }

    #[test]
    fn test_flatten() {
        let mut root = UiaNode::new(1, "window", "Root");
        let child = UiaNode::new(2, "button", "Click");
        let mut sub = UiaNode::new(3, "text", "Label");
        sub.children.push(UiaNode::new(4, "text", "Nested"));
        root.children.push(child);
        root.children.push(sub);
        let flat = root.flatten();
        assert_eq!(flat.len(), 4);
    }

    #[test]
    fn test_text_content() {
        let mut root = UiaNode::new(1, "window", "App");
        root.value = "v1.0".into();
        root.children.push(UiaNode::new(2, "button", "Go"));
        let text = root.text_content();
        assert!(text.contains("App"));
        assert!(text.contains("v1.0"));
        assert!(text.contains("Go"));
    }

    #[test]
    fn test_scan_stub() {
        let mut src = UiaTreeSource::new();
        let roots = src.scan();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].label, "NeoTrix");
        assert_eq!(roots[0].children.len(), 3);
    }

    #[test]
    fn test_to_source_contents() {
        let src = UiaTreeSource::new();
        let trees = src.scan_stub();
        let contents = src.to_source_contents(&trees);
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].source_type, ExplorationSourceType::System);
        assert!(contents[0].text.contains("NeoTrix"));
    }

    #[test]
    fn test_explore() {
        let mut src = UiaTreeSource::new();
        let result = src.explore();
        assert!(result.is_ok());
        let contents = result.unwrap();
        assert!(!contents.is_empty());
        assert_eq!(contents[0].source_type, ExplorationSourceType::System);
    }

    #[test]
    fn test_tracker_fields() {
        let mut src = UiaTreeSource::new();
        let _ = src.explore();
        assert!(src.total_scans >= 1);
        assert!(src.last_poll_ns > 0);
    }

    #[test]
    fn test_is_ready() {
        let src = UiaTreeSource::new();
        assert!(src.is_ready());
    }
}
