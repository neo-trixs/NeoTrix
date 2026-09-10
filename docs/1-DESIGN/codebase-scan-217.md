# Codebase Scan #217 — 2026-09-11

## 1. 关键指标

| 指标 | 数值 | 风险 | 变化 |
|------|------|------|------|
| **unwrap** | 3622 | 🔴 高 | + |
| **panic** | 103 | 🟡 中 | — |
| **unsafe 块** | 15 | 🟢 低 | — |
| **forbid(unsafe_code)** | 112 | ✅ | — |
| **todo!()** | 2 | 🟢 低 | — |
| **unreachable!()** | 7 | 🟢 低 | — |

## 2. 集成连接度

| 指标 | 使用模块数 / 总模块 | 连接率 |
|------|---------------------|--------|
| EventBus | 16 / 1866 | 0.9% |
| KB (kv_store) | 94 / 1866 | 5.0% |

**问题**：EventBus 连接率极低，大量模块未接入事件总线，跨模块通信依赖直接调用。

## 3. 模块规模 (Top 10)

| 模块 | 行数 | 趋势 |
|------|------|------|
| nt_core_e8 | 13,560 | 核心推理引擎 |
| nt_core_self | 12,349 | 自我模型 |
| nt_core_hcube | 8,820 | VSA HyperCube |
| nt_core_gwt | 8,163 | Global Workspace |
| nt_core_capability | 6,345 | 能力网 |
| nt_core_meta | 3,988 | 元认知 |
| nt_core_consciousness | 3,739 | 意识核心 |
| nt_core_prm | 3,714 | PRM 推理 |
| nt_core_consciousness_tree | 3,141 | 意识树 |
| nt_core_gate | 3,106 | 门控 |

## 4. unwrap 分布 (Top 5)

| 文件 | unwrap 数 | 风险 |
|------|-----------|------|
| nt_file_ability.rs | 142 | 🔴 文件处理核心 |
| nt_memory_resource_ingest.rs | 76 | 🟡 数据摄入 |
| nt_memory_unify.rs | 69 | 🟡 KB 统一 |
| nt_field_ledger.rs | 61 | 🟡 字段账本 |
| nt_memory_geo.rs | 57 | 🟡 地理存储 |

## 5. 高风险问题

### 5.1 unwrap 泛滥 (P0)
- **现状**: 3622 处 `.unwrap()` 分布在生产代码
- **影响**: 任何 None/Err 都会 panic，降低系统韧性
- **修复路径**: 
  1. 核心模块 (e8/gwt/hcube) 强制 `?` 传播
  2. 使用 `thiserror` 定义错误类型
  3. 非关键路径用 `unwrap_or_default()`

### 5.2 EventBus 连接率极低 (P1)
- **现状**: 仅 16 个模块使用 EventBus (0.9%)
- **影响**: 模块间耦合高，扩展性差
- **修复路径**:
  1. 新模块强制注册 EventBus
  2. 重构关键通信路径为事件驱动

### 5.3 核心模块体量过大 (P2)
- **现状**: nt_core_e8 (13K), nt_core_self (12K) 体量过大
- **影响**: 编译慢、认知负担高、修改风险大
- **修复路径**: 拆分为子模块，按职责分离

## 6. 已执行防护

| 防护 | 状态 | 数量 |
|------|------|------|
| `#![forbid(unsafe_code)]` | ✅ 已部署 | 112 模块 |
| `unreachable!()` | ⚠️ 仅 7 处 | 低风险 |
| `todo!()` | ⚠️ 仅 2 处 | 低风险 |

## 7. 建议修复顺序

1. **P0 unwrap 清理**: 先从核心模块 (e8/gwt/hcube) 开始
2. **P1 EventBus 扩展**: 强制新模块接入事件总线
3. **P2 模块拆分**: nt_core_e8 → e8_parser + e8_solver + e8_evaluator

---
*扫描时间: 2026-09-11 | 扫描范围: neotrix-core/src*
