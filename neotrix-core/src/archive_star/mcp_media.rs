use super::mcp_bridge::{McpServerInfo, McpToolInfo, McpTransport, StarPulseMcpBridge};

pub struct MediaMcpConsumer {
    bridge: StarPulseMcpBridge,
    registrations: Vec<MediaMcpRegistration>,
}

#[derive(Debug, Clone)]
pub struct MediaMcpRegistration {
    pub modality: &'static str,
    pub server_name: String,
    pub tool_name: String,
    pub is_connected: bool,
}

impl MediaMcpConsumer {
    pub fn new() -> Self {
        let mut bridge = StarPulseMcpBridge::new();
        Self::register_default_servers(&mut bridge);
        Self {
            registrations: Self::default_registrations(),
            bridge,
        }
    }

    fn register_default_servers(bridge: &mut StarPulseMcpBridge) {
        bridge.register_server(McpServerInfo {
            name: "stable-diffusion".into(),
            transport: McpTransport::Stdio { command: "sd-mcp-server".into(), args: vec![] },
            tools: vec![
                McpToolInfo {
                    name: "txt2img".into(),
                    description: "Generate image from text prompt".into(),
                    input_schema: serde_json::json!({"type":"object","properties":{"prompt":{"type":"string"}}}),
                },
            ],
        });
        bridge.register_server(McpServerInfo {
            name: "voicebox".into(),
            transport: McpTransport::Stdio { command: "voicebox-mcp".into(), args: vec![] },
            tools: vec![
                McpToolInfo {
                    name: "tts".into(),
                    description: "Text-to-speech synthesis".into(),
                    input_schema: serde_json::json!({"type":"object","properties":{"text":{"type":"string"}}}),
                },
            ],
        });
        bridge.register_server(McpServerInfo {
            name: "openmontage".into(),
            transport: McpTransport::Stdio { command: "om-mcp".into(), args: vec![] },
            tools: vec![
                McpToolInfo {
                    name: "generate_video".into(),
                    description: "Generate video from description".into(),
                    input_schema: serde_json::json!({"type":"object","properties":{"prompt":{"type":"string"}}}),
                },
            ],
        });
    }

    fn default_registrations() -> Vec<MediaMcpRegistration> {
        vec![
            MediaMcpRegistration { modality: "image", server_name: "stable-diffusion".into(), tool_name: "txt2img".into(), is_connected: true },
            MediaMcpRegistration { modality: "audio", server_name: "voicebox".into(), tool_name: "tts".into(), is_connected: true },
            MediaMcpRegistration { modality: "video", server_name: "openmontage".into(), tool_name: "generate_video".into(), is_connected: true },
        ]
    }

    pub fn find_tool(&self, modality: &str) -> Option<&MediaMcpRegistration> {
        self.registrations.iter().find(|r| r.modality == modality && r.is_connected)
    }

    pub fn available_modalities(&self) -> Vec<&str> {
        self.registrations.iter()
            .filter(|r| r.is_connected)
            .map(|r| r.modality)
            .collect()
    }

    pub fn bridge(&self) -> &StarPulseMcpBridge {
        &self.bridge
    }

    pub fn bridge_mut(&mut self) -> &mut StarPulseMcpBridge {
        &mut self.bridge
    }

    pub fn add_registration(&mut self, server_name: &str, tool_name: &str, modality: &'static str) {
        self.bridge.register_server(McpServerInfo {
            name: server_name.into(),
            transport: McpTransport::Stdio { command: format!("{}-mcp", server_name), args: vec![] },
            tools: vec![McpToolInfo {
                name: tool_name.into(),
                description: format!("{} via {}", modality, server_name),
                input_schema: serde_json::json!({"type":"object","properties":{}}),
            }],
        });
        self.registrations.push(MediaMcpRegistration {
            modality,
            server_name: server_name.into(),
            tool_name: tool_name.into(),
            is_connected: true,
        });
    }
}

impl Default for MediaMcpConsumer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_consumer_new() {
        let consumer = MediaMcpConsumer::new();
        assert_eq!(consumer.bridge().server_count(), 3);
        assert_eq!(consumer.available_modalities().len(), 3);
    }

    #[test]
    fn test_find_tool_by_modality() {
        let consumer = MediaMcpConsumer::new();
        let image = consumer.find_tool("image").unwrap();
        assert_eq!(image.server_name, "stable-diffusion");
        assert_eq!(image.tool_name, "txt2img");
    }

    #[test]
    fn test_find_tool_unknown_modality() {
        let consumer = MediaMcpConsumer::new();
        assert!(consumer.find_tool("hologram").is_none());
    }

    #[test]
    fn test_add_registration() {
        let mut consumer = MediaMcpConsumer::new();
        consumer.add_registration("custom-gen", "create_3d", "model");
        assert_eq!(consumer.bridge().server_count(), 4);
        assert!(consumer.find_tool("model").is_some());
    }

    #[test]
    fn test_available_modalities_list() {
        let consumer = MediaMcpConsumer::new();
        let modalities = consumer.available_modalities();
        assert!(modalities.contains(&"image"));
        assert!(modalities.contains(&"audio"));
        assert!(modalities.contains(&"video"));
    }

    #[test]
    fn test_all_tools_accessible() {
        let consumer = MediaMcpConsumer::new();
        let tools = consumer.bridge().all_tools();
        assert_eq!(tools.len(), 3);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"txt2img"));
        assert!(names.contains(&"tts"));
        assert!(names.contains(&"generate_video"));
    }
}
