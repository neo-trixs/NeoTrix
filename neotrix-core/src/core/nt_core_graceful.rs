use std::fmt::Debug;

/// GracefulDegradation — 人脑式优雅降级.
///
/// 人脑: 一个神经元死了, 脑子不崩溃, 只是某个能力减弱.
/// 传统软件: unwrap() 到 None → panic → 整个进程崩溃.
///
/// 本 trait: 每个子系统定义"健康"和"降级"两个模式.
/// - 健康时: 正常工作
/// - 降级时: 返回 fallback, 记录日志, 缩小能力范围但不崩溃
///
/// 这是 P3.0 原则的工程实现: 零生产 panic, 降级不死亡.
pub trait GracefulDegradation: Debug {
    /// 当前是否健康 (所有依赖就绪)
    fn is_healthy(&self) -> bool { true }

    /// 单行健康状态描述 (用于仪表盘)
    fn health_status(&self) -> &'static str { "unknown" }

    /// 降级时的 fallback 行为描述
    fn degradation_policy(&self) -> &'static str { "no-op" }
}

/// 为 Option<T> 提供降级安全的 unwrap.
///
/// 代替 `.unwrap()` 和 `.expect("...")`:
/// - 如果 Some: 正常返回
/// - 如果 None: 记录警告, 返回 degrade::default()
pub trait DegradeSafe: Debug {
    type Output;
    fn unwrap_or_graceful(&mut self, component: &'static str) -> Self::Output;
}

impl<T: Default + Debug> DegradeSafe for Option<T> {
    type Output = T;

    fn unwrap_or_graceful(&mut self, component: &'static str) -> T {
        match self.take() {
            Some(val) => val,
            None => {
                eprintln!("[graceful] {} was None — using degraded fallback", component);
                T::default()
            }
        }
    }
}

/// 全局恢复包装器: 将 Result<T,E> 包装为"最多降级, 从不崩溃".
pub fn recover_or_degrade<T, E: Debug>(
    result: Result<T, E>,
    component: &'static str,
    fallback: T,
) -> T {
    match result {
        Ok(val) => val,
        Err(e) => {
            eprintln!(
                "[graceful] {} failed with {:?} — using degraded fallback",
                component, e
            );
            fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_degrade_safe() {
        let mut some: Option<i32> = Some(42);
        assert_eq!(some.unwrap_or_graceful("test"), 42);

        let mut none: Option<i32> = None;
        assert_eq!(none.unwrap_or_graceful("test"), 0);
    }

    #[test]
    fn test_recover_or_degrade() {
        let ok: Result<i32, String> = Ok(42);
        assert_eq!(recover_or_degrade(ok, "test", -1), 42);

        let err: Result<i32, String> = Err("broken".into());
        assert_eq!(recover_or_degrade(err, "test", -1), -1);
    }
}
