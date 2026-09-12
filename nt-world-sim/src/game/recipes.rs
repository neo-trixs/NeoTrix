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
            // Utility items
            CraftingRecipe { id: 11, name: "Energy Tincture".to_string(), ingredients: vec![(1001, 3), (1006, 2)], result: (2011, 2), category: "Consumable".to_string(), required_level: 1 },
            CraftingRecipe { id: 12, name: "Focus Charm".to_string(), ingredients: vec![(1007, 2), (3006, 1)], result: (2012, 1), category: "Accessory".to_string(), required_level: 3 },
            CraftingRecipe { id: 13, name: "Empathy Pendant".to_string(), ingredients: vec![(1004, 4), (3004, 2)], result: (2013, 1), category: "Accessory".to_string(), required_level: 4 },
            CraftingRecipe { id: 14, name: "Creativity Palette".to_string(), ingredients: vec![(1005, 3), (1002, 3)], result: (2014, 1), category: "Tool".to_string(), required_level: 3 },
            CraftingRecipe { id: 15, name: "Memory Crystal".to_string(), ingredients: vec![(1006, 5), (3003, 2)], result: (2015, 1), category: "Gem".to_string(), required_level: 5 },
            // Machine recipes
            CraftingRecipe { id: 16, name: "Preserve Jar".to_string(), ingredients: vec![(3001, 5), (3002, 3)], result: (4101, 1), category: "Machine".to_string(), required_level: 2 },
            CraftingRecipe { id: 17, name: "Keg".to_string(), ingredients: vec![(3001, 8), (3005, 2)], result: (4102, 1), category: "Machine".to_string(), required_level: 4 },
            CraftingRecipe { id: 18, name: "Seed Maker".to_string(), ingredients: vec![(3001, 10), (3006, 3)], result: (4103, 1), category: "Machine".to_string(), required_level: 3 },
            // Decorative
            CraftingRecipe { id: 19, name: "Thought Lantern".to_string(), ingredients: vec![(3001, 3), (3004, 1)], result: (5001, 1), category: "Decorative".to_string(), required_level: 1 },
            CraftingRecipe { id: 20, name: "Wisdom Statue".to_string(), ingredients: vec![(3007, 5), (3003, 3)], result: (5002, 1), category: "Decorative".to_string(), required_level: 6 },
            // Quest items
            CraftingRecipe { id: 21, name: "Unity Key".to_string(), ingredients: vec![(2001, 2), (2002, 2), (2003, 2)], result: (6001, 1), category: "Quest".to_string(), required_level: 5 },
            CraftingRecipe { id: 22, name: "Consciousness Core".to_string(), ingredients: vec![(2007, 1), (2005, 1), (2009, 1)], result: (6002, 1), category: "Quest".to_string(), required_level: 10 },
            // Additional recipes
            CraftingRecipe { id: 23, name: "Mana Potion".to_string(), ingredients: vec![(1001, 2), (1003, 2), (1006, 1)], result: (2016, 2), category: "Consumable".to_string(), required_level: 2 },
            CraftingRecipe { id: 24, name: "Thought Quill".to_string(), ingredients: vec![(1002, 4), (3007, 2)], result: (2017, 1), category: "Tool".to_string(), required_level: 3 },
            CraftingRecipe { id: 25, name: "Insight Lens".to_string(), ingredients: vec![(1007, 3), (1003, 2), (3006, 1)], result: (2018, 1), category: "Accessory".to_string(), required_level: 5 },
            CraftingRecipe { id: 26, name: "Clarity Bomb".to_string(), ingredients: vec![(1001, 5), (1005, 3), (3002, 2)], result: (2019, 1), category: "Tool".to_string(), required_level: 4 },
            CraftingRecipe { id: 27, name: "Courage Potion".to_string(), ingredients: vec![(1004, 3), (1008, 1)], result: (2020, 2), category: "Consumable".to_string(), required_level: 2 },
            CraftingRecipe { id: 28, name: "Knowledge Shelf".to_string(), ingredients: vec![(3001, 6), (3003, 2), (3007, 1)], result: (5003, 1), category: "Decorative".to_string(), required_level: 3 },
            CraftingRecipe { id: 29, name: "Harmony Bell".to_string(), ingredients: vec![(3001, 4), (3005, 3)], result: (5004, 1), category: "Decorative".to_string(), required_level: 2 },
            CraftingRecipe { id: 30, name: "Enchanted Sarcophagus".to_string(), ingredients: vec![(3001, 12), (3004, 4), (3006, 2)], result: (4104, 1), category: "Machine".to_string(), required_level: 6 },
        ]
    }

    pub fn register_all(db: &mut RecipeDatabase) {
        for recipe in Self::all_recipes() {
            db.register(recipe);
        }
    }
}
