//! ASCII 关卡/瓦片图 — 吸收 `kaplay` 的 `k.addLevel` 关卡思想.
//!
//! KAPLAY 侧：`addLevel(["  ===  ", "  ...  "], {tileW, tileH,
//! "=": () => [rect(), area(), solid()], ...})` —— 字符画即关卡，
//! 字符→组件工厂解耦美术与布局。
//! 本模块是无头数据侧：`Tilemap::from_ascii` 解析字符矩阵，字符语义由
//! 调用方解释（`tiles_matching(ch)` / `spawn_points(ch)` 取世界坐标，
//! `is_solid` 供物理查询）；渲染/组件挂载由游戏侧完成，保持引擎中性。
//!
//! 公理：行列即数据（不等长行 → `Err`，不用 panic）；`tile` 边长必须 > 0；
//! 世界坐标原点在左上，瓦片中心 = `(tx+0.5)*tile`。

/// 无头瓦片图.
#[derive(Debug, Clone)]
pub struct Tilemap {
    rows: Vec<Vec<char>>,
    w: usize,
    h: usize,
    tile: f32,
}

impl Tilemap {
    /// 从 ASCII 行解析. 空关卡 / 不等长行 / 非正 tile → `Err`.
    pub fn from_ascii(rows: &[&str], tile: f32) -> Result<Self, String> {
        if !(tile > 0.0) {
            return Err("tile 边长必须 > 0".to_string());
        }
        if rows.is_empty() {
            return Err("关卡行为空".to_string());
        }
        let w = rows[0].chars().count();
        if w == 0 {
            return Err("关卡首行为空".to_string());
        }
        let mut grid = Vec::with_capacity(rows.len());
        for (i, r) in rows.iter().enumerate() {
            let chars: Vec<char> = r.chars().collect();
            if chars.len() != w {
                return Err(format!("第 {} 行宽 {} 与首行 {} 不一致", i, chars.len(), w));
            }
            grid.push(chars);
        }
        Ok(Self { rows: grid, w, h: rows.len(), tile })
    }

    pub fn width(&self) -> usize {
        self.w
    }
    pub fn height(&self) -> usize {
        self.h
    }
    pub fn tile(&self) -> f32 {
        self.tile
    }
    pub fn pixel_width(&self) -> f32 {
        self.w as f32 * self.tile
    }
    pub fn pixel_height(&self) -> f32 {
        self.h as f32 * self.tile
    }

    /// 瓦片字符（越界 → `None`）.
    pub fn at(&self, tx: usize, ty: usize) -> Option<char> {
        if tx < self.w && ty < self.h {
            self.rows.get(ty)?.get(tx).copied()
        } else {
            None
        }
    }

    /// 世界坐标 → 瓦片下标（图外 → `None`）.
    pub fn world_to_tile(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let tx = (x / self.tile) as usize;
        let ty = (y / self.tile) as usize;
        if tx < self.w && ty < self.h {
            Some((tx, ty))
        } else {
            None
        }
    }

    /// 按谓词判定实心（调用方传入 solid 字符集语义，如 `|c| c == '='`）.
    pub fn is_solid(&self, tx: usize, ty: usize, solid: impl Fn(char) -> bool) -> bool {
        self.at(tx, ty).map_or(false, solid)
    }

    /// 世界点是否落在实心瓦片上（图外按空处理）.
    pub fn solid_at_world(&self, x: f32, y: f32, solid: impl Fn(char) -> bool) -> bool {
        match self.world_to_tile(x, y) {
            Some((tx, ty)) => self.is_solid(tx, ty, solid),
            None => false,
        }
    }

    /// 全部匹配字符的瓦片下标（行序）.
    pub fn tiles_matching(&self, ch: char) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for ty in 0..self.h {
            for tx in 0..self.w {
                if self.rows[ty][tx] == ch {
                    out.push((tx, ty));
                }
            }
        }
        out
    }

    /// 某字符全部出生点（世界坐标 = 瓦片中心，行序）.
    pub fn spawn_points(&self, ch: char) -> Vec<(f32, f32)> {
        self.tiles_matching(ch)
            .into_iter()
            .map(|(tx, ty)| ((tx as f32 + 0.5) * self.tile, (ty as f32 + 0.5) * self.tile))
            .collect()
    }

    /// 实心瓦片的 AABB 列表 `(cx, cy, w, h)`（中心系，供 `nt_platform` 批量接入）.
    pub fn solid_aabbs(&self, solid: impl Fn(char) -> bool) -> Vec<(f32, f32, f32, f32)> {
        let mut out = Vec::new();
        for ty in 0..self.h {
            for tx in 0..self.w {
                if solid(self.rows[ty][tx]) {
                    out.push((
                        (tx as f32 + 0.5) * self.tile,
                        (ty as f32 + 0.5) * self.tile,
                        self.tile,
                        self.tile,
                    ));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level() -> Tilemap {
        Tilemap::from_ascii(&["P= ", "===", "   "], 32.0).unwrap()
    }

    #[test]
    fn parse_and_dims() {
        let m = level();
        assert_eq!((m.width(), m.height()), (3, 3));
        assert_eq!(m.at(0, 0), Some('P'));
        assert_eq!(m.at(9, 9), None);
        assert!((m.pixel_width() - 96.0).abs() < 1e-3);
    }

    #[test]
    fn ragged_rows_rejected() {
        assert!(Tilemap::from_ascii(&["==", "="], 16.0).is_err());
        assert!(Tilemap::from_ascii(&[], 16.0).is_err());
        assert!(Tilemap::from_ascii(&["="], 0.0).is_err());
    }

    #[test]
    fn solid_queries() {
        let m = level();
        let solid = |c: char| c == '=';
        assert!(!m.is_solid(0, 0, solid));
        assert!(m.is_solid(1, 0, solid));
        // 世界坐标：(48,16) 落在 (1,0) 瓦片
        assert!(m.solid_at_world(48.0, 16.0, solid));
        assert!(!m.solid_at_world(16.0, 16.0, solid));
        assert!(!m.solid_at_world(-5.0, 5.0, solid));
        assert!(!m.solid_at_world(9999.0, 9999.0, solid));
    }

    #[test]
    fn spawn_points_are_centers() {
        let m = level();
        let ps = m.spawn_points('P');
        assert_eq!(ps.len(), 1);
        assert!((ps[0].0 - 16.0).abs() < 1e-3 && (ps[0].1 - 16.0).abs() < 1e-3);
        // '=' 共 4 块 → 4 个 AABB
        assert_eq!(m.solid_aabbs(|c| c == '=').len(), 4);
    }
}
