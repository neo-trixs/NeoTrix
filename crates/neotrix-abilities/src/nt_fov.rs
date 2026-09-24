//! 可见性 — bracket 对称阴影投射（f64 斜率零依赖移植）+ 连续空间几何 LOS.
//!
//! 公理（roguelike skill）：必须对称 shadowcasting（a 见 b ⟺ b 见 a），朴素逐格
//! raycast 会闪烁不对称；FOV 只在移动后重算（调用方持 fov_recompute 思想）；
//! lit/visited/dim 三态由调用方存，本模块只给可见集。
//! 移植说明：bracket 用 num_rational 精确斜率；本模块用 f64（半径 ≤ 64 精度充足，
//! 零新依赖），对称性由单测锁定。

use std::collections::HashSet;

#[derive(Clone, Copy)]
struct Tile {
    depth: i32,
    column: i32,
}

struct Scanline {
    depth: i32,
    start_slope: f64,
    end_slope: f64,
}

fn round_ties_up(r: f64) -> i32 {
    (r + 0.5).floor() as i32
}

fn round_ties_down(r: f64) -> i32 {
    (r - 0.5).ceil() as i32
}

fn slope(t: Tile) -> f64 {
    (2 * t.column - 1) as f64 / (2 * t.depth) as f64
}

fn is_symmetric(line: &Scanline, t: Tile) -> bool {
    let c = t.column as f64;
    let d = line.depth as f64;
    c >= d * line.start_slope && c <= d * line.end_slope
}

/// 网格 FOV：返回可见格（含原点；越界格不返回）。
/// `opaque(x,y)` 越界时应返回 true（墙外不可见），模块内也做边界钳制。
pub fn compute_fov(
    w: i32,
    h: i32,
    ox: i32,
    oy: i32,
    radius: i32,
    opaque: &dyn Fn(i32, i32) -> bool,
) -> Vec<(i32, i32)> {
    let mut visible: HashSet<(i32, i32)> = HashSet::new();
    visible.insert((ox, oy));
    if radius <= 0 {
        return visible
            .into_iter()
            .filter(|&(x, y)| x >= 0 && y >= 0 && x < w && y < h)
            .collect();
    }
    let r2 = (radius * radius) as f64;
    let rp = radius as f64 + 0.5;
    let rp2 = rp * rp;
    // 四象限变换：(depth,column) -> 世界格
    let quads: [fn(i32, i32, i32, i32) -> (i32, i32); 4] = [
        |ox, oy, d, c| (ox + c, oy - d), // 北
        |ox, oy, d, c| (ox + d, oy + c), // 东（bracket East 用 +depth 行进，此处对称覆盖即可）
        |ox, oy, d, c| (ox + c, oy + d), // 南
        |ox, oy, d, c| (ox - d, oy + c), // 西
    ];
    for xf in quads {
        let mut stack = vec![Scanline { depth: 1, start_slope: -1.0, end_slope: 1.0 }];
        while let Some(mut line) = stack.pop() {
            if (line.depth * line.depth) as f64 > r2 {
                continue;
            }
            let sc = round_ties_up(line.depth as f64 * line.start_slope);
            let ec = round_ties_down(line.depth as f64 * line.end_slope);
            let mut prev: Option<Tile> = None;
            for col in sc..=ec {
                let t = Tile { depth: line.depth, column: col };
                let (wx, wy) = xf(ox, oy, t.depth, t.column);
                let in_map = wx >= 0 && wy >= 0 && wx < w && wy < h;
                let dx = (wx - ox) as f64;
                let dy = (wy - oy) as f64;
                if dx * dx + dy * dy <= rp2 {
                    let is_op = if in_map { opaque(wx, wy) } else { true };
                    if is_op || is_symmetric(&line, t) {
                        if in_map {
                            visible.insert((wx, wy));
                        }
                    }
                    if let Some(p) = prev {
                        let p_op = {
                            let (px2, py2) = xf(ox, oy, p.depth, p.column);
                            if px2 >= 0 && py2 >= 0 && px2 < w && py2 < h {
                                opaque(px2, py2)
                            } else {
                                true
                            }
                        };
                        if p_op && !is_op {
                            line.start_slope = slope(t);
                        }
                        if !p_op && is_op {
                            stack.push(Scanline {
                                depth: line.depth + 1,
                                start_slope: line.start_slope,
                                end_slope: slope(t),
                            });
                        }
                    }
                    prev = Some(t);
                }
            }
            if let Some(p) = prev {
                let (px2, py2) = xf(ox, oy, p.depth, p.column);
                let p_op = if px2 >= 0 && py2 >= 0 && px2 < w && py2 < h {
                    opaque(px2, py2)
                } else {
                    true
                };
                if !p_op {
                    stack.push(Scanline {
                        depth: line.depth + 1,
                        start_slope: line.start_slope,
                        end_slope: line.end_slope,
                    });
                }
            }
        }
    }
    visible.into_iter()
        .filter(|&(x, y)| x >= 0 && y >= 0 && x < w && y < h)
        .collect()
}

/// 连续空间 LOS：线段 (ax,ay)→(bx,by) 是否不穿过任一实心 AABB。
/// `solids` 为左上系 (x,y,w,h)（调用方把中心系 Sprite 转好）。
/// fail-open 方向：嵌墙端点判遮挡（→Patrol，P12；追墙+蹦跳刷屏更丑）；擦角掠过算可见。
pub fn los_clear(
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
    solids: &[(f32, f32, f32, f32)],
) -> bool {
    const EPS: f32 = 1e-3;
    let dx = bx - ax;
    let dy = by - ay;
    for &(x, y, w, h) in solids {
        if w <= 0.0 || h <= 0.0 {
            continue;
        }
        // slab：求线段与矩形相交的 t 区间
        let (mut tmin, mut tmax) = (0.0_f32, 1.0_f32);
        let mut ok = true;
        for (p, d, lo, hi) in [(ax, dx, x, x + w), (ay, dy, y, y + h)] {
            if d.abs() < 1e-9 {
                if p < lo || p > hi {
                    ok = false;
                    break;
                }
            } else {
                let mut t1 = (lo - p) / d;
                let mut t2 = (hi - p) / d;
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    ok = false;
                    break;
                }
            }
        }
        if !ok || tmax - tmin <= 1e-6 {
            continue;
        }
        // 取区间中点：严格落在矩形内（缩 EPS）且 t∈(0,1) 才算遮挡
        let mid = ((tmin + tmax) / 2.0).clamp(0.0, 1.0);
        if mid <= EPS || mid >= 1.0 - EPS {
            continue;
        }
        let mx = ax + dx * mid;
        let my = ay + dy * mid;
        if mx > x + EPS && mx < x + w - EPS && my > y + EPS && my < y + h - EPS {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(_x: i32, _y: i32) -> bool {
        false
    }

    #[test]
    fn open_field_radius_shape() {
        let v = compute_fov(11, 11, 5, 5, 4, &open);
        let set: HashSet<(i32, i32)> = v.into_iter().collect();
        assert!(set.contains(&(5, 5))); // 原点恒可见
        assert!(set.contains(&(9, 5)) && set.contains(&(5, 9))); // 轴向半径内
        assert!(!set.contains(&(10, 5))); // 半径外
        // 半径 4（+0.5 容差）：dx²+dy² ≤ 20.25，(3,3)=18 可见，(4,4)=32 不可见
        assert!(set.contains(&(8, 8)));
        assert!(!set.contains(&(9, 9)));
    }

    #[test]
    fn wall_casts_shadow() {
        // x=5 竖墙（y 3..7），观察者 (2,5)：墙后 (8,5) 不可见，墙体可见，侧绕可见
        let opaque = |x: i32, y: i32| x == 5 && (3..=7).contains(&y);
        let set: HashSet<(i32, i32)> =
            compute_fov(11, 11, 2, 5, 8, &opaque).into_iter().collect();
        assert!(set.contains(&(5, 5))); // 墙体本身可见（lit wall）
        assert!(!set.contains(&(8, 5))); // 墙后阴影
        assert!(set.contains(&(2, 0))); // 开阔方向可见
    }

    #[test]
    fn symmetry_holds() {
        // roguelike 大坑：朴素 raycast 不对称；对称算法必须 a见b ⟺ b见a
        let opaque = |x: i32, y: i32| (x * 7 + y * 13) % 11 < 3; // 确定性伪迷宫
        let vis = |ax: i32, ay: i32| -> HashSet<(i32, i32)> {
            compute_fov(12, 12, ax, ay, 6, &opaque).into_iter().collect()
        };
        for ay in 0..12 {
            for ax in 0..12 {
                if opaque(ax, ay) {
                    continue;
                }
                let va = vis(ax, ay);
                for &(bx, by) in &va {
                    if opaque(bx, by) {
                        continue;
                    }
                    let vb = vis(bx, by);
                    assert!(
                        vb.contains(&(ax, ay)),
                        "不对称：({},{}) 见 ({},{}) 反之不见",
                        ax, ay, bx, by
                    );
                }
            }
        }
    }

    #[test]
    fn zero_radius_and_oob_safe() {
        let v = compute_fov(5, 5, 2, 2, 0, &open);
        assert_eq!(v.len(), 1);
        let v2 = compute_fov(5, 5, -3, -3, 4, &open); // 界外原点不 panic
        assert!(!v2.iter().any(|&(x, y)| x < 0 || y < 0 || x >= 5 || y >= 5));
    }

    #[test]
    fn los_open_and_blocked() {
        let wall = vec![(10.0, 0.0, 4.0, 100.0)];
        assert!(los_clear(0.0, 50.0, 5.0, 50.0, &wall)); // 墙前
        assert!(!los_clear(0.0, 50.0, 20.0, 50.0, &wall)); // 穿墙
        assert!(los_clear(0.0, 50.0, 20.0, 50.0, &[])); // 无墙
        assert!(los_clear(20.0, 50.0, 30.0, 50.0, &wall)); // 墙后（墙在身后不挡）
    }

    #[test]
    fn los_embedded_endpoint_blocked() {
        // 其一端点嵌在实心内（如出生重叠）→ 不可见 → Patrol（P12 fail-open；
        // 反之会追墙+反复起跳刷屏。中点判定天然对称，无需特殊分支。）
        let wall = vec![(0.0, 0.0, 10.0, 10.0)];
        assert!(!los_clear(5.0, 5.0, 50.0, 50.0, &wall));
        assert!(!los_clear(50.0, 50.0, 5.0, 5.0, &wall));
    }

    #[test]
    fn los_grazing_corner_visible() {
        // 擦墙角掠过算可见（不因浮点毛刺失明）
        let wall = vec![(10.0, 10.0, 10.0, 10.0)];
        assert!(los_clear(0.0, 0.0, 30.0, 9.0, &wall));
    }
}
