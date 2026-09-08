# NeoTrix 后续任务清单

> 生成时间: 2026-09-03
> 基于: C6 升级完成 + 本次会话经验

---

## 一、C6 进化循环 (已完成 ✅)

| 任务 | 状态 | 说明 |
|------|------|------|
| 42+ 核心模块实现 EvolutionCapable | ✅ | 11 域全覆盖 |
| capability_registry.json 更新 | ✅ | 132/251 模块 c6evolutionloop |
| C6 特定编译错误修复 | ✅ | 0 个 C6 相关错误 |
| 经验落盘 (experience-tree) | ✅ | 6 条经验已写入 pending-absorb.json |

---

## 二、编译错误修复 (37 个预存错误)

### P0 - 高优先级

| # | 文件 | 错误数 | 类型 | 说明 |
|---|------|--------|------|------|
| 1 | `cli/commands/agent_cmds.rs` | 14 | 字段缺失/方法不存在 | SubagentManager API 不匹配 |
| 2 | `core/nt_game/roguelike/run.rs` | 5 | 字段缺失 | roguelike 游戏模块字段缺失 |
| 3 | `cli/commands/kanban_cmds.rs` | 3 | 字段缺失 | Kanban 命令字段缺失 |

### P1 - 中优先级

| # | 文件 | 错误数 | 类型 | 说明 |
|---|------|--------|------|------|
| 4 | `l1_action/nt_act/nt_act_orchestrator/mod.rs` | 2 | 类型不匹配 | 编排器类型错误 |
| 5 | `l5_cognition/nt_core/video_prompt_cache.rs` | 2 | borrow/move | 借用检查器冲突 |
| 6 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/harness_adapter.rs` | 2 | 类型不匹配 | 适配器类型错误 |
| 7 | `core/nt_game/roguelike/cards.rs` | 2 | trait bound | Deserialize 不满足 |
| 8 | `core/nt_game/arc/env.rs` | 2 | 方法不存在 | grid_hash/total_transitions |

### P2 - 低优先级

| # | 文件 | 错误数 | 类型 | 说明 |
|---|------|--------|------|------|
| 9 | `l5_cognition/nt_core/prompt_cache.rs` | 1 | 类型不匹配 | 缓存类型错误 |
| 10 | `l1_action/nt_act/nt_act_code/code_writer.rs` | 1 | 类型不匹配 | 代码生成器错误 |
| 11 | `l1_action/nt_act/nt_act_code/pipeline_autofixer.rs` | 1 | 类型不匹配 | 自动修复器错误 |
| 12 | `l1_action/nt_act/observability_stack.rs` | 1 | 类型不匹配 | 可观测性错误 |
| 13 | `l6_meta/nt_meta/video_quality_scorer.rs` | 1 | unused variable | 未使用变量 |
| 14 | `core/nt_core_sense/sensory_types.rs` | 1 | 字段缺失 | 感官类型字段缺失 |
| 15 | `agent.rs` | 1 | 类型不匹配 | Agent 类型错误 |
| 16 | `cli/commands/file_cmds.rs` | 1 | 类型不匹配 | 文件命令类型错误 |
| 17 | `cli/commands/types.rs` | 1 | 类型不匹配 | 类型定义错误 |
| 18 | `l1_action/nt_memory/nt_memory_kb/mod.rs` | 1 | 类型不匹配 | 知识库类型错误 |
| 19 | `l1_action/traits.rs` | 1 | 类型不匹配 | Trait 定义错误 |
| 20 | `lib.rs` | 1 | 未使用导入 | 清理未使用导入 |
| 21 | `neotrix/nt_core_capability_tree/src/cli.rs` | 1 | 类型不匹配 | 能力树 CLI 错误 |

---

## 三、质量保障

| # | 任务 | 优先级 | 说明 |
|---|------|--------|------|
| 1 | SelfTest T1-T3 全量注册 | P1 | 42+ 核心模块单元测试 |
| 2 | `cargo test --lib` 全量通过 | P1 | 依赖预存错误修复 |
| 3 | `cargo check --all-targets` 通过 | P1 | 全量编译检查 |
| 4 | 能力矩阵文档更新 | P2 | 架构图/能力网文档 |
| 5 | 集成测试编写 | P2 | C6 进化循环端到端测试 |

---

## 四、加密 HTML 价格查询系统

| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 1 | 单重列添加 | ✅ | TCOLS + tbody 渲染代码均已修复 |
| 2 | 数据验证 | ⏳ | 需浏览器测试确认单重数据正确显示 |
| 3 | 样式优化 | ⏳ | 单重列可能需要数值格式化 (小数位) |

---

## 五、架构优化 (长期)

| # | 任务 | 优先级 | 说明 |
|---|------|--------|------|
| 1 | 预存错误根因分析 | P1 | 37 个错误可能有共同根因 (API 变更/类型重构) |
| 2 | 构建缓存清理 | P2 | 确保 cargo clean 后真实错误计数 |
| 3 | 文档同步 | P2 | AGENTS.md/CONTEXT.md 与代码同步 |
| 4 | 性能基准测试 | P3 | C6 进化循环性能对比 |

---

## 六、经验教训 (本次会话)

| 编号 | 类型 | 内容 |
|------|------|------|
| E1 | pattern | Rust 模块修复模式: Serialize/flat_map/match arms/blake2 |
| E2 | defect | 加密 HTML 表格: TCOLS 只控表头, tbody 硬编码需同步修改 |
| E3 | rule | 加密修改流程: 解密→修改→重新编码→替换 |
| E4 | insight | C6 双轨策略: 直接实现 + 注册表批量升级 |
| E5 | pattern | BMonitor 集成: as_ref().and_then() 模式 |

---

## 七、下一步行动 (Recommended)

### 立即 (本次会话)
1. ✅ 经验落盘 → `~/.neotrix/pending-absorb.json`

### 短期 (1-2 天)
1. 浏览器测试价格查询系统单重列
2. 修复 `agent_cmds.rs` (14 个错误) - 影响最大
3. 修复 `roguelike/run.rs` (5 个错误)

### 中期 (1 周)
1. 全量编译通过 (`cargo check --all-targets`)
2. SelfTest T1-T3 注册
3. 能力矩阵文档更新

### 长期 (1 月)
1. 架构优化/重构
2. 性能基准测试
3. 生产部署准备
