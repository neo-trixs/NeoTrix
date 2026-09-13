use std::collections::HashMap;
use crate::engine::renderer::{Vec2, Rect};

// ---------------------------------------------------------------------------
// Spatial Hash Grid — O(n) broad-phase collision detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SpatialCell {
    pub entities: Vec<u64>,
}

pub struct SpatialHashGrid {
    cell_size: f32,
    inv_cell_size: f32,
    cells: HashMap<(i32, i32), SpatialCell>,
    entity_cells: HashMap<u64, Vec<(i32, i32)>>,
}

impl SpatialHashGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            inv_cell_size: 1.0 / cell_size,
            cells: HashMap::new(),
            entity_cells: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_cells.clear();
    }

    fn cell_key(&self, x: f32, y: f32) -> (i32, i32) {
        ((x * self.inv_cell_size).floor() as i32, (y * self.inv_cell_size).floor() as i32)
    }

    fn aabb_cells(&self, rect: &Rect) -> Vec<(i32, i32)> {
        let min = self.cell_key(rect.x, rect.y);
        let max = self.cell_key(rect.x + rect.width, rect.y + rect.height);
        let mut cells = Vec::new();
        for y in min.1..=max.1 {
            for x in min.0..=max.0 {
                cells.push((x, y));
            }
        }
        cells
    }

    /// Insert an entity with an AABB
    pub fn insert(&mut self, entity_id: u64, aabb: &Rect) {
        let cells = self.aabb_cells(aabb);
        for &key in &cells {
            self.cells.entry(key)
                .or_insert_with(|| SpatialCell { entities: Vec::new() })
                .entities.push(entity_id);
        }
        self.entity_cells.insert(entity_id, cells);
    }

    /// Remove an entity
    pub fn remove(&mut self, entity_id: u64) {
        if let Some(cells) = self.entity_cells.remove(&entity_id) {
            for key in cells {
                if let Some(cell) = self.cells.get_mut(&key) {
                    cell.entities.retain(|&e| e != entity_id);
                }
            }
        }
    }

    /// Update an entity's position (remove + insert)
    pub fn update(&mut self, entity_id: u64, new_aabb: &Rect) {
        self.remove(entity_id);
        self.insert(entity_id, new_aabb);
    }

    /// Query all entities that might overlap a given AABB
    pub fn query_aabb(&self, rect: &Rect) -> Vec<u64> {
        let cells = self.aabb_cells(rect);
        let mut seen = HashMap::new();
        let mut result = Vec::new();

        for key in &cells {
            if let Some(cell) = self.cells.get(key) {
                for &entity_id in &cell.entities {
                    *seen.entry(entity_id).or_insert(0u32) += 1;
                }
            }
        }

        let threshold = cells.len().max(1) as u32;
        for (id, count) in seen {
            if count >= threshold {
                result.push(id);
            }
        }
        result
    }

    /// Query entities near a point
    pub fn query_point(&self, point: Vec2, radius: f32) -> Vec<u64> {
        let rect = Rect::new(point.x - radius, point.y - radius, radius * 2.0, radius * 2.0);
        self.query_aabb(&rect)
    }

    /// Get all potential collision pairs
    pub fn potential_pairs(&self) -> Vec<(u64, u64)> {
        let mut pairs = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cell in self.cells.values() {
            let entities = &cell.entities;
            for i in 0..entities.len() {
                for j in (i + 1)..entities.len() {
                    let a = entities[i].min(entities[j]);
                    let b = entities[i].max(entities[j]);
                    if seen.insert((a, b)) {
                        pairs.push((a, b));
                    }
                }
            }
        }
        pairs
    }

    pub fn cell_count(&self) -> usize { self.cells.len() }
    pub fn entity_count(&self) -> usize { self.entity_cells.len() }
    pub fn cell_size(&self) -> f32 { self.cell_size }
}

impl Default for SpatialHashGrid {
    fn default() -> Self { Self::new(64.0) }
}

// ---------------------------------------------------------------------------
// Viewport Culler — cull entities outside camera view
// ---------------------------------------------------------------------------

pub struct ViewportCuller {
    margin: f32,
    culled_count: usize,
    visible_count: usize,
}

impl ViewportCuller {
    pub fn new(margin: f32) -> Self {
        Self { margin, culled_count: 0, visible_count: 0 }
    }

    /// Check if a world-space AABB is visible in the viewport
    pub fn is_visible(&self, aabb: &Rect, viewport: &Rect) -> bool {
        let m = self.margin;
        let expanded = Rect::new(
            viewport.x - m,
            viewport.y - m,
            viewport.width + m * 2.0,
            viewport.height + m * 2.0,
        );
        expanded.intersects(aabb)
    }

    /// Cull a list of entities by their AABBs, returning only visible ones
    pub fn cull<'a, T>(&mut self, items: &'a [(T, Rect)], viewport: &Rect) -> Vec<&'a T> {
        self.visible_count = 0;
        self.culled_count = 0;

        items.iter()
            .filter(|(_, aabb)| {
                let vis = self.is_visible(aabb, viewport);
                if vis { self.visible_count += 1; } else { self.culled_count += 1; }
                vis
            })
            .map(|(item, _)| item)
            .collect()
    }

    /// Cull with indices (returns indices of visible items)
    pub fn cull_indices(&mut self, aabbs: &[Rect], viewport: &Rect) -> Vec<usize> {
        self.visible_count = 0;
        self.culled_count = 0;

        aabbs.iter().enumerate()
            .filter(|(_, aabb)| {
                let vis = self.is_visible(aabb, viewport);
                if vis { self.visible_count += 1; } else { self.culled_count += 1; }
                vis
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn visible_count(&self) -> usize { self.visible_count }
    pub fn culled_count(&self) -> usize { self.culled_count }
    pub fn cull_ratio(&self) -> f32 {
        let total = self.visible_count + self.culled_count;
        if total > 0 { self.visible_count as f32 / total as f32 } else { 1.0 }
    }

    pub fn set_margin(&mut self, margin: f32) { self.margin = margin; }
}

impl Default for ViewportCuller {
    fn default() -> Self { Self::new(64.0) }
}

// ---------------------------------------------------------------------------
// Object Pool — generic reusable object pool
// ---------------------------------------------------------------------------

pub struct ObjectPool<T> {
    objects: Vec<T>,
    active: Vec<bool>,
    free_stack: Vec<usize>,
    capacity: usize,
}

impl<T: Default> ObjectPool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut objects = Vec::with_capacity(capacity);
        let active = vec![false; capacity];
        let mut free_stack = Vec::with_capacity(capacity);

        for i in (0..capacity).rev() {
            objects.push(T::default());
            free_stack.push(i);
        }

        Self { objects, active, free_stack, capacity }
    }

    pub fn acquire(&mut self) -> Option<usize> {
        let idx = self.free_stack.pop()?;
        self.active[idx] = true;
        Some(idx)
    }

    pub fn release(&mut self, idx: usize) {
        if idx < self.capacity && self.active[idx] {
            self.active[idx] = false;
            self.objects[idx] = T::default();
            self.free_stack.push(idx);
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.capacity && self.active[idx] {
            Some(&self.objects[idx])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.capacity && self.active[idx] {
            Some(&mut self.objects[idx])
        } else {
            None
        }
    }

    pub fn active_count(&self) -> usize { self.active.iter().filter(|&&a| a).count() }
    pub fn free_count(&self) -> usize { self.free_stack.len() }
    pub fn capacity(&self) -> usize { self.capacity }

    pub fn iter_active(&self) -> impl Iterator<Item = (usize, &T)> {
        self.active.iter().enumerate()
            .filter(|(_, &a)| a)
            .map(|(i, _)| (i, &self.objects[i]))
    }

    pub fn iter_active_mut(&mut self) -> Vec<(usize, &mut T)> {
        let active_indices: Vec<usize> = self.active.iter().enumerate()
            .filter(|(_, &a)| a)
            .map(|(i, _)| i)
            .collect();
        let mut result = Vec::with_capacity(active_indices.len());
        let len = self.objects.len();
        for i in active_indices {
            if i < len {
                let ptr = &mut self.objects[i] as *mut T;
                result.push((i, unsafe { &mut *ptr }));
            }
        }
        result
    }

    pub fn clear(&mut self) {
        for i in 0..self.capacity {
            self.active[i] = false;
            self.objects[i] = T::default();
        }
        self.free_stack.clear();
        for i in (0..self.capacity).rev() {
            self.free_stack.push(i);
        }
    }
}

// ---------------------------------------------------------------------------
// Frame Time Monitor
// ---------------------------------------------------------------------------

pub struct FrameTimeMonitor {
    frame_times: Vec<f64>,
    max_samples: usize,
    idx: usize,
    min_frame: f64,
    max_frame: f64,
    pub target_fps: f64,
}

impl FrameTimeMonitor {
    pub fn new(max_samples: usize, target_fps: f64) -> Self {
        Self {
            frame_times: vec![0.0; max_samples],
            max_samples,
            idx: 0,
            min_frame: f64::MAX,
            max_frame: 0.0,
            target_fps,
        }
    }

    pub fn record_frame(&mut self, dt: f64) {
        self.frame_times[self.idx % self.max_samples] = dt;
        self.idx += 1;
        if dt < self.min_frame { self.min_frame = dt; }
        if dt > self.max_frame { self.max_frame = dt; }
    }

    pub fn avg_frame_time(&self) -> f64 {
        let count = self.idx.min(self.max_samples);
        if count == 0 { return 0.0; }
        let sum: f64 = self.frame_times[..count].iter().sum();
        sum / count as f64
    }

    pub fn avg_fps(&self) -> f64 {
        let avg = self.avg_frame_time();
        if avg > 0.0 { 1.0 / avg } else { 0.0 }
    }

    pub fn min_fps(&self) -> f64 {
        if self.max_frame > 0.0 { 1.0 / self.max_frame } else { 0.0 }
    }

    pub fn max_fps(&self) -> f64 {
        if self.min_frame > 0.0 && self.min_frame < f64::MAX { 1.0 / self.min_frame } else { 0.0 }
    }

    pub fn frame_time_ms(&self) -> f64 { self.avg_frame_time() * 1000.0 }

    pub fn fps_1_low(&self) -> f64 {
        let mut sorted = self.frame_times[..self.idx.min(self.max_samples)].to_vec();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(&worst) = sorted.first() {
            if worst > 0.0 { 1.0 / worst } else { 0.0 }
        } else {
            0.0
        }
    }

    pub fn fps_01_low(&self) -> f64 {
        let count = self.idx.min(self.max_samples);
        let outlier_count = (count / 100).max(1);
        let mut sorted = self.frame_times[..count].to_vec();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        if outlier_count < sorted.len() {
            let worst = sorted[outlier_count - 1];
            if worst > 0.0 { 1.0 / worst } else { 0.0 }
        } else {
            0.0
        }
    }

    pub fn is_below_target(&self) -> bool { self.avg_fps() < self.target_fps }

    pub fn reset(&mut self) {
        self.frame_times.iter_mut().for_each(|f| *f = 0.0);
        self.idx = 0;
        self.min_frame = f64::MAX;
        self.max_frame = 0.0;
    }
}

impl Default for FrameTimeMonitor {
    fn default() -> Self { Self::new(300, 60.0) }
}

// ---------------------------------------------------------------------------
// Memory Tracker
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub allocated_bytes: usize,
    pub peak_bytes: usize,
    pub pool_count: usize,
    pub entity_count: usize,
    pub component_count: usize,
}

pub struct MemoryTracker {
    allocated: usize,
    peak: usize,
    pool_sizes: HashMap<String, usize>,
    entity_count: usize,
    component_count: usize,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            allocated: 0,
            peak: 0,
            pool_sizes: HashMap::new(),
            entity_count: 0,
            component_count: 0,
        }
    }

    pub fn track_alloc(&mut self, bytes: usize) {
        self.allocated += bytes;
        if self.allocated > self.peak {
            self.peak = self.allocated;
        }
    }

    pub fn track_dealloc(&mut self, bytes: usize) {
        self.allocated = self.allocated.saturating_sub(bytes);
    }

    pub fn set_pool_size(&mut self, name: &str, size: usize) {
        self.pool_sizes.insert(name.to_string(), size);
    }

    pub fn set_entity_count(&mut self, count: usize) { self.entity_count = count; }
    pub fn set_component_count(&mut self, count: usize) { self.component_count = count; }

    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            allocated_bytes: self.allocated,
            peak_bytes: self.peak,
            pool_count: self.pool_sizes.len(),
            entity_count: self.entity_count,
            component_count: self.component_count,
        }
    }

    pub fn reset_peak(&mut self) { self.peak = self.allocated; }

    pub fn pool_sizes(&self) -> &HashMap<String, usize> { &self.pool_sizes }

    pub fn format_stats(&self) -> String {
        let stats = self.stats();
        format!(
            "Memory: {:.2} KB (peak: {:.2} KB) | Entities: {} | Components: {} | Pools: {}",
            stats.allocated_bytes as f64 / 1024.0,
            stats.peak_bytes as f64 / 1024.0,
            stats.entity_count,
            stats.component_count,
            stats.pool_count,
        )
    }
}

impl Default for MemoryTracker {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Performance Aggregator — ties all perf systems together
// ---------------------------------------------------------------------------

pub struct PerfAggregator {
    pub frame_monitor: FrameTimeMonitor,
    pub memory: MemoryTracker,
    pub culler: ViewportCuller,
    pub spatial_grid: SpatialHashGrid,
    update_interval: f64,
    update_timer: f64,
    pub stats: PerfSnapshot,
}

#[derive(Debug, Clone)]
pub struct PerfSnapshot {
    pub fps: f64,
    pub frame_time_ms: f64,
    pub fps_1_low: f64,
    pub fps_01_low: f64,
    pub entities_visible: usize,
    pub entities_culled: usize,
    pub cull_ratio: f32,
    pub spatial_cells: usize,
    pub memory_allocated_kb: f64,
    pub memory_peak_kb: f64,
}

impl Default for PerfSnapshot {
    fn default() -> Self {
        Self {
            fps: 0.0, frame_time_ms: 0.0, fps_1_low: 0.0, fps_01_low: 0.0,
            entities_visible: 0, entities_culled: 0, cull_ratio: 1.0,
            spatial_cells: 0, memory_allocated_kb: 0.0, memory_peak_kb: 0.0,
        }
    }
}

impl PerfAggregator {
    pub fn new() -> Self {
        Self {
            frame_monitor: FrameTimeMonitor::new(300, 60.0),
            memory: MemoryTracker::new(),
            culler: ViewportCuller::new(64.0),
            spatial_grid: SpatialHashGrid::new(64.0),
            update_interval: 0.5,
            update_timer: 0.0,
            stats: PerfSnapshot::default(),
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.frame_monitor.record_frame(dt);
        self.update_timer += dt;
        if self.update_timer >= self.update_interval {
            self.update_timer = 0.0;
            self.refresh_stats();
        }
    }

    fn refresh_stats(&mut self) {
        let mem = self.memory.stats();
        self.stats.fps = self.frame_monitor.avg_fps();
        self.stats.frame_time_ms = self.frame_monitor.frame_time_ms();
        self.stats.fps_1_low = self.frame_monitor.fps_1_low();
        self.stats.fps_01_low = self.frame_monitor.fps_01_low();
        self.stats.entities_visible = self.culler.visible_count();
        self.stats.entities_culled = self.culler.culled_count();
        self.stats.cull_ratio = self.culler.cull_ratio();
        self.stats.spatial_cells = self.spatial_grid.cell_count();
        self.stats.memory_allocated_kb = mem.allocated_bytes as f64 / 1024.0;
        self.stats.memory_peak_kb = mem.peak_bytes as f64 / 1024.0;
    }

    pub fn format_overlay(&self) -> String {
        format!(
            "FPS: {:.0} ({:.1}ms) | 1%: {:.0} | 0.1%: {:.0}\nVisible: {} | Culled: {} ({:.0}%)\nSpatial cells: {} | Memory: {:.1}KB",
            self.stats.fps,
            self.stats.frame_time_ms,
            self.stats.fps_1_low,
            self.stats.fps_01_low,
            self.stats.entities_visible,
            self.stats.entities_culled,
            self.stats.cull_ratio * 100.0,
            self.stats.spatial_cells,
            self.stats.memory_allocated_kb,
        )
    }
}

impl Default for PerfAggregator {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_insert_query() {
        let mut grid = SpatialHashGrid::new(32.0);
        grid.insert(1, &Rect::new(10.0, 10.0, 20.0, 20.0));
        grid.insert(2, &Rect::new(100.0, 100.0, 20.0, 20.0));

        let results = grid.query_aabb(&Rect::new(0.0, 0.0, 50.0, 50.0));
        assert!(results.contains(&1));
        assert!(!results.contains(&2));
        assert_eq!(grid.entity_count(), 2);
    }

    #[test]
    fn test_spatial_hash_remove() {
        let mut grid = SpatialHashGrid::new(32.0);
        grid.insert(1, &Rect::new(0.0, 0.0, 10.0, 10.0));
        grid.remove(1);
        assert!(grid.query_aabb(&Rect::new(0.0, 0.0, 100.0, 100.0)).is_empty());
    }

    #[test]
    fn test_spatial_hash_potential_pairs() {
        let mut grid = SpatialHashGrid::new(32.0);
        grid.insert(1, &Rect::new(0.0, 0.0, 16.0, 16.0));
        grid.insert(2, &Rect::new(8.0, 8.0, 16.0, 16.0));
        grid.insert(3, &Rect::new(200.0, 200.0, 16.0, 16.0));
        let pairs = grid.potential_pairs();
        assert!(pairs.iter().any(|&(a, b)| (a == 1 && b == 2) || (a == 2 && b == 1)));
    }

    #[test]
    fn test_viewport_culler() {
        let mut culler = ViewportCuller::new(0.0);
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        assert!(culler.is_visible(&Rect::new(100.0, 100.0, 32.0, 32.0), &viewport));
        assert!(!culler.is_visible(&Rect::new(900.0, 900.0, 32.0, 32.0), &viewport));
    }

    #[test]
    fn test_viewport_cull_batch() {
        let mut culler = ViewportCuller::new(0.0);
        let viewport = Rect::new(0.0, 0.0, 100.0, 100.0);
        let items: Vec<((), Rect)> = vec![
            ((), Rect::new(50.0, 50.0, 10.0, 10.0)),
            ((), Rect::new(200.0, 200.0, 10.0, 10.0)),
        ];
        let visible = culler.cull(&items, &viewport);
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn test_object_pool() {
        let mut pool = ObjectPool::<u32>::new(4);
        assert_eq!(pool.capacity(), 4);
        assert_eq!(pool.free_count(), 4);

        let idx1 = pool.acquire().unwrap();
        let idx2 = pool.acquire().unwrap();
        assert_eq!(pool.active_count(), 2);
        assert_eq!(pool.free_count(), 2);

        pool.release(idx1);
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.free_count(), 3);

        pool.release(idx2);
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.free_count(), 4);
    }

    #[test]
    fn test_object_pool_overflow() {
        let mut pool = ObjectPool::<bool>::new(2);
        pool.acquire();
        pool.acquire();
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn test_frame_time_monitor() {
        let mut monitor = FrameTimeMonitor::new(60, 60.0);
        for _ in 0..60 {
            monitor.record_frame(1.0 / 60.0);
        }
        assert!((monitor.avg_fps() - 60.0).abs() < 2.0);
    }

    #[test]
    fn test_frame_time_percentiles() {
        let mut monitor = FrameTimeMonitor::new(100, 60.0);
        // Mix of fast and slow frames
        for _ in 0..90 {
            monitor.record_frame(1.0 / 60.0);
        }
        for _ in 0..10 {
            monitor.record_frame(1.0 / 30.0); // slower frames
        }
        assert!(monitor.fps_1_low() > 0.0);
        assert!(monitor.fps_01_low() > 0.0);
    }

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();
        tracker.track_alloc(1024);
        let stats = tracker.stats();
        assert_eq!(stats.allocated_bytes, 1024);
        assert_eq!(stats.peak_bytes, 1024);

        tracker.track_dealloc(512);
        let stats = tracker.stats();
        assert_eq!(stats.allocated_bytes, 512);
        assert_eq!(stats.peak_bytes, 1024);
    }

    #[test]
    fn test_perf_aggregator() {
        let mut agg = PerfAggregator::new();
        agg.tick(1.0 / 60.0);
        agg.tick(1.0 / 60.0);
        assert!(agg.stats.fps > 0.0);
    }

    #[test]
    fn test_format_overlay() {
        let agg = PerfAggregator::new();
        let overlay = agg.format_overlay();
        assert!(overlay.contains("FPS"));
        assert!(overlay.contains("Visible"));
    }
}
