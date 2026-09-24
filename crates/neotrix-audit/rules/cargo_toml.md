# Cargo.toml 审计规则 (system 层)

1. **版本收敛**: 新依赖优先复用 `[workspace.dependencies]`，禁止同包多版本并存（`cargo deny` 视角）。
2. **feature 门**: 重型依赖（模型/浏览器/虚拟化）必须 `optional` + feature 门，默认关闭。
3. **member 对齐**: 新增 crate 必须进 workspace members；`path` 依赖必须存在。
4. **bin 膨胀**: 新增 `[[bin]]` 说明用途，单文件脚本优先放 `scripts/`。
