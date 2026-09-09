use super::offline_index::OfflineIndex;
use super::types::*;

pub struct OfflineDownloader {
    index: OfflineIndex,
}

impl OfflineDownloader {
    pub fn new() -> Self {
        Self {
            index: OfflineIndex::new(),
        }
    }

    pub async fn download(
        &mut self,
        item: &MediaItem,
        quality: Quality,
        sources: &[Box<dyn MediaSource>],
    ) -> Result<String, String> {
        if self.index.contains(&item.id) {
            return Ok(self.index.get(&item.id).unwrap().local_path.clone());
        }

        for source in sources {
            match source.play_url(item, quality).await {
                Ok(view_source) => {
                    let local_path = format!("/tmp/neotrix_offline/{}_{}", item.id, quality.label());
                    self.index.add(item.clone(), local_path.clone());
                    return Ok(local_path);
                }
                Err(_) => continue,
            }
        }
        Err(format!("No source available for item: {}", item.id))
    }

    pub fn is_available(&self, id: &str) -> bool {
        self.index.contains(id)
    }

    pub fn index(&self) -> &OfflineIndex {
        &self.index
    }

    pub fn index_mut(&mut self) -> &mut OfflineIndex {
        &mut self.index
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.index.remove(id).is_some()
    }

    pub fn count(&self) -> usize {
        self.index.len()
    }
}

impl Default for OfflineDownloader {
    fn default() -> Self {
        Self::new()
    }
}
