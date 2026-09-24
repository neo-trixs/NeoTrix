# TypeScript/JavaScript 审计规则 (system 层)

1. **类型安全**: 禁 `any` 泄漏到公开接口；`!` 非空断言需注释理由。
2. **异步**: 未 `await` 的 Promise、缺 `catch` 的顶层调用。
3. **密钥**: 硬编码 token/key/secret（`.env.example` 除外）。
4. **依赖**: 新增 npm 包检查体积与维护状态。
