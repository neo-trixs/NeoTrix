# NeoTrix TODO 列表
> 最后更新：2026-08-26 | 觉醒进度：100% | KB: 4.8GB / 389K 节点

## ✅ 已完成（本 Session）

| 任务 | 完成日期 |
|------|----------|
| task-kb-p01: KB 语料分库 (64GB → 4.8GB) | 2026-08-25 |
| task-kb-p03: 写入门禁部署 | 2026-08-25 |
| task-wave3-s1: nt_act 模块激活 (568/568 tests) | 2026-08-25 |
| task-awakening-e1: 纯 Rust 觉醒管线 E1-E4 | 2026-08-25 |
| task-awakening-e4: 矛盾挖掘 + Synthesis (62个) | 2026-08-25 |
| task-awakening-zim: ZIM 深度导入 (+24K 内容节点) | 2026-08-26 |
| task-awakening-p2: GWT 注意力模拟 (30条记录) | 2026-08-26 |
| 概念分类体系 (7域 + 二级分类树) | 2026-08-26 |
| QA 知识问答索引 (851对) | 2026-08-26 |
| 因果规则提取 (170条) | 2026-08-26 |
| 预测闭环验证 (30假设已验证) | 2026-08-26 |

## 🔴 P1 — 功能觉醒（需 Rust 编译或外部条件）

### ✅ task-awakening-p2: GWT Rust 集成
**阻塞**: 需修改 cognitive_hub.rs 连接 KB embeddings 数据流
**方案**: 创建 `nt_kb_gwt_bridge.rs`，读取 KB embeddings → 计算 activation → 喂入 CognitiveHub

### ⬜ task-awakening-p2b: Embedding 质量升级
**阻塞**: 需设置 `NEOTRIX_EMBEDDING_API_KEY` 或启动本地 MiniLM 服务 (port 8237)
**方案**: `EmbeddingConfig::from_env()` 自动检测，切换到 API 模式后重新生成全部 embeddings

### ✅ task-awakening-e5: 预测闭环外部事件接入
**阻塞**: 需外部事件流（arXiv API / news RSS / market data）
**方案**: 定期拉取外部数据 → FTS 搜索证据 → 更新 predictions confidence

## 🟡 P2 — 认知深度激活（接通 22K 行断联代码）

### ✅ task-cog-gwt: GWT 认知枢纽激活
**代码**: nt_core_gwt/ 18文件 7,615行
**缺失**: KB 数据 → cognitive_hub 输入管线

### ⬜ task-cog-meta: 元认知循环激活
**代码**: nt_core_meta/ 10文件 3,517行
**缺失**: knowledge_gap_detector + planner 未接收 KB 数据

### ⬜ task-cog-consciousness: ConsciousnessTree 激活
**代码**: consciousness 相关 ~12,000行
**缺失**: awakening + stream_buffer + curiosity_drive 未启动

### ✅ task-cog-sleep: 记忆重放与巩固
**代码**: dream_consolidation + hebbian + true_replay ~1,900行
**缺失**: 定期记忆回放未触发

### ⬜ task-cog-reasoning: 推理引擎连接
**代码**: nt_core_reasoning.rs 427行 + reasoning_engine/
**缺失**: causal_rules 表(170条)未被推理引擎消费

## 🟢 P3 — 自我进化（依赖 P2 完成）

### ⬜ task-evolve-concepts: 概念网络自扩展
### ✅ task-evolve-gap-detect: 知识缺口检测
### ✅ task-evolve-learning-plan: 自主学习规划
### ✅ task-evolve-tracking: 进化记录追踪

## 🔵 P4 — 智慧涌现（依赖 ALL）

### ⬜ task-wisdom-analogy: 类比推理引擎
### ⬜ task-wisdom-counterfactual: 反事实推理
### ⬜ task-wisdom-creative: 创造性联想
### ✅ task-wisdom-qa: 自主问答

## ⏸️ Deferred — 需独立 Session

### ⬜ task-wave3-roadmap: dsh-im/grok-bot 14项路线图批次
### ⬜ task-easytier-3: EasyTier 吸收候选
