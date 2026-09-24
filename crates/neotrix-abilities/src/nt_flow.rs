//! 流场 — bracket DijkstraMap 精神，二叉堆真 Dijkstra（无均匀代价限制）.
//!
//! 公理：多源最短路一次算完，全员下山（追击）/上山（逃跑）查表即可；
//! bracket 原版 VecDeque 松弛只对近均匀代价正确（源码自带 WARNING），本模块用
//! 二叉堆（`total_cmp` 排序，无新依赖），加权代价亦正确。
//! 对角线禁穿角（两侧正交邻居其一不可走则禁），防穿墙斜行。

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// 不可达（保持 f32::MAX，与 bracket 一致语义）
pub const INF: f32 = f32::MAX;

#[derive(Clone, Copy, PartialEq)]
struct Key(f32);

impl Eq for Key {}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// Dijkstra 流场：`dist[i]` = 到最近源的最短代价（不可达/超深 = INF）
#[derive(Debug, Clone)]
pub struct FlowMap {
    pub w: i32,
    pub h: i32,
    pub dist: Vec<f32>,
}

impl FlowMap {
    fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }

    pub fn dist(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return INF;
        }
        self.dist[self.idx(x, y)]
    }

    /// 下山：8 邻域中代价最低邻（追击用）；已在底/孤立返回 None
    pub fn descend(&self, x: i32, y: i32, walkable: &dyn Fn(i32, i32) -> bool) -> Option<(i32, i32)> {
        self.extreme(x, y, walkable, false)
    }

    /// 上山：8 邻域中代价最高邻（逃跑用）；无路返回 None
    pub fn ascend(&self, x: i32, y: i32, walkable: &dyn Fn(i32, i32) -> bool) -> Option<(i32, i32)> {
        self.extreme(x, y, walkable, true)
    }

    fn extreme(
        &self,
        x: i32,
        y: i32,
        walkable: &dyn Fn(i32, i32) -> bool,
        want_max: bool,
    ) -> Option<(i32, i32)> {
        let cur = self.dist(x, y);
        let mut best: Option<(i32, i32)> = None;
        let mut best_d = cur;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if !walkable(nx, ny) {
                    continue;
                }
                if dx != 0 && dy != 0
                    && (!walkable(x + dx, y) || !walkable(x, y + dy))
                {
                    continue; // 禁穿角
                }
                let d = self.dist(nx, ny);
                let better = if want_max { d > best_d } else { d < best_d };
                if better {
                    best_d = d;
                    best = Some((nx, ny));
                }
            }
        }
        best
    }
}

/// 构建流场：`targets` 多源（代价 0 起）；`max_depth` 截断（≥截断的不扩展）；
/// `cost(x,y)` 为**进入**该格代价，≤0 或 INF 视为不可走。
pub fn build_flow(
    w: i32,
    h: i32,
    targets: &[(i32, i32)],
    max_depth: f32,
    cost: &dyn Fn(i32, i32) -> f32,
) -> FlowMap {
    let mut dist = vec![INF; (w * h) as usize];
    let idx = |x: i32, y: i32| (y * w + x) as usize;
    let mut heap: BinaryHeap<(Reverse<Key>, i32, i32)> = BinaryHeap::new();
    for &(tx, ty) in targets {
        if tx < 0 || ty < 0 || tx >= w || ty >= h {
            continue;
        }
        if dist[idx(tx, ty)] > 0.0 {
            dist[idx(tx, ty)] = 0.0;
            heap.push((Reverse(Key(0.0)), tx, ty));
        }
    }
    const DIRS: [(i32, i32); 8] =
        [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    while let Some((Reverse(Key(d)), x, y)) = heap.pop() {
        if d > dist[idx(x, y)] {
            continue; // 过期条目
        }
        for (dx, dy) in DIRS {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            if dx != 0 && dy != 0 {
                // 对角禁穿角：用 walkable 语义（cost>0 可走）判断正交邻
                let c1 = cost(x + dx, y);
                let c2 = cost(x, y + dy);
                if !(c1 > 0.0 && c1 < INF) || !(c2 > 0.0 && c2 < INF) {
                    continue;
                }
            }
            let step = cost(nx, ny);
            if !(step > 0.0) || step >= INF {
                continue;
            }
            let nd = d + step;
            if nd < dist[idx(nx, ny)] && nd < max_depth {
                dist[idx(nx, ny)] = nd;
                heap.push((Reverse(Key(nd)), nx, ny));
            }
        }
    }
    FlowMap { w, h, dist }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_cost(_x: i32, _y: i32) -> f32 {
        1.0
    }

    fn open_walk(_x: i32, _y: i32) -> bool {
        true
    }

    #[test]
    fn open_field_chebyshev() {
        // 8 向单位代价：距离 = 切比雪夫距离
        let m = build_flow(9, 9, &[(4, 4)], 100.0, &open_cost);
        assert_eq!(m.dist(4, 4), 0.0);
        assert_eq!(m.dist(5, 4), 1.0);
        assert_eq!(m.dist(5, 5), 1.0);
        assert_eq!(m.dist(6, 6), 2.0);
        assert_eq!(m.dist(0, 0), 4.0);
    }

    #[test]
    fn wall_forces_detour() {
        // x=4 竖墙（y 1..7 留上下缺口）：直穿被 INF 墙挡，须绕行
        let cost = |x: i32, y: i32| {
            if x == 4 && (1..=7).contains(&y) {
                INF
            } else {
                1.0
            }
        };
        let m = build_flow(9, 9, &[(0, 4)], 100.0, &cost);
        assert_eq!(m.dist(4, 4), INF); // 墙内不可达
        let direct = m.dist(8, 4);
        assert!(direct > 8.0); // 绕行远于直线 8
        assert!(direct < INF);
    }

    #[test]
    fn multi_target_takes_min_and_depth_cutoff() {
        let m = build_flow(9, 9, &[(0, 0), (8, 8)], 100.0, &open_cost);
        assert_eq!(m.dist(4, 4), 4.0); // 到任一源的最小值
        let m2 = build_flow(9, 9, &[(0, 0)], 3.0, &open_cost);
        assert_eq!(m2.dist(8, 8), INF); // 超深截断
        assert_eq!(m2.dist(2, 0), 2.0);
    }

    #[test]
    fn descend_reaches_target() {
        let m = build_flow(9, 9, &[(0, 0)], 100.0, &open_cost);
        let (mut x, mut y) = (8, 8);
        for _ in 0..16 {
            match m.descend(x, y, &open_walk) {
                Some((nx, ny)) => {
                    x = nx;
                    y = ny;
                }
                None => break,
            }
        }
        assert_eq!((x, y), (0, 0)); // 下山必达源
        assert!(m.descend(0, 0, &open_walk).is_none()); // 已在底
    }

    #[test]
    fn ascend_runs_away_and_corner_cut_blocked() {
        let m = build_flow(9, 9, &[(4, 4)], 100.0, &open_cost);
        let up = m.ascend(4, 5, &open_walk).unwrap();
        // 上山远离源：(4,5) 的邻域最高代价应在外侧
        assert!(m.dist(up.0, up.1) > m.dist(4, 5));
        // 穿角：L 形墙角，对角捷径被禁
        let lwall = |x: i32, y: i32| {
            if (x == 4 && y == 5) || (x == 5 && y == 4) {
                INF
            } else {
                1.0
            }
        };
        let m2 = build_flow(9, 9, &[(0, 0)], 100.0, &lwall);
        // (5,5) 不能经 (4,4) 对角直达（两侧被墙）：距离 > 8
        assert!(m2.dist(5, 5) > 8.0);
    }

    #[test]
    fn weighted_costs_respected() {
        // 单格沼泽 (4,1) 代价 5：真 Dijkstra 绕开它（VecDeque 松弛版在此会错）
        let cost = |x: i32, y: i32| if x == 4 && y == 1 { 5.0 } else { 1.0 };
        let m = build_flow(9, 3, &[(0, 1)], 100.0, &cost);
        // 直穿 (8,1)：8 步含 1 格沼泽 = 7×1+5=12；绕行上下两行 8 步 = 8
        assert_eq!(m.dist(8, 1), 8.0);
    }
}

/// A* 单目标最优路径（Stanford 决策规则：单对单用 A*，多对一用流场）。
/// 8 向 + 对角禁穿角（与 `build_flow` 同规则）；启发为 octile 距离，
/// 假设最小步代价 1.0（cost<1 的格子启发可能高估→次优；标准 cost≥1 无影响）。
/// `max_expand` 扩展上限（超限 None，防大图卡死）；返回含起终点的格序列。
pub fn astar(
    w: i32,
    h: i32,
    start: (i32, i32),
    goal: (i32, i32),
    max_expand: u32,
    cost: &dyn Fn(i32, i32) -> f32,
) -> Option<Vec<(i32, i32)>> {
    let walkable =
        |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && cost(x, y) > 0.0 && cost(x, y) < INF;
    if start == goal {
        return if walkable(start.0, start.1) { Some(vec![start]) } else { None };
    }
    if !walkable(start.0, start.1) || !walkable(goal.0, goal.1) {
        return None;
    }
    let idx = |x: i32, y: i32| (y * w + x) as usize;
    // octile 启发（8 向单位步最优下界）
    let heur = |x: i32, y: i32| {
        let dx = (x - goal.0).abs();
        let dy = (y - goal.1).abs();
        let (a, b) = if dx > dy { (dx, dy) } else { (dy, dx) };
        a as f32 + 0.41421356 * b as f32
    };
    const DIRS: [(i32, i32); 8] =
        [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    let n = (w * h) as usize;
    let mut g = vec![INF; n];
    let mut came: Vec<Option<usize>> = vec![None; n];
    let mut closed = vec![false; n];
    let mut heap: BinaryHeap<(Reverse<Key>, usize)> = BinaryHeap::new();
    g[idx(start.0, start.1)] = 0.0;
    heap.push((Reverse(Key(heur(start.0, start.1))), idx(start.0, start.1)));
    let mut expanded = 0u32;
    while let Some((_, cur)) = heap.pop() {
        if closed[cur] {
            continue;
        }
        if cur == idx(goal.0, goal.1) {
            // 重建路径（pop 时出 goal ⇒ 最优，启发一致）
            let mut path = vec![goal];
            let mut c = cur;
            while let Some(p) = came[c] {
                let (px, py) = ((p as i32) % w, (p as i32) / w);
                path.push((px, py));
                c = p;
            }
            path.reverse();
            return Some(path);
        }
        closed[cur] = true;
        expanded += 1;
        if expanded > max_expand {
            return None;
        }
        let (cx, cy) = ((cur as i32) % w, (cur as i32) / w);
        for (dx, dy) in DIRS {
            let (nx, ny) = (cx + dx, cy + dy);
            if !walkable(nx, ny) {
                continue;
            }
            if dx != 0 && dy != 0 && (!walkable(cx + dx, cy) || !walkable(cx, cy + dy)) {
                continue; // 禁穿角
            }
            let ni = idx(nx, ny);
            let ng = g[cur] + cost(nx, ny);
            if ng < g[ni] {
                g[ni] = ng;
                came[ni] = Some(cur);
                heap.push((Reverse(Key(ng + heur(nx, ny))), ni));
            }
        }
    }
    None
}

#[cfg(test)]
mod astar_tests {
    use super::*;

    fn open_cost(_x: i32, _y: i32) -> f32 {
        1.0
    }

    fn adjacent_path(p: &[(i32, i32)]) -> bool {
        p.windows(2).all(|w| (w[0].0 - w[1].0).abs() <= 1 && (w[0].1 - w[1].1).abs() <= 1)
    }

    #[test]
    fn open_field_optimal_diagonal() {
        let p = astar(9, 9, (0, 0), (8, 8), 1000, &open_cost).unwrap();
        assert_eq!(p.len(), 9); // 8 对角步最优
        assert_eq!(p[0], (0, 0));
        assert_eq!(p[8], (8, 8));
        assert!(adjacent_path(&p));
    }

    #[test]
    fn start_equals_goal() {
        assert_eq!(astar(9, 9, (3, 3), (3, 3), 1000, &open_cost), Some(vec![(3, 3)]));
    }

    #[test]
    fn wall_detour_avoids_blocked() {
        let cost = |x: i32, y: i32| {
            if x == 4 && (1..=7).contains(&y) {
                INF
            } else {
                1.0
            }
        };
        let p = astar(9, 9, (0, 4), (8, 4), 1000, &cost).unwrap();
        assert!(adjacent_path(&p));
        assert!(!p.iter().any(|&(x, y)| x == 4 && (1..=7).contains(&y)));
        assert!(p.len() > 9); // 绕行必长于直线
    }

    #[test]
    fn enclosed_goal_returns_none() {
        // 终点被 8 邻墙围死
        let cost = |x: i32, y: i32| {
            if (x - 4).abs() <= 1 && (y - 4).abs() <= 1 && (x, y) != (0, 0) {
                INF
            } else {
                1.0
            }
        };
        assert!(astar(9, 9, (0, 0), (4, 4), 1000, &cost).is_none());
    }

    #[test]
    fn weighted_avoids_swamp_and_respects_budget() {
        let cost = |x: i32, y: i32| if x == 4 && y == 1 { 5.0 } else { 1.0 };
        let p = astar(9, 3, (0, 1), (8, 1), 1000, &cost).unwrap();
        assert!(!p.contains(&(4, 1))); // 绕开沼泽
        assert_eq!(p.len(), 9);
        assert!(astar(9, 9, (0, 0), (8, 8), 2, &open_cost).is_none()); // 扩展上限熔断
    }
}
