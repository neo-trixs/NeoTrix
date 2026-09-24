# Rust 审计规则 (system 层)

> 移植 OCR `rule_docs/rust.md` + rev-officer D4/D3: 只评新增代码。

1. **Ownership/Lifetime**: 不必要的 `clone()`、可疑的生命周期标注。
2. **Error Handling**: 生产代码禁 `unwrap`/`expect`/`panic!`/`todo!`（测试内放行），错误用 `?` 传播；`unwrap_or_default` 需说明。
3. **Unsafe 边界**: 新增 `unsafe` 一律打回（除非 FFI 隔离 crate + `forbid(unsafe_code)` 其余部分）；`allow(unsafe_code)` 必须有 reason。
4. **并发/共享状态**: `await` 跨锁持有、`Mutex` 裸锁、全局可变静态。
5. **Async 取消安全**: `select!` 分支的丢弃语义、`Drop` 内的阻塞调用。
6. **分层纪律**: 模块名 `nt_` 前缀；`l0` 不得引用上层；新增跨层 `use` 必须走 facade。
7. **API 设计**: 公开函数错误类型用本 crate `Error` 枚举，不泄漏实现细节。
