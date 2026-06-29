use super::content::SourceContent;

/// 探索源 trait — 统一外部探索接口
///
/// 每个具体的探索源 (浏览器/API/搜索/文件/爬虫) 实现此 trait,
/// 由 ExplorationOrchestrator 统一调度。
pub trait ExplorationSource: Send {
    /// 来源名称 (用于日志和调度)
    fn name(&self) -> &'static str;

    /// 来源类型权重 [0, 1] — 越高表示该来源的信息可信度/价值越高
    fn confidence(&self) -> f64 {
        0.7
    }

    /// 执行一次探索, 返回发现的内容
    fn explore(&mut self) -> Result<Vec<SourceContent>, String>;

    /// 本来源是否就绪 (如浏览器是否已登录)
    fn is_ready(&self) -> bool {
        true
    }

    /// 重置状态 (如登录过期需重新认证)
    fn reset(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// 当前待处理队列大小 (用于调度优先级)
    fn pending_count(&self) -> usize {
        0
    }
}
