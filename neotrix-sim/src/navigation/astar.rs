use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn manhattan_distance(&self, other: &GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn neighbors(&self) -> Vec<GridPos> {
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)];
        dirs.iter()
            .map(|(dx, dy)| GridPos::new(self.x + dx, self.y + dy))
            .collect()
    }
}

#[derive(Debug, Clone)]
struct Node {
    pos: GridPos,
    g: f32,
    h: f32,
    f: f32,
    _parent: Option<GridPos>,
}

impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct AStar {
    width: i32,
    height: i32,
    obstacles: Vec<Vec<bool>>,
    movement_cost: Vec<Vec<f32>>,
}

impl AStar {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            obstacles: vec![vec![false; height as usize]; width as usize],
            movement_cost: vec![vec![1.0; height as usize]; width as usize],
        }
    }

    pub fn set_obstacle(&mut self, x: i32, y: i32, blocked: bool) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.obstacles[x as usize][y as usize] = blocked;
        }
    }

    pub fn set_movement_cost(&mut self, x: i32, y: i32, cost: f32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.movement_cost[x as usize][y as usize] = cost;
        }
    }

    pub fn is_walkable(&self, pos: &GridPos) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
            && !self.obstacles[pos.x as usize][pos.y as usize]
    }

    pub fn find_path(&self, start: GridPos, goal: GridPos) -> Option<Vec<GridPos>> {
        if !self.is_walkable(&start) || !self.is_walkable(&goal) {
            return None;
        }

        let mut open: BinaryHeap<Node> = BinaryHeap::new();
        let mut closed: HashMap<GridPos, f32> = HashMap::new();
        let mut parents: HashMap<GridPos, GridPos> = HashMap::new();

        let h = start.manhattan_distance(&goal) as f32;
        open.push(Node {
            pos: start,
            g: 0.0,
            h,
            f: h,
            _parent: None,
        });

        while let Some(current) = open.pop() {
            if current.pos == goal {
                let mut path = vec![goal];
                let mut current_pos = goal;
                while let Some(parent) = parents.get(&current_pos) {
                    path.push(*parent);
                    current_pos = *parent;
                }
                path.reverse();
                return Some(path);
            }

            if let Some(&best_g) = closed.get(&current.pos) {
                if current.g > best_g {
                    continue;
                }
            }
            closed.insert(current.pos, current.g);

            for neighbor in current.pos.neighbors() {
                if !self.is_walkable(&neighbor) {
                    continue;
                }

                let dx = (neighbor.x - current.pos.x).abs();
                let dy = (neighbor.y - current.pos.y).abs();
                let move_cost = if dx + dy == 2 {
                    1.414 * self.movement_cost[neighbor.x as usize][neighbor.y as usize]
                } else {
                    self.movement_cost[neighbor.x as usize][neighbor.y as usize]
                };

                let new_g = current.g + move_cost;

                if let Some(&best_g) = closed.get(&neighbor) {
                    if new_g >= best_g {
                        continue;
                    }
                }

                let h = neighbor.manhattan_distance(&goal) as f32;
                parents.insert(neighbor, current.pos);
                open.push(Node {
                    pos: neighbor,
                    g: new_g,
                    h,
                    f: new_g + h,
                    _parent: Some(current.pos),
                });
            }
        }

        None
    }

    pub fn find_path_with_limit(&self, start: GridPos, goal: GridPos, max_steps: usize) -> Option<Vec<GridPos>> {
        if !self.is_walkable(&start) || !self.is_walkable(&goal) {
            return None;
        }

        let mut open: BinaryHeap<Node> = BinaryHeap::new();
        let mut closed: HashMap<GridPos, f32> = HashMap::new();
        let mut parents: HashMap<GridPos, GridPos> = HashMap::new();
        let mut steps = 0;

        let h = start.manhattan_distance(&goal) as f32;
        open.push(Node {
            pos: start,
            g: 0.0,
            h,
            f: h,
            _parent: None,
        });

        while let Some(current) = open.pop() {
            steps += 1;
            if steps > max_steps {
                return None;
            }

            if current.pos == goal {
                let mut path = vec![goal];
                let mut current_pos = goal;
                while let Some(parent) = parents.get(&current_pos) {
                    path.push(*parent);
                    current_pos = *parent;
                }
                path.reverse();
                return Some(path);
            }

            if let Some(&best_g) = closed.get(&current.pos) {
                if current.g > best_g {
                    continue;
                }
            }
            closed.insert(current.pos, current.g);

            for neighbor in current.pos.neighbors() {
                if !self.is_walkable(&neighbor) {
                    continue;
                }

                let dx = (neighbor.x - current.pos.x).abs();
                let dy = (neighbor.y - current.pos.y).abs();
                let move_cost = if dx + dy == 2 {
                    1.414 * self.movement_cost[neighbor.x as usize][neighbor.y as usize]
                } else {
                    self.movement_cost[neighbor.x as usize][neighbor.y as usize]
                };

                let new_g = current.g + move_cost;

                if let Some(&best_g) = closed.get(&neighbor) {
                    if new_g >= best_g {
                        continue;
                    }
                }

                let h = neighbor.manhattan_distance(&goal) as f32;
                parents.insert(neighbor, current.pos);
                open.push(Node {
                    pos: neighbor,
                    g: new_g,
                    h,
                    f: new_g + h,
                    _parent: Some(current.pos),
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astar_straight_line() {
        let astar = AStar::new(10, 10);
        let path = astar.find_path(GridPos::new(0, 0), GridPos::new(5, 0));
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 6);
        assert_eq!(path[0], GridPos::new(0, 0));
        assert_eq!(path[5], GridPos::new(5, 0));
    }

    #[test]
    fn test_astar_avoids_obstacle() {
        let mut astar = AStar::new(10, 10);
        for y in 0..10 {
            astar.set_obstacle(5, y, true);
        }
        let path = astar.find_path(GridPos::new(0, 5), GridPos::new(9, 5));
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(!path.iter().any(|p| p.x == 5 && p.y == 5));
    }

    #[test]
    fn test_astar_no_path() {
        let mut astar = AStar::new(10, 10);
        for y in 0..10 {
            astar.set_obstacle(5, y, true);
        }
        let path = astar.find_path(GridPos::new(0, 0), GridPos::new(9, 9));
        assert!(path.is_none());
    }

    #[test]
    fn test_astar_with_step_limit() {
        let astar = AStar::new(20, 20);
        let path = astar.find_path_with_limit(GridPos::new(0, 0), GridPos::new(19, 19), 10);
        assert!(path.is_none());
    }
}
