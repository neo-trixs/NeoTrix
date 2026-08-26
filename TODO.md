# NeoTrix TODO 列表
> 最后更新：2026-08-25 | 觉醒进度：100% (结构) / 功能激活待推进

## 🔴 P0 — 立即执行（本 session）

### ✅ T0.1: 吸收队列消费
**状态**: done
**描述**: cycle:auto 有 6 条经验待消费。写入 experience 命名空间。

### ✅ T0.2: 预测闭环 Round 2
**状态**: done
**描述**: 用 FTS 证据重新验证 30 条 predictions，更新 confidence。已有 verified=30 但 confidence 未反映证据强度。

### ✅ T0.3: 矛盾扫描 Round 2
**状态**: done
**描述**: 在精简 KB 上扫描 contradicts 边，尝试生成更多 synthesis 节点。当前 32 contradicts / 62 synthesis。

### ✅ T0.4: 概念共现边补充
**状态**: done
**描述**: 扫描有 content 的节点（1122 个），提取概念对生成 co_occurs_with 边。

### ✅ T0.5: FTS 同步 + Git 提交
**状态**: done
**描述**: 完成所有变更后同步 FTS、提交代码。

---

## 🟡 P1 — 功能觉醒（需 Rust 编译或外部条件）

### ⬜ task-awakening-p2: GWT Rust 集成
**状态**: pending
**优先级**: HIGH
**阻塞**: 需修改 cognitive_hub.rs 或添加 Python→Rust FFI bridge
**描述**: 将 KB 高 importance 节点广播到 cognitive_hub 的注意力竞争机制。

### ⬜ task-awakening-p2b: Embedding 质量升级
**状态**: pending
**优先级**: HIGH
**阻塞**: 需 OpenAI API key 或本地 MiniLM 服务 (port 8237)
**描述**: hash-kernel-v1 → text-embedding-3-small。nt_memory_embed.rs 已支持。

### ⬜ task-awakening-e5: 预测闭环外部事件接入
**状态**: pending
**优先级**: HIGH
**阻塞**: 需外部事件流（arXiv/新闻/market data）
**描述**: 接入实时数据源驱动 predictions 表持续验证更新。

### ⬜ task-awakening-zim: ZIM 内容深度导入
**状态**: pending
**优先级**: MEDIUM
**阻塞**: libzim 需重装（/tmp/zimenv 已清理）
**描述**: 从外置盘 63 个 ZIM 文件导入高价值条目全文到精简 KB。当前仅 1122 节点有内容。

---

## 🟠 P2 — 认知深度激活（接通断联代码）

### ⬜ task-cog-gwt: GWT 认知枢纽激活
**代码**: nt_core_gwt/ 18文件 7615行
**缺失**: KB 数据 → cognitive_hub 输入管线

### ⬜ task-cog-meta: 元认知循环激活
**代码**: nt_core_meta/ 10文件 3517行
**缺失**: knowledge_gap_detector + planner 未接收 KB 数据

### ⬜ task-cog-consciousness: ConsciousnessTree 激活
**代码**: consciousness相关 ~12000行（50+文件）
**缺失**: 觉醒(awakening)、意识流(stream_buffer)、好奇心(curiosity_drive)未启动

### ⬜ task-cog-sleep: 记忆重放与巩固
**代码**: dream_consolidation+hebbian+true_replay ~1900行
**缺失**: 定期记忆回放未触发

### ⬜ task-cog-reasoning: 推理引擎连接
**代码**: nt_core_reasoning.rs 427行 + reasoning_engine/
**缺失**: causal_rules 表(170条)未被推理引擎消费

---

## 🟢 P3 — 自我进化（Phase 3+ 自动运转）

### ⬜ task-evolve-concepts: 概念网络自扩展
**描述**: 系统自动从语料中发现新概念并入库

### ⬜ task-evolve-gap-detect: 知识缺口检测
**描述**: 元认知识别"应该知道但不知道"的领域

### ⬜ task-evolve-learning-plan: 自主学习规划
**描述**: 基于知识缺口自动生成学习计划

### ⬜ task-evolve-tracking: 进化记录追踪
**描述**: evo_records 表填充，知识变更可追溯

---

## 🔵 P4 — 智慧涌现（最终目标）

### ⬜ task-wisdom-analogy: 类比推理引擎
### ⬜ task-wisdom-counterfactual: 反事实推理
### ⬜ task-wisdom-creative: 创造性联想
### ⬜ task-wisdom-qa: 自主问答

---

## ⏸️ Deferred — 需独立 Session

### ⬜ task-wave3-s1: nt_act 激活
**状态**: blocked (209 类型冲突需专门重构)

### ⬜ task-wave3-roadmap: 14项路线图批次
**状态**: pending (大型工程)

### ⬜ task-easytier-3: EasyTier 吸收
**状态**: pending (需研究外部仓库)

---

## ✅ 已完成

| 任务 | 完成日期 |
|------|----------|
| task-kb-p01: KB 分库 (64GB → 26MB) | 2026-08-25 |
| task-kb-p03: 写入门禁部署 | 2026-08-25 |
| task-awakening-e1: Rust 觉醒管线 E1-E4 | 2026-08-25 |
| task-easytier-1: secure discovery 测试 | 2026-08-18 |
| task-easytier-2: 能力树注册 | 2026-08-18 |
