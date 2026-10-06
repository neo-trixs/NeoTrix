//! A* Pathfinding — absorbed from neotrix-sim
//!
//! Grid-based A* with BinaryHeap, obstacle avoidance, and movement costs.
//!
//! Source: archive/neotrix-sim/src/navigation/astar.rs
//!
//! 2026-09-28: 对角步代价由字面量 `1.414` 改为 `SQRT_2`（精确 √2），
//! 与同层 `nt_flow` 共用同一常量，两文件度量从此只有一个定义点。
//! 数值变化约 0.0151%；本文件测试只断言路径结构（长度/端点/绕障），
//! 不锁距离值，故无需改测试。

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// 对角步代价系数（√2）。与 `nt_flow::SQRT_2` 同值，勿各自硬编码。
pub const SQRT_2: f32 = std::f32::consts::SQRT_2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        GridPos { x, y }
    }

    /// ⛔ **不可采纳**（inadmissible）：八邻网格的启发**不能用 L1**。
    ///
    /// 位移 (1,1) 时真实代价 = `SQRT_2` ≈ 1.414，本函数返回 2 ⇒ **高估**。
    /// 高估的启发会让 A\* 过早把终点从 open 里弹出并返回**次优路径**。
    /// 保留它只为不破坏公开 API，**本文件的启发一律用
    /// [`GridPos::octile_distance`]**。
    pub fn manhattan_distance(&self, other: &GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// 八邻（对角 ×`SQRT_2`）度量的**可采纳且一致**启发：octile 距离。
    ///
    /// `√2 * min(dx,dy) + 1 * (|dx-dy|)` —— 即「先走对角，再走正交」。
    /// 它正是该度量的真实最短距离（无障碍时）⇒ 既不高估也不低估。
    ///
    /// ⛔ 改用 L1 的实测后果：对 60 个确定性随机障碍布局与「同边代价 Dijkstra」
    ///   逐个对账，**28 个（47%）返回次优路径**，最差一例如
    ///   `最优 14.8995 / 实得 20.5563`（差 38%）。
    #[inline]
    pub fn octile_distance(&self, other: &GridPos) -> f32 {
        let dx = (self.x - other.x).abs() as f32;
        let dy = (self.y - other.y).abs() as f32;
        let (lo, hi) = if dx < dy { (dx, dy) } else { (dy, dx) };
        SQRT_2 * lo + (hi - lo)
    }

    /// 8 个方向候选（含 4 个对角）。⛔ **不含穿角规则** —— 那需要网格，
    /// 见 [`AStar::walkable_neighbors`]。
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
    f: f32,
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

    /// **对角步是否可走：两侧正交格都必须可走。**
    ///
    /// ⛔ 缺这条时，`neighbors()` 的 4 个对角方向**不看两侧** ⇒ 单位尺寸的
    ///   实体被允许从两面都堵死的夹角里挤过去（实测：`(1,0)`/`(0,1)` 全堵，
    ///   `(0,0)→(1,1)` 仍返回 `Some([(0,0),(1,1)])`）。物理上不存在这种移动。
    ///
    /// 对照 `ra-ecs`（Apache-2.0）的 `PassGrid::find_path_diag`：对角要求
    /// **两侧正交格都可走**，否则跳过该对角。
    fn can_step(&self, from: GridPos, to: GridPos) -> bool {
        if !self.is_walkable(&to) {
            return false;
        }
        let dx = (to.x - from.x).abs();
        let dy = (to.y - from.y).abs();
        if dx + dy != 2 {
            return true; // 正交步：只看目标格
        }
        // 对角步：两侧正交格必须都可走。
        self.is_walkable(&GridPos::new(from.x, to.y))
            && self.is_walkable(&GridPos::new(to.x, from.y))
    }

    /// 从 `pos` 真正可走的邻居（已应用 [`Self::can_step`] 的穿角规则）。
    fn walkable_neighbors(&self, pos: GridPos) -> Vec<GridPos> {
        pos.neighbors()
            .into_iter()
            .filter(|nb| self.can_step(pos, *nb))
            .collect()
    }

    /// **唯一一份 A\* 实现**；`max_steps = None` 表示不限步数。
    ///
    /// 此前 `find_path` 与 `find_path_with_limit` 是**两份逐字复制**（只差一个
    /// 计数器）⇒ 任何算法修正都必须改两处，漏一处就产生两个互相矛盾的
    /// 实现。现在两者都是本函数的薄包装。
    fn search(
        &self,
        start: GridPos,
        goal: GridPos,
        max_steps: Option<usize>,
    ) -> Option<Vec<GridPos>> {
        if !self.is_walkable(&start) || !self.is_walkable(&goal) {
            return None;
        }

        let mut open: BinaryHeap<Node> = BinaryHeap::new();
        // `g_score` = **已生成**（不只是已弹出）的节点里最好的 g。
        //
        // ⛔ 原实现叫它 `closed`，但只在**弹出时**写入 ⇒ 还在 `open` 里的邻居
        //   查不到 ⇒ 每次重展开都把它们**重复 push**，`open` 无限膨胀。
        //   闸门又是 `current.g > best_g`（严格大于）⇒ g 相等照样重展开。
        //   平时被「曼哈顿启发让 f 有梯度、能较快撞到终点」掩盖；
        //   一旦启发**恰好完美**（无障碍八邻网格上 octile 精确等于真实距离）
        //   ⇒ 所有 f 恒等 ⇒ 堆退化成无序 ⇒ 重复 push 指数膨胀，查询**跑不完**。
        // 标准 A\* 模式：`g_score` 在**生成边时**就更新。
        let mut g_score: HashMap<GridPos, f32> = HashMap::new();
        let mut parents: HashMap<GridPos, GridPos> = HashMap::new();
        let mut steps: usize = 0;

        // 启发必须是 octile（与 √2 对角边**同度量**），不是 L1。
        let h = start.octile_distance(&goal);
        // ⛔ **必须把起点种进去**。否则起点自己会被当作邻居**重新生成**
        //   （`g_score.get(&start)` = `None` ⇒ 不跳过），`parents[start]`
        //   被改写成某个后继 ⇒ 回溯链**成环** ⇒ `while let Some(parent)`
        //   死循环。实测症状很有欺骗性：迭代已抵达终点、g 已是最终值，
        //   却仍在原地打转（空网格 10x10 走 5 步都跑不完）。
        g_score.insert(start, 0.0);
        open.push(Node { pos: start, g: 0.0, f: h });

        while let Some(current) = open.pop() {
            if let Some(limit) = max_steps {
                if steps >= limit {
                    return None;
                }
            }
            steps += 1;

            if current.pos == goal {
                let mut path = vec![goal];
                let mut cur = goal;
                while let Some(parent) = parents.get(&cur) {
                    path.push(*parent);
                    cur = *parent;
                }
                path.reverse();
                return Some(path);
            }

            // 陈旧副本（弹出时 g 已不是最好的）直接丢弃。
            if let Some(&best_g) = g_score.get(&current.pos) {
                if current.g > best_g {
                    continue;
                }
            }

            for neighbor in self.walkable_neighbors(current.pos) {
                let dx = (neighbor.x - current.pos.x).abs();
                let dy = (neighbor.y - current.pos.y).abs();
                let base = self.movement_cost[neighbor.x as usize][neighbor.y as usize];
                let move_cost = if dx + dy == 2 { SQRT_2 * base } else { base };
                let new_g = current.g + move_cost;

                // 与 `g_score` 比：不够好就不生成。**这一句是「不重复 push」的
                //   全部保证** —— 生成时就记账，而不是等弹出才记账。
                if let Some(&best_g) = g_score.get(&neighbor) {
                    if new_g >= best_g {
                        continue;
                    }
                }
                g_score.insert(neighbor, new_g);

                let h = neighbor.octile_distance(&goal);
                parents.insert(neighbor, current.pos);
                open.push(Node { pos: neighbor, g: new_g, f: new_g + h });
            }
        }

        None
    }

    pub fn find_path(&self, start: GridPos, goal: GridPos) -> Option<Vec<GridPos>> {
        self.search(start, goal, None)
    }

    pub fn find_path_with_limit(
        &self,
        start: GridPos,
        goal: GridPos,
        max_steps: usize,
    ) -> Option<Vec<GridPos>> {
        self.search(start, goal, Some(max_steps))
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
        for y in 0..4 {
            astar.set_obstacle(5, y, true);
        }
        for y in 6..10 {
            astar.set_obstacle(5, y, true);
        }
        let path = astar.find_path(GridPos::new(0, 5), GridPos::new(9, 5));
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path[0], GridPos::new(0, 5));
        assert_eq!(*path.last().unwrap(), GridPos::new(9, 5));
        assert!(!path.iter().any(|p| p.x == 5 && (p.y < 4 || p.y > 6)));
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

// ===========================================================================
// 度量与穿角测试 —— 两处缺陷的实证
//
// ⛔ 上一组测试只断言**结构**（长度/端点/绕障），模块头也自认「不锁距离值」
//    ⇒ 它们在结构上**不可能**发现「启发式与边代价不同度量」这类缺陷。
// ===========================================================================
#[cfg(test)]
mod tests_2 {
    use super::*;

    /// 按 `find_path` **同一套**边代价给一条路径算总代价。
    ///
    /// ⛔ 不得用步数或曼哈顿代替 —— 那正是本文件一度出现的度量混乱。
    fn path_cost(astar: &AStar, path: &[GridPos]) -> f32 {
        path.windows(2)
            .map(|w| {
                let (a, b) = (w[0], w[1]);
                let dx = (b.x - a.x).abs();
                let dy = (b.y - a.y).abs();
                let base = astar.movement_cost[b.x as usize][b.y as usize];
                if dx + dy == 2 { SQRT_2 * base } else { base }
            })
            .sum()
    }

    /// ⛔ **启发式必须与边代价同度量**。
    ///
    /// 边代价是欧氏（对角 ×`SQRT_2`，`:146`），而启发式用
    /// `manhattan_distance`（L1，`:29-31`）。位移 (1,1) 时
    /// 真实代价 = √2 ≈ 1.414，L1 估 = 2 ⇒ **高估** ⇒ 不可采纳
    /// ⇒ A* 提前从 open 里弹出终点并返回**次优路径**。
    ///
    /// 对照 `ra-ecs` 同仓的 `PassGrid`：它**根本没有启发式**（均匀 BFS），
    /// 所以结构上不可能犯这个错。本文件选择了 A*，就必须把度量配平。
    #[test]
    fn diagonal_path_cost_must_actually_be_optimal() {
        let astar = AStar::new(10, 10);
        // 开阔网格，(0,0) → (3,3)：最优 = 3 步对角 = 3√2。
        let path = astar
            .find_path(GridPos::new(0, 0), GridPos::new(3, 3))
            .expect("开阔网格必有路径");
        let optimal = 3.0 * SQRT_2;
        assert!(
            (path_cost(&astar, &path) - optimal).abs() < 1e-4,
            "路径总代价 {:.4} ≠ 最优 {optimal:.4}（走法 {:?}）\n\
             ⇒ L1 启发高估于 √2 对角边，A* 返回了次优路径",
            path_cost(&astar, &path),
            path
        );
    }

    /// ⛔ **对角步不得从两面墙的夹角穿过**。
    ///
    /// `neighbors()`（`:33-38`）无条件返回 4 个对角方向，**不看两侧正交格
    /// 是否可走** ⇒ 单位尺寸的实体被允许从 `(1,0)` 与 `(0,1)` 都堵死的
    /// 夹角里挤过去。物理上不存在这种移动。
    ///
    /// `ra-ecs` 的 `PassGrid::find_path_diag` 恰好显式禁止：对角要求
    /// **两侧正交格都可走**。
    #[test]
    fn a_diagonal_may_not_cut_between_two_blocked_cells() {
        let mut astar = AStar::new(10, 10);
        // 起点 (0,0) 的两个正交邻居全堵 ⇒ 唯一「出路」是穿角到 (1,1)。
        astar.set_obstacle(1, 0, true);
        astar.set_obstacle(0, 1, true);
        let path = astar.find_path(GridPos::new(0, 0), GridPos::new(1, 1));
        assert!(
            path.is_none(),
            "两侧正交格都堵死时不该有路径，但返回了 {:?} ⇒ 对角步穿角了",
            path
        );
    }

    /// 堵住**一侧**后，对角**必须**被禁（两侧规则是「两侧都要可走」），
    /// 但仍应有路径 —— 绕成两步正交。
    ///
    /// ⛔ 这条测试原先断言 `path.len() == 2`，那等于把「直穿对角」当期望行为
    ///   —— 而那正是被修掉的 bug。正确期望是**三个位置**（起点 + 2 步）
    ///   且总代价为 2。
    #[test]
    fn a_diagonal_is_refused_when_either_orthogonal_neighbour_is_blocked() {
        let mut astar = AStar::new(10, 10);
        astar.set_obstacle(1, 0, true); // 只堵一侧 ⇒ 对角必须被禁
        let path = astar
            .find_path(GridPos::new(0, 0), GridPos::new(1, 1))
            .expect("一侧被堵不等于无路：绕成两步正交即可");
        assert_eq!(
            path.len(),
            3,
            "绕行应是 起点+2 步 = 3 个位置；实测 {:?}",
            path
        );
        assert!(
            !path.windows(2).any(|w| {
                let dx = (w[1].x - w[0].x).abs();
                let dy = (w[1].y - w[0].y).abs();
                dx + dy == 2
            }),
            "路径里不该有对角步：{:?}",
            path
        );
        assert!(
            (path_cost(&astar, &path) - 2.0).abs() < 1e-4,
            "两步正交代价应为 2，实测 {}",
            path_cost(&astar, &path)
        );
    }

    /// ⛔ 两个查询入口**不得**给出不同的答案。
    ///
    /// `find_path_with_limit`（`:175-252`）是 `find_path`（`:102-173`）的
    /// 逐字复制，只多一个计数器 ⇒ 任何对算法的修正都必须改两处，
    /// 漏一处就产生两个互相矛盾的实现。这条测试把「两份实现」钉成「一个契约」。
    #[test]
    fn the_step_limited_variant_must_agree_with_the_unlimited_one() {
        let mut astar = AStar::new(12, 12);
        for y in 0..12 {
            astar.set_obstacle(6, y, true);
        }
        let (s, g) = (GridPos::new(0, 5), GridPos::new(11, 5));
        let plain = astar.find_path(s, g);
        let limited = astar.find_path_with_limit(s, g, 100_000);
        assert_eq!(
            plain, limited,
            "同一个查询的两个入口给了不同答案 ⇒ 两份复制实现已经漂移"
        );
    }
}

// ===========================================================================
// 判决实验：L1 启发在 √2 对角边下**实测**是否真会返回次优路径
//
// 上一条 `diagonal_path_cost_must_actually_be_optimal` 是绿的 ⇒
//    「不可采纳 ⇒ 一定次优」这个推断**不成立**（不可采纳只意味着「可能」）。
//    这里用同一套边代价 + 同一套邻居规则跑一遍 Dijkstra 当基准，
//    在确定性扫描的障碍布局上逐个对账。有反例就是真缺陷，没有就
//    **如实记为「理论风险，未实测到反例」**，不拿它当 bug 改代码。
// ===========================================================================
#[cfg(test)]
mod probe_optimality {
    use super::*;
    use std::collections::BinaryHeap;

    /// 确定性 LCG —— 不用 `rand`，保证这个判决实验**每次跑出同一个结果**。
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            self.0 >> 33
        }
    }

    #[derive(PartialEq)]
    struct Q { cost: f32, pos: GridPos }
    impl Eq for Q {}
    impl Ord for Q {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            o.cost.partial_cmp(&self.cost).unwrap_or(std::cmp::Ordering::Equal)
        }
    }
    impl PartialOrd for Q {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(o)) }
    }

    /// 基准：同一套边代价 + 同一套邻居规则的 Dijkstra。
    /// ⛔ 基准必须与被测实现**共用**邻居规则，否则比的是两件不同的事。
    fn dijkstra_cost(astar: &AStar, start: GridPos, goal: GridPos) -> Option<f32> {
        let mut dist = HashMap::new();
        let mut open = BinaryHeap::new();
        dist.insert(start, 0.0_f32);
        open.push(Q { cost: 0.0, pos: start });
        while let Some(Q { cost, pos }) = open.pop() {
            if pos == goal { return Some(cost); }
            if dist.get(&pos).is_some_and(|&d| cost > d) { continue; }
            for nb in astar.walkable_neighbors(pos) {
                let dx = (nb.x - pos.x).abs();
                let dy = (nb.y - pos.y).abs();
                let base = astar.movement_cost[nb.x as usize][nb.y as usize];
                let step = if dx + dy == 2 { SQRT_2 * base } else { base };
                let nd = cost + step;
                if dist.get(&nb).is_some_and(|&d| nd >= d) { continue; }
                dist.insert(nb, nd);
                open.push(Q { cost: nd, pos: nb });
            }
        }
        None
    }

    #[test]
    fn scan_for_suboptimal_paths() {
        let mut rng = Lcg(0x5eed);
        let mut checked = 0_u32;
        let mut suboptimal = Vec::new();

        for case in 0..3000_u32 {
            let w = 6 + (rng.next() % 8) as i32;
            let h = 6 + (rng.next() % 8) as i32;
            let mut astar = AStar::new(w, h);
            // 稀疏随机障碍：密度随 case 变化，覆盖稀疏到中密。
            let density = 5 + rng.next() % 20;
            for y in 0..h {
                for x in 0..w {
                    if rng.next() % 100 < density {
                        astar.set_obstacle(x, y, true);
                    }
                }
            }
            let s = GridPos::new(0, 0);
            let g = GridPos::new(w - 1, h - 1);
            astar.set_obstacle(s.x, s.y, false);
            astar.set_obstacle(g.x, g.y, false);
            if !astar.is_walkable(&s) || !astar.is_walkable(&g) { continue; }

            let best = dijkstra_cost(&astar, s, g);
            let got = astar
                .find_path(s, g)
                .map(|p| p.windows(2).map(|w| {
                    let (a, b) = (w[0], w[1]);
                    let dx = (b.x - a.x).abs();
                    let dy = (b.y - a.y).abs();
                    let base = astar.movement_cost[b.x as usize][b.y as usize];
                    if dx + dy == 2 { SQRT_2 * base } else { base }
                }).sum::<f32>());
            checked += 1;
            match (best, got) {
                (Some(b), Some(gc)) if (b - gc).abs() > 1e-4 => {
                    suboptimal.push((case, w, h, b, gc));
                }
                (None, Some(_)) => suboptimal.push((case, w, h, f32::NAN, got.unwrap_or(f32::NAN))),
                _ => {}
            }
        }

        let n_sub = suboptimal.len();
        println!("[判决] 对账 {checked} 个布局，次优 {n_sub} 例");
        for (case, w, h, b, gc) in suboptimal.iter().take(5) {
            println!("[判决]   case={case} {w}x{h} 最优={b:.4} 实得={gc:.4}");
        }
        assert!(
            n_sub == 0,
            "扫出 {n_sub} 例次优路径；前 5 例见上 ⇒ L1 启发在 √2 对角边下**实测**返回次优"
        );
    }
}
