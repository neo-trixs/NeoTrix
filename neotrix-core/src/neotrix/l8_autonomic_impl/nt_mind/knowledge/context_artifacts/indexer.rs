use std::fs;
use std::path::{Path, PathBuf};

use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};

use super::store::{ArtifactBuilder, ArtifactStore};
use super::types::{ArtifactType, ArtifactsConfig};

pub struct ArtifactIndexer {
    config_path: PathBuf,
    store: ArtifactStore,
}

impl ArtifactIndexer {
    pub fn new(config_path: &Path) -> Self {
        Self {
            config_path: config_path.to_path_buf(),
            store: ArtifactStore::new(),
        }
    }

    pub fn build(&mut self) -> NeoTrixResult<usize> {
        let config_content = fs::read_to_string(&self.config_path).map_err(|e| {
            NeoTrixError::Config(format!(
                "Cannot read artifacts config {:?}: {}",
                self.config_path, e
            ))
        })?;

        let config: ArtifactsConfig = serde_json::from_str(&config_content)
            .map_err(|e| NeoTrixError::Serde(format!("Invalid artifacts config: {}", e)))?;

        let mut built = 0;
        for source in &config.sources {
            let path = if source.path.starts_with('~') {
                shellexpand::tilde(&source.path).to_string()
            } else if source.path.starts_with('/') {
                source.path.clone()
            } else {
                let parent = self.config_path.parent().unwrap_or(Path::new("."));
                parent.join(&source.path).to_string_lossy().to_string()
            };

            if !Path::new(&path).exists() {
                eprintln!("[warn] Artifact source path not found: {}", path);
                continue;
            }

            let artifact = if let Some(ref at) = source.artifact_type {
                let content = fs::read_to_string(&path)?;
                let atype = ArtifactType::parse_artifact_kind(at).unwrap_or(ArtifactType::ConfigFile);
                let mut art = match atype {
                    ArtifactType::DatabaseSchema => {
                        ArtifactBuilder::parse_sql(&source.name, &content, Some(&path))
                    }
                    ArtifactType::ApiSpec => {
                        ArtifactBuilder::parse_openapi_yaml(&source.name, &content, Some(&path))
                    }
                    ArtifactType::ArchitectureDoc => Some(ArtifactBuilder::parse_markdown(
                        &source.name,
                        &content,
                        Some(&path),
                    )),
                    ArtifactType::ConfigFile => Some(ArtifactBuilder::parse_config(
                        &source.name,
                        &content,
                        if path.ends_with(".toml") {
                            "toml"
                        } else {
                            "yaml"
                        },
                        Some(&path),
                    )),
                    ArtifactType::InfrastructureSpec => {
                        if path.ends_with(".md") {
                            Some(ArtifactBuilder::parse_markdown(
                                &source.name,
                                &content,
                                Some(&path),
                            ))
                        } else {
                            Some(ArtifactBuilder::parse_config(
                                &source.name,
                                &content,
                                "yaml",
                                Some(&path),
                            ))
                        }
                    }
                };
                if let Some(ref mut a) = art {
                    if let Some(ref tags) = source.tags {
                        for t in tags {
                            if !a.tags.contains(t) {
                                a.tags.push(t.clone());
                            }
                        }
                    }
                }
                art
            } else {
                ArtifactBuilder::build_from_file(&source.name, &path)
            };

            if let Some(mut art) = artifact {
                if let Some(ref tags) = source.tags {
                    for t in tags {
                        if !art.tags.contains(t) {
                            art.tags.push(t.clone());
                        }
                    }
                }
                self.store.store(art);
                built += 1;
            }
        }

        Ok(built)
    }

    pub fn store(&self) -> &ArtifactStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut ArtifactStore {
        &mut self.store
    }
}
