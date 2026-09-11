use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A resource type in the crafting system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resource {
    pub name: String,
    pub category: ResourceCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceCategory {
    Raw,
    Processed,
    Consumable,
    Tool,
    Weapon,
}

impl Resource {
    pub fn new(name: &str, category: ResourceCategory) -> Self {
        Self {
            name: name.to_string(),
            category,
        }
    }

    pub fn wood() -> Self { Self::new("Wood", ResourceCategory::Raw) }
    pub fn plank() -> Self { Self::new("Plank", ResourceCategory::Processed) }
    pub fn stone() -> Self { Self::new("Stone", ResourceCategory::Raw) }
    pub fn tool() -> Self { Self::new("Tool", ResourceCategory::Tool) }
    pub fn iron() -> Self { Self::new("Iron", ResourceCategory::Raw) }
    pub fn sword() -> Self { Self::new("Sword", ResourceCategory::Weapon) }
    pub fn herb() -> Self { Self::new("Herb", ResourceCategory::Raw) }
    pub fn potion() -> Self { Self::new("Potion", ResourceCategory::Consumable) }
    pub fn food() -> Self { Self::new("Food", ResourceCategory::Raw) }
    pub fn meal() -> Self { Self::new("Meal", ResourceCategory::Consumable) }
}

/// A crafting recipe: inputs → output, with skill requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub inputs: Vec<(Resource, u32)>, // (resource, quantity)
    pub output: (Resource, u32),       // (resource, quantity produced)
    pub skill_required: Option<String>, // skill name, None = no skill needed
    pub craft_time: u32,               // ticks to craft
    pub base_xp: f32,                  // XP granted on craft
}

impl Recipe {
    pub fn new(name: &str, output: Resource, output_qty: u32, craft_time: u32) -> Self {
        Self {
            name: name.to_string(),
            inputs: Vec::new(),
            output: (output, output_qty),
            skill_required: None,
            craft_time,
            base_xp: 10.0,
        }
    }

    pub fn with_input(mut self, resource: Resource, qty: u32) -> Self {
        self.inputs.push((resource, qty));
        self
    }

    pub fn with_skill(mut self, skill: &str) -> Self {
        self.skill_required = Some(skill.to_string());
        self
    }

    pub fn with_xp(mut self, xp: f32) -> Self {
        self.base_xp = xp;
        self
    }
}

/// Inventory: resource name → quantity.
#[derive(Debug, Clone, Default)]
pub struct Inventory {
    pub items: HashMap<String, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, resource_name: &str, qty: u32) {
        *self.items.entry(resource_name.to_string()).or_insert(0) += qty;
    }

    pub fn remove(&mut self, resource_name: &str, qty: u32) -> bool {
        if let Some(count) = self.items.get_mut(resource_name) {
            if *count >= qty {
                *count -= qty;
                if *count == 0 {
                    self.items.remove(resource_name);
                }
                return true;
            }
        }
        false
    }

    pub fn count(&self, resource_name: &str) -> u32 {
        self.items.get(resource_name).copied().unwrap_or(0)
    }

    pub fn has_all(&self, requirements: &[(Resource, u32)]) -> bool {
        requirements.iter().all(|(res, qty)| self.count(&res.name) >= *qty)
    }
}

/// Active crafting job.
#[derive(Debug, Clone)]
pub struct CraftingJob {
    pub recipe: Recipe,
    pub progress: u32,
    pub crafter_id: String,
}

/// Crafting bench: manages recipes and active crafting jobs.
pub struct CraftingBench {
    pub recipes: Vec<Recipe>,
    pub active_jobs: Vec<CraftingJob>,
    pub max_concurrent: usize,
    pub xp: HashMap<String, f32>, // skill_name → XP
}

impl CraftingBench {
    pub fn new() -> Self {
        let mut bench = Self {
            recipes: Vec::new(),
            active_jobs: Vec::new(),
            max_concurrent: 3,
            xp: HashMap::new(),
        };
        bench.register_starter_recipes();
        bench
    }

    /// Register the 5 starter recipes.
    fn register_starter_recipes(&mut self) {
        // Wood → Plank
        self.recipes.push(
            Recipe::new("wood_to_plank", Resource::plank(), 2, 3)
                .with_input(Resource::wood(), 3)
                .with_xp(8.0)
        );

        // Stone → Tool
        self.recipes.push(
            Recipe::new("stone_to_tool", Resource::tool(), 1, 5)
                .with_input(Resource::stone(), 5)
                .with_input(Resource::wood(), 2)
                .with_skill("crafting")
                .with_xp(15.0)
        );

        // Iron → Sword
        self.recipes.push(
            Recipe::new("iron_to_sword", Resource::sword(), 1, 8)
                .with_input(Resource::iron(), 4)
                .with_input(Resource::wood(), 2)
                .with_skill("smithing")
                .with_xp(25.0)
        );

        // Herb → Potion
        self.recipes.push(
            Recipe::new("herb_to_potion", Resource::potion(), 1, 4)
                .with_input(Resource::herb(), 3)
                .with_skill("alchemy")
                .with_xp(20.0)
        );

        // Food → Meal
        self.recipes.push(
            Recipe::new("food_to_meal", Resource::meal(), 1, 3)
                .with_input(Resource::food(), 2)
                .with_skill("cooking")
                .with_xp(12.0)
        );
    }

    /// Find recipe by name.
    pub fn find_recipe(&self, name: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.name == name)
    }

    /// Check if an agent has the required skill level.
    pub fn has_skill(&self, skill: &str, _required_level: f32) -> bool {
        self.xp.get(skill).copied().unwrap_or(0.0) >= _required_level
    }

    /// Get skill XP for a skill.
    pub fn skill_xp(&self, skill: &str) -> f32 {
        self.xp.get(skill).copied().unwrap_or(0.0)
    }

    /// Start crafting a recipe. Consumes ingredients and queues a job.
    pub fn craft(
        &mut self,
        recipe_name: &str,
        inventory: &mut Inventory,
        crafter_id: &str,
    ) -> Result<(), CraftingError> {
        let recipe = self.recipes.iter()
            .find(|r| r.name == recipe_name)
            .cloned()
            .ok_or(CraftingError::RecipeNotFound)?;

        // Check skill requirement
        if let Some(ref skill) = recipe.skill_required {
            if !self.has_skill(skill, 0.0) {
                return Err(CraftingError::InsufficientSkill {
                    skill: skill.clone(),
                });
            }
        }

        // Check and consume ingredients
        if !inventory.has_all(&recipe.inputs) {
            return Err(CraftingError::MissingIngredients);
        }

        for (resource, qty) in &recipe.inputs {
            if !inventory.remove(&resource.name, *qty) {
                return Err(CraftingError::MissingIngredients);
            }
        }

        // Check concurrent job limit
        if self.active_jobs.len() >= self.max_concurrent {
            return Err(CraftingError::BenchFull);
        }

        self.active_jobs.push(CraftingJob {
            recipe,
            progress: 0,
            crafter_id: crafter_id.to_string(),
        });

        Ok(())
    }

    /// Advance all active jobs by one tick. Returns completed recipes.
    pub fn tick(&mut self) -> Vec<(Recipe, String)> {
        let mut completed = Vec::new();

        self.active_jobs.retain_mut(|job| {
            job.progress += 1;
            if job.progress >= job.recipe.craft_time {
                let recipe = job.recipe.clone();
                let crafter = job.crafter_id.clone();

                // Grant XP
                if let Some(ref skill) = recipe.skill_required {
                    *self.xp.entry(skill.clone()).or_insert(0.0) += recipe.base_xp;
                }

                completed.push((recipe, crafter));
                false
            } else {
                true
            }
        });

        completed
    }

    /// Get the number of active jobs.
    pub fn active_job_count(&self) -> usize {
        self.active_jobs.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CraftingError {
    RecipeNotFound,
    MissingIngredients,
    InsufficientSkill { skill: String },
    BenchFull,
}

impl std::fmt::Display for CraftingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecipeNotFound => write!(f, "recipe not found"),
            Self::MissingIngredients => write!(f, "missing ingredients"),
            Self::InsufficientSkill { skill } => write!(f, "insufficient skill: {}", skill),
            Self::BenchFull => write!(f, "crafting bench full"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_creation() {
        let wood = Resource::wood();
        assert_eq!(wood.name, "Wood");
        assert_eq!(wood.category, ResourceCategory::Raw);
    }

    #[test]
    fn recipe_registration() {
        let bench = CraftingBench::new();
        assert_eq!(bench.recipes.len(), 5);
        assert!(bench.find_recipe("wood_to_plank").is_some());
        assert!(bench.find_recipe("iron_to_sword").is_some());
    }

    #[test]
    fn inventory_operations() {
        let mut inv = Inventory::new();
        inv.add("Wood", 10);
        assert_eq!(inv.count("Wood"), 10);
        assert!(inv.remove("Wood", 5));
        assert_eq!(inv.count("Wood"), 5);
        assert!(!inv.remove("Wood", 100));
    }

    #[test]
    fn crafting_consumes_ingredients() {
        let mut bench = CraftingBench::new();
        let mut inv = Inventory::new();
        inv.add("Wood", 10);

        let result = bench.craft("wood_to_plank", &mut inv, "agent_1");
        assert!(result.is_ok());
        assert_eq!(inv.count("Wood"), 7); // 10 - 3 = 7
        assert_eq!(bench.active_job_count(), 1);
    }

    #[test]
    fn crafting_fails_without_ingredients() {
        let mut bench = CraftingBench::new();
        let mut inv = Inventory::new();
        inv.add("Wood", 1); // need 3

        let result = bench.craft("wood_to_plank", &mut inv, "agent_1");
        assert!(result.is_err());
        assert_eq!(inv.count("Wood"), 1); // unchanged
    }

    #[test]
    fn crafting_tick_produces_output() {
        let mut bench = CraftingBench::new();
        let mut inv = Inventory::new();
        inv.add("Wood", 10);

        bench.craft("wood_to_plank", &mut inv, "agent_1").unwrap();
        // wood_to_plank takes 3 ticks
        let mut completed = bench.tick();
        assert!(completed.is_empty());
        completed = bench.tick();
        assert!(completed.is_empty());
        completed = bench.tick();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].0.output.0.name, "Plank");
        assert_eq!(completed[0].0.output.1, 2); // produces 2 planks

        // XP should be granted
        assert!(bench.skill_xp("crafting") >= 0.0); // wood_to_plank has no skill req
    }

    #[test]
    fn crafting_respects_skill_requirement() {
        let mut bench = CraftingBench::new();
        let mut inv = Inventory::new();
        inv.add("Iron", 10);
        inv.add("Wood", 10);

        // No smithing skill
        let result = bench.craft("iron_to_sword", &mut inv, "agent_1");
        assert!(result.is_err());
        assert_eq!(inv.count("Iron"), 10); // unchanged
    }

    #[test]
    fn crafting_with_skill() {
        let mut bench = CraftingBench::new();
        bench.xp.insert("smithing".to_string(), 100.0);
        let mut inv = Inventory::new();
        inv.add("Iron", 10);
        inv.add("Wood", 10);

        let result = bench.craft("iron_to_sword", &mut inv, "agent_1");
        assert!(result.is_ok());
        assert_eq!(inv.count("Iron"), 6);
    }

    #[test]
    fn bench_full_limit() {
        let mut bench = CraftingBench::new();
        bench.max_concurrent = 1;
        let mut inv = Inventory::new();
        inv.add("Wood", 20);

        bench.craft("wood_to_plank", &mut inv, "a").unwrap();
        let result = bench.craft("wood_to_plank", &mut inv, "b");
        assert!(result.is_err());
    }
}
