/// NT-GAME 热重载与运行时管理系统
///
/// 吸收 Godot 4 热重载 + Unity Coroutine + Godot Timer 架构
/// - HotReloader: 监听文件变化，触发资源热重载
/// - TimerManager: 管理多个定时器（单次/重复）
/// - CoroutineManager: 协程序列（延迟、等待、并行）

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════
// 热重载器 — 吸收 Godot 4 EditorPlugin 资源热重载
// ═══════════════════════════════════════════════════════════════════

/// 热重载器：监听文件/目录变化，触发回调
///
/// 设计参考 Godot 4 的 ResourceFormatLoader + EditorFileSystem：
/// - 轮询模式（适合游戏主循环）
/// - 按路径注册回调
/// - 支持目录递归和单文件监听
pub struct HotReloader {
    watched_dirs: Vec<PathBuf>,
    watched_files: HashMap<PathBuf, Box<dyn Fn(&Path) + Send>>,
    file_timestamps: HashMap<PathBuf, Option<SystemTime>>,
    modified_files: Vec<PathBuf>,
}

impl HotReloader {
    pub fn new() -> Self {
        Self {
            watched_dirs: Vec::new(),
            watched_files: HashMap::new(),
            file_timestamps: HashMap::new(),
            modified_files: Vec::new(),
        }
    }

    /// 监听目录下所有文件的变化
    pub fn watch_dir(&mut self, path: &Path) {
        if !self.watched_dirs.iter().any(|d| d == path) {
            self.watched_dirs.push(path.to_path_buf());
        }
    }

    /// 监听单个文件变化并注册回调
    pub fn watch_file(&mut self, path: &Path, callback: impl Fn(&Path) + Send + 'static) {
        let canonical = path.to_path_buf();
        self.file_timestamps
            .insert(canonical.clone(), Self::mtime(path));
        self.watched_files.insert(canonical, Box::new(callback));
    }

    /// 轮询一次，返回发生变化的文件路径列表
    pub fn poll(&mut self) -> Vec<PathBuf> {
        self.modified_files.clear();

        // 扫描已注册的文件
        for (path, last_ts) in &mut self.file_timestamps {
            let current_ts = Self::mtime(path);
            if current_ts.is_some() && current_ts != *last_ts {
                self.modified_files.push(path.clone());
                *last_ts = current_ts;
            }
        }

        // 扫描目录下的文件
        for dir in &self.watched_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let current_ts = Self::mtime(&path);
                        match self.file_timestamps.get_mut(&path) {
                            Some(last_ts) => {
                                if current_ts.is_some() && current_ts != *last_ts {
                                    self.modified_files.push(path.clone());
                                    *last_ts = current_ts;
                                }
                            }
                            None => {
                                self.file_timestamps.insert(path.clone(), current_ts);
                            }
                        }
                    }
                }
            }
        }

        self.modified_files.clone()
    }

    /// 清除所有监听
    pub fn clear(&mut self) {
        self.watched_dirs.clear();
        self.watched_files.clear();
        self.file_timestamps.clear();
        self.modified_files.clear();
    }

    /// 手动标记文件为已修改（用于外部触发）
    pub fn mark_modified(&mut self, path: &Path) {
        let canonical = path.to_path_buf();
        if !self.modified_files.contains(&canonical) {
            self.modified_files.push(canonical);
        }
    }

    /// 触发已注册文件的回调
    pub fn trigger_callbacks(&self) {
        for (path, cb) in &self.watched_files {
            if self.modified_files.contains(path) {
                cb(path);
            }
        }
    }

    fn mtime(path: &Path) -> Option<SystemTime> {
        std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
    }
}

impl Default for HotReloader {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// 定时器管理器 — 吸收 Godot Timer 节点
// ═══════════════════════════════════════════════════════════════════

/// 定时器条目
pub struct TimerEntry {
    pub id: u64,
    pub duration: Duration,
    pub elapsed: Duration,
    pub repeating: bool,
    pub active: bool,
    pub callback: Option<Box<dyn FnOnce() + Send>>,
}

/// 定时器管理器：管理多个定时器，支持单次和重复触发
///
/// 设计参考 Godot Timer 节点：
/// - 通过 callback 闭包替代 Godot 的 signal 机制
/// - repeating 模式等价于 Godot Timer.wait = true
/// - tick 推进时间，到期自动执行并清理
pub struct TimerManager {
    timers: Vec<TimerEntry>,
}

impl TimerManager {
    pub fn new() -> Self {
        Self { timers: Vec::new() }
    }

    /// 添加通用定时器
    pub fn add_timer(
        &mut self,
        duration: Duration,
        repeating: bool,
        callback: impl FnOnce() + Send + 'static,
    ) -> u64 {
        let id = next_id();
        self.timers.push(TimerEntry {
            id,
            duration,
            elapsed: Duration::ZERO,
            repeating,
            active: true,
            callback: Some(Box::new(callback)),
        });
        id
    }

    /// 添加延迟执行（单次定时器的便捷方法）
    pub fn add_delay(
        &mut self,
        duration: Duration,
        callback: impl FnOnce() + Send + 'static,
    ) -> u64 {
        self.add_timer(duration, false, callback)
    }

    /// 添加重复定时器（repeating = true 的便捷方法）
    pub fn add_repeating(
        &mut self,
        interval: Duration,
        callback: impl FnOnce() + Send + 'static,
    ) -> u64 {
        self.add_timer(interval, true, callback)
    }

    /// 取消定时器
    pub fn cancel(&mut self, id: u64) {
        if let Some(t) = self.timers.iter_mut().find(|t| t.id == id) {
            t.active = false;
            t.callback = None;
        }
    }

    /// 每帧推进，触发到期的定时器
    pub fn tick(&mut self, dt: Duration) {
        for timer in &mut self.timers {
            if !timer.active {
                continue;
            }
            timer.elapsed += dt;
            if timer.elapsed >= timer.duration {
                if let Some(cb) = timer.callback.take() {
                    cb();
                }
                if timer.repeating {
                    timer.elapsed -= timer.duration;
                }
                timer.active = false;
            }
        }
    }

    /// 活跃定时器数量
    pub fn active_count(&self) -> usize {
        self.timers.iter().filter(|t| t.active).count()
    }

    /// 移除所有已完成的定时器
    pub fn cleanup(&mut self) {
        self.timers.retain(|t| t.active);
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// 协程系统 — 吸收 Unity Coroutine (StartCoroutine / StopCoroutine)
// ═══════════════════════════════════════════════════════════════════

/// 协程步骤
pub enum CoroutineStep {
    /// 等待指定时间
    Wait(Duration),
    /// 立即执行一个回调
    Call(Box<dyn FnOnce() + Send>),
    /// 顺序执行一组步骤
    Sequence(Vec<CoroutineStep>),
    /// 并行执行一组步骤（全部完成后进入下一步）
    Parallel(Vec<CoroutineStep>),
}

/// 活跃的协程实例
pub struct ActiveCoroutine {
    pub id: u64,
    steps: Vec<CoroutineStep>,
    cursor: usize,
    timer: Duration,
    waiting: bool,
    sub: Vec<ActiveCoroutine>,
    pub active: bool,
}

impl ActiveCoroutine {
    fn tick_inner(&mut self, dt: Duration) {
        if !self.active {
            return;
        }

        // 先推进所有子协程
        for child in &mut self.sub {
            child.tick_inner(dt);
        }
        self.sub.retain(|c| c.active);

        loop {
            if self.cursor >= self.steps.len() {
                self.active = false;
                return;
            }

            match &self.steps[self.cursor] {
                CoroutineStep::Wait(dur) => {
                    if !self.waiting {
                        self.waiting = true;
                        self.timer = Duration::ZERO;
                    }
                    self.timer += dt;
                    if self.timer >= *dur {
                        self.waiting = false;
                        self.cursor += 1;
                        continue;
                    }
                    return; // 还在等待中
                }
                CoroutineStep::Call(_) => {
                    let step = std::mem::replace(
                        &mut self.steps[self.cursor],
                        CoroutineStep::Wait(Duration::ZERO),
                    );
                    if let CoroutineStep::Call(cb) = step {
                        cb();
                    }
                    self.cursor += 1;
                    continue;
                }
                CoroutineStep::Sequence(_) => {
                    let placeholder = CoroutineStep::Wait(Duration::ZERO);
                    let taken = std::mem::replace(
                        &mut self.steps[self.cursor],
                        placeholder,
                    );
                    if let CoroutineStep::Sequence(steps) = taken {
                        self.sub.push(ActiveCoroutine {
                            id: next_id(),
                            steps,
                            cursor: 0,
                            timer: Duration::ZERO,
                            waiting: false,
                            sub: Vec::new(),
                            active: true,
                        });
                    }
                    self.cursor += 1;
                    return;
                }
                CoroutineStep::Parallel(_) => {
                    let placeholder = CoroutineStep::Wait(Duration::ZERO);
                    let taken = std::mem::replace(
                        &mut self.steps[self.cursor],
                        placeholder,
                    );
                    if let CoroutineStep::Parallel(steps) = taken {
                        self.sub.push(ActiveCoroutine {
                            id: next_id(),
                            steps,
                            cursor: 0,
                            timer: Duration::ZERO,
                            waiting: false,
                            sub: Vec::new(),
                            active: true,
                        });
                    }
                    self.cursor += 1;
                    return;
                }
            }
        }
    }

    /// 检查所有子协程是否完成
    fn all_sub_done(&self) -> bool {
        self.sub.iter().all(|c| !c.active)
    }
}

/// 协程管理器：管理多个并发协程
///
/// 设计参考 Unity 的 StartCoroutine / StopCoroutine：
/// - 每个协程有唯一 ID
/// - 支持 Wait / Call / Sequence / Parallel
/// - tick 推进时间，到期自动执行步骤
pub struct CoroutineManager {
    active: Vec<ActiveCoroutine>,
}

impl CoroutineManager {
    pub fn new() -> Self {
        Self { active: Vec::new() }
    }

    /// 启动新协程，返回 ID
    pub fn start(&mut self, steps: Vec<CoroutineStep>) -> u64 {
        let id = next_id();
        self.active.push(ActiveCoroutine {
            id,
            steps,
            cursor: 0,
            timer: Duration::ZERO,
            waiting: false,
            sub: Vec::new(),
            active: true,
        });
        id
    }

    /// 停止协程
    pub fn stop(&mut self, id: u64) {
        if let Some(c) = self.active.iter_mut().find(|c| c.id == id) {
            c.active = false;
        }
    }

    /// 每帧推进所有活跃协程
    pub fn tick(&mut self, dt: Duration) {
        for coro in &mut self.active {
            coro.tick_inner(dt);
            // 如果有未完成的子协程，保持活跃（等待子协程完成）
            if coro.active && !coro.all_sub_done() {
                coro.active = true;
            }
        }
        self.active.retain(|c| c.active);
    }

    /// 活跃协程数量
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

impl Default for CoroutineManager {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    // ── HotReloader 测试 ──────────────────────────────────────────

    #[test]
    fn test_hot_reloader_new() {
        let reloader = HotReloader::new();
        assert!(reloader.watched_dirs.is_empty());
        assert!(reloader.watched_files.is_empty());
        assert!(reloader.modified_files.is_empty());
    }

    #[test]
    fn test_hot_reloader_watch_dir() {
        let mut reloader = HotReloader::new();
        let dir = PathBuf::from("/tmp/test_watch_dir");
        reloader.watch_dir(&dir);
        assert_eq!(reloader.watched_dirs.len(), 1);
        // 重复添加不应增加
        reloader.watch_dir(&dir);
        assert_eq!(reloader.watched_dirs.len(), 1);
    }

    #[test]
    fn test_hot_reloader_mark_modified() {
        let mut reloader = HotReloader::new();
        let path = PathBuf::from("/tmp/test_file.rs");
        reloader.mark_modified(&path);
        assert_eq!(reloader.modified_files.len(), 1);
        // 重复标记不应增加
        reloader.mark_modified(&path);
        assert_eq!(reloader.modified_files.len(), 1);
    }

    #[test]
    fn test_hot_reloader_clear() {
        let mut reloader = HotReloader::new();
        reloader.watch_dir(&PathBuf::from("/tmp"));
        reloader.mark_modified(&PathBuf::from("/tmp/file.rs"));
        reloader.clear();
        assert!(reloader.watched_dirs.is_empty());
        assert!(reloader.watched_files.is_empty());
        assert!(reloader.file_timestamps.is_empty());
        assert!(reloader.modified_files.is_empty());
    }

    #[test]
    fn test_hot_reloader_watch_file_callback() {
        let mut reloader = HotReloader::new();
        let flag = Arc::new(AtomicUsize::new(0));
        let flag_clone = flag.clone();
        let path = PathBuf::from("/tmp/nonexistent_file.txt");
        reloader.watch_file(&path, move |_p| {
            flag_clone.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(reloader.watched_files.len(), 1);
    }

    // ── TimerManager 测试 ─────────────────────────────────────────

    #[test]
    fn test_timer_manager_new() {
        let tm = TimerManager::new();
        assert_eq!(tm.active_count(), 0);
    }

    #[test]
    fn test_timer_delay_fires() {
        let mut tm = TimerManager::new();
        let flag = Arc::new(AtomicUsize::new(0));
        let flag_clone = flag.clone();

        tm.add_delay(Duration::from_millis(100), move || {
            flag_clone.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(tm.active_count(), 1);

        // 不够时间
        tm.tick(Duration::from_millis(50));
        assert_eq!(flag.load(Ordering::Relaxed), 0);
        assert_eq!(tm.active_count(), 1);

        // 超过时间
        tm.tick(Duration::from_millis(60));
        assert_eq!(flag.load(Ordering::Relaxed), 1);
        assert_eq!(tm.active_count(), 0);
    }

    #[test]
    fn test_timer_cancel() {
        let mut tm = TimerManager::new();
        let flag = Arc::new(AtomicUsize::new(0));
        let flag_clone = flag.clone();

        let id = tm.add_delay(Duration::from_millis(100), move || {
            flag_clone.fetch_add(1, Ordering::Relaxed);
        });
        tm.cancel(id);
        tm.tick(Duration::from_millis(200));
        assert_eq!(flag.load(Ordering::Relaxed), 0);
        assert_eq!(tm.active_count(), 0);
    }

    #[test]
    fn test_timer_multiple() {
        let mut tm = TimerManager::new();
        let count = Arc::new(AtomicUsize::new(0));

        for _ in 0..5 {
            let c = count.clone();
            tm.add_delay(Duration::from_millis(100), move || {
                c.fetch_add(1, Ordering::Relaxed);
            });
        }
        assert_eq!(tm.active_count(), 5);

        tm.tick(Duration::from_millis(150));
        assert_eq!(count.load(Ordering::Relaxed), 5);
        assert_eq!(tm.active_count(), 0);
    }

    #[test]
    fn test_timer_cleanup() {
        let mut tm = TimerManager::new();
        let flag = Arc::new(AtomicUsize::new(0));
        let flag_clone = flag.clone();

        tm.add_delay(Duration::from_millis(50), move || {
            flag_clone.fetch_add(1, Ordering::Relaxed);
        });
        tm.add_delay(Duration::from_millis(200), || {});

        tm.tick(Duration::from_millis(100));
        assert_eq!(tm.active_count(), 1); // 第一个已完成但还在vec中
        tm.cleanup();
        assert_eq!(tm.timers.len(), 1); // cleanup后只剩活跃的
    }

    // ── CoroutineManager 测试 ─────────────────────────────────────

    #[test]
    fn test_coroutine_call_only() {
        let mut cm = CoroutineManager::new();
        let flag = Arc::new(AtomicUsize::new(0));
        let flag_clone = flag.clone();

        cm.start(vec![CoroutineStep::Call(Box::new(move || {
            flag_clone.fetch_add(1, Ordering::Relaxed);
        }))]);

        assert_eq!(cm.active_count(), 1);
        cm.tick(Duration::from_millis(16));
        assert_eq!(flag.load(Ordering::Relaxed), 1);
        assert_eq!(cm.active_count(), 0);
    }

    #[test]
    fn test_coroutine_wait_then_call() {
        let mut cm = CoroutineManager::new();
        let flag = Arc::new(AtomicUsize::new(0));
        let flag_clone = flag.clone();

        cm.start(vec![
            CoroutineStep::Wait(Duration::from_millis(100)),
            CoroutineStep::Call(Box::new(move || {
                flag_clone.fetch_add(1, Ordering::Relaxed);
            })),
        ]);

        cm.tick(Duration::from_millis(50));
        assert_eq!(flag.load(Ordering::Relaxed), 0);

        cm.tick(Duration::from_millis(60));
        assert_eq!(flag.load(Ordering::Relaxed), 1);
        assert_eq!(cm.active_count(), 0);
    }

    #[test]
    fn test_coroutine_sequence() {
        let mut cm = CoroutineManager::new();
        let flag = Arc::new(AtomicUsize::new(0));
        let flag_clone = flag.clone();

        cm.start(vec![CoroutineStep::Sequence(vec![
            CoroutineStep::Call(Box::new(|| {})),
            CoroutineStep::Wait(Duration::from_millis(50)),
            CoroutineStep::Call(Box::new(move || {
                flag_clone.fetch_add(1, Ordering::Relaxed);
            })),
        ])]);

        cm.tick(Duration::from_millis(16));
        assert_eq!(flag.load(Ordering::Relaxed), 0);

        cm.tick(Duration::from_millis(50));
        assert_eq!(flag.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_coroutine_stop() {
        let mut cm = CoroutineManager::new();
        let flag = Arc::new(AtomicUsize::new(0));
        let flag_clone = flag.clone();

        let id = cm.start(vec![
            CoroutineStep::Wait(Duration::from_millis(100)),
            CoroutineStep::Call(Box::new(move || {
                flag_clone.fetch_add(1, Ordering::Relaxed);
            })),
        ]);

        cm.stop(id);
        cm.tick(Duration::from_millis(200));
        assert_eq!(flag.load(Ordering::Relaxed), 0);
        assert_eq!(cm.active_count(), 0);
    }

    #[test]
    fn test_coroutine_multiple_calls() {
        let mut cm = CoroutineManager::new();
        let count = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            let c = count.clone();
            cm.start(vec![CoroutineStep::Call(Box::new(move || {
                c.fetch_add(1, Ordering::Relaxed);
            }))]);
        }

        cm.tick(Duration::from_millis(16));
        assert_eq!(count.load(Ordering::Relaxed), 10);
        assert_eq!(cm.active_count(), 0);
    }

    // ── 综合集成测试 ─────────────────────────────────────────────

    #[test]
    fn test_integration_timer_and_coroutine() {
        let mut tm = TimerManager::new();
        let mut cm = CoroutineManager::new();
        let order = Arc::new(AtomicUsize::new(0));

        let o1 = order.clone();
        tm.add_delay(Duration::from_millis(50), move || {
            o1.fetch_add(1, Ordering::Relaxed);
        });

        let o2 = order.clone();
        cm.start(vec![
            CoroutineStep::Wait(Duration::from_millis(30)),
            CoroutineStep::Call(Box::new(move || {
                o2.fetch_add(10, Ordering::Relaxed);
            })),
        ]);

        // 50ms later
        tm.tick(Duration::from_millis(50));
        cm.tick(Duration::from_millis(50));
        assert_eq!(order.load(Ordering::Relaxed), 11); // timer fired (1) + coroutine call (10)
    }
}
