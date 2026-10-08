//! `nt_routing` — 路由组（Magpie-style `group/<name>`）：把同池多端点收成
//! 一个逻辑成员，`order`/`rotate`/`usage` 三档挑选 + 可重试失败自动 failover。
//!
//! 接线面：`neobot run --provider <name>` 先按**路由组名**解析，命中则走
//! [`RoutingEngine`]；否则按普通 provider 解析。组内成员是 `providers` 表里
//! 的 provider 名；不新增平行引擎，`HttpEngine` 原样透传各自的 key_env。
//!
//! 失败分类复用 `nt_error::classify_engine_failure`：仅 `Transport`/`RateLimit`
//! 触发 failover；`Authentication`/`ContextOverflow`/`ResumeNotFound`/`Unknown`
//! 立即上报（换账号/改模型才是正解，重试无意义）。

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::nt_engine::{EngineAdapter, EngineTurn};
use crate::nt_error::{NtBotError, classify_engine_failure};
use crate::nt_types::TranscriptItem;

/// 挑选模式（对齐 Magpie routing group 的 order/rotate/usage 三档）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMode {
    /// 按成员顺序，第一个可服务者即用（廉价优先/主备）。
    Order,
    /// 每轮起点轮询（均摊账号压力）。
    Rotate,
    /// 已用次数最少者优先。
    Usage,
    /// 额度感知：最近一次失败距今最久的成员优先（失败即额度可能已耗尽，
    /// 冷却时间越久的账号额度恢复可能性越高；全新成员 last_fail=0 排最前）。
    Quota,
}

impl RouteMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Order => "order",
            Self::Rotate => "rotate",
            Self::Usage => "usage",
            Self::Quota => "quota",
        }
    }
    pub fn parse(s: &str) -> Result<Self, NtBotError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "order" => Ok(Self::Order),
            "rotate" => Ok(Self::Rotate),
            "usage" => Ok(Self::Usage),
            "quota" => Ok(Self::Quota),
            other => Err(NtBotError::Invalid(format!(
                "unknown route mode '{other}' (order|rotate|usage)"
            ))),
        }
    }
}

/// 路由组行（`route_groups` 表 1:1）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouteGroup {
    pub name: String,
    /// 成员 provider 名（顺序即 order 模式的尝试顺序）。
    pub members: Vec<String>,
    /// order | rotate | usage。
    pub mode: String,
    pub enabled: bool,
}

/// 路由组成员：provider 名 + 已构建引擎。
pub struct RouteMember {
    pub provider: String,
    pub engine: Box<dyn EngineAdapter>,
}

/// 组行为：按模式排引擎顺序，逐个尝试直到成功或遇到不可重试错误。
pub struct RoutingEngine {
    name: String,
    members: Vec<RouteMember>,
    mode: RouteMode,
    rotation: AtomicUsize,
    use_counts: Vec<AtomicUsize>,
    /// 成员上次失败的 unix 秒（0=从未失败）；失败只写这里，不攒计数。
    fail_last: Vec<AtomicUsize>,
}

impl RoutingEngine {
    pub fn new(name: &str, mode: RouteMode, members: Vec<RouteMember>) -> Result<Self, NtBotError> {
        if members.is_empty() {
            return Err(NtBotError::Invalid(format!(
                "route group '{name}' has no enabled members"
            )));
        }
        Ok(Self {
            name: name.to_owned(),
            use_counts: members.iter().map(|_| AtomicUsize::new(0)).collect(),
            fail_last: members.iter().map(|_| AtomicUsize::new(0)).collect(),
            members,
            mode,
            rotation: AtomicUsize::new(0),
        })
    }

    pub fn group_name(&self) -> &str {
        &self.name
    }

    /// 当前模式下的尝试顺序（rotate 每轮递增起点；usage 按成功次数升序）。
    fn attempt_order(&self) -> Vec<usize> {
        let n = self.members.len();
        match self.mode {
            RouteMode::Order => (0..n).collect(),
            RouteMode::Rotate => {
                let start = self.rotation.fetch_add(1, Ordering::Relaxed) % n;
                (0..n).map(|i| (start + i) % n).collect()
            }
            RouteMode::Usage => {
                let mut idx: Vec<usize> = (0..n).collect();
                idx.sort_by_key(|&i| self.use_counts[i].load(Ordering::Relaxed));
                idx
            }
            RouteMode::Quota => {
                // 失败越久（fail_last 越小）越先——额度越可能恢复。
                let mut idx: Vec<usize> = (0..n).collect();
                idx.sort_by_key(|&i| self.fail_last[i].load(Ordering::Relaxed));
                idx
            }
        }
    }

    fn record_fail(&self, idx: usize) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize)
            .unwrap_or(0);
        self.fail_last[idx].store(now, Ordering::Relaxed);
    }

    fn record_success(&self, idx: usize) {
        self.use_counts[idx].fetch_add(1, Ordering::Relaxed);
    }
}

fn is_failoverable(err: &NtBotError) -> bool {
    classify_engine_failure(&err.to_string()).is_retryable()
}

impl EngineAdapter for RoutingEngine {
    fn engine_id(&self) -> &str {
        "route"
    }

    fn model_name(&self) -> &str {
        self.members
            .first()
            .map(|m| m.engine.model_name())
            .unwrap_or("")
    }

    fn vision_capable(&self) -> bool {
        // 诚实披露：仅当首选成员支持视觉时才宣称（换成员后可能不支持）。
        self.members
            .first()
            .map(|m| m.engine.vision_capable())
            .unwrap_or(false)
    }

    fn probe(&self) -> Result<String, NtBotError> {
        let mut last = None;
        for idx in self.attempt_order() {
            match self.members[idx].engine.probe() {
                Ok(msg) => return Ok(format!("{} -> {}", self.members[idx].provider, msg)),
                Err(err) => last = Some(err),
            }
        }
        Err(last.unwrap_or_else(|| {
            NtBotError::Engine {
                engine: "route".to_owned(),
                reason: "no members".to_owned(),
            }
        }))
    }

    fn run_turn(&self, prompt: &str, inbox: &[String]) -> Result<EngineTurn, NtBotError> {
        let mut last = None;
        for idx in self.attempt_order() {
            match self.members[idx].engine.run_turn(prompt, inbox) {
                Ok(turn) => {
                    self.record_success(idx);
                    return Ok(turn);
                }
                Err(err) if is_failoverable(&err) => {
                    self.record_fail(idx);
                    last = Some(err)
                }
                Err(err) => return Err(err),
            }
        }
        Err(last.unwrap_or_else(|| NtBotError::Engine {
            engine: "route".to_owned(),
            reason: "no members".to_owned(),
        }))
    }

    fn run_turn_with_history(
        &self,
        prompt: &str,
        history: &[TranscriptItem],
    ) -> Result<EngineTurn, NtBotError> {
        let mut last = None;
        for idx in self.attempt_order() {
            match self.members[idx]
                .engine
                .run_turn_with_history(prompt, history)
            {
                Ok(turn) => {
                    self.record_success(idx);
                    return Ok(turn);
                }
                Err(err) if is_failoverable(&err) => {
                    self.record_fail(idx);
                    last = Some(err)
                }
                Err(err) => return Err(err),
            }
        }
        Err(last.unwrap_or_else(|| NtBotError::Engine {
            engine: "route".to_owned(),
            reason: "no members".to_owned(),
        }))
    }

    fn run_turn_stream(
        &self,
        prompt: &str,
        history: &[TranscriptItem],
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<EngineTurn, NtBotError> {
        let mut last = None;
        for idx in self.attempt_order() {
            let mut produced = false;
            let mut forward = |delta: &str| {
                produced = true;
                on_delta(delta);
            };
            match self.members[idx]
                .engine
                .run_turn_stream(prompt, history, &mut forward)
            {
                Ok(turn) => {
                    self.record_success(idx);
                    return Ok(turn);
                }
                // 已吐过增量就不能再 failover（会重复内容）；无增量才可换成员。
                Err(err) if !produced && is_failoverable(&err) => {
                    self.record_fail(idx);
                    last = Some(err)
                }
                Err(err) => return Err(err),
            }
        }
        Err(last.unwrap_or_else(|| NtBotError::Engine {
            engine: "route".to_owned(),
            reason: "no members".to_owned(),
        }))
    }
}

/// `--provider <name>` 的统一解析：先路由组，再 provider；都不在返回 `None`。
pub fn build_engine_by_name(
    store: &crate::NeobotStore,
    name: &str,
    model_override: Option<&str>,
    memory: Option<String>,
) -> Result<Option<Box<dyn EngineAdapter>>, NtBotError> {
    if let Some(group) = store.get_route_group(name)? {
        if !group.enabled {
            return Err(NtBotError::Store(format!("route group '{name}' is off")));
        }
        let mode = RouteMode::parse(&group.mode)?;
        let mut members = Vec::new();
        for member_name in &group.members {
            let provider = store.get_provider(member_name)?.ok_or_else(|| {
                NtBotError::Store(format!(
                    "route group '{name}' member '{member_name}' has no such provider"
                ))
            })?;
            if !provider.enabled {
                continue; // 跳过禁用成员（主成员在最后一个也能继续）
            }
            let engine = provider.http_engine(model_override)?.with_memory_context(memory.clone());
            members.push(RouteMember {
                provider: member_name.clone(),
                engine: Box::new(engine),
            });
        }
        if members.is_empty() {
            return Err(NtBotError::Store(format!(
                "route group '{name}' has no enabled members"
            )));
        }
        return Ok(Some(Box::new(RoutingEngine::new(name, mode, members)?)));
    }
    match store.get_provider(name)? {
        Some(provider) => {
            if !provider.enabled {
                return Err(NtBotError::Store(format!("provider '{name}' is off")));
            }
            let engine = provider.http_engine(model_override)?.with_memory_context(memory);
            Ok(Some(Box::new(engine)))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubEngine {
        model: &'static str,
        fail_with: Option<String>,
        calls: AtomicUsize,
    }

    impl StubEngine {
        fn ok(model: &'static str) -> Self {
            Self {
                model,
                fail_with: None,
                calls: AtomicUsize::new(0),
            }
        }
        fn failing(model: &'static str, diagnostic: &str) -> Self {
            Self {
                model,
                fail_with: Some(diagnostic.to_owned()),
                calls: AtomicUsize::new(0),
            }
        }
    }

    impl EngineAdapter for StubEngine {
        fn engine_id(&self) -> &str {
            "stub"
        }
        fn model_name(&self) -> &str {
            self.model
        }
        fn probe(&self) -> Result<String, NtBotError> {
            Ok("stub ready".to_owned())
        }
        fn run_turn(&self, _prompt: &str, _inbox: &[String]) -> Result<EngineTurn, NtBotError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            match &self.fail_with {
                Some(reason) => Err(NtBotError::Engine {
                    engine: "stub".to_owned(),
                    reason: reason.clone(),
                }),
                None => Ok(EngineTurn {
                    assistant_text: format!("from {}", self.model),
                    status: crate::nt_types::TurnStatus::Done,
                    tool_calls: Vec::new(),
                    usage: None,
                    side_effects: Vec::new(),
                }),
            }
        }
    }

    fn member(name: &str, engine: StubEngine) -> RouteMember {
        RouteMember {
            provider: name.to_owned(),
            engine: Box::new(engine),
        }
    }

    #[test]
    fn order_fails_over_on_transport_error() {
        let engine = RoutingEngine::new(
            "g",
            RouteMode::Order,
            vec![
                member(
                    "a",
                    StubEngine::failing("m1", "transport error: connection refused"),
                ),
                member("b", StubEngine::ok("m2")),
            ],
        )
        .expect("build");
        let turn = engine.run_turn("hi", &[]).expect("failover succeeded");
        assert_eq!(turn.assistant_text, "from m2");
    }

    #[test]
    fn authentication_error_does_not_failover() {
        let engine = RoutingEngine::new(
            "g",
            RouteMode::Order,
            vec![
                member("a", StubEngine::failing("m1", "authentication failed")),
                member("b", StubEngine::ok("m2")),
            ],
        )
        .expect("build");
        assert!(engine.run_turn("hi", &[]).is_err());
    }

    #[test]
    fn rotate_spreads_across_members() {
        let engine = RoutingEngine::new(
            "g",
            RouteMode::Rotate,
            vec![member("a", StubEngine::ok("m1")), member("b", StubEngine::ok("m2"))],
        )
        .expect("build");
        let first = engine.run_turn("hi", &[]).expect("ok").assistant_text;
        let second = engine.run_turn("hi", &[]).expect("ok").assistant_text;
        assert_ne!(first, second);
    }

    #[test]
    fn usage_prefers_least_used_member() {
        let engine = RoutingEngine::new(
            "g",
            RouteMode::Usage,
            vec![member("a", StubEngine::ok("m1")), member("b", StubEngine::ok("m2"))],
        )
        .expect("build");
        let first = engine.run_turn("hi", &[]).expect("ok").assistant_text;
        let second = engine.run_turn("hi", &[]).expect("ok").assistant_text;
        assert_ne!(first, second);
    }

    #[test]
    fn empty_group_rejected() {
        assert!(RoutingEngine::new("g", RouteMode::Order, vec![]).is_err());
    }

    #[test]
    fn mode_parse_roundtrip() {
        assert_eq!(RouteMode::parse("order").expect("ok"), RouteMode::Order);
        assert_eq!(RouteMode::parse("ROTATE").expect("ok"), RouteMode::Rotate);
        assert!(RouteMode::parse("smart").is_err());
    }
}
