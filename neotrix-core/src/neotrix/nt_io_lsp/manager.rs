use std::collections::HashMap;
use std::path::PathBuf;

use super::LspClient;
use crate::neotrix::nt_io_lsp::LspError;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub language_id: String,
}

pub struct LspServerManager {
    servers: HashMap<String, ServerConfig>,
    active_servers: HashMap<String, LspClient>,
    workspace_root: Option<PathBuf>,
}

impl Default for LspServerManager {
    fn default() -> Self {
        let mut servers = HashMap::new();
        servers.insert(
            "rust".into(),
            ServerConfig {
                command: "rust-analyzer".into(),
                args: vec![],
                language_id: "rust".into(),
            },
        );
        servers.insert(
            "typescript".into(),
            ServerConfig {
                command: "typescript-language-server".into(),
                args: vec!["--stdio".into()],
                language_id: "typescript".into(),
            },
        );
        servers.insert(
            "javascript".into(),
            ServerConfig {
                command: "typescript-language-server".into(),
                args: vec!["--stdio".into()],
                language_id: "javascript".into(),
            },
        );
        servers.insert(
            "python".into(),
            ServerConfig {
                command: "pylsp".into(),
                args: vec![],
                language_id: "python".into(),
            },
        );
        Self {
            servers,
            active_servers: HashMap::new(),
            workspace_root: None,
        }
    }
}

impl LspServerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_workspace_root(mut self, root: PathBuf) -> Self {
        self.workspace_root = Some(root);
        self
    }

    pub fn register_server(&mut self, lang: &str, config: ServerConfig) {
        self.servers.insert(lang.to_string(), config);
    }

    pub fn unregister_server(&mut self, lang: &str) {
        self.servers.remove(lang);
    }

    pub fn detect_workspace_root() -> Option<PathBuf> {
        std::env::current_dir().ok()
    }

    pub fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_root = Some(root);
    }

    pub fn workspace_root(&self) -> Option<&PathBuf> {
        self.workspace_root.as_ref()
    }

    pub async fn spawn_for_lang(
        &mut self,
        lang: &str,
    ) -> Result<&mut LspClient, LspError> {
        if self.active_servers.contains_key(lang) {
            return Ok(self.active_servers.get_mut(lang).unwrap());
        }
        let config = self
            .servers
            .get(lang)
            .ok_or_else(|| LspError::ConnectionFailed(format!("no server config for language: {}", lang)))?;

        let root = self
            .workspace_root
            .clone()
            .or_else(Self::detect_workspace_root)
            .unwrap_or_else(|| PathBuf::from("."));

        let root_uri = format!("file://{}", root.display());

        let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
        let mut client =
            LspClient::spawn(&config.command, &args, &root_uri, &config.language_id).await?;
        client.initialize().await?;

        self.active_servers.insert(lang.to_string(), client);
        Ok(self.active_servers.get_mut(lang).unwrap())
    }

    pub async fn initialize_lang(&mut self, lang: &str) -> Result<(), LspError> {
        let client = self
            .active_servers
            .get_mut(lang)
            .ok_or_else(|| LspError::NotInitialized)?;
        client.initialize().await?;
        Ok(())
    }

    pub fn get_client(&mut self, lang: &str) -> Option<&mut LspClient> {
        self.active_servers.get_mut(lang)
    }

    pub fn has_client(&self, lang: &str) -> bool {
        self.active_servers.contains_key(lang)
    }

    pub fn active_languages(&self) -> Vec<String> {
        self.active_servers.keys().cloned().collect()
    }

    pub fn active_count(&self) -> usize {
        self.active_servers.len()
    }

    pub async fn kill(&mut self, lang: &str) -> Result<(), LspError> {
        if let Some(mut client) = self.active_servers.remove(lang) {
            client.shutdown().await
        } else {
            Ok(())
        }
    }

    pub async fn kill_all(&mut self) -> Vec<(String, Result<(), LspError>)> {
        let langs: Vec<String> = self.active_servers.keys().cloned().collect();
        let mut results = Vec::with_capacity(langs.len());
        for lang in langs {
            let result = self.kill(&lang).await;
            results.push((lang, result));
        }
        results
    }
}
