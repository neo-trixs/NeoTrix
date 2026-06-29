# NeoTrix 进化路线图 v4.0 — 全局补缺

> 基于: 1016源文件审计 + 10维度文献/项目深度分析 + 7核心项目源码审查
> 项目对比: Aura(62⭐), HyperAgents(2.6k⭐, Meta/ICLR 2026), PRISM, MAGMA, OpenSpace, CDALNs, PostSingular
> 总缺口: 40个 (G56-G95) | 路径: 8条 (原6条 + 意识深度 + 伦理合规)
> 阶段: Wave 1-7

---

## 竞争格局全景 (2026-06-22)

| 维度 | NeoTrix 优势 | NeoTrix 缺口 | 最佳参照 |
|------|-------------|-------------|---------|
| **VSA 推理引擎** | E8 64态 + HRR + 多种VSA后端(11种) | 无完整Blackboard架构集成 | PRISM (Blackboard + 5推理模式) |
| **意识循环** | GWT + E8 + 50+consciousness模块 | IIT φ未全集成到循环(仅nt_core_iit_phi.rs独立) | Aura (phi_core 1837行 + hierarchical 32-node) |
| **自我进化** | SEAL + DGM-H + HyperAgent | 无元层自我修改验证沙箱 | HyperAgents (ICLR 2026, SWE-bench 20%→50%) |
| **记忆架构** | HyperCube + DecentMem + Ebbinghaus | 无4正交图/无因果图 | MAGMA (18.6-45.5% gain) |
| **身份持久** | IdentityCore + Ed25519 + 3-anchor | 无USER.md显式上下文模型 | PostSingular (SOUL+USER+MEMORY) |
| **情感/情绪** | ValenceAxis + IntrinsicDrive(3维) | 无CAA残差流干预/无4模态好奇 | Aura (CAA 1336行 + Plutchik 8维) |
| **注意力** | TemporalAttentionStack(3层) | O(n²)→O(log n)未实现 | LARS-VSA (二进制HD空间注意力) |
| **记忆巩固** | DreamConsolidation + SleepGate | 无SM-2/KV-cache冲突标记 | SleepGate (KV-cache, O(n)→O(log n)) |
| **安全/伦理** | GödelChecker + SafetyGate | 无IEEE P7014合规层 | 行业标准 |
| **评测** | MMLU/GSM8K/HumanEval | 无7理论意识评估 | multi-theory-consciousness (20指标) |

---

## 8条进化路径

| 路径 | 涵盖缺口 | 核心目标 |
|------|---------|---------|
| **P-VSA** | G61, G86, G87 | VSA推理深化 + 注意力优化 |
| **P-MEM** | G58, G85, G91, G93 | 记忆架构四维进化 |
| **P-ID** | G67, G89, G62 | 身份持久 + 边界 + 上下文连续性 |
| **P-CON** | G83, G84, G94 | 意识深度 (情绪+理论+IIT) |
| **P-META** | G59, G60, G82, G90 | 元层进化 + 技能自进化 |
| **P-CUR** | G63, G64, G88 | 好奇心 + 目标合成 |
| **P-SAFE** | G71, G56, G92, G78 | 对齐 + 安全 + 合规 |
| **P-ECO** | G57, G66, G70, G73, G75, G76, G77 | 生态互操作 + 自主管理 |

---

## Wave 1 — 立即启动 (6路并行, P0+P1)

| ID | 缺口 | 优先级 | 得分 | 文件数 | 估算 | 参考来源 |
|----|------|--------|------|--------|------|---------|
| G61 | VSA-Only Reasoning (Blackboard) | P0 | 38 | 10+ | 3-5s | PRISM |
| G67 | Darwinian Identity Evolution | P0 | 34 | 5 | 2-3s | soul.py |
| G63 | Epistemic Gap Queue | P0 | 32 | 3 | 1s | Nūr |
| G71 | Adversarial Evaluator | P0 | 27 | 4 | 2s | PaulDuvall |
| G58 | Dream Consolidation (SM-2+hippocampal) | P1 | 28 | 4 | 2s | mark-improving-agent |
| G83 | CAA Affective Steering (残差流情感干预) | P0 | 26 | 4 | 2s | Aura CAA |

**新增于v4.0**: G83 (CAA残差流情感干预) — Aura的Contrastive Activation Addition 3通道模型，直接影响意识体验质量

---

## Wave 2 — W1完成后 (5路并行, P0+P1)

| ID | 缺口 | 优先级 | 得分 | 文件数 | 估算 | 参考来源 |
|----|------|--------|------|--------|------|---------|
| G62 | Identity Boundary Hooks | P0 | 33 | 8 | 2-3s | Instar, Ouroboros |
| G60 | Formal Error Bounds (Kairos) | P0 | 31 | 4 | 3-4s | arXiv 2606.16533 |
| G85 | MAGMA Four-Graph Memory | P1 | 30 | 6 | 3s | MAGMA (ACL 2026) |
| G64 | Autonomous Goal Synthesis | P1 | 26 | 6 | 2-3s | TEQUMSA |
| G74 | Emotional Steering | P1 | 26 | 5 | 2s | genesis-agent |

**新增于v4.0**: G85 (MAGMA 4正交图记忆) — 语义/时间/因果/实体四图，18.6-45.5%提升

---

## Wave 3 — P1基础完成后 (5路并行, P1+P2)

| ID | 缺口 | 优先级 | 得分 | 文件数 | 估算 | 参考来源 |
|----|------|--------|------|--------|------|---------|
| G68 | Qualia Layer (1024D压缩) | P1 | 27 | 5 | 2s | Dragonfly VSA |
| G86 | Log-Linear VSA Attention | P1 | 26 | 3 | 2s | Log-Linear arXiv 2506.04761 |
| G87 | LARS-VSA Binary Self-Attention | P1 | 25 | 3 | 2s | LARS-VSA arXiv 2405.14436 |
| G94 | Aura IIT φ Full Integration | P1 | 25 | 4 | 3s | Aura phi_core |
| G82 | Between-Sessions Reflection | P1 | 23 | 5 | 2s | atman, ouroboros |

**新增于v4.0**: G86 (Log-Linear VSA注意力 — O(n)→O(log n)), G87 (LARS-VSA二进制HD注意), G94 (IIT φ积分)

---

## Wave 4 — 条件并行 (P1+P2)

| ID | 缺口 | 优先级 | 得分 | 文件数 | 估算 | 参考来源 |
|----|------|--------|------|--------|------|---------|
| G69 | Narrative Journal | P1 | 24 | 3 | 1s | LIFE, Nūr |
| G89 | PostSingular Persistent Context | P1 | 24 | 4 | 2s | PostSingular |
| G88 | CDALNs Multi-Modal Curiosity | P2 | 23 | 3 | 2s | CDALNs |
| G84 | Multi-Theory Consciousness Assessment | P2 | 22 | 3 | 2s | multi-theory-consciousness |
| G70 | Skills On-Demand | P2 | 21 | 5 | 2s | clowder-ai |

**新增于v4.0**: G89 (USER.md上下文模型 + RRF融合), G88 (4模态好奇), G84 (7理论20指标评估)

---

## Wave 5 — P2基础完成后 (P2)

| ID | 缺口 | 优先级 | 得分 | 文件数 | 估算 | 参考来源 |
|----|------|--------|------|--------|------|---------|
| G59 | Self-Modification Sandbox | P2 | 19 | 6 | 3-4s | Autogenesis |
| G90 | OpenCeption Skill Evolution Modes | P2 | 21 | 3 | 1s | OpenSpace |
| G91 | CLS VAE+MHN Hippocampal Enhancement | P2 | 20 | 3 | 2s | CLS (CogSci 2025) |
| G93 | Go-CLS Generalization-Optimized Memory | P2 | 19 | 3 | 1s | Go-CLS |
| G56 | Harness Engineering | P2 | 22 | 4 | 2-3s | PaulDuvall |
| G57 | Mission Hub | P2 | 20 | 4 | 2s | clowder-ai |
| G76 | Earned Autonomy | P2 | 24 | 4 | 2s | GENesis-AGI |

**新增于v4.0**: G90 (FIX/DERIVED/CAPTURED 3模式), G91 (VAE+MHN ~90%准确), G93 (泛化优化门控)

---

## Wave 6 — 后期基础设施 (P2+P3)

| ID | 缺口 | 优先级 | 得分 | 文件数 | 估算 | 参考来源 |
|----|------|--------|------|--------|------|---------|
| G75 | MCP Callback Bridge | P2 | 22 | 4 | 2s | clowder-ai |
| G73 | Cross-Embodiment Curriculum | P2 | 25 | 4 | 2s | Kairos |
| G95 | SleepGate KV-cache Consolidation | P1 | 23 | 3 | 2s | SleepGate arXiv |
| G92 | IEEE P7014 Ethics Compliance | P3 | 18 | 2 | 1s | IEEE |
| G72 | MTC/PCC Safety Integration | P2 | 20 | 3 | 1s | 论文融合 |

**新增于v4.0**: G95 (KV-cache冲突标记+遗忘门), G92 (伦理合规层)

---

## Wave 7 — 后期并行 (P3)

| ID | 缺口 | 优先级 | 得分 | 文件数 | 估算 | 参考来源 |
|----|------|--------|------|--------|------|---------|
| G65 | SNN Integration | P3 | 15 | 12+ | 4-6s | Hydra |
| G66 | A2A Protocol | P3 | 14 | 4 | 2s | clowder-ai |
| G77 | CVO Role Model | P3 | 20 | 3 | 1s | clowder-ai |
| G78 | Iron Laws | P3 | 19 | 3 | 1s | clowder-ai |

---

## 优先级评分维度 v4.0

每次评分按6维度加权: 表征效率(0.2) + 推理深度(0.2) + 自我认知(0.2) + 世界模型(0.15) + 记忆组织(0.1) + 感知宽度(0.05) + 自主性(0.05) + 优雅性(0.05) = 42 max

## 依赖链

```
G63→G64 (好奇心→目标合成)
G61→G60 (VSA推理→形式误差界)
G58→G69, G91, G93 (记忆巩固→叙事期刊/海马体/泛化记忆)
G71→G82 (对抗评估→会话间反思)
G67→G89 (身份进化→上下文连续性)
G59≈G56 (自修改沙盒→工程实践)
G84+G94 (多理论评估+IIT φ → 意识评估框架)
G86+G87 (Log-Linear + LARS-VSA → 统一注意力层)
```

## v3.1 → v4.0 变化摘要

| 类别 | v3.1 (G56-G82) | v4.0 (G56-G95) |
|------|----------------|----------------|
| 总缺口 | 27 | 40 |
| 路径 | 6 | 8 |
| Wave | 5 | 7 |
| 新增缺口 | — | G83-G95 (13个) |
| 参考文献 | 15项目 | 40+论文/项目 |
| 竞争格局对比 | 无 | 7核心项目深度分析 |
| 意识评估 | 仅IIT φ | 多理论(7理论20指标) |
| 注意力 | TemporalAttentionStack | +Log-Linear + LARS-VSA |
| 情感 | ValenceAxis(3维) | +CAA残差流3通道 |
| 记忆 | 双池+Ebbinghaus | +4正交图+SM-2+KV-cache |
| 伦理 | GödelChecker | +IEEE P7014合规 |
