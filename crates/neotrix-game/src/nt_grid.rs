//! 寻路网格 — 吸收 `OpenWarcraft3` 的 `g_pathing.c` 网格思想.
//!
//! OW3 侧：RTS 地图是行走网格（walkable 网格 + 寻路 + 阻塞标记），
//! 单位指令（`g_commands`）的目标点需先落到可走格再寻路。
//! 本模块是无头数据侧：`WalkGrid` 存行列可走性 + BFS 最短路
//!（`find_path` 四向，返回含起终点的格序列）；大地图长路调用方
//! 自行分片/缓存，本模块保证确定性（邻居序固定：右/下/左/上）。
//!
//! 公理：越界/不可走起终点 → `None`（不用空路径糊弄）；对角穿越墙角
//! 不允许（四向移动无切角问题）。

use std::collections::VecDeque;

/// 行走网格（`true` = 可走）.
#[derive(Debug, Clone)]
pub struct WalkGrid {
    cells: Vec<Vec<bool>>,
    w: usize,
    h: usize,
}

impl WalkGrid {
    /// 全可走网格（宽高任一为 0 → `Err`）.
    pub fn open(w: usize, h: usize) -> Result<Self, String> {
        if w == 0 || h == 0 {
            return Err("网格宽高必须 > 0".to_string());
        }
        Ok(Self { cells: vec![vec![true; w]; h], w, h })
    }

    /// 从行布尔值构造（不等长行 → `Err`）.
    pub fn from_rows(rows: &[Vec<bool>]) -> Result<Self, String> {
        if rows.is_empty() || rows[0].is_empty() {
            return Err("网格行为空".to_string());
        }
        let w = rows[0].len();
        for (i, r) in rows.iter().enumerate() {
            if r.len() != w {
                return Err(format!("第 {} 行宽 {} 与首行 {} 不一致", i, r.len(), w));
            }
        }
        Ok(Self { cells: rows.to_vec(), w, h: rows.len() })
    }

    /// 从 ASCII 构造（`walkable(c)` 为真即行走格；不等长行 → `Err`）.
    pub fn from_ascii(rows: &[&str], walkable: impl Fn(char) -> bool) -> Result<Self, String> {
        if rows.is_empty() {
            return Err("网格行为空".to_string());
        }
        let w = rows[0].chars().count();
        if w == 0 {
            return Err("网格首行为空".to_string());
        }
        let mut cells = Vec::with_capacity(rows.len());
        for (i, r) in rows.iter().enumerate() {
            let row: Vec<char> = r.chars().collect();
            if row.len() != w {
                return Err(format!("第 {} 行宽 {} 与首行 {} 不一致", i, row.len(), w));
            }
            cells.push(row.into_iter().map(&walkable).collect());
        }
        Ok(Self { cells, w, h: rows.len() })
    }

    pub fn width(&self) -> usize {
        self.w
    }
    pub fn height(&self) -> usize {
        self.h
    }

    pub fn walkable(&self, x: usize, y: usize) -> bool {
        x < self.w && y < self.h && self.cells[y][x]
    }

    /// 设置可走性（越界 → `Err`）.
    pub fn set(&mut self, x: usize, y: usize, v: bool) -> Result<(), String> {
        if x >= self.w || y >= self.h {
            return Err(format!("越界格 ({}, {})", x, y));
        }
        self.cells[y][x] = v;
        Ok(())
    }

    /// BFS 最短路（四向；不可达/非法起终点 → `None`）.
    /// 返回含起终点的格序列；起点==终点且可走 → 单点路径.
    pub fn find_path(&self, start: (usize, usize), goal: (usize, usize)) -> Option<Vec<(usize, usize)>> {
        let (sx, sy) = start;
        let (gx, gy) = goal;
        if !self.walkable(sx, sy) || !self.walkable(gx, gy) {
            return None;
        }
        if start == goal {
            return Some(vec![start]);
        }
        // 邻居固定序：右/下/左/上（确定性）
        const DIRS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        let mut prev: Vec<Vec<Option<(usize, usize)>>> = vec![vec![None; self.w]; self.h];
        let mut seen = vec![vec![false; self.w]; self.h];
        let mut q = VecDeque::new();
        seen[sy][sx] = true;
        q.push_back((sx, sy));
        while let Some((x, y)) = q.pop_front() {
            if (x, y) == (gx, gy) {
                break;
            }
            for (dx, dy) in DIRS {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let (nx, ny) = (nx as usize, ny as usize);
                if nx >= self.w || ny >= self.h || seen[ny][nx] || !self.cells[ny][nx] {
                    continue;
                }
                seen[ny][nx] = true;
                prev[ny][nx] = Some((x, y));
                q.push_back((nx, ny));
            }
        }
        if !seen[gy][gx] {
            return None;
        }
        // 回溯
        let mut path = vec![(gx, gy)];
        let mut cur = (gx, gy);
        while cur != (sx, sy) {
            match prev[cur.1][cur.0] {
                Some(p) => {
                    cur = p;
                    path.push(cur);
                }
                None => return None,
            }
        }
        path.reverse();
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_grid_straight_path() {
        let g = WalkGrid::open(5, 3).unwrap();
        let p = g.find_path((0, 0), (4, 0)).unwrap();
        assert_eq!(p.first(), Some(&(0, 0)));
        assert_eq!(p.last(), Some(&(4, 0)));
        assert_eq!(p.len(), 5); // 直线最短
    }

    #[test]
    fn wall_detours() {
        // 中间一堵墙，上下绕行
        let g = WalkGrid::from_ascii(&[".....", ".###.", "....."], |c| c == '.').unwrap();
        let p = g.find_path((0, 1), (4, 1)).unwrap();
        // 必须绕行：路径长 > 直线 5
        assert!(p.len() > 5);
        assert_eq!(*p.first().unwrap(), (0, 1));
        assert_eq!(*p.last().unwrap(), (4, 1));
        for (x, y) in &p {
            assert!(g.walkable(*x, *y));
        }
    }

    #[test]
    fn unreachable_and_bad_endpoints() {
        let g = WalkGrid::from_ascii(&[".#.", "###", ".#."], |c| c == '.').unwrap();
        assert!(g.find_path((0, 0), (2, 0)).is_none()); // 隔断
        assert!(g.find_path((1, 0), (0, 0)).is_none()); // 起点不可走
        assert!(g.find_path((0, 0), (9, 9)).is_none()); // 越界
        let h = WalkGrid::open(3, 3).unwrap();
        assert_eq!(h.find_path((1, 1), (1, 1)), Some(vec![(1, 1)]));
    }

    #[test]
    fn bad_shapes_rejected() {
        assert!(WalkGrid::open(0, 3).is_err());
        assert!(WalkGrid::from_rows(&[]).is_err());
        assert!(WalkGrid::from_rows(&[vec![true], vec![true, false]]).is_err());
        assert!(WalkGrid::from_ascii(&["..", "."], |c| c == '.').is_err());
        let mut g = WalkGrid::open(2, 2).unwrap();
        assert!(g.set(5, 5, false).is_err());
        assert!(g.set(1, 1, false).is_ok());
        assert!(!g.walkable(1, 1));
    }
}
