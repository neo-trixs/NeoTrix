// 金丝雀窗口接线测试（2026-10-05）
//
// 背景：`nt_capability_canary::tick()` / `::reset()` 此前**零生产调用者**
// ⇒ `window_ticks()` 恒 0
// ⇒ 健康判据 `fired>0 || ticks<3` 的第二项**恒真**
// ⇒ 任何能力永远被判「健康」。
//
// 本模块的测试都**必须能证伪**：否则删掉 tick 接线它们仍全绿。
// 故每条都先归零、再断言增量。
//
// 共享状态纪律：金丝雀窗口是**进程全局**的，所以这些测试不能断言绝对值，
// 否则并行执行时互相踩。需要跨过一条边界取绝对值时，必须持有 `WINDOW_LOCK`。
//
// 这把锁不是权宜之计，是该数据结构的真实约束：窗口按设计是「整个进程一个」，
// 于是「在某个 turn 前后读窗口」这件事天然不可并行化。
// 2026-10-05 首版不持锁 ⇒ 串行 5 绿 / 并行 5 红。

use super::nt_loop_tests::{backend_with, tool_call, MockCalc};
use super::nt_loop_types::AgentLoop;
use crate::l1_action::nt_io::nt_io_provider::types::FinishReason;
use neotrix_neobot::nt_capability_canary as canary;
use std::sync::{Arc, Mutex};

static WINDOW_LOCK: Mutex<()> = Mutex::new(());

#[tokio::test]
async fn 每次工具派发推进一个金丝雀窗口() {
    let _g = WINDOW_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    canary::reset();
    assert_eq!(canary::window_ticks(), 0, "前置：窗口应从 0 起");

    // 第 1 轮：模型请求 calc；第 2 轮：给出最终答案。
    let (llm, _seen) = backend_with(vec![
        (
            "".into(),
            FinishReason::Tool,
            vec![tool_call("calc", "call_1", r#"{"expr":"1+1"}"#)],
        ),
        ("done".into(), FinishReason::Stop, vec![]),
    ]);
    let mut loop_ = AgentLoop::new(llm, "mock", "")
        .with_tools(vec![Box::new(MockCalc {
            calls: Arc::new(Mutex::new(Vec::new())),
        })])
        .with_max_tool_rounds(4);

    loop_.turn("go").await.expect("turn ok");
    assert_eq!(
        canary::window_ticks(),
        1,
        "一次工具派发应推进恰好 1 tick（不是 0，也不是按 turn 数计）"
    );
}

#[tokio::test]
async fn 同一轮多个工具调用各自推进窗口() {
    let _g = WINDOW_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    canary::reset();
    let (llm, _seen) = backend_with(vec![
        (
            "".into(),
            FinishReason::Tool,
            vec![
                tool_call("calc", "c1", r#"{"expr":"1"}"#),
                tool_call("calc", "c2", r#"{"expr":"2"}"#),
                tool_call("calc", "c3", r#"{"expr":"3"}"#),
            ],
        ),
        ("done".into(), FinishReason::Stop, vec![]),
    ]);
    let mut loop_ = AgentLoop::new(llm, "mock", "")
        .with_tools(vec![Box::new(MockCalc {
            calls: Arc::new(Mutex::new(Vec::new())),
        })])
        .with_max_tool_rounds(4);

    loop_.turn("go").await.expect("turn ok");
    assert_eq!(
        canary::window_ticks(),
        3,
        "3 个 tool call 应推进 3 tick（若按轮计则只有 1）"
    );
}

/// 反事实价值：若有人在 `turn()` 入口也打一次 tick，本测试会得到 1 而非 0，
/// 从而精确抓出「窗口被无关轮次灌水」。
#[tokio::test]
async fn 不调工具的轮次不推进窗口() {
    let _g = WINDOW_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    canary::reset();
    let (llm, _seen) = backend_with(vec![("just text".into(), FinishReason::Stop, vec![])]);
    let mut loop_ = AgentLoop::new(llm, "mock", "").with_max_tool_rounds(4);

    let out = loop_.turn("hello").await.expect("turn ok");
    assert_eq!(out, "just text");
    assert_eq!(
        canary::window_ticks(),
        0,
        "模型没调任何能力时窗口不应推进，否则阈值被无关轮次灌水而形同虚设"
    );
}

#[tokio::test]
async fn 会话起点归零金丝雀窗口() {
    let _g = WINDOW_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    canary::reset();
    for _ in 0..5 {
        canary::tick();
    }
    assert_eq!(canary::window_ticks(), 5);

    let (llm, _seen) = backend_with(vec![("x".into(), FinishReason::Stop, vec![])]);
    let mut loop_ = AgentLoop::new(llm, "mock", "");
    loop_.begin_session_window();

    assert_eq!(
        canary::window_ticks(),
        0,
        "会话起点必须归零，否则一次信号能让金丝雀在整个进程生命周期保持健康"
    );
}

/// 共享状态纪律的守门测试：`new()` 不应归零。
/// 归零必须是**显式的会话生命周期事件**，不能挂在「对象被构造过」这个事实上
/// ——否则并行测试会互相清零对方的计数，门与测试将读到随机值。
#[tokio::test]
async fn 构造loop不归零窗口() {
    let _g = WINDOW_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    canary::reset();
    for _ in 0..3 {
        canary::tick();
    }
    let (llm, _seen) = backend_with(vec![("x".into(), FinishReason::Stop, vec![])]);
    let _loop_ = AgentLoop::new(llm, "mock", "");

    assert_eq!(canary::window_ticks(), 3, "`new()` 不该归零");
}
