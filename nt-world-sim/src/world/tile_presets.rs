use super::tile::{Tile, TileType};

pub struct TilePresets;

impl TilePresets {
    pub fn grass() -> Tile { Tile::new(TileType::Grass) }
    pub fn dirt() -> Tile { Tile::new(TileType::Dirt) }
    pub fn path() -> Tile { Tile::new(TileType::Path) }
    pub fn water() -> Tile { Tile::new(TileType::Water) }
    pub fn tree() -> Tile { Tile::new(TileType::Tree) }
    pub fn rock() -> Tile { Tile::new(TileType::Rock) }
    pub fn farm_plot() -> Tile { Tile::new(TileType::Dirt) }
    pub fn watered_plot() -> Tile { Tile::new(TileType::WateredSoil) }
    pub fn tilled_plot() -> Tile { Tile::new(TileType::TilledSoil) }
    pub fn house() -> Tile { Tile::new(TileType::Door) }
    pub fn wall() -> Tile { Tile::new(TileType::Wall) }
    pub fn shop() -> Tile { Tile::new(TileType::ShopTile) }
    pub fn npc_spot() -> Tile { Tile::new(TileType::NPCSpot) }
    pub fn spawn() -> Tile { Tile::new(TileType::SpawnPoint) }
    pub fn exit() -> Tile { Tile::new(TileType::ExitPoint) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_return_correct_types() {
        assert_eq!(TilePresets::grass().tile_type, TileType::Grass);
        assert_eq!(TilePresets::dirt().tile_type, TileType::Dirt);
        assert_eq!(TilePresets::path().tile_type, TileType::Path);
        assert_eq!(TilePresets::water().tile_type, TileType::Water);
        assert_eq!(TilePresets::tree().tile_type, TileType::Tree);
        assert_eq!(TilePresets::rock().tile_type, TileType::Rock);
        assert_eq!(TilePresets::farm_plot().tile_type, TileType::Dirt);
        assert_eq!(TilePresets::watered_plot().tile_type, TileType::WateredSoil);
        assert_eq!(TilePresets::tilled_plot().tile_type, TileType::TilledSoil);
        assert_eq!(TilePresets::house().tile_type, TileType::Door);
        assert_eq!(TilePresets::wall().tile_type, TileType::Wall);
        assert_eq!(TilePresets::shop().tile_type, TileType::ShopTile);
        assert_eq!(TilePresets::npc_spot().tile_type, TileType::NPCSpot);
        assert_eq!(TilePresets::spawn().tile_type, TileType::SpawnPoint);
        assert_eq!(TilePresets::exit().tile_type, TileType::ExitPoint);
    }

    #[test]
    fn test_presets_walkability() {
        assert!(TilePresets::grass().walkable);
        assert!(TilePresets::path().walkable);
        assert!(!TilePresets::water().walkable);
        assert!(!TilePresets::wall().walkable);
    }
}
