use std::collections::HashMap;
use std::sync::RwLock;

/// 模块化网关 — 组件化架构，支持热插拔中间件
pub(crate) struct ModularGateway {
    middlewares: RwLock<Vec<Box<dyn Middleware>>>,
    routes: RwLock<HashMap<String, Route>>,
    config: ModularConfig,
}

#[derive(Debug, Clone)]
pub(crate) struct ModularConfig {
    pub max_middlewares: usize,
    pub request_timeout_ms: u64,
}

impl Default for ModularConfig {
    fn default() -> Self {
        Self {
            max_middlewares: 20,
            request_timeout_ms: 30000,
        }
    }
}

/// 中间件 trait
pub(crate) trait Middleware: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32;
    fn before_request(&self, ctx: &mut RequestCtx) -> Result<(), MiddlewareError>;
    fn after_response(&self, ctx: &mut ResponseCtx) -> Result<(), MiddlewareError>;
}

#[derive(Debug, Clone)]
pub(crate) struct RequestCtx {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseCtx {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub latency_ms: u64,
}

#[derive(Debug)]
pub(crate) enum MiddlewareError {
    Abort(String),
    Skip,
}

/// 路由定义
#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub provider: String,
    pub model: String,
    pub middlewares: Vec<String>,
}

impl ModularGateway {
    pub fn new(config: ModularConfig) -> Self {
        Self {
            middlewares: RwLock::new(Vec::new()),
            routes: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub fn add_middleware(&self, mw: Box<dyn Middleware>) -> Result<(), String> {
        let mut mws = self.middlewares.write().unwrap();
        if mws.len() >= self.config.max_middlewares {
            return Err("max middlewares reached".into());
        }
        mws.push(mw);
        mws.sort_by_key(|m| m.priority());
        Ok(())
    }

    pub fn remove_middleware(&self, name: &str) -> bool {
        let mut mws = self.middlewares.write().unwrap();
        let before = mws.len();
        mws.retain(|m| m.name() != name);
        mws.len() < before
    }

    pub fn add_route(&self, route: Route) {
        self.routes
            .write()
            .unwrap()
            .insert(route.path.clone(), route);
    }

    pub fn process_request(&self, ctx: &mut RequestCtx) -> Result<(), MiddlewareError> {
        let mws = self.middlewares.read().unwrap();
        for mw in mws.iter() {
            mw.before_request(ctx)?;
        }
        Ok(())
    }

    pub fn process_response(&self, ctx: &mut ResponseCtx) -> Result<(), MiddlewareError> {
        let mws = self.middlewares.read().unwrap();
        for mw in mws.iter().rev() {
            mw.after_response(ctx)?;
        }
        Ok(())
    }

    pub fn list_middlewares(&self) -> Vec<String> {
        self.middlewares
            .read()
            .unwrap()
            .iter()
            .map(|m| m.name().to_string())
            .collect()
    }

    pub fn list_routes(&self) -> Vec<String> {
        self.routes.read().unwrap().keys().cloned().collect()
    }
}

impl Default for ModularGateway {
    fn default() -> Self {
        Self::new(ModularConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LogMiddleware {
        name: String,
    }

    impl Middleware for LogMiddleware {
        fn name(&self) -> &str {
            &self.name
        }
        fn priority(&self) -> i32 {
            0
        }
        fn before_request(&self, _ctx: &mut RequestCtx) -> Result<(), MiddlewareError> {
            Ok(())
        }
        fn after_response(&self, _ctx: &mut ResponseCtx) -> Result<(), MiddlewareError> {
            Ok(())
        }
    }

    #[test]
    fn test_add_remove_middleware() {
        let gw = ModularGateway::default();
        gw.add_middleware(Box::new(LogMiddleware {
            name: "log".into(),
        }))
        .unwrap();
        assert_eq!(gw.list_middlewares(), vec!["log"]);
        assert!(gw.remove_middleware("log"));
        assert!(gw.list_middlewares().is_empty());
    }

    #[test]
    fn test_route_management() {
        let gw = ModularGateway::default();
        gw.add_route(Route {
            path: "/chat".into(),
            provider: "openai".into(),
            model: "gpt-4".into(),
            middlewares: vec![],
        });
        assert_eq!(gw.list_routes(), vec!["/chat"]);
    }
}
