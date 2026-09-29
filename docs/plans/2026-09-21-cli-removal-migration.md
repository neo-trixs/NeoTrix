# CLI 目录完全移除迁移计划

> 2026-09-21 确立并执行完毕 ✅
> 状态：`neotrix-core/src/cli/` 已删除，`cargo check --all-targets` 零 error 零 warning，
> `nt_auto_orchestrator` 单测 13/13，详见 `RUST-STANDARDS.md §16`。

## 1. 架构变更概述

### 移除前
```
neotrix-core/src/
├── cli/                    # ❌ 移除
│   ├── commands/           # CLI 命令系统
│   ├── tui/                # 终端 UI
│   ├── approval.rs         # 审批系统
│   ├── cost_tracker.rs     # 成本追踪
│   ├── sandbox.rs          # 沙箱
│   ├── shield_enforcer.rs  # 安全执行器
│   ├── permission_profiles.rs  # 权限配置
│   ├── laws.rs             # 项目规则
│   ├── jsonl_stream.rs     # JSONL 流
│   ├── nt_conn/            # 连接管理
│   ├── nt_router/          # 路由管理
│   └── nt_subagent/        # 子代理管理
```

### 移除后
```
neotrix-core/src/
├── l0_substrate/
│   └── nt_core_jsonl.rs        # jsonl_stream.rs
├── l1_action/
│   ├── nt_conn/                # nt_conn/
│   └── nt_router/              # nt_router/
├── l3_embodiment/
│   ├── nt_sandbox.rs           # sandbox.rs
│   ├── nt_sandboxed_shell.rs   # sandboxed_shell.rs
│   └── nt_shield_enforcer.rs   # shield_enforcer.rs
├── l6_meta/
│   ├── nt_approval.rs          # approval.rs
│   ├── nt_cost_tracker.rs      # cost_tracker.rs
│   ├── nt_permission_profiles.rs  # permission_profiles.rs
│   ├── nt_laws.rs              # laws.rs
│   └── nt_auto_orchestrator.rs # 自动编排器
```

## 2. 迁移路径

### 2.1 高优先级（核心功能）

| 原文件 | 目标位置 | 功能 | 迁移难度 |
|--------|----------|------|----------|
| `approval.rs` | `l6_meta/nt_approval.rs` | 审批系统 | 中 |
| `cost_tracker.rs` | `l6_meta/nt_cost_tracker.rs` | 成本追踪 | 低 |
| `permission_profiles.rs` | `l6_meta/nt_permission_profiles.rs` | 权限配置 | 中 |
| `laws.rs` | `l6_meta/nt_laws.rs` | 项目规则 | 低 |
| `sandbox.rs` | `l3_embodiment/nt_sandbox.rs` | 沙箱执行 | 中 |
| `shield_enforcer.rs` | `l3_embodiment/nt_shield_enforcer.rs` | 安全执行器 | 中 |

### 2.2 中优先级（基础设施）

| 原文件 | 目标位置 | 功能 | 迁移难度 |
|--------|----------|------|----------|
| `jsonl_stream.rs` | `l0_substrate/nt_core_jsonl.rs` | JSONL 流 | 低 |
| `sandboxed_shell.rs` | `l3_embodiment/nt_sandboxed_shell.rs` | 沙箱 shell | 低 |
| `nt_conn/` | `l1_action/nt_conn/` | 连接管理 | 中 |
| `nt_router/` | `l1_action/nt_router/` | 路由管理 | 中 |

### 2.3 低优先级（移除）

| 原文件 | 处理方式 | 原因 |
|--------|----------|------|
| `commands/` | 完全移除 | CLI 命令迁移到自动编排 |
| `nt_subagent/` | 完全移除 | 迁移到 AutoOrchestrator |
| `tui/` | 移除或保留为独立 crate | 终端 UI 非核心 |

## 3. 依赖分析

### 3.1 内部依赖（cli 内部模块相互依赖）
- `commands/` 依赖 `approval.rs`, `cost_tracker.rs`, `sandbox.rs`
- `tui/` 依赖 `commands/`

### 3.2 外部依赖（其他模块依赖 cli）
- `l5_cognition/nt_mind/` 依赖 `cli::approval`
- `l1_action/nt_io/` 依赖 `cli::sandbox`
- `l3_embodiment/nt_shield/` 依赖 `cli::shield_enforcer`

### 3.3 迁移策略
1. 先迁移被依赖少的模块
2. 更新所有 import 路径
3. 最后移除 commands/ 和 tui/

## 4. 执行步骤

### Phase 1: 迁移核心模块
```bash
# 1. 创建新模块文件
touch neotrix-core/src/l6_meta/nt_approval.rs
touch neotrix-core/src/l6_meta/nt_cost_tracker.rs
touch neotrix-core/src/l6_meta/nt_permission_profiles.rs
touch neotrix-core/src/l6_meta/nt_laws.rs
touch neotrix-core/src/l3_embodiment/nt_sandbox.rs
touch neotrix-core/src/l3_embodiment/nt_shield_enforcer.rs

# 2. 迁移代码（复制 + 修改 import）
# 3. 更新 mod.rs
# 4. 更新所有引用
```

### Phase 2: 迁移基础设施
```bash
# 1. 迁移 jsonl_stream.rs
# 2. 迁移 sandboxed_shell.rs
# 3. 迁移 nt_conn/
# 4. 迁移 nt_router/
```

### Phase 3: 移除 CLI
```bash
# 1. 移除 commands/ 目录
# 2. 移除 nt_subagent/ 目录
# 3. 移除 tui/ 目录
# 4. 更新 lib.rs
# 5. 验证编译
```

## 5. 验证清单

- [ ] 所有功能模块迁移完成
- [ ] 所有 import 路径更新
- [ ] 编译通过（cargo check）
- [ ] 单元测试通过
- [ ] 集成测试通过
- [ ] 文档更新

## 6. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 循环依赖 | 高 | 先分析依赖图，按顺序迁移 |
| 功能丢失 | 中 | 逐个模块迁移，保留测试 |
| 编译错误 | 低 | 分阶段迁移，每阶段验证 |
| 性能回归 | 低 | 迁移后运行基准测试 |
