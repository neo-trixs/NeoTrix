use crate::game::crafting::{CraftingRecipe, RecipeDatabase};

pub struct RecipeBook;

impl RecipeBook {
    pub fn all_recipes() -> Vec<CraftingRecipe> {
        vec![
            CraftingRecipe { id: 1, name: "Inspiration Gem".to_string(), ingredients: vec![(1001, 3), (1002, 2)], result: (2001, 1), category: "Gem".to_string(), required_level: 1 },
            CraftingRecipe { id: 2, name: "Logic Crystal".to_string(), ingredients: vec![(1003, 5)], result: (2002, 1), category: "Gem".to_string(), required_level: 3 },
            CraftingRecipe { id: 3, name: "Empathy Charm".to_string(), ingredients: vec![(1004, 4), (1006, 2)], result: (2003, 1), category: "Accessory".to_string(), required_level: 2 },
            CraftingRecipe { id: 4, name: "Memory Lantern".to_string(), ingredients: vec![(1006, 5), (2001, 1)], result: (2004, 1), category: "Tool".to_string(), required_level: 2 },
            CraftingRecipe { id: 5, name: "Focus Amulet".to_string(), ingredients: vec![(1007, 3), (2002, 1)], result: (2005, 1), category: "Accessory".to_string(), required_level: 4 },
            CraftingRecipe { id: 6, name: "Creativity Paint".to_string(), ingredients: vec![(1005, 3), (1002, 2)], result: (2006, 2), category: "Consumable".to_string(), required_level: 1 },
            CraftingRecipe { id: 7, name: "Wisdom Tome".to_string(), ingredients: vec![(1008, 1), (2002, 2), (2004, 1)], result: (2007, 1), category: "Structure".to_string(), required_level: 5 },
            CraftingRecipe { id: 8, name: "Thought Elixir".to_string(), ingredients: vec![(1001, 5), (1004, 3)], result: (2008, 1), category: "Consumable".to_string(), required_level: 1 },
            CraftingRecipe { id: 9, name: "Insight Prism".to_string(), ingredients: vec![(2001, 2), (2003, 2)], result: (2009, 1), category: "Gem".to_string(), required_level: 6 },
            CraftingRecipe { id: 10, name: "Consciousness Core".to_string(), ingredients: vec![(2007, 1), (2005, 1), (2009, 1)], result: (2010, 1), category: "Structure".to_string(), required_level: 10 },
        ]
    }

    pub fn register_all(db: &mut RecipeDatabase) {
        for recipe in Self::all_recipes() {
            db.register(recipe);
        }
    }
}
