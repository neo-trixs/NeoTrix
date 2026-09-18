# 旧架构 → 晶体辐射架构 迁移分析

> **日期**: 2026-09-18
> **目标**: 量化迁移工作量，识别阻塞点

---

## 一、架构对比

### 1.1 旧架构 (L0-L6 层级)

| 层 | 文件数 | 主要模块 |
|----|--------|---------|
| L0 Substrate | 15 | error, time, cache, self_test |
| L1 Action | 562 | nt_act, nt_io, nt_memory |
| L2 Perception | 289 | nt_world, nt_sense, knowledge |
| L3 Embodiment | 201 | nt_shield, nt_physical |
| L4 Emotion | 11 | nt_feel |
| L5 Cognition | 621 | nt_core, nt_mind |
| L6 Meta | 144 | nt_meta, nt_repair, nt_nexus |
| **总计** | **1843** | |

### 1.2 新架构 (Crystal-Radiant)

| 组件 | 文件数 | 状态 |
|------|--------|------|
| Crystal Core | 28 | ✅ 已实现 |
| NT-MEMORY 模块 | 1 | ✅ 已实现 |
| NT-PERCEPTION 模块 | 1 | ✅ 已实现 |
| NT-ACT 模块 | 1 | ✅ 已实现 |
| NT-FEEL 模块 | 1 | ✅ 已实现 |
| NT-SHIELD 模块 | 1 | ✅ 已实现 |
| NT-META 模块 | 1 | ✅ 已实现 |
| **NT-WORLD 模块** | 0 | ❌ 未实现 |
| **NT-MIND 模块** | 0 | ❌ 未实现 |
| **NT-IO 模块** | 0 | ❌ 未实现 |
| **总计** | **35** | |

---

## 二、迁移映射

### 2.1 完全可迁移 (无阻塞)

| 旧模块 | → 新组件 | 文件数 | 难度 |
|--------|---------|--------|------|
| nt_core::consciousness | CrystalConsciousness | ~50 | ⭐⭐ |
| nt_core::e8 | CrystalConsciousness (内嵌) | ~30 | ⭐⭐ |
| nt_core::hypercube | CrystalConsciousness (内嵌) | ~20 | ⭐⭐ |
| nt_core::gwt | CrystalConsciousness (内嵌) | ~25 | ⭐⭐ |
| nt_core::capability_tree | CrystalConsciousness (内嵌) | ~15 | ⭐ |
| nt_core::capability_registry | CrystalConsciousness (内嵌) | ~10 | ⭐ |
| nt_core::self_model | CrystalConsciousness (内嵌) | ~20 | ⭐⭐ |
| nt_core::heartbeat | CrystalConsciousness (内嵌) | ~5 | ⭐ |
| nt_meta | NT-META | ~40 | ⭐⭐ |
| nt_repair | NT-META | ~30 | ⭐⭐ |
| nt_nexus | NT-META | ~20 | ⭐⭐ |
| nt_feel | NT-FEEL | ~11 | ⭐ |
| nt_physical | NT-SHIELD + NT-ACT | ~50 | ⭐⭐⭐ |
| **小计** | | **~326** | |

### 2.2 需要适配器迁移 (有接口依赖)

| 旧模块 | → 新组件 | 文件数 | 阻塞点 |
|--------|---------|--------|--------|
| nt_memory | NT-MEMORY | ~120 | 依赖 L0/L2 类型 |
| nt_act | NT-ACT | ~200 | 依赖 L5 推理 |
| nt_shield | NT-SHIELD | ~201 | 依赖 L6 SelfTest |
| **小计** | | **~521** | |

### 2.3 需要重构迁移 (跨层耦合严重)

| 旧模块 | → 新组件 | 文件数 | 阻塞点 |
|--------|---------|--------|--------|
| nt_world | NT-WORLD | ~289 | 依赖 L5 能力向量 |
| nt_mind | NT-MIND | ~400 | 依赖 L6 自我模型 |
| nt_io | NT-IO | ~300 | 依赖 L1/L5 接口 |
| **小计** | | **~989** | |

---

## 三、迁移可行性评估

### 3.1 可迁移率

```
完全可迁移:     326 / 1843 = 17.7%  ✅
需适配器:       521 / 1843 = 28.3%  🟡
需重构:         989 / 1843 = 53.7%  🔴
```

### 3.2 按辐射臂统计

| 辐射臂 | 可迁移文件 | 阻塞文件 | 迁移率 |
|--------|-----------|---------|--------|
| NT-MEMORY | 80 | 40 | 67% |
| NT-ACT | 100 | 100 | 50% |
| NT-SHIELD | 100 | 101 | 50% |
| NT-WORLD | 0 | 289 | 0% |
| NT-MIND | 0 | 400 | 0% |
| NT-IO | 0 | 300 | 0% |
| NT-FEEL | 11 | 0 | 100% |
| NT-META | 90 | 0 | 100% |
| Crystal Core | 145 | 0 | 100% |

---

## 四、迁移任务清单

### Phase 1: Crystal Core 完善 (3 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T1.1 | E8 状态机迁入 CrystalConsciousness | 30 |
| T1.2 | HyperCube VSA 迁入 CrystalConsciousness | 20 |
| T1.3 | GWT 注意力路由迁入 CrystalConsciousness | 25 |
| T1.4 | CapabilityTree 迁入 CrystalConsciousness | 15 |
| T1.5 | SelfModel 迁入 CrystalConsciousness | 20 |
| **小计** | | **110** |

### Phase 2: 无阻塞辐射臂 (5 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T2.1 | NT-FEEL 完善 (VAD + 奖励塑形) | 11 |
| T2.2 | NT-META 完善 (Ω 操作 + AutoMem) | 90 |
| T2.3 | NT-SHIELD 适配器 (Override 机制) | 100 |
| T2.4 | NT-MEMORY 适配器 (向量/图/键值) | 80 |
| T2.5 | NT-ACT 适配器 (工具调用) | 100 |
| **小计** | | **381** |

### Phase 3: 高难度辐射臂 (10 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T3.1 | NT-WORLD 重构 (感知 + 爬虫) | 289 |
| T3.2 | NT-MIND 重构 (推理 + 进化) | 400 |
| T3.3 | NT-IO 重构 (CLI + Tauri) | 300 |
| **小计** | | **989** |

### Phase 4: CTM 通信 + 治理 (5 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T4.1 | CTM 10步循环完善 | 50 |
| T4.2 | Governance 层实现 | 30 |
| T4.3 | 辐射臂隔离验证 | 20 |
| **小计** | | **100** |

---

## 五、总结

### 5.1 迁移统计

| 指标 | 值 |
|------|-----|
| 旧架构总文件 | 1843 |
| 已迁移到晶体 | 35 (1.9%) |
| 完全可迁移 | 326 (17.7%) |
| 需适配器 | 521 (28.3%) |
| 需重构 | 989 (53.7%) |
| **预计总工期** | **23 天** |

### 5.2 关键阻塞

1. **NT-WORLD (289 文件)** — 依赖 L5 能力向量，需要先完成 Crystal Core
2. **NT-MIND (400 文件)** — 依赖 L6 自我模型，需要先完成 Crystal Core
3. **NT-IO (300 文件)** — 依赖 L1/L5 接口，需要先完成辐射臂

### 5.3 建议

1. **先完成 Crystal Core** — 这是所有辐射臂的基础
2. **先迁移无阻塞模块** — NT-FEEL, NT-META 可立即迁移
3. **高难度模块用适配器** — NT-WORLD/NT-MIND/NT-IO 先写适配器，后续重构
4. **CTM 通信是关键** — 所有辐射臂必须通过 CTM 通信，这是架构核心
