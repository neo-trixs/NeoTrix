//! 事件派发调度器 — 吸收自 deepseek-harness vendor/cordis/src/events.ts
//! (4+1 dispatch modes: emit / waterfall / parallel / serial) 与
//! cordiverse/paper §4.3.3 (asynchrony/inertia) + §5.1.2 (notify → refresh)。
//!
//! 机制:
//! - `Emit`    广播: 所有 handler 收到事件, 互不短路。
//! - `Parallel`并行: 语义同 Emit (Rust 同步调度); 保留模式名以对应 Cordis 语义。
//! - `Serial`  顺序: 首个 handler 返回 `true` (已处理) 即短路 (bail)。
//! - `Waterfall` 链式中间件: 每个 handler 可调 `next()` 委托给下一环 (around
//!   middleware), 或返回 `true` 短路; 都不做则顺延 (fall-through)。
//!
//! NeoTrix 消费方 (R-P79): McpServer 工具调用 pre/post 钩子 (Waterfall 中间件链),
//! 对应 dsh tools.md "工具管线 = 可扩展 waterfall" 范式。

/// 事件派发模式 (Cordis events.ts dispatch modes)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchMode {
    /// 广播到全部 handler (NeoTrix EventBus 现行为)
    Emit,
    /// around 中间件链: handler 可 next() 委托 / 返回 true 短路 / 静默顺延
    Waterfall,
    /// 并行独立处理 (同步场景等价 Emit)
    Parallel,
    /// 顺序处理, 首个 handler 返回 true 即短路 (bail)
    Serial,
}

impl DispatchMode {
    pub fn name(&self) -> &'static str {
        match self {
            DispatchMode::Emit => "emit",
            DispatchMode::Waterfall => "waterfall",
            DispatchMode::Parallel => "parallel",
            DispatchMode::Serial => "serial",
        }
    }
}

/// 类型化事件调度器。
///
/// Handler 签名: `Fn(&E, &dyn Fn()) -> bool`
/// - `&E`: 事件载荷
/// - `&dyn Fn()`: `next()` 委托 — 仅在 Waterfall 模式有意义 (调用后运行剩余链)
/// - 返回 `true`: 声明"已处理" (Serial/Waterfall 短路; Emit/Parallel 仅计数)
pub struct Dispatcher<E> {
    handlers: Vec<Box<dyn Fn(&E, &dyn Fn()) -> bool + Send + Sync>>,
}

impl<E> Default for Dispatcher<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> Dispatcher<E> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// 注册 handler。返回自增标识。
    pub fn register<F>(&mut self, handler: F) -> usize
    where
        F: Fn(&E, &dyn Fn()) -> bool + Send + Sync + 'static,
    {
        self.handlers.push(Box::new(handler));
        self.handlers.len() - 1
    }

    /// 便捷注册 (忽略 next 的简单 handler)。
    pub fn on<F>(&mut self, handler: F) -> usize
    where
        F: Fn(&E) -> bool + Send + Sync + 'static,
    {
        self.register(move |e, _next| handler(e))
    }

    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// 按模式派发。返回实际运行的 handler 数。
    pub fn dispatch(&self, mode: DispatchMode, event: &E) -> usize {
        match mode {
            DispatchMode::Emit | DispatchMode::Parallel => {
                let mut ran = 0;
                for h in &self.handlers {
                    h(event, &|| {});
                    ran += 1;
                }
                ran
            }
            DispatchMode::Serial => {
                let mut ran = 0;
                for h in &self.handlers {
                    ran += 1;
                    if h(event, &|| {}) {
                        break;
                    }
                }
                ran
            }
            DispatchMode::Waterfall => dispatch_chain(&self.handlers, 0, event).0,
        }
    }

    /// Waterfall 派发并报告是否发生短路拦截。
    /// 语义: 任一 handler 返回 `true` (声明已处理) 即短路, 剩余链不运行。
    /// 用于 EventBus 过滤链 / 守卫链: 短路 = 拦截。
    pub fn dispatch_waterfall(&self, event: &E) -> bool {
        dispatch_chain(&self.handlers, 0, event).1
    }
}

/// Waterfall 链式调度: next() 委托 / true 短路 / 静默顺延。
/// 返回 (运行 handler 数, 是否发生短路拦截)。
fn dispatch_chain<E>(
    handlers: &[Box<dyn Fn(&E, &dyn Fn()) -> bool + Send + Sync>],
    idx: usize,
    event: &E,
) -> (usize, bool) {
    if idx >= handlers.len() {
        return (0, false);
    }
    let advanced = std::cell::Cell::new(false);
    let ran = std::cell::Cell::new(0usize);
    let short = std::cell::Cell::new(false);
    {
        let next = || {
            advanced.set(true);
            let (r, s) = dispatch_chain(handlers, idx + 1, event);
            ran.set(r);
            short.set(s);
        };
        let terminal = handlers[idx](event, &next);
        if advanced.get() {
            // 调用过 next(): 本环 1 + 后继环
            return (1 + ran.get(), short.get());
        }
        if terminal {
            // 短路
            return (1, true);
        }
    }
    // 静默顺延: 未调 next 也未短路
    let (r, s) = dispatch_chain(handlers, idx + 1, event);
    (1 + r, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Mutex};

    fn event_counter(counter: Arc<AtomicU32>) -> impl Fn(&u32) -> bool + Send + Sync + 'static {
        move |_e| {
            counter.fetch_add(1, Ordering::SeqCst);
            false
        }
    }

    #[test]
    fn test_empty_dispatcher_runs_none() {
        let d: Dispatcher<u32> = Dispatcher::new();
        assert_eq!(d.dispatch(DispatchMode::Emit, &1), 0);
        assert_eq!(d.dispatch(DispatchMode::Waterfall, &1), 0);
        assert_eq!(d.dispatch(DispatchMode::Serial, &1), 0);
        assert!(d.is_empty());
    }

    #[test]
    fn test_emit_runs_all_handlers() {
        let mut d = Dispatcher::new();
        let c1 = Arc::new(AtomicU32::new(0));
        let c2 = Arc::new(AtomicU32::new(0));
        d.on(event_counter(c1.clone()));
        d.on(event_counter(c2.clone()));
        let ran = d.dispatch(DispatchMode::Emit, &42);
        assert_eq!(ran, 2);
        assert_eq!(c1.load(Ordering::SeqCst), 1);
        assert_eq!(c2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_serial_bails_on_first_handled() {
        let mut d = Dispatcher::new();
        let first = Arc::new(AtomicU32::new(0));
        let second = Arc::new(AtomicU32::new(0));
        {
            let f = first.clone();
            d.on(move |_e| {
                f.fetch_add(1, Ordering::SeqCst);
                true // 已处理 → 短路
            });
        }
        {
            let s = second.clone();
            d.on(move |_e| {
                s.fetch_add(1, Ordering::SeqCst);
                false
            });
        }
        let ran = d.dispatch(DispatchMode::Serial, &7);
        assert_eq!(ran, 1);
        assert_eq!(first.load(Ordering::SeqCst), 1);
        assert_eq!(second.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_serial_runs_all_when_none_handle() {
        let mut d = Dispatcher::new();
        let count = Arc::new(AtomicU32::new(0));
        for _ in 0..3 {
            let c = count.clone();
            d.on(move |_e| {
                c.fetch_add(1, Ordering::SeqCst);
                false
            });
        }
        let ran = d.dispatch(DispatchMode::Serial, &0);
        assert_eq!(ran, 3);
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_waterfall_next_delegation() {
        // 中间件 1 调 next 后包一层; 中间件 2 处理
        let mut d = Dispatcher::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        d.register({
            let log = log.clone();
            move |e, next| {
                log.lock().unwrap().push(format!("enter1:{}", e));
                next();
                log.lock().unwrap().push("exit1".to_string());
                false
            }
        });
        d.register({
            let log = log.clone();
            move |e, _next| {
                log.lock().unwrap().push(format!("handler2:{}", e));
                true
            }
        });
        let ran = d.dispatch(DispatchMode::Waterfall, &5);
        assert_eq!(ran, 2);
        let log = log.lock().unwrap();
        assert_eq!(
            log.as_slice(),
            &[
                "enter1:5".to_string(),
                "handler2:5".to_string(),
                "exit1".to_string()
            ]
        );
    }

    #[test]
    fn test_waterfall_short_circuit() {
        let mut d = Dispatcher::new();
        let second = Arc::new(AtomicU32::new(0));
        d.on(move |_e| true); // 直接短路
        {
            let s = second.clone();
            d.on(move |_e| {
                s.fetch_add(1, Ordering::SeqCst);
                false
            });
        }
        let ran = d.dispatch(DispatchMode::Waterfall, &1);
        assert_eq!(ran, 1);
        assert_eq!(second.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_waterfall_fall_through() {
        // handler 既未调 next 也未短路 → 顺延到下一环
        let mut d = Dispatcher::new();
        let count = Arc::new(AtomicU32::new(0));
        for _ in 0..3 {
            let c = count.clone();
            d.on(move |_e| {
                c.fetch_add(1, Ordering::SeqCst);
                false
            });
        }
        let ran = d.dispatch(DispatchMode::Waterfall, &1);
        assert_eq!(ran, 3);
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_parallel_is_non_short_circuit() {
        let mut d = Dispatcher::new();
        let count = Arc::new(AtomicU32::new(0));
        for _ in 0..2 {
            let c = count.clone();
            d.on(move |_e| {
                c.fetch_add(1, Ordering::SeqCst);
                true
            });
        }
        let ran = d.dispatch(DispatchMode::Parallel, &9);
        assert_eq!(ran, 2); // 全部运行 (true 不短路)
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_dispatch_waterfall_reports_short_circuit() {
        let mut d = Dispatcher::new();
        let count = Arc::new(AtomicU32::new(0));
        d.on(move |_e| false); // 放行
        {
            let c = count.clone();
            d.on(move |_e| {
                c.fetch_add(1, Ordering::SeqCst);
                true // 拦截
            });
        }
        d.on(move |_e| {
            panic!("must be short-circuited");
        });
        assert!(d.dispatch_waterfall(&1)); // 有拦截
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_dispatch_waterfall_allows_pass_through() {
        let mut d = Dispatcher::new();
        let count = Arc::new(AtomicU32::new(0));
        for _ in 0..2 {
            let c = count.clone();
            d.on(move |_e| {
                c.fetch_add(1, Ordering::SeqCst);
                false
            });
        }
        assert!(!d.dispatch_waterfall(&1)); // 无拦截
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_mode_names() {
        assert_eq!(DispatchMode::Emit.name(), "emit");
        assert_eq!(DispatchMode::Waterfall.name(), "waterfall");
        assert_eq!(DispatchMode::Parallel.name(), "parallel");
        assert_eq!(DispatchMode::Serial.name(), "serial");
    }
}
