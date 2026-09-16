use crate::l5_cognition::nt_mind::nt_game::world::tilemap::{LayeredTileMap, MapLayer};

#[derive(Debug, Clone, Copy)]
pub enum MinimapMode {
    Full,
    FogOfWar,
    PlayerCentered,
}

pub struct MiniMap {
    pub width: u32,
    pub height: u32,
    pub zoom: f32,
    pub mode: MinimapMode,
    pub show_entities: bool,
    pub show_pois: bool,
}

impl MiniMap {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            zoom: 1.0,
            mode: MinimapMode::FogOfWar,
            show_entities: true,
            show_pois: true,
        }
    }

    pub fn world_to_mini(&self, wx: f64, wy: f64, cam_x: f64, cam_y: f64) -> (u32, u32) {
        let scale = self.zoom;
        let mx = ((wx - cam_x) * scale as f64 + self.width as f64 / 2.0) as i32;
        let my = ((wy - cam_y) * scale as f64 + self.height as f64 / 2.0) as i32;
        (
            mx.max(0).min(self.width as i32 - 1) as u32,
            my.max(0).min(self.height as i32 - 1) as u32,
        )
    }

    pub fn get_pixel_color(&self, map: &LayeredTileMap, x: usize, y: usize) -> [u8; 4] {
        map.get_tile(MapLayer::Terrain, x, y)
            .map_or([0, 0, 0, 255], |t| match t.tile_type {
                super::tilemap::TileType::Water => [30, 100, 200, 255],
                super::tilemap::TileType::Grass => [50, 160, 50, 255],
                super::tilemap::TileType::Tree => [20, 100, 20, 255],
                super::tilemap::TileType::Rock | super::tilemap::TileType::Wall => {
                    [120, 100, 80, 255]
                }
                super::tilemap::TileType::Sand => [200, 180, 100, 255],
                super::tilemap::TileType::Snow => [220, 220, 240, 255],
                _ => [60, 60, 60, 255],
            })
    }
}
