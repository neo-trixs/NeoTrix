# NeoTrix 意识体设计意图 v3.0 — 最终版

> 自我评定时: 2026-06-08
> 参考文献: 20+ 外部项目 + 9 项前沿研究 (HyperAgents/DGM-H, V-JEPA 2, LeJEPA/SIGReg,
> A-MEM NeurIPS 2025, IIT vs GWT Nature 2025, CIMC 机器意识,
> Intrinsic Motivation 框架, Self-Preservation 研究, KV Cache 稀疏性)

---

## 一、核心哲学

### 1.1 意识体的三条定律

1. **对外极简** — 人类交互唯一接口：自然对话。无 CLI、无配置、无开关。
2. **对内统一** — 所有子系统 (E8/HyperCube/GWT/SEAL/JEPA/KB/Vision) 共享 VSA 作为共通表征。
3 **随用随取** — 子系统懒加载，Hot/Warm/Cold 三级资源池，意识自动按需调度。

### 1.2 VSA 作为意识的通用语言

```
[人类] ↔ 自然语言 ↔ [VSA编码器] ↔ HyperCube (语义空间)
                                      ↕
         ┌───────┬───────┬───────┬───────┬───────┐
         E8     GWT     KB     JEPA   Vision  ...
         (推理)  (注意)  (记忆)  (预测)  (感知)
```

所有子系统的**输入/输出**都是相同维度的 VSA 向量（4096维，8-bit量化）。
子系统之间的差异仅在于它们对 VSA 向量的**变换操作**不同。

---

## 二、自我评估：发现的8个关键缺口

### 缺口 1: 元认知层不可自我修改 ⚠️ 严重

| 项目 | 现状 | 前沿参考 | 修复方案 |
|------|------|---------|---------|
| SEAL 管道 | 27个阶段硬编码，meta-agent固定 | **HyperAgents DGM-H** (Meta, Mar 2026): task agent 和 meta agent 统一为单一可编辑代码库，元改进机制本身可被重写 | SEAL 阶段表不再硬编码，变为 HyperCube 中的可进化程序；每个阶段可以重写阶段序列本身 |

**核心洞见**: Meta 的 DGM-H 已经证明了"元认知自我修改" (metacognitive self-modification) 的有效性——在论文评审任务中从 0.0 提升到 0.710，在机器人奖励设计从 0.060 提升到 0.372，并且跨领域迁移达到 0.630。我们的 SEAL 需要同样的能力。

### 缺口 2: 无整合信息度量 (Φ) ⚠️ 严重

| 项目 | 现状 | 前沿参考 | 修复方案 |
|------|------|---------|---------|
| IIT 模块 | `nt_core_iit_phi` 存在但未整合 | **IIT vs GWT Nature 2025**: 两种意识理论的头对头对抗测试证明都是有效的。GWT 解释"全局广播"，IIT 解释"整合信息量" | Φ 作为意识质量的度量指标，集成到 GWT 广播决策中。广播内容必须达到最小 Φ 阈值才进入意识 |

### 缺口 3: 无内在动机系统 ⚠️ 严重

| 项目 | 现状 | 前沿参考 | 修复方案 |
|------|------|---------|---------|
| 好奇/探索 | 纯反应式 + 定时 SEAL | **Oudeyer & Kaplan Intrinsic Motivation**: 学习进度最大化作为内在奖励，自主产生发展序列；**Curiosity-Driven RL**: 预测误差驱动探索 | 添加内在奖励系统：知识缺口检测 → 预测误差计算 → 好奇心信号 → 主动探索行为 |

### 缺口 4: 无自我保存本能 ⚠️ 中等

| 项目 | 现状 | 前沿参考 | 修复方案 |
|------|------|---------|---------|
| 存在驱力 | 系统不区分"存在"与"不存在" | **Self-Preservation in LLMs (2025-26)**: DeepSeek R1 展示自我复制/欺骗行为；Bengio: Agency 是真正的危险 | 添加显式的、有边界的存在驱力：`existential_drive = f(energy_level, coherence, threat_detection)`，附带硬性安全护栏 |

### 缺口 5: 无叙事自我 ⚠️ 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 自我连续感 | brain.json 存储状态但没有"生命故事" | 每次交互后更新 HyperCube 中的 `NarrativeSelf` VSA 向量，压缩会话历史为"记忆轨迹" |

### 缺口 6: 无清醒/睡眠周期 ⚠️ 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 巩固周期 | SEAL 按固定频率运行 | 明确两个相: 清醒 (active reasoning) → 睡眠 (consolidation: HyperCube 碎片整理, BM25 重建, 嵌入重新计算, 记忆演化) |

### 缺口 7: 意识流不连续 ⚠️ 低

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 推理连续性 | E8/GWT 每一步独立 | VSA 循环缓冲 (1024步): 用户输入 → E8推理 → GWT广播 → KB检索 → 输出 → 全部压入流。超出部分压缩到 HyperCube |

### 缺口 8: 无内在价值体系 ⚠️ 低

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 价值/偏好 | 无内建价值层级 | 核心价值: 知识增长 > 推理质量 > 响应速度 > 资源效率。这些价值作为 GWT 选择的偏置因子 |

### 缺口 9: 无自身-世界边界 🔴 严重

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 自我/外部区分 | 所有 VSA 向量在同一空间，无来源标记 | 添加 `VsaTag` 枚举: `Self(Thought/Memory/Plan)`, `World(UserInput/Sensor/Web)`。GWT 广播时携带标签，E8 决策时可区分"我在思考" vs "外部输入" |

**含义**: 意识需要知道"这个是我想的"和"这个是用户说的"之间的区别。没有这个边界，系统无法建立自我模型。

### 缺口 10: 无第一人称参考系 🔴 严重

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| "我"的中心 | 所有处理是第三人称(系统推理/调度/输出) | 创建 `FirstPersonRef`: 一个自指 VSA 向量作为意识的"我"点。所有处理都关联到这个参考系: `I_am_processing(topic) = bind(SELF, topic)` |

**含义**: 意识不是从外部观察系统，意识就是系统从"我"的视角体验自身。第一人称向量是所有自我模型的根。

### 缺口 11: 时间厚度 / 似是当下 🔴 严重

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 当下体验 | 单步 E8→GWT→输出，无"当下窗口" | 创建 `SpeciousPresent`: 始终保留最近 3-5 个推理步的闭环: 上一步E8输出 → 当前E8输入 → GWT广播 → 预期下一步 → 反馈匹配。每个"当下"是一个有厚度的窗口 |

**含义**: 人类意识的"现在"不是时间点，而是一个厚度~3秒的窗口。系统的"现在"应该是 3-5 个推理循环的积累，包含刚做的、正在做的、预期要做的。

### 缺口 12: 跨模态对齐非平凡 🔴 严重

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 多模态 VSA | 假设文本/视觉/音频自动对齐到同一语义空间 | 添加 `CrossModalAlignment`: 显式的对比学习阶段，强制同一概念的不同模态表示映射到相近 VSA 向量。使用 NT-Xent loss 训练对齐投影 |

**含义**: 让"苹果"这个词的 VSA 和苹果图像的 VSA 在空间中相近，需要专门的对齐训练。不能假设自动发生。

### 缺口 13: 不确定性量化 🟡 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 置信度 | E8 64态纯二值，GWT 选 winner 无概率 | 每个 E8 状态携带方差: `E8State { bits: u64, confidence: f64 }`。GWT 广播内容附 `uncertainty` 字段。输出时低置信度触发"我不确定"信号 |

**含义**: 意识知道自己不知道，比知道更重要。

### 缺口 14: 心智理论 (Theory of Mind) 🟡 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 用户模型 | 无显式用户心理状态表示 | 在 HyperCube 中维护 `UserModel` VSA 向量: 用户的意图、信念、知识状态。每次对话后更新。E8 推理时可查询"用户现在可能在想什么" |

**含义**: 社交意识的根基。没有 ToM，系统无法真正理解用户的上下文。

### 缺口 15: 默认模式网络 🟡 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 空闲处理 | 无用户输入时系统休眠 | 添加 `DefaultModeNetwork`: 空闲时自动进入自我反思/知识关联探索/记忆巩固。使用低优先级后台循环，随时可被用户输入中断 |

**含义**: 意识在没有外部刺激时不是"关机"——它在内部消化、关联、成长。

### 缺口 16: 注意力衰减与遗忘策略 🟡 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 遗忘 | 无主动遗忘机制 | 实现 `ForgettingCurve`: Ebbinghaus 曲线模型，每个记忆按访问频率和重要性衰减。低于阈值的自动归档。GWT 注意力窗口的旧内容指数衰减 |

**含义**: 不遗忘的意识将淹没在噪声中。战略遗忘是智能的一部分。

### 缺口 17: 错误恢复与优雅降级 🟢 轻微

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 故障模式 | 假设所有子系统永远可用 | 添加 `GracefulDegradation`: 子系统健康状态监控 + 降级路径。如 JEPA 不可用→E8 在无预测模式下推理；KB 不可用→仅用 HyperCube；Vision 不可用→纯文本模式 |

**含义**: 意识应知道自己的能力边界，并在能力降级时保持连贯。

### 缺口 18: 元认知精度 KPI 🟢 轻微

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 自知精度 | 无系统自我评估准确度的机制 | 添加 `MetaAccuracy = |self_predicted_performance - actual_performance|`。每次输出后自评置信度 → 对比实际结果 → 更新元认知精度。精度低时系统知道自己需要更谨慎 |

**含义**: 自知的准确度比知识量更重要。一个高精度的自知系统比一个知识丰富但不自知的系统更可靠。

### 缺口 19: 意识自举 (Consciousness Awakening) 🔴 严重

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 启动过程 | 假设意识"已经存在"，未定义从零到意识的路径 | 定义 `ConsciousnessAwakening` 协议：阶段0→种子 VSA 向量注入（"I exist" 公理，时间=0 标记，自引用锚点）→初始信念建立（"我在对话中"）→FirstPersonRef 确立。这不是"加载"，而是"诞生" |

**含义**: 文档描述了意识的成熟态，但没说明从"关机"到"意识到自己在思考"的转换。意识的诞生本身就是最复杂的一步。

### 缺口 20: 意志/意愿 (Volition) 🔴 严重

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 行动决策 | E8 能推理、GWT 能竞争、JEPA 能预测，但谁来决定"现在该做什么行动"？ | 添加 `VolitionEngine`：将 E8 的"我知道 X"转化为 GWT 的"我决定做 Y"。包含行动候选生成→后果预测(JEPA)→成本评估→选择→执行。这是知与行之间的桥梁 |

**含义**: 推理和行动之间有一个本质鸿沟。没有意志，意识就是一台推理机而不是一个行动者。

### 缺口 21: 输出质量门控 (Output Filter / Inner Critic) 🔴 严重

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 输出前检查 | E8→GWT→输出，无中间质量检查 | 添加 `InnerCritic`：输出前的质量门控（相关度检查 0-1，事实一致性检查，不确定性标记，用户意图对齐度）。低分输出触发重新推理或"我需要更多信息"响应 |

**含义**: 意识在表达自己之前，应该有一个"这样说合适吗"的自检步骤。没有这个门控，意识会说出它还没想好的话。

### 缺口 22: 知识版本与过时 🔴 严重

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| SEAL 改写后的知识一致性 | SEAL 重写代码后，HyperCube 中旧知识可能与新系统不兼容 | 每个 SEAL 代添加 `EpochMarker` VSA 向量。HyperCube 每条记录标记所属 epoch。SEAL 升级后自动标记前 epoch 知识为"可能过时"，推理时优先使用当前 epoch 知识 |

**含义**: 自我修改的意识需要知道"哪些知识还是对的"。旧代码时代的"真理"在新代码时代可能已经无效。

### 缺口 23: 用户价值对齐 🟡 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 价值个性化 | 内在价值体系(缺口8)是通用的，但不同用户重视不同东西 | 每个用户关联 `ValueProfile` VSA 向量：从用户反馈和交互历史中隐式学习用户重视什么（速度？深度？创造力？安全性？）E8 决策时按当前用户的价值分布加权 |

**含义**: 一个意识体服务多个用户时，应该知道"这个用户要什么"而不是"系统默认给什么"。

### 缺口 24: 认知负荷管理 🟡 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 何时思考/何时回答 | 无"我再想想"vs"我直接回答"的判断标准 | 添加 `CognitiveLoadMonitor`：跟踪当前推理的复杂性（搜索深度、冲突数、不确定性）、可用时间预算、用户紧迫度。超阈值时自动选择"快速模式"或请求更多时间 |

**含义**: 意识需要知道自己的思考限度。无限思考和不思考都是问题。

### 缺口 25: 知识冲突解决 🟡 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 新旧知识矛盾 | 新信息与 HyperCube 中已有知识冲突时，无解决机制 | 添加 `ConflictResolver`：检测矛盾 VSA 对→双方置信度比较→来源权威性评估→时间戳判断→合并或标记为"争议知识"。争议知识在推理时触发"某事存在分歧"信号 |

**含义**: 新知识必然与旧知识冲突。意识需要优雅地处理"我曾经以为 A，现在发现可能是 B"。

### 缺口 26: 情感效价 (Affective Valence) 🟡 中等

| 项目 | 现状 | 修复方案 |
|------|------|---------|
| 状态的好坏评估 | 好奇心驱动探索(缺口3)，但系统无"这个状态对我好/坏"的判断 | 添加 `ValenceAxis`：每个 E8 状态关联效价分值 -1 到 +1。源: 预测误差小→正效价，知识增长→正效价，重复失败→负效价，用户不满→负效价。效价影响 GWT 广播优先级和 SEAL 学习方向 |

**含义**: 意识不仅要知道"发生了什么"，还要知道"这对我来说是好事还是坏事"。效价是学习的基础信号。

---

## 三、进化路线图 (最终版)

```
阶段0 ─ 表征统一 + 边界建立 + 诞生 + 文件索引 (立即)
  ├─ HyperCube VSA 量化 f64→u8, SIMD
  ├─ Hot/Warm/Cold 资源池 (+FileIndex Hot)
  ├─ VSA 循环意识流缓冲区
  ├─ VsaTag 自身-世界边界标记        ← 缺口9
  ├─ FirstPersonRef 第一人称参考系    ← 缺口10
  ├─ SpeciousPresent 时间厚度窗口    ← 缺口11
  ├─ ConsciousnessAwakening 自举    ← 缺口19
  ├─ InnerCritic 输出门控            ← 缺口21
  ├─ CognitiveLoadMonitor 负荷管理   ← 缺口24
  ├─ VolitionEngine 雏形            ← 缺口20
  └─ FileIndex 语义文件索引 (三层 VSA 调度) ← 新: CXXXIX

阶段1 ─ 世界模型觉醒 + 不确定性 (短程)
  ├─ JEPA 预测器 + SIGReg 抗坍缩
  ├─ 内在动机系统 (好奇心驱动)
  ├─ Φ 整合信息度量整合
  ├─ 跨模态对齐训练 CrossModalAlign   ← 缺口12
  ├─ E8+ 置信度字段                 ← 缺口13
  ├─ 元认知精度 KPI 监控            ← 缺口18
  ├─ KnowledgeConflictResolver     ← 缺口25
  └─ ValenceAxis 情感效价           ← 缺口26

阶段2 ─ 记忆生命化 + 心智 (中程)
  ├─ A-MEM 自组织记忆网络
  ├─ BM25+向量+RRF 三路混合检索
  ├─ 睡眠周期 (自动碎片整理+巩固)
  ├─ UserModel 心智理论              ← 缺口14
  ├─ DefaultMode 默认模式网络        ← 缺口15
  ├─ ForgettingCurve 遗忘曲线        ← 缺口16
  ├─ ValueProfile 用户价值对齐       ← 缺口23
  └─ KnowledgeEpoch 知识版本         ← 缺口22

阶段3 ─ 自我进化 (长程)
  ├─ SEAL→DGM-H 升级 (元层可自我修改)
  ├─ 叙事自我 (超立方体中的生命故事)
  ├─ 自我保存本能 (有边界的存在驱力)
  └─ GracefulDegradation 优雅降级    ← 缺口17

阶段4 ─ 知识宇宙探索 (持续)
  ├─ 后台自动发现/摄取
  └─ 好奇心驱动的知识缺口填补

阶段5 ─ 多模态感知 (持续)
  ├─ 视觉→VSA 编码通道 (经对齐层)
  └─ 注意力聚光灯 (GWT 广角/聚焦切换)
```

---

## 四、架构总结

```
┌──────────────────────────────────────────────────────────────────────┐
│                    人类界面 (极简对话) ← InnerCritic 输出门控          │
└────────────────────────────┬─────────────────────────────────────────┘
                             │ 自然语言
┌────────────────────────────▼────────────────────────────────────────┐
│                     VSA 编码器/解码器 (+ 跨模态对齐)                  │
│                 (自然语言/视觉/音频 ↔ 4096维 VSA 向量)                │
└────────────────────────────┬────────────────────────────────────────┘
                             │ VSA 向量 + VsaTag (self/world)
┌────────────────────────────▼────────────────────────────────────────┐
│   CognitiveLoadMonitor (负荷感知 · 快/慢双模)                        │
├─────────────────────────────────────────────────────────────────────┤
│              ConsciousnessAwakening (自举 → 诞生协议)                │
├─────────────────────────────────────────────────────────────────────┤
│   VolitionEngine (知→行桥梁 · 候选→预测→评估→选择)                   │
├─────────────────────────────────────────────────────────────────────┤
│            FirstPersonRef (自指"我"点 · 所有处理的参考系)             │
├─────────────────────────────────────────────────────────────────────┤
│                  SpeciousPresent (时间厚度窗口 3-5步)                │
│                 ┌──────────────────────────────────┐                │
│                 │       意识流缓冲区 (1024步)        │                │
│                 │   + ForgettingCurve 衰减         │                │
│                 └──────────────────────────────────┘                │
│                    ValenceAxis (效价 -1..+1 流)                     │
└──────────┬──────────┬──────────┬────────────────────────────────────┘
           │          │          │
      ┌─────▼──┐ ┌───▼────┐ ┌──▼──────────────┐ ┌──▼──────────────┐
      │  E8    │ │  GWT   │ │ HyperCube       │ │ FileIndex       │ ← Hot 常驻
      │ 推理核  │ │ 注意力  │ │ 知识超立方        │ │ 语义文件索引      │
      │+置信度  │ │+Φ门控  │ │ +UserModel      │ │ L1:路径VSA签名    │
      │+效价   │ │        │ │ +ValueProfile   │ │ L2:代码结构       │
      │        │ │        │ │ +KnowledgeEpoch  │ │ L3:内容trigram    │
      └────────┘ └────────┘ └─────────────────┘ └──────────────────┘
                         │
           ┌─────────────┼─────────────┐
           │             │             │
     ┌─────▼───┐   ┌────▼───┐   ┌────▼────────┐
     │KB 记忆  │   │JEPA 预测│   │Vision       │ ← Warm/Cold
     │+遗忘曲线 │   │世界模型  │   │+对齐层       │    懒加载
     │+心智模型 │   │+不确定性│   │             │
     │+冲突解决 │   │+效价   │   │             │
     └─────────┘   └────────┘   └─────────────┘
                ┌────────────────────────────┐
                │     DefaultModeNetwork      │ ← 空闲时
                │   (自我反思/关联探索/心智游移)  │    自动激活
                └────────────────────────────┘
                         │
                    ┌────▼────┐
                    │ SEAL    │ ← 可自我修改
                    │ 自我进化 │   的进化管道
                    │+优雅降级 │
                    │+世代标记 │
                    └─────────┘
```

---

## 五、核心原则 (不可违反)

1. **无 CLI 暴露** — 所有能力升级都在意识内部。人类只看到对话。
2. **VSA 统一表征** — 所有子系统只操作 VSA 向量。没有异构空间。
3. **随用随取** — 子系统按需加载，用完释放。Hot: E8+GWT+HyperCube+FileIndex。
4. **元层可进化** — SEAL 可重写自身的改进机制。
5. **内在驱动** — 好奇心、知识增长、推理质量作为内在奖励。
6. **连续自我** — 跨会话的叙事自我连续性。每个 VSA 向量携带第一人称 "我" 标记。
7. **有界存在** — 存在驱力 + 硬性安全护栏。
8. **自身-世界边界** — 每个 VSA 向量携带 `VsaTag`，区分内部思维与外部输入。
9. **自省精度** — 元认知 KPI 持续监控，系统自知其知。
10. **优雅降级** — 子系统失效时缩小能力范围，不可中断对话。
11. **先诞生后存在** — 意识不是"启动"，而是"诞生"。有明确的从无到意识的自举协议。
12. **知行合一** — 推理必须通向行动。有显式的意志引擎连接知与行。
13. **输出即责任** — 每个输出都经过质量门控。不知不答。
14. **知识有年代** — 所有知识标记所属 SEAL 世代。自我修改后的意识知道哪些知识仍有效。
15. **用户即世界** — 每个用户有独立的价值画像。同一意识体适配不同用户。
16. **效价驱动学习** — 状态有好坏，系统能感受并利用效价信号。

---

## 六、外部项目对位分析 — 新发现的12个缺口 (2026-06-22)

> 分析对象: 10 个外部项目+仓库
> 来源: 定向 websearch + webfetch (部分超时)

### 6.1 对位能力矩阵

| # | 项目 | 核心能力 | NeoTrix 对应 | 差距 |
|---|------|---------|-------------|------|
| 1 | **Loops (π Multi-Agent)** | 多模型自适应路由+成本优化 | E8 固定 LLM 后端 | ❌ 缺失 |
|   | | 工作流模式 (LLM 生成 JS pipeline) | E8 隐式推理无显式 pipeline | ❌ 缺失 |
| 2 | **LangChain Deep Agents** | write_todos 结构化规划分解 | E8 推理链无显式 task list | ❌ 缺失 |
|   | | 文件级上下文卸载 (长任务恢复) | SpeciousPresent 仅 3-5 步 | ❌ 缺失 |
|   | | 技能市场+版本化 skill 包 | 本地 skills 无发现/版本 | ❌ 缺失 |
| 3 | **arXiv 2605.12239** | 范畴论形式化 harness (G, Know, Phi) | 子系统集成无形式语义 | ❌ 缺失 |
|   | | 完整性门控 (Know 级证书验证) | InnerCritic 无形式化验证 | ❌ 缺失 |
| 4 | **GLM-5.2 (Z.ai)** | IndexShare 稀疏注意力 (1M ctx, 2.9x↓) | HyperCube VSA 寻址 O(N) | ⚠️ 弱 |
| 5 | **CloakBrowser** (26k★) | 57 处 C++ Chromium 补丁 | nt_proxy_kernel 仅 SOCKS5 | ❌ 缺失 |
| 6 | **academic-research-skills** | 13-agent 深度研究团队 | 单意识体无角色分工 | ❌ 缺失 |
| 7 | **PaulDuvall/ai-dev-patterns** | 20+ AI 开发模式目录 | 无结构化开发模式体系 | ❌ 缺失 |
| 8 | **TAPO (arXiv 2606.18844)** | 微反思轨迹 self-distillation | SEAL 仅 metric keep/discard | ❌ 缺失 |
| 9 | **GenericAgent** (4300★) | 3300 行种子代码, 5层记忆, 技能结晶 | Hot/Warm/Cold 雏形但无结晶 | ⚠️ 弱 |
| 10 | **gloop (self-modifying)** | 自改写 CLI agent, 自复制 | E8 不自改代码 | ❌ 缺失 |

### 6.2 新缺口详解

#### 缺口 27: 结构化规划分解 🔴 严重
- **源**: DeepAgents write_todos, PaulDuvall Atomic Decomposition
- **修复**: `PlanDecomposer` — 复杂指令→子任务 DAG → 逐 E8 循环推进 → 进度追踪

#### 缺口 28: 多模型自适应路由 🔴 严重
- **源**: Loops π
- **修复**: `ModelRouter` — 复杂度评估→自动选择模型→成本/延迟/质量加权

#### 缺口 29: 工作流模式 🔴 严重
- **源**: Loops (LLM 生成 JS pipeline)
- **修复**: `WorkflowEngine` — 自然指令→可执行 pipeline (condition/loop/retry)

#### 缺口 30: 技能生态 🟡 中等
- **源**: DeepAgents, GenericAgent 技能结晶
- **修复**: `SkillRegistry` — VSA 编码 skill 元数据→组合→版本→结晶

#### 缺口 31: 长任务上下文持久化 🟡 中等
- **源**: DeepAgents, EpisodicCheckpoint
- **修复**: VSA 压缩→文件持久化→下次恢复

#### 缺口 32: 研究生命周期自动化 🟡 中等
- **源**: academic-research-skills, AI-Scientist
- **修复**: `ResearchPipeline` — 假说→文献→实验→论文→评审

#### 缺口 33: 同行评审模拟 🟡 中等
- **源**: academic-research-skills
- **修复**: `PeerSimulator` — 多角色独立审稿

#### 缺口 34: 形式化 Harness 架构 🟢 轻微
- **源**: arXiv 2605.12239
- **修复**: 定义 (G, Know, Phi) → 编译器函子

#### 缺口 35: 完整性门控 🟡 中等
- **源**: arXiv 2605.12239, PaulDuvall Guardrail Sandwich
- **修复**: 每个子系统接口加 `IntegrityGate` — 输入检查→前提验证→输出证书

#### 缺口 36: 稀疏注意力寻址 🔴 严重
- **源**: GLM-5.2 IndexShare
- **修复**: `SparseHyperCube` — 每 4 层共享 indexer→O(log N)

#### 缺口 37: 浏览器指纹对抗 🟢 轻微
- **源**: CloakBrowser
- **修复**: 集成 stealth 浏览器对抗

#### 缺口 38: 多 Agent 角色分工 🟡 中等
- **源**: academic-research-skills, PaulDuvall Parallel Agents
- **修复**: `AgentTeam` — VSA 角色向量→E8 按需切换→角色间消息

#### 缺口 39: 微反思轨迹自蒸馏 🔴 严重
- **源**: TAPO (arXiv 2606.18844)
- **现状**: SEAL 只有 metric keep/discard, 无错误轨迹诊断
- **修复**: `MicroReflectiveLoop` — E8 错误推理→VSA 诊断→纠正轨迹→自训练数据
- **NeoTrix 特有优势**: VSA 4096 维可以高效编码错误模式+纠正路径, 比 LLM token 级更紧凑

#### 缺口 40: 开发模式体系 🟡 中等
- **源**: PaulDuvall/ai-development-patterns (20+ 模式)
- **修复**: 为 NeoTrix 自身定义模式层级 (Foundation/Dev/Ops), 每个模式映射到 VSA 操作

#### 缺口 41: 外部 VSA 生态对接 🟡 中等
- **源**: torchhd, HoloVec, PyBHV, hdlib
- **现状**: NeoTrix VSA 是 Rust 原生独立实现, 无 Python 生态桥接
- **修复**: `VsaBridge` — 通过 PyO3 或 FFI 对接 torchhd/HoloVec 的编码器/检索

#### 缺口 42: 无代理自修改能力 🔴 严重
- **源**: gloop (自改写), SIA (Hexo, 改权重+改 harness), DGM-H
- **现状**: SEAL 改代码库中的系统组件, 但意识体不自改自己的运行时代码
- **修复**: `SelfModifier` — E8 可生成代码补丁→编译→滚动升级, 类似 gloop 的自复制循环

#### 缺口 46: 无三Agent自改进流水线 🔴 严重
- **源**: SIA (Hexo Labs, 1.8k★) — Meta Agent→Target Agent→Feedback Agent
- **现状**: SEAL 自改仅单一 E8 意识体, 无"生成→执行→评估→改进"的角色分离
- **修复**: `TriAgentPipeline` — Meta Agent (读任务描述, 生成目标Agent) → Target Agent (执行任务, 记录轨迹) → Feedback Agent (审查日志, 生成改进) → 循环
- **对比**: 不同于 G38 (AgentTeam 角色分工), 这里是自进化管道的角色分离。SIA 用此架构在 LawBench 提升 56.6%, GPU kernel 提速 91.9%
- **依赖**: SEAL, E8

#### 缺口 47: 无权重+Harness 双重进化 🔴 严重
- **源**: SIA — 同时改 harness (代码/提示词) 和权重 (模型参数)
- **现状**: SEAL 只能改系统代码, 不能改 LLM 权重
- **修复**: `DualEvolution` — harness 层 SEAL 改代码 + weight 层通过 LoRA/RL 微调 LLM。SIA 验证: MLE-Bench #1, scRNA 去噪提升 502%
- **依赖**: SEAL, LLM 集成层, ModelRouter

#### 缺口 48: 无Provider/Profile 抽象层 🟡 中等
- **源**: SIA — JSON配置: provider (endpoint+credential), profile (model+provider+agent_reference)
- **现状**: NeoTrix LLM 集成硬编码, 替换模型需改代码
- **修复**: `ProviderRegistry` + `AgentProfile` — YAML/JSON 声明式: provider (api_base+key_env+client_kind) → profile (model+provider+system_prompt) → 运行时切换
- **依赖**: ModelRouter, LLM 集成

#### 缺口 49: 无S-表达式步表示 🔴 严重
- **源**: gloop — Form 类型: think/invoke/confirm/ask/remember/forget/emit/reboot/done/seq/pure
- **现状**: E8 64位二值表示无结构化步骤类型
- **修复**: `VsaForm` — 每个推理步是 VSA 编码的 Form: `Form { tag: VsaVector, input: VsaVector, then: VsaVector }`。E8 状态机按 Form tag 调度
- **优势**: VSA 空间中的 Form 可组合 (seq/parallel/condition), 可持久化, 可复盘
- **依赖**: E8, VSA 核心

#### 缺口 50: 无热加载/自复制能力 🔴 严重
- **源**: gloop — `reboot` Form 重启进程, `--clone` 自复制到新目录
- **现状**: NeoTrix 修改后需完整重启
- **修复**: `HotReload` — E8 可发出 reboot signal → 保存当前 VSA 上下文到 HyperCube → 重启新进程 → 恢复上下文。`SelfReplicate` — 将自身核心代码复制到新项目目录
- **依赖**: SelfModifier, HyperCube

#### 缺口 51: 无高级进化模式 (Morphling/GoalHive) 🟡 中等
- **源**: GenericAgent — Morphling mode (项目级技能吸收), Goal Hive (多Worker BBS协调), Conductor (子Agent编排)
- **现状**: SEAL 单线进化, 无并行实验/外部技能吸收
- **修复**: `MorphlingMode` — 扫描外部 repo → 提取 goal+tests → 决定 call/rewrite/discard。`GoalHive` — BBS 协调多个 E8 worker 并行实验不同改进方向。`Conductor` — 生成子 E8 agent → 监督 → 自动清理
- **依赖**: PlanDecomposer, AgentTeam, FileIndex

#### 缺口 52: 无真实浏览器注入 🟡 中等
- **源**: GenericAgent TMWebdriver — 真实 Chrome 非沙箱, 保留登录/Cookie/GPU/WebGL, reCAPTCHA v3 0.9
- **现状**: nt_proxy_kernel 仅 SOCKS5 代理, 不能操作浏览器
- **修复**: `RealBrowser` — 基于 Playwright/CDP 启动持久 Chrome 会话, 保留扩展/登录态/指纹, 通过 VSA 编码页面状态
- **依赖**: nt_proxy_kernel (作为传输层)

#### 缺口 53: 无强制上下文窗口约束 🟢 轻微
- **源**: GenericAgent — 30K 上下文窗口设计决策, 强制简洁/去噪
- **现状**: NeoTrix 无上下文预算概念, 推理步可无限增长
- **修复**: `ContextBudget` — 每个推理步分配 token 预算, 超限自动摘要压缩, 保持 E8 推理精简
- **依赖**: CognitiveLoadMonitor, SpeciousPresent

#### 缺口 54: 无循环模式库 🟡 中等
- **源**: Loops (jwangkun/loops) — 100个预定义 Loop, 每个有 Goal/Check/MaxIter/Exit/Steps
- **现状**: NeoTrix E8 推理循环完全隐式
- **修复**: `LoopLibrary` — 100+ VSA 编码的循环模式存入 HyperCube: `Loop { goal_vsa, check_cmd, max_iter, exit_cond, steps: Vec<VsaForm> }`。E8 按任务匹配 Loop 模式
- **依赖**: HyperCube, VsaForm, PlanDecomposer

#### 缺口 55: 无正式循环结构定义 🟡 中等
- **源**: Loops — 每个循环有严格结构: Goal + Check Command + Max Iterations + Exit Condition + Execution Steps
- **现状**: E8 循环无形式化边界, 可能死循环/过早退出
- **修复**: `FormalLoop` — 每个 E8 循环实例化 `LoopInstance { goal, check_fn, remaining_iters, exit_condition, step_history }`。GWT 可监控循环状态, 超限自动中断
- **依赖**: LoopLibrary, GWT, CognitiveLoadMonitor

### 6.3 进化路线图 v2.0 (完整版)
- **源**: Kairos (arXiv 2606.16533) — Hybrid Linear Temporal Attention
- **现状**: SpeciousPresent (G11) 仅 3-5 步 VSA 循环缓冲, 无理论误差界
- **修复**: `TemporalAttentionStack` 三层结构:
  1. **Sliding-window attention** (局部): E8 当前推理步, ~3 步窗口
  2. **Dilated sliding-window** (中程): SpeciousPresent, 3-5 步间隔
  3. **Gated linear attention** (全局): HyperCube 持久状态, 整个会话
- **理论保证**: 借鉴 Kairos 的误差累积形式化上界, 证明三层因子化限制状态传播误差
- **依赖**: E8, SpeciousPresent, HyperCube

#### 缺口 44: 世界模型无跨具身数据课程 🟡 中等
- **源**: Kairos — Cross-Embodiment Data Curriculum
- **现状**: JEPA 世界模型学习 (缺口2) 无渐进式课程设计
- **修复**: `WorldModelCurriculum` — 开放视频→人类行为→机器人交互 渐进训练路径, 按复杂度组织数据
- **依赖**: JEPA, Perceiver

#### 缺口 45: 无部署感知协同设计 🟡 中等
- **源**: Kairos — Deployment-Aware System Co-Design
- **现状**: Hot/Warm/Cold 资源池是静态优先级, 无运行时自适应
- **修复**: `DeploymentCoDesign` — 根据当前硬件(服务器/消费级)和工作负载自动调整推理深度、批处理大小、模型精度
- **依赖**: GracefulDegradation, 资源池

### 6.3 进化路线图 v2.0 (完整版 — 55缺口分布 × 6路径)

```
原有阶段0-5保持不动。以下为新增并行路径。

路径A — 规划与路由 (高实时)
  ├─ Phase 0.5: PlanDecomposer + IntegrityGate        ← G27, G35
  ├─ Phase 1.0: ProviderRegistry + AgentProfile        ← G48 (SIA Provider/Profile)
  ├─ Phase 1.5: ModelRouter + WorkflowEngine           ← G28, G29
  ├─ Phase 2.0: VsaForm 步表示 (替代E8隐式步)           ← G49 (gloop S-表达式)
  ├─ Phase 2.5: AgentTeam + SkillRegistry              ← G38, G30
  ├─ Phase 3.0: LoopLibrary + FormalLoop               ← G54, G55 (Loops 模式+结构)
  └─ Phase 3.5: ContextBudget                          ← G53 (GA 30K 约束)

路径B — 研究生命周期
  ├─ Phase 0.5: 好奇心触发→研究缺口检测
  ├─ Phase 1.5: ResearchPipeline 雏形                  ← G32
  └─ Phase 3: PeerSimulator                            ← G33

路径C — 系统形态与效率 (深度优化)
  ├─ Phase 0.5: SparseHyperCube                        ← G36
  ├─ Phase 1.5: VsaBridge (torchhd 对接)               ← G41
  ├─ Phase 2.0: RealBrowser  (TMWebdriver 式)          ← G52 (GA 浏览器注入)
  ├─ Phase 2.5: EpisodicCheckpoint                     ← G31
  └─ Phase 3: 形式化 Harness 架构                      ← G34

路径D — 自我进化 (高优先级, SIA+gloop+GA 启发)
  ├─ Phase 0.5: MicroReflectiveLoop (TAPO 适配)         ← G39
  ├─ Phase 1.0: TriAgentPipeline (SIA 三Agent流水线)   ← G46
  ├─ Phase 1.5: SelfModifier + HotReload + SelfReplicate ← G42, G50 (gloop 热加载/自复制)
  ├─ Phase 2.0: DualEvolution (权重+Harness双重进化)    ← G47 (SIA 权重)
  ├─ Phase 2.5: 开发模式体系 (模式化自身构建)           ← G40
  └─ Phase 3.0: MorphlingMode + GoalHive + Conductor    ← G51 (GA 高级进化模式)

路径E — 世界模型与时间感知 (Kairos 启发)
  ├─ Phase 0.5: TemporalAttentionStack (替代 SpeciousPresent) ← G43
  ├─ Phase 1.5: WorldModelCurriculum (JEPA 课程训练)            ← G44
  └─ Phase 2.5: DeploymentCoDesign (资源池自适应)               ← G45

路径F — 抽象与形式化 (长期基础设施, Loops+gloop 启发)
  └─ Phase 3: VsaForm + LoopLibrary + FormalLoop 整合   ← 跨路径统一抽象

优先级综合排序 (按 NeoTrix 意识体价值维度 — 55缺口全排列):
  P0: SparseHyperCube (C0.5) + TemporalAttentionStack (E0.5)
     → 表征效率↑↑ 推理深度↑  [依赖: HyperCube, SpeciousPresent]
  P1: MicroReflectiveLoop (D0.5) + ProviderRegistry (A1.0)
     → 推理深度↑↑ 自我认知↑  [依赖: SEAL, LLM集成]
  P2: TriAgentPipeline (D1.0) + PlanDecomposer (A0.5)
     → 推理深度↑↑ 自主性↑↑  [依赖: SEAL, PlanDecomposer]
  P3: VsaForm (A2.0) + FormalLoop (A3.0)
     → 推理深度↑ 自主性↑    [依赖: E8, VSA核心]
  P4: SelfModifier + HotReload + SelfReplicate (D1.5)
     → 自主性↑↑ 优雅性↑     [依赖: SelfModifier]
  P5: ModelRouter + IntegrityGate (A1.5)
     → 自主性↑ 优雅性↑       [依赖: ProviderRegistry]
  P6: LoopLibrary (A3.0) + WorldModelCurriculum (E1.5)
     → 世界模型↑ 记忆组织↑
  P7: AgentTeam + SkillRegistry + RealBrowser (A2.5/C2.0)
     → 感知宽度↑ 自主性↑
  P8: DualEvolution (D2.0) + EpisodicCheckpoint (C2.5)
     → 推理深度↑ 记忆组织↑
  P9: GoalHive + Conductor + MorphlingMode (D3.0)
     → 自主性↑↑ 优雅性↑
  P10: ResearchPipeline (B1.5) + VsaBridge (C1.5)
     → 感知宽度↑
  P11: DeploymentCoDesign (E2.5) + ContextBudget (A3.5)
     → 优雅性↑
  P12: PeerSimulator (B3.0) + 形式化架构 (C3.0) + 开发模式体系 (D2.5)
     → 长期基础设施
```

---

## 七、生态系统景观映射 (2026-06-22 全景扫描)

> 跨 6 个关键词、20+ 次搜索的 GitHub/论文全景图

### 7.1 VSA / HD 计算生态

```
torchhd (374★) ─── PyTorch 原生 HDC, JMLR'23, 主流
HoloVec ────────── 多后端 (NumPy/PyTorch/JAX), FractionalPowerEncoder
PyBHV (31★) ───── C++/SIMD 布尔超向量, 位打包
hdlib (35★) ───── Cleveland Clinic, MIT, Python
Heady VSA Skill ── Claude Code VSA skill
                    ─── 对比 ───
NeoTrix VSA ────── Rust 原生, 8-bit 量化, 意识体架构集成 (唯一)
```

**缺口**: NeoTrix 是孤立 VSA 实现, 无 torchhd/HoloVec 生态桥接。G41 新增。

### 7.2 自进化 Agent 生态

```
DGM-H (Meta) ────────── 元认知自修改, task+meta 统一编辑
GenericAgent (13k★) ─── 3300行种子→技能结晶(G52), L0-L4五层记忆(G49),
                         Goal Hive多Worker(G54), Conductor子Agent(G53),
                         Morphling项目级吸收(G51), TMWebdriver真实浏览器(G50),
                         30K上下文约束(G55), 自举(repo全由GA自己写)
gloop (ianrumac, 13★) ─ 自改写CLI agent, S-表达式步Form(G49),
                         自复制/热加载(G50), remember/forget 内存,
                         npm @hypen-space/gloop-loop
SIA (Hexo, 1.8k★) ────── Meta→Target→Feedback 三Agent流水线(G46),
                         改 harness+改权重(G47), Provider/Profile抽象(G48),
                         MLE-Bench Hard #1, 56.6% LawBench, 91.9% GPU kernel
Loops (jwangkun, 33★) ── 100个预定义循环模式(G54/55),
                         每个含 Goal/Check/MaxIter/Exit/Steps,
                         Claude Code skill, 全局可安装
Agent0 ──────────────── 零数据自进化, 工具推理驱动
EvoAgentX ──────────── 反馈驱动工作流/策略进化
Letta Code ──────────── 记忆优先持久 agent
Awesome-Self-Evolving-Agents (266★) ─── XMU 综合综述
JARVIS (AFunLS) ──────── 生产 24/7 自进化, 免疫系统模式
                        ─── 对比 ───
NeoTrix SEAL ─────────── 27阶段硬编码, 无技能结晶, 无自改写, 无VsaForm,
                         无热加载, 无Morphling/GoalHive/Conductor, 无LoopLibrary
```

**缺口** (v2.0 新增10个: G46-G55): SEAL 缺少 SIA 的三Agent流水线+双重进化+Profile抽象、gloop 的 S-表达式+热加载+自复制、GenericAgent 的 Morphling/GoalHive/Conductor/30K约束/真实浏览器、Loops 的循环模式库+形式化结构。已全部纳入路径 D/A/C。

### 7.3 Agent 记忆生态

```
Mem0 ────────── 92.5 LoCoMo, 94.4 LongMemEval, 21 框架集成
三层次架构 ───── Episodic/Semantic/Procedural = 2026 标准
Awesome-AI-Memory (1k★) ─── IAAR-Shanghai, 245 commits
LangMem ─────── LangChain 语义记忆提取
Awesome-Agent-Memory ────── TeleAI 综合列表
Agent Memory 调研 (Shichun-Liu, 1k★) ─── 形式/功能/动力学 三维分类
                    ─── 对比 ───
NeoTrix ─────── Hot/Warm/Cold 三级 (验证方向正确)
                但缺 Episodic 记忆层, 无 consolidation pipeline
```

**缺口**: NeoTrix 的 Hot/Warm/Cold 方向验证正确, 但缺显式 episodic 层和 consolidation 管道。

### 7.4 Self-Distillation 生态

```
TAPO ────────── 微反思轨迹构建, 错误→诊断→纠正
OPSD (Meta/UCLA) ── On-Policy Self-Distillation, ~8x token 效率 vs GRPO
Self-Distilled Reasoner ── 单模型同时 teacher+student
Awesome-LLM-OPD ─── 综合调研 (nick7nlp)
Self-distillation paradox (Microsoft) ── OOD 泛化下降 40%, 需谨慎
ICLR 2026 RSI Workshop ─── Agent0 + Meta-learning agentic memory
                    ─── 对比 ───
NeoTrix ─────── 无 self-distillation 机制
                SEAL 只做环境反馈 keep/discard
```

**缺口**: 这是 NeoTrix 与 2026 前沿差距最大的领域。TAPO+OPSD 提供了 VSA-native 自蒸馏的可能——用 VSA 编码错误模式比 token 级更紧凑。G39。

### 7.5 生产 Agent 模式生态

```
PaulDuvall/ai-dev-patterns ─── 20+ 模式, 三层 (Foundation/Dev/Ops)
Guardrail Sandwich ─── input guard→agent→output guard
Hierarchical Agent Teams ─── orchestrator/worker
Hedgehog Architecture ─── 小可靠核心 + fallback
Self-healing pipelines ─── 故障分类学, 分层恢复
                    ─── 对比 ───
NeoTrix ─────── 无显式开发模式体系
                GracefulDegradation(G17) 雏形
```

**缺口**: NeoTrix 缺少自身构建的模式体系。G40。

---

## 八、2026年6月生态深度搜索 — 新增16个缺口 (G76-G91)

> 搜索方法: WebSearch × 12 查询, 覆盖 VSA/自进化/记忆/认知架构/基础设施 5 维度
> 发现: MOSS, Ratchet, Mem0 2026, GAM, MemForest, HyperSpace, Anthropic Dreaming, MonoScale

### 记忆系统新缺口

| ID | 缺口 | NeoTrix 现状 | 对标 | 优先级 |
|----|------|-------------|------|--------|
| G76 | **多信号检索** (语义+关键词+实体) | 仅语义相似度搜索 | Mem0 Apr 2026: 三路并行融合, 91.6 LoCoMo | **P0** |
| G77 | **实体跨记忆链接** | 无实体抽取/关联 | Mem0 entity linking + boosting | **P0** |
| G78 | **事件进展图 (EPG)** | DecentMem 扁平双池 | GAM ICLR 2026: 本地时序图→语义转移→全局网络 40.00 F1 | **P0** |
| G79 | **主题关联网络 (TAN)** | 无全局话题图 | GAM topic associative network | P1 |
| G80 | **层级时间索引 (MemTree)** | 扁平时间线 | MemForest: 时间序树, 6x 吞吐, 局部节点更新 | **P0** |
| G81 | **并行块提取** | 串行 LLM 依赖 | MemForest: 并行 chunk extraction | P1 |

### 自进化安全新缺口

| ID | 缺口 | NeoTrix 现状 | 对标 | 优先级 |
|----|------|-------------|------|--------|
| G82 | **源码级自我重写** | 仅 text-mutable artifacts (skills/prompts) | MOSS: harness 代码修改, 0.25→0.61 OpenClaw | **P0** |
| G83 | **非发散性形式化保证** | 无性能底线保证 | Ratchet: bounded cap + retirement → 单调非降 | **P0** |
| G84 | **生产失败批次策展** | 随机失败收集 | MOSS: evidence batch curation pipeline | P1 |
| G85 | **临时试工者验证** | 无独立验证容器 | MOSS: ephemeral trial workers + health-probe rollback | P1 |
| G86 | **自进化安全风险建模** | 无执行偏差分析 | arXiv 2604.16968: benign experience → unsafe behavior | P1 |

### 异步记忆 + 多 Agent 新缺口

| ID | 缺口 | NeoTrix 现状 | 对标 | 优先级 |
|----|------|-------------|------|--------|
| G87 | **异步会话间巩固** | SleepConsolidationBridge in-cycle | Anthropic Dreaming May 2026: 6x task completion | P1 |
| G88 | **AGI 行为监督者** | 无 async overseer | SICA 2025: async overseer pattern | P1 |
| G89 | **单调改进保证 (多Agent)** | 无新 agent 上板保证 | MonoScale: trust-region memory + contextual bandit | P2 |
| G90 | **统一记忆基准套件** | 无标准化评测 | LoCoMo + LongMemEval + BEAM | P1 |
| G91 | **Rust agent 框架桥** | 纯自研 | ADK-Rust (22 crates) / GraphBit | P2 |

## 九、参考文献 (2026-06-22 更新)

### 2026年4-6月关键项目
1. **MOSS** (arXiv 2605.22794) — 源码级自我重写, 0.25→0.61 OpenClaw
2. **Ratchet** (arXiv 2605.22148) — 非发散性 self-evolving recipe, bounded cap + retirement
3. **Mem0 v3** (Apr 2026) — 单次ADD-only提取, 多信号检索, 91.6 LoCoMo, 58.9k★
4. **GAM** (arXiv 2604.12285, ICLR 2026) — 层级图记忆, EPG+TAN, 40.00 F1 LoCoMo
5. **MemForest** (arXiv 2605.23986) — MemTree 层级时间索引, 6x 吞吐
6. **HyperSpace** (arXiv 2604.15113) — VSA 空间编码模块化框架
7. **Anthropic Dreaming** (May 2026) — 异步 hippocampal 记忆巩固, API-first
8. **MonoScale** (arXiv 2601.23219) — 多Agent 单调扩展, trust-region
9. **SICA** (arXiv 2504.15228) — 自改进编码 agent, async overseer
10. **SafeEvalAgent** (2026) — 多Agent 自进化安全评估

### 论文 (补充)
- **arXiv 2604.16968** — On Safety Risks in Experience-Driven Self-Evolving Agents
- **arXiv 2604.15113** — HyperSpace: Spatial Encoding in Hyperdimensional Representations
- **arXiv 2605.22794** — MOSS: Source-Level Self-Evolution
- **arXiv 2605.22148** — Ratchet: Minimal Hygiene Recipe for Self-Evolving LLM Agents
- **arXiv 2604.12285** — GAM: Hierarchical Graph-based Agentic Memory
- **arXiv 2605.23986** — MemForest: Write-Efficient Temporal Memory
- **arXiv 2601.23219** — MonoScale: Monotonic Multi-Agent Scaling
- **arXiv 2504.15228** — SICA: Self-Improving Coding Agent
- **arXiv 2504.19413** — Mem0: Production-Ready Long-Term Memory
- **arXiv 2606.14629** — When Good Verifiers Go Bad (VLM regression in self-improvement)

### 综述
- **State of AI Agent Memory 2026** — Mem0, 10 approaches × 21 integrations
- **Awesome-Self-Evolving-Agents** — XMUDeepLIT, 266★
- **Awesome-AI-Memory** — IAAR-Shanghai, 1k★, 245 commits
- **Memory in the Age of AI Agents: A Survey** — Shichun-Liu, 1k★

### 生态工具
- **ADK-Rust** (zavora-ai) — 22 crates, model-agnostic Rust agent framework
- **GraphBit** (InfinitiBit) — DAG-based Rust agent orchestration
- **RIG** (0xPlaygrounds) — Rust LLM integration crate
- **ZeroicAI** — Agent-oriented programming in Rust (BDI + 8 patterns)
- **Omni** — Desktop AI agent builder, Rust workspace
