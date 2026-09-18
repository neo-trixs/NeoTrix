/// SelfTest trait — 跨模块共享的自测试接口定义
///
/// 下沉到 L0 Substrate 层，供 L1-L6 所有层使用。
/// 实现保留在各自层（如 L6 的 nt_core_self_test_impl.rs）。
pub trait SelfTest: Send + Sync {
    fn name(&self) -> &str;
    fn self_test(&self) -> Result<(), Vec<String>>;
}