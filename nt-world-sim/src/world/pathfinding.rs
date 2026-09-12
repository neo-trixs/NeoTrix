use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathNode {
    pub x: i32,
    pub y: i32,
}

impl PathNode {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn heuristic(&self, other: &PathNode) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Clone)]
struct OpenNode {
    node: PathNode,
    #[allow(dead_code)]
    g_score: i32,
    f_score: i32,
}

impl PartialEq for OpenNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score
    }
}

impl Eq for OpenNode {}

impl Ord for OpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for OpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn astar(
    start: PathNode,
    goal: PathNode,
    is_walkable: impl Fn(i32, i32) -> bool,
    max_steps: usize,
) -> Option<Vec<PathNode>> {
    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<PathNode, PathNode> = HashMap::new();
    let mut g_score: HashMap<PathNode, i32> = HashMap::new();

    g_score.insert(start, 0);
    open.push(OpenNode {
        node: start,
        g_score: 0,
        f_score: start.heuristic(&goal),
    });

    let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    let mut steps = 0;

    while let Some(current) = open.pop() {
        steps += 1;
        if steps > max_steps {
            return None;
        }

        if current.node == goal {
            let mut path = vec![goal];
            let mut cur = goal;
            while let Some(&prev) = came_from.get(&cur) {
                path.push(prev);
                cur = prev;
            }
            path.reverse();
            return Some(path);
        }

        for (dx, dy) in &dirs {
            let nx = current.node.x + dx;
            let ny = current.node.y + dy;
            if !is_walkable(nx, ny) {
                continue;
            }

            let neighbor = PathNode::new(nx, ny);
            let move_cost = if *dx != 0 && *dy != 0 { 14 } else { 10 };
            let tent_g = g_score[&current.node] + move_cost;

            if tent_g < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                came_from.insert(neighbor, current.node);
                g_score.insert(neighbor, tent_g);
                let f = tent_g + neighbor.heuristic(&goal);
                open.push(OpenNode {
                    node: neighbor,
                    g_score: tent_g,
                    f_score: f,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astar_straight_line() {
        let walkable = |_x: i32, _y: i32| true;
        let path = astar(
            PathNode::new(0, 0),
            PathNode::new(5, 0),
            walkable,
            1000,
        );
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&PathNode::new(0, 0)));
        assert_eq!(path.last(), Some(&PathNode::new(5, 0)));
    }

    #[test]
    fn test_astar_around_wall() {
        let walls: std::collections::HashSet<(i32, i32)> =
            (2..8).map(|y| (5, y)).collect();
        let walkable = |x: i32, y: i32| !walls.contains(&(x, y));

        let path = astar(
            PathNode::new(1, 5),
            PathNode::new(8, 5),
            walkable,
            1000,
        );
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() > 5); // Must go around wall
        // Verify no path node is inside the wall
        for node in &path {
            assert!(!walls.contains(&(node.x, node.y)));
        }
    }

    #[test]
    fn test_astar_no_path() {
        // Two-wide wall blocks all movement within a bounded grid
        let walls: std::collections::HashSet<(i32, i32)> =
            (0..10).flat_map(|y| [(5, y), (6, y)]).collect();
        let walkable = |x: i32, y: i32| {
            x >= 0 && y >= 0 && x < 10 && y < 10 && !walls.contains(&(x, y))
        };

        let path = astar(
            PathNode::new(0, 5),
            PathNode::new(9, 5),
            walkable,
            1000,
        );
        assert!(path.is_none());
    }

    #[test]
    fn test_astar_max_steps() {
        let walkable = |_x: i32, _y: i32| true;
        let path = astar(
            PathNode::new(0, 0),
            PathNode::new(100, 100),
            walkable,
            5,
        );
        assert!(path.is_none());
    }

    #[test]
    fn test_astar_same_start_goal() {
        let walkable = |_x: i32, _y: i32| true;
        let path = astar(
            PathNode::new(3, 3),
            PathNode::new(3, 3),
            walkable,
            100,
        );
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 1);
    }

    #[test]
    fn test_astar_diagonal() {
        let walkable = |_x: i32, _y: i32| true;
        let path = astar(
            PathNode::new(0, 0),
            PathNode::new(3, 3),
            walkable,
            1000,
        );
        assert!(path.is_some());
        let path = path.unwrap();
        // Diagonal path should be shorter than Manhattan
        assert!(path.len() <= 4);
    }
}
