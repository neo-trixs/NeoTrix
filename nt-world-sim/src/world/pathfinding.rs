use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use super::tile::TileMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos(pub u32, pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    pos: Pos,
    g: u32,
    h: u32,
}

impl Node {
    fn f(&self) -> u32 { self.g + self.h }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f().cmp(&self.f())
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: Pos, b: Pos) -> u32 {
    ((a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()) as u32
}

pub fn astar(map: &TileMap, start: Pos, goal: Pos) -> Option<Vec<Pos>> {
    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<Pos, Pos> = HashMap::new();
    let mut g_score: HashMap<Pos, u32> = HashMap::new();

    g_score.insert(start, 0);
    open.push(Node { pos: start, g: 0, h: heuristic(start, goal) });

    let dirs = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)];

    while let Some(current) = open.pop() {
        if current.pos == goal {
            let mut path = vec![goal];
            let mut current = goal;
            while let Some(&prev) = came_from.get(&current) {
                path.push(prev);
                current = prev;
            }
            path.reverse();
            return Some(path);
        }

        for (dx, dy) in &dirs {
            let nx = current.pos.0 as i32 + dx;
            let ny = current.pos.1 as i32 + dy;
            if nx < 0 || ny < 0 { continue; }
            let nx = nx as u32;
            let ny = ny as u32;

            if !map.is_walkable(nx, ny) { continue; }

            let tentative_g = g_score[&current.pos] + 1;
            let neighbor = Pos(nx, ny);

            if tentative_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                came_from.insert(neighbor, current.pos);
                g_score.insert(neighbor, tentative_g);
                open.push(Node { pos: neighbor, g: tentative_g, h: heuristic(neighbor, goal) });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tile::{Tile, TileType};

    #[test]
    fn test_pathfinding() {
        let mut map = TileMap::new(10, 10, 16.0);
        // Add a wall
        for y in 2..8 {
            map.set(5, y, Tile::new(TileType::Wall));
        }

        let path = astar(&map, Pos(1, 1), Pos(8, 1));
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() > 2); // Must go around wall
        assert_eq!(path.first(), Some(&Pos(1, 1)));
        assert_eq!(path.last(), Some(&Pos(8, 1)));
    }

    #[test]
    fn test_no_path() {
        let mut map = TileMap::new(5, 5, 16.0);
        // Complete wall
        for y in 0..5 {
            map.set(2, y, Tile::new(TileType::Wall));
        }

        let path = astar(&map, Pos(0, 0), Pos(4, 0));
        assert!(path.is_none());
    }
}
