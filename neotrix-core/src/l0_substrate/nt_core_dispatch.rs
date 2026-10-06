#![forbid(unsafe_code)]

//! 事件派发调度器 — 吸收自 deepseek-harness vendor/cordis/src/events.ts
//! (4+1 dispatch modes: emit / waterfall / independent / serial) 与
//! cordiverse/paper §4.3.3 (asynchrony/inertia) + §5.1.2 (notify → refresh)。
//!
//! 机制:
//! - `Emit`    广播: 所有 handler 收到事件, 互不短路。
//! - `Independent` 独立: **逐 handler 隔离 panic**（真「独立」，⛔ 不是并发）。
//!   ⛔ **刻意不 spawn 线程**：L0 是无运行时依赖的同步基元，
//!   每事件 spawn 会把「同步基元」变成「线程工厂」。
//! - `Serial`  顺序: 首个 handler 返回 `true` (已处理) 即短路 (bail)。
//! - `Waterfall` 链式中间件: 每个 handler 可调 `next()` 委托给下一环 (around
//!   middleware), 或返回 `true` 短路; 都不做则顺延 (fall-through)。
//!
//! NeoTrix 消费方 (R-P79): McpServer 工具调用 pre/post 钩子 (Waterfall 中间件链),
//! 对应 dsh tools.md "工具管线 = 可扩展 waterfall" 范式。
// 2026-10-03 **从 `l5_cognition/` 下沉到 `l0_substrate/`**。
//
// ## 下沉的依据（三条，全部实测，非品味）
//
// ① **本模块零 `use`、零 `crate::` 引用** ⇒ 纯 std、**完全自包含**
//    ⇒ 它对「认知」没有任何依赖，**层次归属是历史偶然**。
//
// ② **它是基座设施，不是认知能力**：`Dispatcher` 是**通用事件/钩子链**
//    （Emit/Waterfall/Parallel/Serial）。而 L0 是「被所有人依赖的基座」——
//    L0 反过来依赖 L5 会构成**近乎循环**（见
//    `docs/architecture/LAYER-DEBT-TIERS-2026-10-03.md` §2 的 S1 级定义）。
//    原先 `l0_substrate/nt_core_event_bus.rs:6` 直接
//    `use crate::l5_cognition::nt_core_dispatch::Dispatcher`
//    ⇒ **正是 L0→L5 近循环倒置的一处**。
//
// ③ **两个消费者都不在 L5**：
//    · `l0_substrate/nt_core_event_bus.rs`（基座）
//    · `l1_action/nt_io/nt_io_mcp_bridge.rs`（动作层，`54a2fa64` 接入）
//    ⇒ **消费者分布与它的实际用途一致**，只有「定义位置」不一致。
//
// ## 兼容性
// `l5_cognition::nt_core_dispatch` 保留为 **`pub use` 再导出**
// ⇒ ⛔ 不改任何调用方的路径 ⇒ 本 commit 是**纯位置变更**，零行为变化。

/// 事件派发模式 (Cordis events.ts dispatch modes)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchMode {
    /// 广播到全部 handler (NeoTrix EventBus 现行为)
    Emit,
    /// around 中间件链: handler 可 next() 委托 / 返回 true 短路 / 静默顺延
    Waterfall,
    /// **独立处理**：跑完**全部** handler，且**每个 handler 的 panic 被隔离**。
    ///
    /// 2026-10-04 **重命名 + 语义修正**（改名零风险：实测生产零调用方）。
    ///
    /// ⛔ **改名前叫 `Parallel`，而它撒谎**：
    ///   原注释写着「并行独立处理 (同步场景等价 Emit)」——
    ///   **前半句说并行，后半句自认等价 Emit（= 串行同步）**
    ///   实现更是 `for h in &self.handlers { h(event, &|| {}) }`
    ///   ⇒ **既不并行，又阻塞 producer**。
    /// 对标 Atlas 的 `atlas-bus` 原文原则：**lagging subscriber
    ///   绝不阻塞 producer**。那个「独立」真正要保证的不是并发度，
    ///   而是 **一个坏 handler 不能拖死/拖慢 producer**。
    ///
    /// 所以这里给的是**真独立**：
    ///   ① `catch_unwind` 逐个隔离 ⇒ **一个 handler panic 不影响其余**
    ///   ② 全程 `catch_unwind` ⇒ **producer 永不 panic**
    ///   ③ ⛔ **刻意不做真并发**（不 spawn 线程）：
    ///      L0 是无运行时依赖的同步基元，每事件 spawn 线程会把
    ///      「同步基元」变成「线程工厂」，代价远大于收益。
    ///      ⇒ 并发留给上层（tokio），L0 只保证**隔离**。
    Independent,
    /// 顺序处理, 首个 handler 返回 true 即短路 (bail)
    Serial,
}

impl DispatchMode {
    pub fn name(&self) -> &'static str {
        match self {
            DispatchMode::Emit => "emit",
            DispatchMode::Waterfall => "waterfall",
            DispatchMode::Independent => "independent",
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
    /// `Independent` 模式下被隔离掉的 handler panic 次数（**实例级**）。
    ///
    /// **为什么必须是实例级、⛔ 不是进程全局**（第一版踩了）：
    ///   ⛔ 我第一版做成 `static AtomicUsize`，于是
    ///   **cargo test 并行跑测试时互相污染** —— 实测
    ///   「1 个 panic 的 handler，差值却测出 **2**」。
    ///   更本质的问题是：调用方要问的是
    ///   **「我这次 dispatch 有没有 handler 崩」**，
    ///   ⛔ 不是「全进程今天崩了几次」⇒ 观测必须**跟着 dispatcher 走**。
    /// 附带收益：无全局可变状态 ⇒ **天然可重入、可并发观测**。
    handler_panics: std::sync::atomic::AtomicUsize,
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
            // 实例级 panic 计数（⛔ 不是 `static`：见字段注释的实测教训）
            handler_panics: std::sync::atomic::AtomicUsize::new(0),
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

    /// 本实例累计隔离掉的 handler panic 次数（`Independent` 模式）。
    ///
    /// 调用方据此判断：「这次派发**是否全部成功**」。
    /// ⛔ **不要**把它当「错误率」指标跨实例相加（不同 dispatcher 语义不同）。
    pub fn handler_panic_count(&self) -> usize {
        self.handler_panics.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// 按模式派发。返回实际运行的 handler 数。
    pub fn dispatch(&self, mode: DispatchMode, event: &E) -> usize {
        match mode {
            DispatchMode::Emit => {
                let mut ran = 0;
                for h in &self.handlers {
                    h(event, &|| {});
                    ran += 1;
                }
                ran
            }
            DispatchMode::Independent => {
                // 真独立：逐个 `catch_unwind` ⇒ **一个 handler
                // panic 不影响其余，也不把 panic 抛回 producer**。
                // 这正是 Atlas `atlas-bus` 的核心契约（lagging/broken
                // subscriber 不得阻塞 producer）在本层的最小实现。
                let mut ran = 0;
                for h in &self.handlers {
                    let evt = event;
                    // ⚠️ `AssertUnwindSafe` 是**必需**的：handler 是 `&dyn Fn`，
                    // 而编译器无法证明它没有内部可变状态（`&mut` 捕获）。
                    // 这里**只用于 unwinding 边界**，⛔ 不引入任何 `unsafe`
                    // （本 crate 是 `#![forbid(unsafe_code)]`，catch_unwind 是安全 API）。
                    let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        h(evt, &|| {});
                    }))
                    .is_ok();
                    // 计数语义：**跑过就算 ran**（含 panic 那个），
                    // ⛔ 但返回值只说「派发到几个」，不该假装它成功了。
                    ran += 1;
                    if !ok {
                        // 诚实：记一笔，⛔ **不静默吞掉**
                        // （静默吞 = 用户以为「都处理了」）。
                        // 记在**实例**上（不是进程全局）——
                        // 第一版做成了 `static` 全局，结果
                        // **cargo test 并行跑测试时互相污染**
                        // （实测：1 个 panic 却测出 2），
                        // 且全局可变状态让 Dispatcher ⛔ 不可重入观测。
                        self.handler_panics.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
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
    /// `Independent` 的**核心契约**：一个 handler panic
    /// **既不打断其余 handler，也不把 panic 抛回 producer**，且**留痕**。
    ///
    /// 这是 Atlas `atlas-bus` 那条原则的可执行形态：
    /// 「lagging / broken subscriber 绝不阻塞 producer」。
    /// 而 **留痕（计数器）** 同样重要 —— 静默吞掉 panic
    /// 会让调用方以为「全部处理成功」。
    #[test]
    fn independent_isolates_panicking_handler_and_still_runs_the_rest() {
        // `Dispatcher::on` 的真签名是 `Fn(&E) -> bool`（`:137`），
        // **不是**裸 `Fn(&E, &dyn Fn())` —— 我第一版照 `dispatch`
        // 的内部 handler 形状写，编译器当场抓住（这就是有门的好处）。
        let mut d: Dispatcher<i32> = Dispatcher::new();
        // 记录「到达了第几个 handler」。
        // **必须 `Arc`**：`Dispatcher::on` 的 bound 是 `F: Fn(&E) -> bool
        //   + Send + Sync + 'static`（`:137`）⇒ **`'static` 要求闭包不借用
        //   栈上局部** ⇒ 直接捕获 `step` 报 E0373。
        // 而 `Mutex<Vec>` 更糟：handler 是 `Fn`（**不可变**捕获），
        // 闭包里拿不到 `Mutex` 的 `&mut` ⇒ **两条路都堵**，
        // 所以用 `Arc<AtomicUsize>`：`Clone` 一份进闭包，全 'static。
        let step = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        // 三个句柄各司其职：`step` 进闭包 1、`step2` 进闭包 3、
        // `reader` 留在测试里读数。**不复用已被 move 的变量**
        // （我第一版复用 ⇒ E0382，编译器当场抓住）。
        let step2 = std::sync::Arc::clone(&step);
        let reader = std::sync::Arc::clone(&step);

        d.on(move |_e: &i32| {
            step.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            false
        });
        // 故意 panic 的 handler
        d.on(|_e: &i32| panic!("induced handler panic"));
        d.on(move |_e: &i32| {
            // 记 10（而不是 3）⇒ 于是「after 被跑到」可与
            // 「停在 2」区分开 （只数个数分不清是否跳过了 3 号）
            step2.fetch_add(10, std::sync::atomic::Ordering::SeqCst);
            false
        });

        // `handler_panic_count()` 直接返回 `usize`（我上一版返回类型写错，
        // 编译器当场抓住 ⇒ **不要**再对它 `.load()`）
        let before = d.handler_panic_count();
        // 关键：**producer 侧不 panic**（改前 Emit 会 panic 出来）
        let ran = d.dispatch(DispatchMode::Independent, &1);
        let after_panics = d.handler_panic_count() - before;

        assert_eq!(ran, 3, "⭐ 三个 handler 都必须被派发到");
        assert_eq!(
            reader.load(std::sync::atomic::Ordering::SeqCst),
            11,
            "⭐⭐ panic 的 handler **不能**阻断前后两个（1 + 10）"
        );
        assert_eq!(after_panics, 1, "⭐⭐ panic 必须留痕（⛔ 不静默吞）");
    }

    #[test]
    fn test_independent_is_non_short_circuit() {
        let mut d = Dispatcher::new();
        let count = Arc::new(AtomicU32::new(0));
        for _ in 0..2 {
            let c = count.clone();
            d.on(move |_e| {
                c.fetch_add(1, Ordering::SeqCst);
                true
            });
        }
        let ran = d.dispatch(DispatchMode::Independent, &9);
        assert_eq!(ran, 2); // 全部运行 (true 不短路)
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_dispatch_waterfall_reports_short_circuit() -> Result<(), String> {
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
        assert!(d.dispatch_waterfall(&1)); // 有拦截
        assert_eq!(count.load(Ordering::SeqCst), 1);
        Ok(())
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
        assert_eq!(DispatchMode::Independent.name(), "independent");
        assert_eq!(DispatchMode::Serial.name(), "serial");
    }
}
