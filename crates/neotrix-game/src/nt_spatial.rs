//! 扁平网格哈希空间索引 — 无头数据侧邻近查询.
//!
//! 大地图实体按固定格长散列到 `(i32, i32)` 格中，
//! 半径 / 矩形查询先算覆盖格全扫、再做精确过滤，
//! 结果去重后按 id 排序，保证确定性。
//! 本模块只存点数据（id + 坐标），调用方负责重建时机。
//!
//! 公理：非法输入按空结果降级（`r <= 0` / 非法矩形 / 非有限坐标 → 空）；
//! 同一实体多条目不去重存储，去重只在查询侧做。

use std::collections::{HashMap, HashSet};

/// 扁平网格哈希（格长 `cell` 恒大于 0）.
#[derive(Debug, Clone, Default)]
pub struct SpatialHash {
    cell: f32,
    cells: HashMap<(i32, i32), Vec<u64>>,
    /// 全量条目（含重复），用于精确距离 / 矩形边界过滤.
    items: Vec<(u64, f32, f32)>,
}

impl SpatialHash {
    /// 新建（`cell <= 0` 或非有限值则按 `64.0`）.
    pub fn new(cell: f32) -> Self {
        let c = if cell.is_finite() && cell > 0.0 {
            cell
        } else {
            64.0
        };
        Self {
            cell: c,
            cells: HashMap::new(),
            items: Vec::new(),
        }
    }

    /// 清空全部条目.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.items.clear();
    }

    /// 先清后建；同一实体多条目原样保留，去重由查询侧处理.
    pub fn rebuild(&mut self, items: &[(u64, f32, f32)]) {
        self.clear();
        for (id, px, py) in items.iter() {
            if !px.is_finite() || !py.is_finite() {
                continue;
            }
            let key = self.cell_of(*px, *py);
            self.cells.entry(key).or_default().push(*id);
            self.items.push((*id, *px, *py));
        }
    }

    /// 半径查询：覆盖格全扫 + 精确距离过滤，去重后按 id 排序；`r <= 0` 返回空.
    pub fn query_radius(&self, x: f32, y: f32, r: f32) -> Vec<u64> {
        if !x.is_finite() || !y.is_finite() || !r.is_finite() || r <= 0.0 {
            return Vec::new();
        }
        let (cx0, cy0) = self.cell_of(x - r, y - r);
        let (cx1, cy1) = self.cell_of(x + r, y + r);
        let mut cand: HashSet<u64> = HashSet::new();
        let mut cx = cx0;
        while cx <= cx1 {
            let mut cy = cy0;
            while cy <= cy1 {
                if let Some(ids) = self.cells.get(&(cx, cy)) {
                    for id in ids.iter() {
                        cand.insert(*id);
                    }
                }
                if cy == cy1 {
                    break;
                }
                cy += 1;
            }
            if cx == cx1 {
                break;
            }
            cx += 1;
        }
        if cand.is_empty() {
            return Vec::new();
        }
        let rr = f64::from(r) * f64::from(r);
        let mut hit: HashSet<u64> = HashSet::new();
        for (id, px, py) in self.items.iter() {
            if !cand.contains(id) || hit.contains(id) {
                continue;
            }
            let dx = f64::from(*px) - f64::from(x);
            let dy = f64::from(*py) - f64::from(y);
            if dx * dx + dy * dy <= rr {
                hit.insert(*id);
            }
        }
        let mut out: Vec<u64> = hit.into_iter().collect();
        out.sort_unstable();
        out
    }

    /// 矩形查询（含边界），去重排序；非法矩形（`x0 > x1` 等）返回空.
    pub fn query_rect(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<u64> {
        if !x0.is_finite() || !y0.is_finite() || !x1.is_finite() || !y1.is_finite() {
            return Vec::new();
        }
        if !(x0 <= x1 && y0 <= y1) {
            return Vec::new();
        }
        let (cx0, cy0) = self.cell_of(x0, y0);
        let (cx1, cy1) = self.cell_of(x1, y1);
        let mut cand: HashSet<u64> = HashSet::new();
        let mut cx = cx0;
        while cx <= cx1 {
            let mut cy = cy0;
            while cy <= cy1 {
                if let Some(ids) = self.cells.get(&(cx, cy)) {
                    for id in ids.iter() {
                        cand.insert(*id);
                    }
                }
                if cy == cy1 {
                    break;
                }
                cy += 1;
            }
            if cx == cx1 {
                break;
            }
            cx += 1;
        }
        if cand.is_empty() {
            return Vec::new();
        }
        let mut hit: HashSet<u64> = HashSet::new();
        for (id, px, py) in self.items.iter() {
            if !cand.contains(id) || hit.contains(id) {
                continue;
            }
            if *px >= x0 && *px <= x1 && *py >= y0 && *py <= y1 {
                hit.insert(*id);
            }
        }
        let mut out: Vec<u64> = hit.into_iter().collect();
        out.sort_unstable();
        out
    }

    /// 全部条目数（含重复）.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 当前格长.
    pub fn cell_size(&self) -> f32 {
        self.cell
    }

    fn cell_of(&self, px: f32, py: f32) -> (i32, i32) {
        let cx = (f64::from(px) / f64::from(self.cell)).floor() as i32;
        let cy = (f64::from(py) / f64::from(self.cell)).floor() as i32;
        (cx, cy)
    }
}

#[cfg(test)]
mod tests {
    use super::SpatialHash;

    #[test]
    fn radius_hit_and_miss() {
        let mut h = SpatialHash::new(10.0);
        h.rebuild(&[(1, 0.0, 0.0), (2, 100.0, 100.0)]);
        let got = h.query_radius(0.0, 0.0, 5.0);
        assert_eq!(got, vec![1]);
        let miss = h.query_radius(-100.0, -100.0, 5.0);
        assert!(miss.is_empty());
    }

    #[test]
    fn radius_across_cells() {
        let mut h = SpatialHash::new(10.0);
        h.rebuild(&[(1, 9.0, 5.0), (2, 11.0, 5.0), (3, 50.0, 50.0)]);
        let got = h.query_radius(10.0, 5.0, 2.0);
        assert_eq!(got, vec![1, 2]);
    }

    #[test]
    fn empty_index_returns_empty() {
        let h = SpatialHash::new(16.0);
        assert!(h.query_radius(0.0, 0.0, 100.0).is_empty());
        assert!(h.query_rect(-10.0, -10.0, 10.0, 10.0).is_empty());
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn rebuild_replaces_old_data() {
        let mut h = SpatialHash::new(10.0);
        h.rebuild(&[(1, 0.0, 0.0)]);
        assert_eq!(h.len(), 1);
        h.rebuild(&[(2, 0.0, 0.0), (3, 1.0, 1.0)]);
        let got = h.query_radius(0.0, 0.0, 5.0);
        assert_eq!(got, vec![2, 3]);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn rect_boundary_inclusive() {
        let mut h = SpatialHash::new(10.0);
        h.rebuild(&[(1, 0.0, 0.0), (2, 10.0, 10.0), (3, 10.1, 10.0)]);
        let got = h.query_rect(0.0, 0.0, 10.0, 10.0);
        assert_eq!(got, vec![1, 2]);
    }

    #[test]
    fn rect_invalid_and_radius_nonpositive_empty() {
        let mut h = SpatialHash::new(10.0);
        h.rebuild(&[(1, 0.0, 0.0)]);
        assert!(h.query_rect(5.0, 0.0, 1.0, 4.0).is_empty());
        assert!(h.query_rect(0.0, 4.0, 4.0, 1.0).is_empty());
        assert!(h.query_radius(0.0, 0.0, 0.0).is_empty());
        assert!(h.query_radius(0.0, 0.0, -3.0).is_empty());
    }

    #[test]
    fn new_bad_cell_falls_back_to_64() {
        let h = SpatialHash::new(0.0);
        assert_eq!(h.cell_size(), 64.0);
        let h2 = SpatialHash::new(-5.0);
        assert_eq!(h2.cell_size(), 64.0);
    }
}
