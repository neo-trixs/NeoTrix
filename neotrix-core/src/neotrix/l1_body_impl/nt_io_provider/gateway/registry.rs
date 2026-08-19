use std::sync::{Arc, LazyLock, Mutex, Weak};

use super::GatewayV2;

// ═══════════════════════════════════════════════════════════════════
// Auto Exacto 周期重估注册表 (R-P79 生产接线)
// GatewayV2 在各子系统各自持有 (SEAL 推理引擎 / subagent 共享缓存 / 会话网关),
// 后台循环无法直接引用某个实例。这里提供进程级注册表 (Weak 防泄漏) —
// NT-MIND 后台循环每 5min 经 run_periodic_re_evaluation() 统一 tick,
// 使市场权重重估由生产调度驱动, 而非仅依赖 route() 惰性触发。
// ═══════════════════════════════════════════════════════════════════

/// 进程级活跃 GatewayV2 注册表 — Weak 持有, 网关释放后自动剔除。
pub static RE_EVALUATION_GATEWAYS: LazyLock<Mutex<Vec<Weak<GatewayV2>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// 注册一个进程级共享 GatewayV2 参与 Auto Exacto 周期重估。
/// 生产共享网关创建点 (SEAL 推理引擎 / subagent 静态缓存) 调用;
/// Weak 持有 — 网关被释放后由下一次 tick 剔除, 不泄漏。
pub fn register_gateway_for_re_evaluation(gateway: &Arc<GatewayV2>) {
    if let Ok(mut registry) = RE_EVALUATION_GATEWAYS.lock() {
        registry.retain(|w| w.strong_count() > 0);
        registry.push(Arc::downgrade(gateway));
    }
}

/// 周期驱动 Auto Exacto 重估 — 遍历注册的活跃 GatewayV2 调用
/// [`GatewayV2::maybe_re_evaluate`]。返回本次实际触发重估的网关数
/// (每个网关内部受各自 5min 间隔约束, 未到期返回 false 不计数)。
pub fn run_periodic_re_evaluation() -> usize {
    let mut registry = match RE_EVALUATION_GATEWAYS.lock() {
        Ok(reg) => reg,
        Err(e) => {
            log::warn!("[gateway] re-evaluation registry poisoned: {}", e);
            e.into_inner()
        }
    };
    registry.retain(|w| w.strong_count() > 0);
    let mut evaluated = 0usize;
    for weak in registry.iter() {
        if let Some(gw) = weak.upgrade() {
            if gw.maybe_re_evaluate() {
                evaluated += 1;
            }
        }
    }
    evaluated
}