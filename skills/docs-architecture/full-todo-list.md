# NeoTrix 全量 TODO 清单

## 项目状态仪表板

| 指标 | 当前值 | 目标值 | 状态 |
|------|--------|--------|------|
| 总 .rs 文件 | 2,711 | - | - |
| 核心文件 | 2,413 | - | - |
| 工作空间 crate | 8 + src-tauri | - | - |
| 编译状态 | 待验证 | 通过 | ⚠️ |
| 层次违规 | 严重 | 0 | ❌ |
| TODO/FIXME/HACK | 46 | <10 | ❌ |
| `todo!()` 宏 | 10 | 0 | ❌ |
| 测试覆盖率 | 70% | >85% | ⚠️ |
| 未文档化公共项 | 78.5% | <20% | ❌ |
| 死代码标记 | 95 | <20 | ❌ |

---

## P0: 紧急修复 (立即执行)

### TODO-001: 验证编译状态
- **优先级**: P0
- **状态**: ⬜ 待开始
- **描述**: 运行 cargo clean && cargo check 验证当前编译状态
- **验收标准**: 编译通过或错误数明确

### TODO-002: 修复 L0→L6 层次违规
- **优先级**: P0
- **状态**: ⬜ 待开始
- **描述**: nt_core_traits.rs 导入 l6_meta::nt_core_kb_types, nt_core_error 导入 l1_action
- **修复方案**: 将共享类型下移到 L0 或 neotrix-types
- **涉及文件**:
  - l0_substrate/nt_core_traits.rs
  - l0_substrate/nt_core_error/mod.rs
  - l0_substrate/nt_core_qtest.rs
  - l0_substrate/nt_core_telemetry.rs
  - l0_substrate/nt_core_schema_watchdog.rs

### TODO-003: 消除 todo!() 宏
- **优先级**: P0
- **状态**: ⬜ 待开始
- **描述**: 10 个生产代码中的 todo!() 宏会导致运行时 panic
- **涉及位置**: 搜索 `todo!()` 并实现或标记为 unimplemented

---

## P1: 高优先级 (本周)

### TODO-004: 添加 L5/L6 lib.rs
- **优先级**: P1
- **状态**: ⬜ 待开始
- **描述**: L5 和 L6 没有顶层模块组织文件
- **验收标准**: 每个层有清晰的 lib.rs

### TODO-005: 修复 L1→L5 层次违规
- **优先级**: P1
- **状态**: ⬜ 待开始
- **描述**: nt_core_bank 和 nt_io 从 L5 导入
- **涉及文件**:
  - l1_action/nt_core_bank/bank/mod.rs (导入 l5_cognition::kron)
  - l1_action/nt_io/nt_io_standalone.rs (重新导出 l5_cognition)
  - l1_action/nt_io/nt_io_provider/gateway/types.rs

### TODO-006: 修复 L3→L5 层次违规
- **优先级**: P1
- **状态**: ⬜ 待开始
- **描述**: nt_shield_stealth_net 依赖 l5_cognition::nt_core_state
- **涉及文件**: l3_embodiment/nt_shield_stealth_net/ 多个文件

### TODO-007: 修复 SelfIteratingBrain FIXME
- **优先级**: P1
- **状态**: ⬜ 待开始
- **描述**: select_operator / selective_state 字段被注释
- **涉及文件**:
  - pipeline.rs:1015
  - seal_loop.rs:1211

### TODO-008: 清理死代码
- **优先级**: P1
- **状态**: ⬜ 待开始
- **描述**: 95 个 #[allow(dead_code)] 标记
- **验收标准**: 减少到 <20 个

---

## P2: 中优先级 (本月)

### TODO-009: 提升 L0 测试覆盖率
- **优先级**: P2
- **状态**: ⬜ 待开始
- **描述**: 从 59% 提升到 70%+
- **验收标准**: L0 测试覆盖率 >70%

### TODO-010: 提升 L2 测试覆盖率
- **优先级**: P2
- **状态**: ⬜ 待开始
- **描述**: 从 62% 提升到 70%+
- **验收标准**: L2 测试覆盖率 >70%

### TODO-011: 解决 fusion-plan-215 TODO
- **优先级**: P2
- **状态**: ⬜ 待开始
- **描述**: 合并 EchoPrmBridge 和 MethodRegistry
- **涉及文件**:
  - echo_terminal.rs:406
  - reasoning_core.rs:66

### TODO-012: 清理 neotrix-core/src/neotrix/ 命名空间
- **优先级**: P2
- **状态**: ⬜ 待开始
- **描述**: 110 个文件在遗留命名空间中
- **验收标准**: 迁移到正确的层结构

### TODO-013: 删除 neotrix-consciousness/src/legacy.rs
- **优先级**: P2
- **状态**: ⬜ 待开始
- **描述**: 所有消费者迁移到新路径后删除
- **验收标准**: 无遗留兼容层

### TODO-014: 添加文档注释
- **优先级**: P2
- **状态**: ⬜ 待开始
- **描述**: 78.5% 公共项未文档化
- **验收标准**: 未文档化率 <20%

---

## P3: 低优先级 (积压)

### TODO-015: 解决 29 个 TODO 注释
- **优先级**: P3
- **状态**: ⬜ 待开始
- **描述**: 生产代码中的 TODO 注释
- **验收标准**: 清理所有 TODO

### TODO-016: 修复 nt_codegen.rs:165
- **优先级**: P3
- **状态**: ⬜ 待开始
- **描述**: 生成 `// TODO: implement system logic` 作为代码输出

### TODO-017: 实现 McpRegistry.gateway()
- **优先级**: P3
- **状态**: ⬜ 待开始
- **描述**: 2 个 FIXME 关于此功能
- **涉及文件**: agent_cmds.rs:408,520

### TODO-018: 整理 nt_core_capability_tree
- **优先级**: P3
- **状态**: ⬜ 待开始
- **描述**: 嵌入式路径 crate 需要评估

---

## 进度跟踪

| Phase | 总任务 | 完成 | 进度 |
|-------|--------|------|------|
| P0 | 3 | 0 | 0% |
| P1 | 5 | 0 | 0% |
| P2 | 6 | 0 | 0% |
| P3 | 4 | 0 | 0% |
| **总计** | **18** | **0** | **0%** |

---

*清单版本: v1.0*
*创建时间: 2026-09-20*
*最后更新: 2026-09-20*
