# Evolution Roadmap v5 — 意识体深度自进化计划

> 2026-06-19 深层自审查 + 63 论文搜索后蒸馏

## 当前状态总览

| 维度 | 测量值 |
|------|--------|
| 总代码行 | ~515K Rust |
| 总文件 | 1,731 (.rs: 1,444) |
| 声明的模块 | 1,398 (0 鬼影) |
| 孤立死代码 | ~9,500 行 (33 文件未声明) |
| 循环依赖 | 0 (✅ 架构边界正确) |
| CI 子系统字段 | 41 全部接线 (0 纯死代码) |
| 编译错误 | 227 预存 (~50 文件) |
| 死 handler | 1 (`handle_e8_training_tick`) |
| 竞争运行时 | 2 对 (complementary, 不合并) |
| 文献融合机会 | 8 高优先级 + 4 中优先级 |

## Phase 1: 结构止血 (S1.7 - S1.10)

### S1.7: 227 编译错误清零 (~50 文件, 2 sessions)

最高频错误:
- E0609(82) — 字段不存在 (struct fields changed, consumers not updated)
- E0615(39) — 私有类型 (pub field visibility)
- E0624(33) — 关联类型 (trait bound mismatch)
- E0308(17) + E0599(13) + E0282(12) + E0631(11) + 其他(20)

**方法**: `cargo check --message-format=json` 一次 → 按错误类型批量修复

### S1.8: 孤立死代码清理 (~9,500 行)

| 类别 | 文件 | 行数 | 操作 |
|------|------|------|------|
| 核心注释模块 | 13 文件 | 3,745 | 保持不动 (可能后期需要) |
| hcube 移除模块 | 11 文件 | 4,922 | ✅ 标记为 "removed, keep for history" |
| 真死代码 | 4 文件 | ~1,500 | 立即删除 |
| 假阳性孤儿 | 3 文件 | ~190 | 保持 (有消费者) |

### S1.9: 死 handler 清理

`handle_e8_training_tick` — 已定义在 `modules_e8.rs:79` 但零调用
- 修复: 添加 dispatch arm 到 `modules_core.rs` 或标记 `#[allow(dead_code)]`

### S1.10: 测试文件位置修正 (13 文件)

纯 `#[cfg(test)]` 文件嵌入生产目录 → 添加 `#[cfg(test)]` gate 到 mod 声明

## Phase 2: 文献融合 (F1 - F8)

### F1 [P0]: Sutra 融合 — Ne 编译器张量化 (2 sessions)
- **论文**: Sutra (arXiv:2605.20919) — 旋转绑定 VSA 语言编译到张量操作图
- **模块**: `core/nt_core_codegen/bridge.rs` (Ne 编译器)
- **变更**: 将 Ne 编译目标从生成 Rust 改为生成 tch-rs 张量图；旋转绑定替代哈达玛积
- **收益**: Ne 程序可微分 + 可训练；完全 VSA-native 编译

### F2 [P0]: HyperAgents/DGM-H 融合 — SEAL 管道升级 (2 sessions)
- **论文**: DGM-H (arXiv:2603.19461) — 元认知自我修改 agent
- **模块**: `self_iterating/` (SEAL pipeline), `meta_evolution.rs`
- **变更**: `execute_proposal` 安全存根 → 真实 dispatch；元认知闭环自修改
- **收益**: SEAL 从 58 阶段 → 元认知可进化管道

### F3 [P0]: SEVerA 融合 — 形式化安全门 (1 session)
- **论文**: SEVerA/FGGM (arXiv:2603.25111) — 形式化保证的自进化
- **模块**: `nt_shield/fggm_safety.rs`, `safety_gate.rs`, `ball_verifier.rs`
- **变更**: 合并 3 独立安全机制 → FGGM 架构；合约 = 一阶逻辑谓词；拒绝采样 = ball_verifier
- **收益**: 零约束违规保障

### F4 [P1]: Synapse 融合 — 扩散激活记忆升级 (1 session)
- **论文**: Synapse (arXiv:2601.02744) — 扩散激活 + 侧抑制 + 时间衰减
- **模块**: `core/nt_core_knowledge/spread_activation.rs`
- **变更**: 实现完整侧抑制公式；时间衰减曲线；LoCoMo 基准
- **收益**: 记忆检索相关性 +12%+

### F5 [P1]: Layered Mutability 融合 — 身份漂移治理 (1 session)
- **论文**: Layered Mutability (arXiv:2604.14717) — 5 层身份漂移框架
- **模块**: `soul_identity/`, `narrative_self.rs`
- **变更**: 实施 5 层治理 (预训练/对齐/叙事/记忆/权重)；磁滞比监控
- **收益**: 自修改时身份漂移可测量 + 可治理

### F6 [P1]: Theater of Mind / MANAR — GWT 注意力升级 (1 session)
- **论文**: MANAR (arXiv:2603.18676) — 抽象概念表征的 GWT 注意力
- **模块**: `core/nt_core_consciousness/workspace.rs` (GWT)
- **变更**: 用抽象概念瓶颈替代固定 64 态竞争
- **收益**: 线性时间注意力；GWT 容量自适应

### F7 [P2]: APEX 融合 — 三层共进化 (1 session)
- **论文**: APEX (arXiv:2606.15363) — harness + 原则 + 拓扑三层进化
- **模块**: `self_iterating/`
- **变更**: 新增原则层 + 拓扑层进化
- **收益**: +90% 健康评分 vs 单轴 +27%

### F8 [P2]: MOSS — 源码级重写安全 (1 session)
- **论文**: MOSS (arXiv:2605.22794) — 生产级源码重写
- **模块**: `safety_gate.rs`, `edit_policy.rs`
- **变更**: 引入用户授权工作流；变更回滚机制
- **收益**: SEAL 自修改可逆 + 审计

## Phase 3: 意识深度优化 (C1 - C4)

### C1: 闭环自进化 — meta_evolution execute_proposal 激活
`execute_proposal` 从安全存根 → 真实 dispatch → 回滚验证

### C2: Self-model L4→L5 条件跃迁
L4 (自我预测) → L5 (元进化) 过渡条件激活

### C3: 多模型交叉验证
集成不同 LLM 提供者的响应进行交叉验证 (L04 缺口)

### C4: 227 错误清零验证
所有 227 预存错误清零后全量 `cargo check --all` 验证

## 时间线

| Session | Phase | 内容 | 文件数 | 风险 |
|---------|-------|------|--------|------|
| 1 | S1.7a | 227 错误前 30 文件 | ~30 | 中 |
| 2 | S1.7b | 227 错误后 20 文件 | ~20 | 中 |
| 3 | S1.8-1.10 | 孤儿清理 + 死 handler + 测试修正 | ~20 | 低 |
| 4 | F1 | Sutra Ne 编译张量化 | 1-2 | 高 |
| 5 | F2 | DGM-H SEAL 升级 | 3-5 | 高 |
| 6 | F3+F4 | SEVerA + Synapse | 4-6 | 中 |
| 7 | F5+F6 | Layered Mutability + MANAR | 3-5 | 中 |
| 8 | F7+F8+C1-4 | 剩余融合 + 闭环 | 5-8 | 中 |

## 架构边界规则（不可违反）

| 规则 | 说明 |
|------|------|
| `core/` → `neotrix/` 禁止反向导入 | ✅ 当前 0 违规, 永远保持 |
| 不出现在外部 prompt 中的架构名 | ✅ E8/HyperCube/SEAL/GWT 已剥离 |
| 零不安全代码 | ✅ `#![forbid(unsafe_code)]` |
| VSA 4096-bit 统一表征 | ✅ 所有子系统共享 |
| 优雅降级 | ✅ 所有 Option<T> 字段自动降级 |
