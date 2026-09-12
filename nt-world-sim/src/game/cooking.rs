use crate::core::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CookingRecipe {
    pub id: u32,
    pub name: String,
    pub ingredients: Vec<(u32, u32)>,
    pub result_id: u32,
    pub result_quantity: u32,
    pub energy_restore: u32,
    pub skill_bonus: Option<(String, f32)>,
}

#[derive(Debug, Clone)]
pub struct CookingSystem {
    pub recipes: HashMap<u32, CookingRecipe>,
    pub cooked_count: HashMap<u32, u32>,
}

impl CookingSystem {
    pub fn new() -> Self {
        let mut system = Self { recipes: HashMap::new(), cooked_count: HashMap::new() };
        system.register_default_recipes();
        system
    }

    fn register_default_recipes(&mut self) {
        let recipes = vec![
            CookingRecipe { id: 7001, name: "Thought Soup".to_string(), ingredients: vec![(1001, 2), (3001, 1)], result_id: 7101, result_quantity: 1, energy_restore: 30, skill_bonus: None },
            CookingRecipe { id: 7002, name: "Curiosity Salad".to_string(), ingredients: vec![(1002, 2), (1004, 1)], result_id: 7102, result_quantity: 1, energy_restore: 40, skill_bonus: Some(("Awareness".to_string(), 0.1)) },
            CookingRecipe { id: 7003, name: "Logic Stew".to_string(), ingredients: vec![(1003, 2), (3002, 2)], result_id: 7103, result_quantity: 1, energy_restore: 50, skill_bonus: Some(("Logic".to_string(), 0.15)) },
            CookingRecipe { id: 7004, name: "Empathy Tea".to_string(), ingredients: vec![(1004, 3), (3004, 1)], result_id: 7104, result_quantity: 2, energy_restore: 25, skill_bonus: Some(("Empathy".to_string(), 0.1)) },
            CookingRecipe { id: 7005, name: "Creativity Cake".to_string(), ingredients: vec![(1005, 2), (1001, 2), (3001, 3)], result_id: 7105, result_quantity: 1, energy_restore: 60, skill_bonus: Some(("Creativity".to_string(), 0.2)) },
            CookingRecipe { id: 7006, name: "Memory Jam".to_string(), ingredients: vec![(1006, 4)], result_id: 7106, result_quantity: 2, energy_restore: 20, skill_bonus: None },
            CookingRecipe { id: 7007, name: "Focus Elixir".to_string(), ingredients: vec![(1007, 2), (3006, 2)], result_id: 7107, result_quantity: 1, energy_restore: 45, skill_bonus: Some(("Focus".to_string(), 0.15)) },
            CookingRecipe { id: 7008, name: "Wisdom Feast".to_string(), ingredients: vec![(1008, 1), (1001, 3), (1002, 2)], result_id: 7108, result_quantity: 1, energy_restore: 100, skill_bonus: Some(("All".to_string(), 0.05)) },
        ];
        for recipe in recipes {
            self.recipes.insert(recipe.id, recipe);
        }
    }

    pub fn can_cook(&self, recipe: &CookingRecipe, inventory: &super::inventory::Inventory) -> bool {
        recipe.ingredients.iter().all(|(id, qty)| inventory.count_item(*id) >= *qty)
    }

    pub fn cook(&mut self, recipe_id: u32, inventory: &mut super::inventory::Inventory) -> Option<(u32, u32, u32)> {
        if let Some(recipe) = self.recipes.get(&recipe_id) {
            if self.can_cook(recipe, inventory) {
                for (id, qty) in &recipe.ingredients {
                    inventory.remove_item(*id, *qty);
                }
                *self.cooked_count.entry(recipe_id).or_insert(0) += 1;
                return Some((recipe.result_id, recipe.result_quantity, recipe.energy_restore));
            }
        }
        None
    }

    pub fn all_recipes(&self) -> Vec<&CookingRecipe> {
        self.recipes.values().collect()
    }

    pub fn get_recipe(&self, id: u32) -> Option<&CookingRecipe> {
        self.recipes.get(&id)
    }
}

impl Default for CookingSystem {
    fn default() -> Self { Self::new() }
}

impl Resource for CookingSystem {}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::inventory::Inventory;
    use super::super::item::ItemQuality;

    #[test]
    fn test_cooking_system_recipes_loaded() {
        let system = CookingSystem::new();
        assert_eq!(system.recipes.len(), 8);
        assert!(system.recipes.contains_key(&7001));
        assert!(system.recipes.contains_key(&7008));
    }

    #[test]
    fn test_can_cook_insufficient_ingredients() {
        let system = CookingSystem::new();
        let inv = Inventory::new(20, 12);
        let recipe = system.recipes.get(&7001).unwrap();
        assert!(!system.can_cook(recipe, &inv));
    }

    #[test]
    fn test_cook_success() {
        let mut system = CookingSystem::new();
        let mut inv = Inventory::new(20, 12);
        inv.add_item(1001, 2, ItemQuality::Normal);
        inv.add_item(3001, 1, ItemQuality::Normal);

        let result = system.cook(7001, &mut inv);
        assert!(result.is_some());
        let (result_id, qty, energy) = result.unwrap();
        assert_eq!(result_id, 7101);
        assert_eq!(qty, 1);
        assert_eq!(energy, 30);
        assert_eq!(inv.count_item(1001), 0);
        assert_eq!(inv.count_item(3001), 0);
        assert_eq!(*system.cooked_count.get(&7001).unwrap(), 1);
    }

    #[test]
    fn test_cook_recipe_not_found() {
        let mut system = CookingSystem::new();
        let mut inv = Inventory::new(20, 12);
        assert!(system.cook(9999, &mut inv).is_none());
    }

    #[test]
    fn test_all_recipes() {
        let system = CookingSystem::new();
        let recipes = system.all_recipes();
        assert_eq!(recipes.len(), 8);
    }
}
