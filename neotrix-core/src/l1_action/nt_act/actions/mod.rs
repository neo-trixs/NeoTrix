// actions 模块 — 按功能分组
pub mod infra;       // 生产基础设施：可观测性、成本、调度、路由
pub mod video;       // 视频管线：作业、存储、拼接、音频
pub mod security;    // 安全防护：磁盘守卫、沙箱、策略
pub mod media;       // 媒体能力：3D、SEO
pub mod core;        // 核心工具：缓存、熔断、限流、事件总线、工作流
pub mod orchestration; // 编排调度：运维、发布、生产管线
