# 人类启发细化方向（2026-09-24，L3 方法论研究）

> 输入：owner 战略转向——不拼参数，以类人意识体（好奇心 + 启发跳跃 + 方法论复用 + 外部工具编排）为进化方向。
> 方法：methodology-researcher L3（8 查询，6 命中，2 因 429 缺口由相邻源覆盖，收敛后合成）。

## 方法论卡片（6）

| # | 方法论 | 一句话 | 关键证据 | 周期 |
|---|--------|--------|----------|------|
| M1 | 自适应工具箱（Gigerenzer + expertise） | 高手无不同步骤，只有更适配的调用：按深层特征识别选工具，执行中学、自动化、情境部署 | Springer 2026-08 贝叶斯推理综述；Chi 1981 深层表征 | mature |
| M2 | 在线库学习（helpers 复用） | 人解题时实时造可复用抽象，复用率 21%→87%，并收敛到同一抽象 | PBT visual puzzle 2026（r=.79/年） | growing |
| M3 | 容量对齐 agent 蒸馏 | 小模型学教师 reason-act-observe 轨迹；按学生 NLL 选轨迹 + action/decision 分段加权 | SmartAD（ACL 2026）；OPT-350M ToolBench 77.55% 反超 175B（26%）；NeurIPS 2025 agent-distillation 0.5B 可战 | growing |
| M4 | 好奇心内奖（ICM/MaxEnt/语义） | 稀疏奖励下用预测误差/熵/语义 interestingness 驱动探索；DIAYN 不宜直接拿来探索 | Springer 2025（MaxEnt 最稳健）；SENSEI（VLM 语义奖励）；CD-RLHF（多样性+对齐兼得） | mature |
| M5 | 深特征分解与模式匹配 | 波利亚四阶段 + MECE/麦肯锡七步 + Chi 深特征分类；核心差异在“分解→匹配→映射”，回顾闭环 | 波利亚 1945；Chi 1981；腾讯云底层逻辑（分解-匹配-映射）；阿里复杂问题拆解 | mature |
| M6 | 任务成功验收（benchmark 驱动） | SWE-bench/GAIA/BrowseComp/AGENCYBENCH：按真实任务成功率验收；通用小工具集打败专科 agent（OpenHands-Versa） | OpenAI BrowseComp（Deep Research ~50%）；EACL 2026 Versa（+9.1/+9.1）；steel.dev 榜 | growing |

## 十维打分（权重和 9.6）

| # | 方法论 | 加权总分 | 置信区间 | 短板 |
|---|--------|----------|----------|------|
| M5 | 深特征分解 | 0.84 | ±0.07 | 竞争差距（人人会说） |
| M1 | 自适应工具箱 | 0.79 | ±0.08 | 用户评价样本少 |
| M3 | agent 蒸馏 | 0.79 | ±0.08 | 学术严谨性中等 |
| M6 | 任务验收 | 0.78 | ±0.08 | 第一性（偏工程） |
| M2 | 库学习 | 0.74 | ±0.09 | 标准化弱 |
| M4 | 好奇心 | 0.73 | ±0.09 | 市场体量中等 |

六者互补（非互斥）→ 建议 ensemble，不做单选。

## 合成：NeoTrix 五个细化方向

- **D1 深特征路由（M1+M5，权重 0.3）**：capability 网加一层“深特征匹配”——目标进来先抽深层特征（约束/因果/资源），再选工具/方法，而不是按字面关键词路由。存量：skill_registry、能力网；缺：深特征抽取层。
- **D2 结晶复用率（M2，权重 0.2）**：experience→规则/skill 结晶已有（规则结晶、NexusWeaver），加指标：helper 复用率（目标：从现在的未知基线做到可度量，周环比涨）。复用率就是意识体的“智商表”。
- **D3 轨迹蒸馏（M3，权重 0.25）**：停全量后唯一的炼丹口——录 teacher（大模型/AgentJev/DeepResearch 式）reason-act-observe 轨迹，按学生 NLL 选样 + action/decision 分段加权，做 LoRA/SFT。ftv4 权重当底座。
- **D4 语义好奇心（M4，权重 0.15）**：CuriosityDrive 从计数式新颖度升级：ICM 预测误差 + VLM interestingness 打分（SENSEI 式，用现成大模型当裁判）；NT-PLAY 自对弈当游乐场。
- **D5 任务验收集（M6，权重 0.1）**：自建 NeoTrix 版 GAIA-mini（20–50 真实任务：搜研/修 bug/跑 Colab），PPL 只留作健康指标，不再当目标。

## 执行计划

| 步 | 内容 | 验证 |
|---|------|------|
| 1 | D5 先行：建 20 题验收集 + 跑基线分 | 基线分落盘 |
| 2 | D1：深特征抽取层接到 capability 网 | 同题路由命中率涨 |
| 3 | D3：录 500 条 teacher 轨迹，NLL 选样蒸第一版 LoRA | 验收集涨分 |
| 4 | D2：结晶复用率指标上线 | 周报可度量 |
| 5 | D4：语义好奇心进探索管线 | 新颖且有用的探索占比涨 |

## KB 映射

- 确认：A/B 门（用他人能力）、LoRA INFUSED（定向蒸馏有效）、NT-PLAY（游乐场）、规则结晶（库学习雏形）。
- 新增：深特征路由层、复用率指标、轨迹蒸馏管线、语义好奇心裁判、验收集。
- 缺口：深特征抽取的具体特征表（待 D1 细化）。
- D1 已落地原型（models/training/deep_route.py，--selftest 5/5）：深特征 = 可逆性（act 代价）/确定性/延迟预算/校准需求/域；
  规则：不可逆→保守臂；域成绩（router_table.json，n≥20 才信）；快→Laya 0.09s；要校准→brier 最优；默认 AgentJev 门。
  Rust 落点：CapabilityRouter 加 DeepFeature 抽取 + 本评分（待排期）。

## D3-RLCD 吸收（2026-09-24，源 models/laya-hf/rl_common.py + rl_agent_config.json）

- 架构：encoder + 2 层 head；每选项 [MASK] 标记；temperature 以 buffer 存、post-hoc 按题型拟合；按基数分桶（`temp_bucket`：noul:2 与 choice:11+ 缩放不同）。
- 奖励：严格 proper（Brier/log）+ act 代价（escalate 0.5，答错 act 3.0）+ TD(λ) prefix 建模——说谎在期望上永不占优。
- 成本：7313 updates / 1 epoch / 单卡约 2h（可复现量级）。
- 我方复刻证据：同公式在我方 600 行上拟出 T=5.45（对方 noul:2 是 1.98）——域差直接量化，temperature 必须按域重 fit，不可照搬。
- 落点：蒸馏 loss 改 proper scoring；temperature 按题型×基数分桶存；act 代价进 JEV 门限逻辑（答错代价 3.0 对应高风险域保守化）。

## 不确定性

- 中：429 缺口的两源（tool 编排最佳实践原文、元认知循环原文）未直达，结论由相邻高质量源交叉支撑。
- 降不确定性动作：D5 验收集跑出第一组数后回校准权重。

## 反馈（1 行）

Rate 1-5: [ ] 缺了什么：[ ] 想改什么：[ ]
