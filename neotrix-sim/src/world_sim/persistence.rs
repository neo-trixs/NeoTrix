use serde::{Serialize, Deserialize};
use crate::world_sim::WorldSim;
use crate::world_sim::config::WorldSimConfig;
use crate::agents::sim_agent::Personality;

/// Full world snapshot for save/load (distinct from the summary `WorldSnapshot` in config.rs)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullWorldSnapshot {
    pub tick: u64,
    pub config: WorldSimConfig,
    pub agents: Vec<AgentSnapshot>,
    pub resources: Vec<ResourceSnapshot>,
    pub terrain_seed: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentSnapshot {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub alive: bool,
    pub energy: f32,
    pub health: f32,
    pub hunger: f32,
    pub age: u64,
    pub phi: f32,
    pub personality: Personality,
    pub needs: [f32; 5],
    pub skills: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResourceSnapshot {
    pub id: String,
    pub resource_type: String,
    pub position: (f32, f32),
    pub amount: f32,
    pub max_amount: f32,
    pub regeneration_rate: f32,
    pub depleted: bool,
}

/// Metadata about a save file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveInfo {
    pub name: String,
    pub path: String,
    pub tick: u64,
    pub timestamp_secs: u64,
    pub agent_count: usize,
    pub file_size_bytes: u64,
}

pub struct PersistenceManager {
    save_dir: String,
    auto_save_interval: u64,
    max_backups: usize,
}

impl PersistenceManager {
    pub fn new(save_dir: &str) -> Self {
        Self {
            save_dir: save_dir.to_string(),
            auto_save_interval: 1000,
            max_backups: 5,
        }
    }

    pub fn with_config(save_dir: &str, auto_save_interval: u64, max_backups: usize) -> Self {
        Self {
            save_dir: save_dir.to_string(),
            auto_save_interval,
            max_backups,
        }
    }

    /// Check if auto-save should trigger at the given tick
    pub fn should_auto_save(&self, tick: u64) -> bool {
        tick > 0 && tick % self.auto_save_interval == 0
    }

    /// Auto-save: saves and prunes old backups beyond max_backups
    pub fn auto_save(&self, sim: &WorldSim) -> Result<String, String> {
        if !self.should_auto_save(sim.tick) {
            return Ok(format!("Skipped: tick {} not at interval {}", sim.tick, self.auto_save_interval));
        }
        let path = self.save(sim)?;
        self.prune_backups()?;
        Ok(path)
    }

    /// List all save files with metadata
    pub fn list_saves(&self) -> Vec<SaveInfo> {
        let mut saves = Vec::new();
        let dir = std::fs::read_dir(&self.save_dir);
        if let Ok(entries) = dir {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json")
                    && path
                        .file_name()
                        .map_or(false, |n| n.to_string_lossy().starts_with("save_"))
                {
                    let metadata = std::fs::metadata(&path).ok();
                    let file_size = metadata.as_ref().map_or(0, |m| m.len());
                    let timestamp_secs = metadata
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map_or(0, |d| d.as_secs());

                    // Try to read tick from JSON
                    let tick = std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|json| {
                            let v: serde_json::Value = serde_json::from_str(&json).ok()?;
                            v.get("tick")?.as_u64()
                        })
                        .unwrap_or(0);

                    let agent_count = std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|json| {
                            let v: serde_json::Value = serde_json::from_str(&json).ok()?;
                            v.get("agents")?.as_array().map(|a| a.len())
                        })
                        .unwrap_or(0);

                    let name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    saves.push(SaveInfo {
                        name,
                        path: path.to_string_lossy().to_string(),
                        tick,
                        timestamp_secs,
                        agent_count,
                        file_size_bytes: file_size,
                    });
                }
            }
        }
        saves.sort_by(|a, b| b.tick.cmp(&a.tick));
        saves
    }

    /// Delete a save file by name (without extension)
    pub fn delete_save(&self, name: &str) -> Result<(), String> {
        let filename = format!("{}/{}.json", self.save_dir, name);
        if std::path::Path::new(&filename).exists() {
            std::fs::remove_file(&filename)
                .map_err(|e| format!("Delete error: {}", e))?;
            Ok(())
        } else {
            Err(format!("Save file '{}' not found", name))
        }
    }

    fn prune_backups(&self) -> Result<(), String> {
        let mut saves = self.list_saves();
        if saves.len() <= self.max_backups {
            return Ok(());
        }
        // Keep the most recent max_backups, delete the rest
        saves.sort_by(|a, b| b.tick.cmp(&a.tick));
        let to_delete = &saves[self.max_backups..];
        for save in to_delete {
            let _ = std::fs::remove_file(&save.path);
        }
        Ok(())
    }

    pub fn save(&self, sim: &WorldSim) -> Result<String, String> {
        let snapshot = self.create_snapshot(sim);
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| format!("Serialize error: {}", e))?;

        let filename = format!("{}/save_{}.json", self.save_dir, sim.tick);
        std::fs::create_dir_all(&self.save_dir)
            .map_err(|e| format!("Dir error: {}", e))?;
        std::fs::write(&filename, &json)
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(filename)
    }

    pub fn load(&self, filename: &str) -> Result<FullWorldSnapshot, String> {
        let json = std::fs::read_to_string(filename)
            .map_err(|e| format!("Read error: {}", e))?;

        serde_json::from_str(&json)
            .map_err(|e| format!("Deserialize error: {}", e))
    }

    pub fn load_latest(&self) -> Result<FullWorldSnapshot, String> {
        let entries: Vec<_> = std::fs::read_dir(&self.save_dir)
            .map_err(|e| format!("Dir error: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "json")
                    && e.path()
                        .file_name()
                        .map_or(false, |n| n.to_string_lossy().starts_with("save_"))
            })
            .collect();

        if entries.is_empty() {
            return Err("No save files found".to_string());
        }

        let mut latest = entries[0].path();
        for entry in &entries[1..] {
            let candidate = entry.path();
            if let (Ok(a_meta), Ok(b_meta)) =
                (candidate.metadata(), latest.metadata())
            {
                if let (Ok(a_time), Ok(b_time)) =
                    (a_meta.modified(), b_meta.modified())
                {
                    if a_time > b_time {
                        latest = candidate;
                    }
                }
            }
        }

        self.load(latest.to_str().unwrap_or_default())
    }

    fn create_snapshot(&self, sim: &WorldSim) -> FullWorldSnapshot {
        let agents = sim.agents.iter().map(|a| AgentSnapshot {
            id: a.core.id.clone(),
            x: a.core.position.x,
            y: a.core.position.y,
            alive: a.core.alive,
            energy: a.core.energy,
            health: a.core.health,
            hunger: a.core.hunger,
            age: a.core.age,
            phi: a.phi,
            personality: a.personality.clone(),
            needs: a.needs,
            skills: a.skills.clone(),
        }).collect();

        let resources = sim.resources.nodes.iter().map(|r| ResourceSnapshot {
            id: r.id.clone(),
            resource_type: format!("{:?}", r.resource_type),
            position: r.position,
            amount: r.amount,
            max_amount: r.max_amount,
            regeneration_rate: r.regeneration_rate,
            depleted: r.depleted,
        }).collect();

        FullWorldSnapshot {
            tick: sim.tick,
            config: sim.config.clone(),
            agents,
            resources,
            terrain_seed: sim.config.seed,
        }
    }

    pub fn restore_from_snapshot(&self, snapshot: FullWorldSnapshot) -> Result<WorldSim, String> {
        let mut sim = WorldSim::new(snapshot.config);
        sim.tick = snapshot.tick;

        for agent_snap in &snapshot.agents {
            if let Some(agent) = sim.agents.iter_mut().find(|a| a.core.id == agent_snap.id) {
                agent.core.position.x = agent_snap.x;
                agent.core.position.y = agent_snap.y;
                agent.core.alive = agent_snap.alive;
                agent.core.energy = agent_snap.energy;
                agent.core.health = agent_snap.health;
                agent.core.hunger = agent_snap.hunger;
                agent.core.age = agent_snap.age;
                agent.phi = agent_snap.phi;
                agent.personality = agent_snap.personality.clone();
                agent.needs = agent_snap.needs;
                agent.skills = agent_snap.skills.clone();
            }
        }

        for res_snap in &snapshot.resources {
            if let Some(node) = sim.resources.nodes.iter_mut().find(|n| n.id == res_snap.id) {
                node.amount = res_snap.amount;
                node.max_amount = res_snap.max_amount;
                node.regeneration_rate = res_snap.regeneration_rate;
                node.depleted = res_snap.depleted;
            }
        }

        Ok(sim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let config = WorldSimConfig::default();
        let sim = WorldSim::new(config);
        let manager = PersistenceManager::new("/tmp/neotrix_test");

        let snapshot = manager.create_snapshot(&sim);
        assert_eq!(snapshot.tick, 0);
        assert!(!snapshot.agents.is_empty());
        assert!(!snapshot.resources.is_empty());
        assert_eq!(snapshot.terrain_seed, 42);
    }

    #[test]
    fn test_save_load_cycle() {
        let config = WorldSimConfig::default();
        let mut sim = WorldSim::new(config);
        sim.tick = 10;
        if let Some(agent) = sim.agents.first_mut() {
            agent.core.energy = 75.0;
            agent.core.health = 80.0;
        }
        let manager = PersistenceManager::new("/tmp/neotrix_test");

        let _ = std::fs::create_dir_all("/tmp/neotrix_test");
        let filename = manager.save(&sim).unwrap();
        let loaded = manager.load(&filename).unwrap();

        assert_eq!(loaded.tick, 10);
        assert_eq!(loaded.agents.len(), sim.agents.len());
        assert_eq!(loaded.agents[0].energy, 75.0);
        assert_eq!(loaded.agents[0].health, 80.0);
    }

    #[test]
    fn test_restore_roundtrip() {
        let config = WorldSimConfig::default();
        let mut sim = WorldSim::new(config);
        sim.tick = 5;
        if let Some(agent) = sim.agents.first_mut() {
            agent.core.energy = 42.0;
            agent.personality.curiosity = 0.9;
        }
        let manager = PersistenceManager::new("/tmp/neotrix_test");

        let _ = std::fs::create_dir_all("/tmp/neotrix_test");
        let snapshot = manager.create_snapshot(&sim);
        let restored = manager.restore_from_snapshot(snapshot).unwrap();

        assert_eq!(restored.tick, 5);
        assert_eq!(restored.agents.len(), sim.agents.len());
        let orig_agent = &sim.agents[0];
        let rest_agent = restored.agents.iter().find(|a| a.core.id == orig_agent.core.id);
        assert!(rest_agent.is_some());
        let rest_agent = rest_agent.unwrap();
        assert_eq!(rest_agent.core.energy, 42.0);
        assert!((rest_agent.personality.curiosity - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_resource_snapshot_roundtrip() {
        let config = WorldSimConfig::default();
        let mut sim = WorldSim::new(config);
        if let Some(node) = sim.resources.nodes.first_mut() {
            node.amount = 25.0;
            node.depleted = true;
        }
        let manager = PersistenceManager::new("/tmp/neotrix_test");

        let _ = std::fs::create_dir_all("/tmp/neotrix_test");
        let snapshot = manager.create_snapshot(&sim);
        let json = serde_json::to_string(&snapshot).unwrap();
        let loaded: FullWorldSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.resources[0].amount, 25.0);
        assert!(loaded.resources[0].depleted);
    }

    #[test]
    fn test_auto_save_interval() {
        let manager = PersistenceManager::with_config("/tmp/neotrix_test", 100, 5);
        assert!(!manager.should_auto_save(0));
        assert!(!manager.should_auto_save(50));
        assert!(manager.should_auto_save(100));
        assert!(!manager.should_auto_save(150));
        assert!(manager.should_auto_save(200));
    }

    #[test]
    fn test_list_saves() {
        let config = WorldSimConfig::default();
        let sim = WorldSim::new(config);
        let manager = PersistenceManager::new("/tmp/neotrix_test_list");

        let _ = std::fs::create_dir_all("/tmp/neotrix_test_list");
        let _ = manager.save(&sim);
        let saves = manager.list_saves();
        assert!(!saves.is_empty());
        assert_eq!(saves[0].tick, 0);
    }

    #[test]
    fn test_delete_save() {
        let config = WorldSimConfig::default();
        let sim = WorldSim::new(config);
        let manager = PersistenceManager::new("/tmp/neotrix_test_delete");

        let _ = std::fs::create_dir_all("/tmp/neotrix_test_delete");
        let path = manager.save(&sim).unwrap();
        let filename = std::path::Path::new(&path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let result = manager.delete_save(&filename);
        assert!(result.is_ok());
        assert!(manager.list_saves().is_empty());
    }

    #[test]
    fn test_delete_nonexistent_save() {
        let manager = PersistenceManager::new("/tmp/neotrix_test_delete2");
        let _ = std::fs::create_dir_all("/tmp/neotrix_test_delete2");
        let result = manager.delete_save("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_info_metadata() {
        let config = WorldSimConfig::default();
        let mut sim = WorldSim::new(config);
        sim.tick = 42;
        let manager = PersistenceManager::new("/tmp/neotrix_test_meta");

        let _ = std::fs::create_dir_all("/tmp/neotrix_test_meta");
        let _ = manager.save(&sim);
        let saves = manager.list_saves();
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].tick, 42);
        assert!(saves[0].file_size_bytes > 0);
        assert!(saves[0].agent_count > 0);
    }
}
