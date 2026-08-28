//! L6 / NT-ACT — omni_route (github.com/diegosouzapw/OmniRoute) 吸收节点 (C1)。
//!
//! 源: OmniRoute — 请求/任务智能路由, 把入站请求按上下文匹配到最优处理通道。
//! NeoTrix 视角: 路由 = 类比 GWT 注意力路由 (resonance-based routing) — 一个
//! 广播的" salient 请求"经共振路由到最合适的 specialist (handler)。本节点提供
//! 无副作用的路由 stub。

use crate::core::nt_core_self_test::SelfTest;

/// 一个可被路由的入站请求 (salient 信号)。
#[derive(Debug, Clone)]
pub struct InboundRequest {
    pub route_key: String,
    pub payload: String,
}

/// 一个候选处理通道 (specialist handler)。
#[derive(Debug, Clone)]
pub struct RouteHandler {
    pub name: String,
    pub accepts: String,
}

/// 请求/任务路由 (GWT 注意力路由类比) (stub)。
pub trait OmniRouter {
    /// 把请求路由到首个接受其 route_key 的 handler, 返回 handler 名。
    fn route(&self, req: &InboundRequest, handlers: &[RouteHandler]) -> Option<String>;
}

/// 纯计算路由器 (无副作用): 前缀/精确匹配 route_key。
pub struct OmniRouteRouter;

impl OmniRouter for OmniRouteRouter {
    fn route(&self, req: &InboundRequest, handlers: &[RouteHandler]) -> Option<String> {
        if req.route_key.is_empty() {
            return None;
        }
        handlers
            .iter()
            .find(|h| h.accepts == req.route_key || req.route_key.starts_with(&format!("{}.", h.accepts)))
            .map(|h| h.name.clone())
    }
}

/// 纯函数: 解析路由目标。
pub fn resolve_route(req: &InboundRequest, handlers: &[RouteHandler]) -> Option<String> {
    OmniRouteRouter.route(req, handlers)
}

#[derive(Default)]
pub struct OmniRouteSelfTest;

impl SelfTest for OmniRouteSelfTest {
    fn name(&self) -> &str {
        "nt_act_omni_route"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let rt = OmniRouteRouter;
        let mut errs = Vec::new();
        let handlers = [
            RouteHandler { name: "chat".into(), accepts: "io.chat".into() },
            RouteHandler { name: "crawl".into(), accepts: "world.crawl".into() },
        ];
        match rt.route(&InboundRequest { route_key: "io.chat".into(), payload: "hi".into() }, &handlers) {
            Some(n) if n == "chat" => {}
            other => errs.push(format!("omni_route: io.chat must hit chat, got {other:?}")),
        }
        match rt.route(&InboundRequest { route_key: "world.crawl.feed".into(), payload: "x".into() }, &handlers) {
            Some(n) if n == "crawl" => {}
            other => errs.push(format!("omni_route: prefix match must hit crawl, got {other:?}")),
        }
        if rt.route(&InboundRequest { route_key: "io.unknown".into(), payload: "x".into() }, &handlers).is_some() {
            errs.push("omni_route: unknown route must fall through to None".into());
        }
        if rt.route(&InboundRequest { route_key: String::new(), payload: "x".into() }, &handlers).is_some() {
            errs.push("omni_route: empty route_key must be None".into());
        }
        if resolve_route(&InboundRequest { route_key: "io.chat".into(), payload: "x".into() }, &handlers) != Some("chat".into()) {
            errs.push("omni_route: resolve_route inconsistent".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_routes() {
        let rt = OmniRouteRouter;
        let handlers = [RouteHandler { name: "a".into(), accepts: "x".into() }];
        assert_eq!(rt.route(&InboundRequest { route_key: "x".into(), payload: "".into() }, &handlers), Some("a".into()));
    }

    #[test]
    fn prefix_match_routes() {
        let rt = OmniRouteRouter;
        let handlers = [RouteHandler { name: "b".into(), accepts: "y".into() }];
        assert_eq!(rt.route(&InboundRequest { route_key: "y.sub".into(), payload: "".into() }, &handlers), Some("b".into()));
    }

    #[test]
    fn unknown_falls_through() {
        let rt = OmniRouteRouter;
        let handlers = [RouteHandler { name: "a".into(), accepts: "x".into() }];
        assert_eq!(rt.route(&InboundRequest { route_key: "z".into(), payload: "".into() }, &handlers), None);
        assert_eq!(rt.route(&InboundRequest { route_key: String::new(), payload: "".into() }, &handlers), None);
    }
}
