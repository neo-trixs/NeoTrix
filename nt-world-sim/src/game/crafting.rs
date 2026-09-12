use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CraftingRecipe {
    pub id: u32,
    pub name: String,
    pub ingredients: Vec<(u32, u32)>, // (item_id, quantity)
    pub result: (u32, u32),            // (item_id, quantity)
    pub category: String,
    pub required_level: u32,
}

pub struct CraftingRegistry {
    recipes: HashMap<u32, CraftingRecipe>,
}

impl CraftingRegistry {
    pub fn new() -> Self {
        let mut recipes = HashMap::new();

        recipes.insert(1, CraftingRecipe {
            id: 1,
            name: "Neural Pathway".to_string(),
            ingredients: vec![(101, 3), (102, 1)],
            result: (301, 1),
            category: "Structure".to_string(),
            required_level: 1,
        });

        recipes.insert(2, CraftingRecipe {
            id: 2,
            name: "Insight Elixir".to_string(),
            ingredients: vec![(103, 2), (104, 1)],
            result: (302, 1),
            category: "Consumable".to_string(),
            required_level: 2,
        });

        Self { recipes }
    }

    pub fn get(&self, id: u32) -> Option<&CraftingRecipe> {
        self.recipes.get(&id)
    }

    pub fn can_craft(&self, recipe_id: u32, inventory_counts: &HashMap<u32, u32>, skill_level: u32) -> bool {
        if let Some(recipe) = self.recipes.get(&recipe_id) {
            if skill_level < recipe.required_level {
                return false;
            }
            recipe.ingredients.iter().all(|&(item_id, qty)| {
                inventory_counts.get(&item_id).copied().unwrap_or(0) >= qty
            })
        } else {
            false
        }
    }

    pub fn craft(&self, recipe_id: u32, inventory_counts: &mut HashMap<u32, u32>, skill_level: u32) -> Option<(u32, u32)> {
        if let Some(recipe) = self.recipes.get(&recipe_id) {
            if !self.can_craft(recipe_id, inventory_counts, skill_level) {
                return None;
            }
            for &(item_id, qty) in &recipe.ingredients {
                if let Some(count) = inventory_counts.get_mut(&item_id) {
                    *count -= qty;
                }
            }
            Some(recipe.result)
        } else {
            None
        }
    }

    pub fn all_recipes(&self) -> impl Iterator<Item = &CraftingRecipe> {
        self.recipes.values()
    }

    pub fn recipe_count(&self) -> usize {
        self.recipes.len()
    }
}

pub struct RecipeDatabase {
    pub recipes: HashMap<u32, CraftingRecipe>,
}

impl RecipeDatabase {
    pub fn new() -> Self {
        Self { recipes: HashMap::new() }
    }

    pub fn register(&mut self, recipe: CraftingRecipe) {
        self.recipes.insert(recipe.id, recipe);
    }

    pub fn get(&self, id: u32) -> Option<&CraftingRecipe> {
        self.recipes.get(&id)
    }

    pub fn can_craft(&self, recipe: &CraftingRecipe, inventory: &super::inventory::Inventory) -> bool {
        recipe.ingredients.iter().all(|(item_id, qty)| inventory.count_item(*item_id) >= *qty)
    }

    pub fn recipes_for_skill(&self, _skill_name: &str, level: u32) -> Vec<&CraftingRecipe> {
        self.recipes.values().filter(|r| {
            r.required_level <= level
        }).collect()
    }

    pub fn recipe_count(&self) -> usize {
        self.recipes.len()
    }
}

impl Default for RecipeDatabase {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crafting() {
        let reg = CraftingRegistry::new();
        let mut counts = HashMap::new();
        counts.insert(101, 5);
        counts.insert(102, 3);

        assert!(reg.can_craft(1, &counts, 1));
        let result = reg.craft(1, &mut counts, 1).unwrap();
        assert_eq!(result, (301, 1));
        assert_eq!(counts[&101], 2);
    }
}
