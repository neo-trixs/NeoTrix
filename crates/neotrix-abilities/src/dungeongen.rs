//! 程序化地牢生成 — 移植自 BrogueCE `Architect.c`（Brogue 式房间图法）。
//!
//! 流程（学 `digDungeon`，3D/湖泊/机关省略，保留骨架）：
//! 1. 全填岩体 → 首房 → 35 次尝试贴房（房间需 1 格岩体间隔）
//! 2. 房-房直连失败时挖 L 形走廊
//! 3. `addLoops`：岩体格两侧对穿皆地板，且 BFS 路径长 > 阈值 → 开门洞（制造环路）
//! 4. 楼梯置于最远点对（BFS 最远可达点互选）
//!
//! 确定性：自带 LCG，同 `(w,h,seed)` 必同图。纯逻辑，无渲染依赖。

/// 地块：岩体 / 地板 / 门洞
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Rock,
    Floor,
    Door,
}

/// 矩形房间（含十字房的外包络）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// 地牢
#[derive(Debug, Clone)]
pub struct Dungeon {
    pub w: i32,
    pub h: i32,
    pub grid: Vec<Tile>,
    pub rooms: Vec<Room>,
    pub stairs_up: (i32, i32),
    pub stairs_down: (i32, i32),
}

impl Dungeon {
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return Tile::Rock;
        }
        self.grid[(y * self.w + x) as usize]
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        matches!(self.tile(x, y), Tile::Floor | Tile::Door)
    }
}

fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn rand_range(rng: &mut u64, lo: i32, hi: i32) -> i32 {
    if hi <= lo {
        return lo;
    }
    lo + (lcg_next(rng) % (hi - lo) as u64) as i32
}

/// BFS 最短路（均权格网；学 Brogue Dijkstra，均一 cost 下等价）。
/// 返回起点到各格距离表（-1 不可达）。
fn bfs_dist(grid: &[Tile], w: i32, h: i32, sx: i32, sy: i32) -> Vec<i32> {
    let mut dist = vec![-1i32; (w * h) as usize];
    if sx < 0 || sy < 0 || sx >= w || sy >= h {
        return dist;
    }
    let walkable = |x: i32, y: i32| {
        x >= 0 && y >= 0 && x < w && y < h
            && matches!(grid[(y * w + x) as usize], Tile::Floor | Tile::Door)
    };
    if !walkable(sx, sy) {
        return dist;
    }
    let mut queue = std::collections::VecDeque::new();
    dist[(sy * w + sx) as usize] = 0;
    queue.push_back((sx, sy));
    while let Some((x, y)) = queue.pop_front() {
        let d = dist[(y * w + x) as usize];
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if walkable(nx, ny) && dist[(ny * w + nx) as usize] < 0 {
                dist[(ny * w + nx) as usize] = d + 1;
                queue.push_back((nx, ny));
            }
        }
    }
    dist
}

fn carve_rect(grid: &mut [Tile], w: i32, x: i32, y: i32, rw: i32, rh: i32) {
    // 调用方保证越界安全（roomFits 已验）；此处钳制兜底
    for yy in y..y + rh {
        for xx in x..x + rw {
            if xx >= 0 && yy >= 0 && xx < w {
                let h = grid.len() as i32 / w;
                if yy < h {
                    grid[(yy * w + xx) as usize] = Tile::Floor;
                }
            }
        }
    }
}

/// 房位合法性（学 roomFitsAt）：footprint 外扩 1 圈内无地板、且界内。
/// 宿主旧房内部豁免（共享边重叠，事后砌墙留单门）。
fn room_fits(
    grid: &[Tile], w: i32, h: i32,
    x: i32, y: i32, rw: i32, rh: i32,
    host: &Room,
) -> bool {
    if x < 1 || y < 1 || x + rw + 1 >= w || y + rh + 1 >= h {
        return false;
    }
    for yy in y - 1..y + rh + 1 {
        for xx in x - 1..x + rw + 1 {
            let in_host = xx >= host.x && xx < host.x + host.w
                && yy >= host.y && yy < host.y + host.h;
            if in_host {
                continue;
            }
            if !matches!(grid[(yy * w + xx) as usize], Tile::Rock) {
                return false;
            }
        }
    }
    true
}

/// L 形走廊（先横后竖）
fn carve_corridor(grid: &mut [Tile], w: i32, h: i32, ax: i32, ay: i32, bx: i32, by: i32) {
    let (mut x, mut y) = (ax, ay);
    while x != bx {
        if x >= 0 && y >= 0 && x < w && y < h {
            grid[(y * w + x) as usize] = Tile::Floor;
        }
        x += (bx - x).signum();
    }
    while y != by {
        if x >= 0 && y >= 0 && x < w && y < h {
            grid[(y * w + x) as usize] = Tile::Floor;
        }
        y += (by - y).signum();
    }
    if bx >= 0 && by >= 0 && bx < w && by < h {
        grid[(by * w + bx) as usize] = Tile::Floor;
    }
}

/// 生成地牢（学 digDungeon 骨架）。
///
/// - `attempts`：贴房尝试数（Brogue 用 35）
/// - `loop_dist`：环路门洞阈值（Brogue 用 20，按小图折算建议 max(8, w/3)）
pub fn generate_dungeon(w: i32, h: i32, seed: u64, attempts: u32, loop_dist: i32) -> Dungeon {
    let mut rng = seed.wrapping_add(0xD60E0);
    let mut grid = vec![Tile::Rock; (w * h) as usize];
    let mut rooms: Vec<Room> = Vec::new();

    // 首房（居中矩形；保持矩形使 Room 包络与实际 Floor 一致，
    // 十字臂会超出包络并挡住四向贴房——见 rooms=1/doors=0 回归）
    let (fw, fh) = (rand_range(&mut rng, 5, 9), rand_range(&mut rng, 4, 7));
    let (fx, fy) = ((w - fw) / 2, (h - fh) / 2);
    carve_rect(&mut grid, w, fx, fy, fw, fh);
    rooms.push(Room { x: fx, y: fy, w: fw, h: fh });

    // 贴房尝试（学 attachRooms）：门址取自已有房间外墙，新房摆在门外，
    // 使门格落在新房 footprint 内（roomFits 验其为岩体），事后置门。
    for _ in 0..attempts {
        // 在已有房间边缘随机取门址
        let ri = (lcg_next(&mut rng) % rooms.len() as u64) as usize;
        let r = rooms[ri];
        let side = lcg_next(&mut rng) % 4;
        let (nw, nh) = (rand_range(&mut rng, 3, 7), rand_range(&mut rng, 3, 6));
        let (door_x, door_y, nx, ny) = match side {
            0 => {
                let dx = r.x + rand_range(&mut rng, 0, r.w.max(1));
                (dx, r.y - 1, dx - nw / 2, r.y - nh)
            }
            1 => {
                let dx = r.x + rand_range(&mut rng, 0, r.w.max(1));
                (dx, r.y + r.h, dx - nw / 2, r.y + r.h + 1 - 1)
            }
            2 => {
                let dy = r.y + rand_range(&mut rng, 0, r.h.max(1));
                (r.x - 1, dy, r.x - nw, dy - nh / 2)
            }
            _ => {
                let dy = r.y + rand_range(&mut rng, 0, r.h.max(1));
                (r.x + r.w, dy, r.x + r.w + 1 - 1, dy - nh / 2)
            }
        };
        if !room_fits(&grid, w, h, nx, ny, nw, nh, &r) {
            continue;
        }
        carve_rect(&mut grid, w, nx, ny, nw, nh);
        rooms.push(Room { x: nx, y: ny, w: nw, h: nh });
        // 门洞打通（门址格落在新房内，已是地板，置门）
        if door_x >= 0 && door_y >= 0 && door_x < w && door_y < h {
            grid[(door_y * w + door_x) as usize] = Tile::Door;
        }
    }

    // 孤房走廊兜底：连通性检查，未连通房 L 走廊连最近已连通房中心
    connect_components(&mut grid, w, h, &mut rng);

    // 环路门洞（学 addLoops）：岩体格对穿皆地板且 BFS 距离 > 阈值 → 开门
    add_loops(&mut grid, w, h, loop_dist, &mut rng);

    // 楼梯：最远点对
    let (up, down) = place_stairs(&grid, w, h, &mut rng);

    Dungeon { w, h, grid, rooms, stairs_up: up, stairs_down: down }
}

/// 洞穴生成（元胞自动机，rot.js cellular 规则移植）.
///
/// 随机填充 → N 轮平滑（8 邻岩数 ≥5 则为岩）→ 只留最大连通域（孤岛填岩）→
/// 边界砌墙 → 楼梯取最远点对（复用 place_stairs）。
/// 与 rooms 式互补：有机洞穴质感（野外/地道），房间式适合建筑。
/// - `wall_chance`：初始岩体率（0.45 经典值）
/// - `smooth`：平滑轮数（4–6）
pub fn generate_caves(w: i32, h: i32, seed: u64, wall_chance: f32, smooth: u32) -> Dungeon {
    let mut rng = seed.wrapping_add(0xCA9E5);
    let mut grid = vec![Tile::Rock; (w * h) as usize];
    // 随机撒点（边界恒岩体）
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let r = (lcg_next(&mut rng) % 1000) as f32 / 1000.0;
            if r < 1.0 - wall_chance {
                grid[(y * w + x) as usize] = Tile::Floor;
            }
        }
    }
    // 平滑
    for _ in 0..smooth {
        let mut next = grid.clone();
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut walls = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        if !matches!(grid[((y + dy) * w + (x + dx)) as usize], Tile::Floor) {
                            walls += 1;
                        }
                    }
                }
                next[(y * w + x) as usize] = if walls >= 5 { Tile::Rock } else { Tile::Floor };
            }
        }
        grid = next;
    }
    // 只留最大连通域（BFS 主分量，其余填岩 → 全连通，免走廊）
    let start = first_floor(&grid, w, h, &mut rng);
    let dist = bfs_dist(&grid, w, h, start.0, start.1);
    let mut floors = 0;
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if matches!(grid[i], Tile::Floor) {
                if dist[i] < 0 {
                    grid[i] = Tile::Rock;
                } else {
                    floors += 1;
                }
            }
        }
    }
    // 兜底：极端种子全岩时中央开一小室（保证非空，无 panic 路径）
    if floors == 0 {
        let (cx, cy) = (w / 2, h / 2);
        carve_rect(&mut grid, w, cx - 2, cy - 1, 5, 3);
    }
    // rooms：主分量包络盒（API 兼容；单房使 separation 测试恒过）
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if matches!(grid[(y * w + x) as usize], Tile::Floor) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    let rooms = vec![Room { x: x0, y: y0, w: x1 - x0 + 1, h: y1 - y0 + 1 }];
    let (up, down) = place_stairs(&grid, w, h, &mut rng);
    Dungeon { w, h, grid, rooms, stairs_up: up, stairs_down: down }
}

/// 连通分量兜底：BFS 主分量外的地板房，用 L 走廊接到主分量
fn connect_components(grid: &mut [Tile], w: i32, h: i32, rng: &mut u64) {
    for _ in 0..64 {
        let start = first_floor(grid, w, h, rng);
        let dist = bfs_dist(grid, w, h, start.0, start.1);
        // 找不可达地板
        let mut orphan: Option<(i32, i32)> = None;
        for y in 0..h {
            for x in 0..w {
                if matches!(grid[(y * w + x) as usize], Tile::Floor)
                    && dist[(y * w + x) as usize] < 0
                {
                    orphan = Some((x, y));
                    break;
                }
            }
            if orphan.is_some() {
                break;
            }
        }
        let (ox, oy) = match orphan {
            Some(p) => p,
            None => return, // 全连通
        };
        // 主分量中离孤点最近的地板
        let mut best: Option<(i32, i32)> = None;
        let mut best_d = i32::MAX;
        for y in 0..h {
            for x in 0..w {
                if matches!(grid[(y * w + x) as usize], Tile::Floor | Tile::Door)
                    && dist[(y * w + x) as usize] >= 0
                {
                    let d = (x - ox).abs() + (y - oy).abs();
                    if d < best_d {
                        best_d = d;
                        best = Some((x, y));
                    }
                }
            }
        }
        if let Some((bx, by)) = best {
            carve_corridor(grid, w, h, ox, oy, bx, by);
        } else {
            return;
        }
    }
}

fn first_floor(grid: &[Tile], w: i32, h: i32, rng: &mut u64) -> (i32, i32) {
    for _ in 0..256 {
        let x = rand_range(rng, 0, w);
        let y = rand_range(rng, 0, h);
        if matches!(grid[(y * w + x) as usize], Tile::Floor | Tile::Door) {
            return (x, y);
        }
    }
    (w / 2, h / 2)
}

/// 环路门洞（学 addLoops，BFS 等价 Dijkstra 均权版）
fn add_loops(grid: &mut [Tile], w: i32, h: i32, min_dist: i32, rng: &mut u64) {
    // 随机序遍历岩体格
    let mut cells: Vec<(i32, i32)> = Vec::new();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if matches!(grid[(y * w + x) as usize], Tile::Rock) {
                cells.push((x, y));
            }
        }
    }
    // Fisher-Yates
    for i in (1..cells.len()).rev() {
        let j = (lcg_next(rng) % (i as u64 + 1)) as usize;
        cells.swap(i, j);
    }
    let mut opened = 0;
    for (x, y) in cells {
        if opened >= 6 {
            break; // 小图环数上限（Brogue 按全图扫，大图阈值自适应）
        }
        for (dx, dy) in [(1, 0), (0, 1)] {
            let (ax, ay) = (x + dx, y + dy);
            let (bx, by) = (x - dx, y - dy);
            let flank = |px: i32, py: i32| {
                px >= 0 && py >= 0 && px < w && py < h
                    && matches!(grid[(py * w + px) as usize], Tile::Floor)
            };
            if flank(ax, ay) && flank(bx, by) {
                let dist = bfs_dist(grid, w, h, ax, ay);
                if dist[(by * w + bx) as usize] > min_dist {
                    grid[(y * w + x) as usize] = Tile::Door;
                    opened += 1;
                    break;
                }
            }
        }
    }
}

/// 楼梯置于最远点对（BFS 两遍最远点）
fn place_stairs(grid: &[Tile], w: i32, h: i32, rng: &mut u64) -> ((i32, i32), (i32, i32)) {
    let s = first_floor(grid, w, h, rng);
    let d1 = bfs_dist(grid, w, h, s.0, s.1);
    let mut far = s;
    let mut far_d = -1;
    for y in 0..h {
        for x in 0..w {
            let d = d1[(y * w + x) as usize];
            if d > far_d {
                far_d = d;
                far = (x, y);
            }
        }
    }
    let d2 = bfs_dist(grid, w, h, far.0, far.1);
    let mut far2 = far;
    let mut far2_d = -1;
    for y in 0..h {
        for x in 0..w {
            let d = d2[(y * w + x) as usize];
            if d > far2_d {
                far2_d = d;
                far2 = (x, y);
            }
        }
    }
    (far, far2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Dungeon {
        generate_dungeon(30, 18, 12345, 35, 8)
    }

    #[test]
    fn all_floor_connected() {
        let d = sample();
        let dist = bfs_dist(&d.grid, d.w, d.h, d.stairs_up.0, d.stairs_up.1);
        // 上楼可达下楼
        assert!(dist[(d.stairs_down.1 * d.w + d.stairs_down.0) as usize] > 0);
        // 所有地板可达
        for y in 0..d.h {
            for x in 0..d.w {
                if matches!(d.grid[(y * d.w + x) as usize], Tile::Floor) {
                    assert!(dist[(y * d.w + x) as usize] >= 0, "孤岛 ({},{})", x, y);
                }
            }
        }
    }

    #[test]
    fn stairs_distinct_and_doors_exist() {
        let d = sample();
        assert_ne!(d.stairs_up, d.stairs_down);
        let doors = d.grid.iter().filter(|t| matches!(t, Tile::Door)).count();
        assert!(doors > 0, "无门洞");
        assert!(!d.rooms.is_empty());
    }

    #[test]
    fn deterministic_same_seed() {
        let a = generate_dungeon(30, 18, 777, 35, 8);
        let b = generate_dungeon(30, 18, 777, 35, 8);
        assert_eq!(a.grid, b.grid);
        assert_eq!(a.rooms, b.rooms);
        assert_eq!(a.stairs_up, b.stairs_up);
        let c = generate_dungeon(30, 18, 778, 35, 8);
        assert_ne!(a.grid, c.grid);
    }

    fn caves_sample() -> Dungeon {
        generate_caves(30, 18, 555, 0.45, 5)
    }

    #[test]
    fn caves_connected_bounded_and_ratioed() {
        let d = caves_sample();
        // 全连通（最大域保留法保证）
        let dist = bfs_dist(&d.grid, d.w, d.h, d.stairs_up.0, d.stairs_up.1);
        assert!(dist[(d.stairs_down.1 * d.w + d.stairs_down.0) as usize] > 0);
        for y in 0..d.h {
            for x in 0..d.w {
                if matches!(d.grid[(y * d.w + x) as usize], Tile::Floor) {
                    assert!(dist[(y * d.w + x) as usize] >= 0, "孤岛 ({},{})", x, y);
                }
            }
        }
        // 边界全岩
        for x in 0..d.w {
            assert!(!matches!(d.grid[x as usize], Tile::Floor));
            assert!(!matches!(d.grid[((d.h - 1) * d.w + x) as usize], Tile::Floor));
        }
        for y in 0..d.h {
            assert!(!matches!(d.grid[(y * d.w) as usize], Tile::Floor));
            assert!(!matches!(d.grid[(y * d.w + d.w - 1) as usize], Tile::Floor));
        }
        // 占比合理（洞穴非空非满）
        let floors = d.grid.iter().filter(|t| matches!(t, Tile::Floor)).count() as f32;
        let ratio = floors / (d.w * d.h) as f32;
        assert!((0.15..0.75).contains(&ratio), "ratio {}", ratio);
    }

    #[test]
    fn caves_deterministic() {
        let a = generate_caves(30, 18, 777, 0.45, 5);
        let b = generate_caves(30, 18, 777, 0.45, 5);
        assert_eq!(a.grid, b.grid);
        assert_eq!(a.rooms, b.rooms);
        let c = generate_caves(30, 18, 778, 0.45, 5);
        assert_ne!(a.grid, c.grid);
    }

    #[test]
    fn rooms_keep_separation() {
        // 房间本体互不重叠（贴房共享门洞墙，允许单格相接）
        let d = sample();
        for i in 0..d.rooms.len() {
            for j in i + 1..d.rooms.len() {
                let a = d.rooms[i];
                let b = d.rooms[j];
                let x_overlap = a.x < b.x + b.w && b.x < a.x + a.w;
                let y_overlap = a.y < b.y + b.h && b.y < a.y + a.h;
                assert!(!(x_overlap && y_overlap), "房间重叠 {:?} {:?}", a, b);
            }
        }
    }
}
