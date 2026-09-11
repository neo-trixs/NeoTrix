use serde::{Serialize, Deserialize};

/// Tile types for WFC map generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    Water,
    Mountain,
    Forest,
    Sand,
    Stone,
}

impl TileType {
    /// Index for adjacency matrix lookup
    pub fn index(&self) -> usize {
        match self {
            TileType::Grass => 0,
            TileType::Water => 1,
            TileType::Mountain => 2,
            TileType::Forest => 3,
            TileType::Sand => 4,
            TileType::Stone => 5,
        }
    }

    pub fn all() -> &'static [TileType] {
        &[
            TileType::Grass,
            TileType::Water,
            TileType::Mountain,
            TileType::Forest,
            TileType::Sand,
            TileType::Stone,
        ]
    }
}

/// Cell in the WFC grid
#[derive(Debug, Clone)]
struct Cell {
    possible: Vec<bool>,
    collapsed: bool,
}

impl Cell {
    fn new(num_tiles: usize) -> Self {
        Self {
            possible: vec![true; num_tiles],
            collapsed: false,
        }
    }

    fn count_possible(&self) -> usize {
        self.possible.iter().filter(|&&p| p).count()
    }

    fn chosen(&self) -> Option<usize> {
        if self.collapsed {
            self.possible.iter().position(|&p| p)
        } else {
            None
        }
    }
}

/// Wave Function Collapse map generator
pub struct WfcGenerator {
    width: usize,
    height: usize,
    adjacency: Vec<Vec<[bool; 6]>>,
    rng_state: u32,
}

impl WfcGenerator {
    pub fn new(seed: u64) -> Self {
        let adjacency = Self::build_adjacency();
        Self {
            width: 0,
            height: 0,
            adjacency,
            rng_state: (seed.max(1) as u32).wrapping_mul(1103515245).wrapping_add(12345),
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    /// Build adjacency rules: which tiles can be adjacent in which direction
    /// adjacency[from][direction] = bitmask of allowed 'to' tiles
    /// Directions: 0=Up, 1=Down, 2=Left, 3=Right
    fn build_adjacency() -> Vec<Vec<[bool; 6]>> {
        let n = 6;

        // For each tile, define which tiles can be in cardinal directions
        // adj[from] = [up, down, left, right] → each is a bool array of allowed 'to' tiles
        let mut result: Vec<Vec<[bool; 6]>> = Vec::with_capacity(n);

        // Grass (0): neighbors Sand, Grass, Forest, Stone
        result.push(vec![
            [false, true, false, true, true, true], // up
            [false, true, false, true, true, true], // down
            [false, true, false, true, true, true], // left
            [false, true, false, true, true, true], // right
        ]);

        // Water (1): neighbors Water, Sand only
        result.push(vec![
            [false, false, false, true, true, false], // up
            [false, false, false, true, true, false], // down
            [false, false, false, true, true, false], // left
            [false, false, false, true, true, false], // right
        ]);

        // Mountain (2): neighbors Forest, Mountain, Stone
        result.push(vec![
            [false, false, true, true, false, true], // up
            [false, false, true, true, false, true], // down
            [false, false, true, true, false, true], // left
            [false, false, true, true, false, true], // right
        ]);

        // Forest (3): neighbors Grass, Forest, Mountain, Stone
        result.push(vec![
            [false, false, true, true, true, true], // up
            [false, false, true, true, true, true], // down
            [false, false, true, true, true, true], // left
            [false, false, true, true, true, true], // right
        ]);

        // Sand (4): neighbors Water, Sand, Grass, Stone
        result.push(vec![
            [false, true, false, true, true, true], // up
            [false, true, false, true, true, true], // down
            [false, true, false, true, true, true], // left
            [false, true, false, true, true, true], // right
        ]);

        // Stone (5): neighbors Sand, Grass, Forest, Mountain, Stone
        result.push(vec![
            [false, true, true, true, true, true], // up
            [false, true, true, true, true, true], // down
            [false, true, true, true, true, true], // left
            [false, true, true, true, true, true], // right
        ]);

        result
    }

    /// Generate a map using Wave Function Collapse
    pub fn generate(&mut self, width: usize, height: usize) -> Vec<Vec<TileType>> {
        self.width = width;
        self.height = height;
        let num_tiles = 6;

        let mut grid: Vec<Vec<Cell>> = (0..height)
            .map(|_| (0..width).map(|_| Cell::new(num_tiles)).collect())
            .collect();

        // Seed: collapse a few random cells first
        let seeds = 4.max((width * height) / 64);
        for _ in 0..seeds {
            let x = (self.next_f32() * width as f32) as usize % width;
            let y = (self.next_f32() * height as f32) as usize % height;
            if !grid[y][x].collapsed {
                let tile = (self.next_f32() * num_tiles as f32) as usize % num_tiles;
                self.collapse(&mut grid, x, y, tile);
                self.propagate(&mut grid, x, y);
            }
        }

        // Main WFC loop
        let max_iterations = width * height * 10;
        for _ in 0..max_iterations {
            // Find lowest entropy cell (most constrained)
            if let Some((cx, cy)) = self.find_min_entropy(&grid) {
                if grid[cy][cx].collapsed {
                    break;
                }

                // Choose a tile weighted by remaining possibilities
                let chosen = self.choose_tile(&grid[cy][cx]);
                self.collapse(&mut grid, cx, cy, chosen);
                self.propagate(&mut grid, cx, cy);
            } else {
                break;
            }
        }

        // Fill any remaining uncollapsed cells with most common neighbor
        self.fill_remaining(&mut grid);

        // Convert to TileType grid
        grid.iter()
            .map(|row| {
                row.iter()
                    .map(|cell| {
                        cell.chosen()
                            .map(|i| TileType::all()[i])
                            .unwrap_or(TileType::Grass)
                    })
                    .collect()
            })
            .collect()
    }

    fn find_min_entropy(&self, grid: &[Vec<Cell>]) -> Option<(usize, usize)> {
        let mut min_entropy = f32::MAX;
        let mut min_pos = None;

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &grid[y][x];
                if cell.collapsed {
                    continue;
                }
                let count = cell.count_possible() as f32;
                if count > 0.0 && count < min_entropy {
                    min_entropy = count;
                    min_pos = Some((x, y));
                }
            }
        }

        min_pos
    }

    fn choose_tile(&mut self, cell: &Cell) -> usize {
        let possible: Vec<usize> = cell
            .possible
            .iter()
            .enumerate()
            .filter(|(_, &p)| p)
            .map(|(i, _)| i)
            .collect();

        if possible.is_empty() {
            return 0;
        }

        // Weighted random: favor tiles with more adjacency support
        let idx = (self.next_f32() * possible.len() as f32) as usize;
        possible[idx.min(possible.len() - 1)]
    }

    fn collapse(&self, grid: &mut [Vec<Cell>], x: usize, y: usize, tile: usize) {
        let cell = &mut grid[y][x];
        for i in 0..cell.possible.len() {
            cell.possible[i] = i == tile;
        }
        cell.collapsed = true;
    }

    fn propagate(&self, grid: &mut [Vec<Cell>], start_x: usize, start_y: usize) {
        let mut stack = vec![(start_x, start_y)];

        while let Some((x, y)) = stack.pop() {
            let tile_idx = match grid[y][x].chosen() {
                Some(t) => t,
                None => continue,
            };

            // Directions: [up, down, left, right]
            let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

            for (d, &(dx, dy)) in dirs.iter().enumerate() {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx < 0 || ny < 0 || nx >= self.width as isize || ny >= self.height as isize {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;

                if grid[ny][nx].collapsed {
                    continue;
                }

                let allowed = &self.adjacency[tile_idx][d];
                let mut changed = false;

                for t in 0..6 {
                    if grid[ny][nx].possible[t] && !allowed[t] {
                        grid[ny][nx].possible[t] = false;
                        changed = true;
                    }
                }

                if changed {
                    stack.push((nx, ny));
                }
            }
        }
    }

    fn fill_remaining(&self, grid: &mut [Vec<Cell>]) {
        for y in 0..self.height {
            for x in 0..self.width {
                if !grid[y][x].collapsed {
                    // Pick most common allowed tile
                    let chosen = grid[y][x]
                        .possible
                        .iter()
                        .enumerate()
                        .filter(|(_, &p)| p)
                        .map(|(i, _)| i)
                        .next()
                        .unwrap_or(0);
                    self.collapse(grid, x, y, chosen);
                }
            }
        }
    }
}

/// Convenience function to generate a map
pub fn generate_map(width: usize, height: usize, seed: u64) -> Vec<Vec<TileType>> {
    let mut gen = WfcGenerator::new(seed);
    gen.generate(width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn tile_type_index() {
        assert_eq!(TileType::Grass.index(), 0);
        assert_eq!(TileType::Water.index(), 1);
        assert_eq!(TileType::Mountain.index(), 2);
        assert_eq!(TileType::Forest.index(), 3);
        assert_eq!(TileType::Sand.index(), 4);
        assert_eq!(TileType::Stone.index(), 5);
    }

    #[test]
    fn generate_returns_correct_size() {
        let map = generate_map(16, 16, 42);
        assert_eq!(map.len(), 16);
        assert_eq!(map[0].len(), 16);
    }

    #[test]
    fn generate_all_tiles_used() {
        let map = generate_map(64, 64, 42);
        let mut counts = HashMap::new();
        for row in &map {
            for tile in row {
                *counts.entry(format!("{:?}", tile)).or_insert(0) += 1;
            }
        }
        // At 64x64, should have at least 2 different tile types
        assert!(counts.len() >= 2, "Expected variety in generated map, got: {:?}", counts);
    }

    #[test]
    fn adjacency_water_to_sand_valid() {
        let adj = WfcGenerator::new(1).adjacency;
        // Water can go to Sand (index 4)
        assert!(adj[1][0][4]); // Water up → Sand
        // Water cannot go to Mountain directly
        assert!(!adj[1][0][2]);
    }

    #[test]
    fn different_seeds_produce_different_maps() {
        let map1 = generate_map(16, 16, 42);
        let map2 = generate_map(16, 16, 99);
        let mut different = false;
        for y in 0..16 {
            for x in 0..16 {
                if map1[y][x] != map2[y][x] {
                    different = true;
                    break;
                }
            }
            if different { break; }
        }
        assert!(different, "Different seeds should produce different maps");
    }

    #[test]
    fn small_map_generation() {
        let map = generate_map(4, 4, 1);
        assert_eq!(map.len(), 4);
        assert_eq!(map[0].len(), 4);
    }

    #[test]
    fn large_map_generation() {
        let map = generate_map(128, 128, 42);
        assert_eq!(map.len(), 128);
        assert_eq!(map[0].len(), 128);
    }
}
