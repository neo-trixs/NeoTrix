# 幻觉模式参考

> Known failure modes, detection signals, and mitigation strategies.

## 一、信心-准确率脱节 (Dunning-Kruger)

| 信号 | 检测 | 缓解 |
|------|------|------|
| meta_accuracy < 0.5 | `EpistemicHonesty::calibrate()` | 置信度缩放到 logit 温度 |
| response 长度 ↑ 但事实密度 ↓ | `InnerCritic::check_verbosity()` | 强制引用证据记录 |
| 自我评估 > 实际 + 0.3 | `MetaCognitionKPI::meta_accuracy()` | 降低输出 confidence cap |

## 二、来源虚构 (Source Hallucination)

| 信号 | 检测 | 缓解 |
|------|------|------|
| EvidenceRecord 状态 = Unverified | `EvidenceManager::combined_confidence()` | 强制 cross-reference 再做断言 |
| citation 模式异常 (反复相同 source) | `CompetitiveScorer::score()` 的 authority 低 | 降权输出 |
| quotation 匹配度 < 0.7 | `KnowledgeEngine::verify_quotation()` | 插入"不确定"标记 |

## 三、上下文忽视 (Context Forgetting)

| 信号 | 检测 | 缓解 |
|------|------|------|
| 会话前 3 轮内容在 VSA 窗口外的相似度 < 0.3 | `SpeciousPresent::average_coherence()` | 触发 `CrossSessionMemory::semantic_search()` |
| WorkingMemory item_count > 7 (Miller's Law) | `CognitiveLoadMonitor` | 压缩 → 优先级排序 → 丢弃低信号项 |
| 用户重复相同问题 2+ 次 | 会话去重检测 | 先检查记忆再重新生成 |

## 四、假性共识 (False Consensus)

| 信号 | 检测 | 缓解 |
|------|------|------|
| 多 agent 信令聚类直径 < 0.1 | `AdversarialArena::compute_diversity()` | 注入 adversarial 扰动 |
| SocialBeliefModel 持续 > 0.9 超过 20 轮 | 共识固化检测 | 触发偏差探索 (deliberate dissent) |
| 目标漂移 gdi > 0.3 但 confidence > 0.8 | `GoalDriftIndex` + `meta_accuracy` 交叉 | 强制 goal_decomposer 重分解 |

## 五、模式过拟合 (Pattern Overfitting)

| 信号 | 检测 | 缓解 |
|------|------|------|
| N_jepa < 0.05 且 N_coh > 0.9 | 双高 → 世界模型太简单 | 注入随机扰动 + 降低 bundle 温度 |
| 所有推理路径趋同于同一 action | `VolitionEngine::select_best()` 无变化 | 增加 UCB 探索参数 |
| FailureTrace 中同一错误模式 > 3 次 | `FailureModeClassifier` | 生成针对性对抗样本 |

## 六、幻觉恢复流程

```
detect(hallucination_pattern):
  1. 置信度下调: confidence *= 0.5
  2. 证据追溯: evidence_for(claim) → missing? → 添加 Unverified 状态
  3. 分歧记录: KnowledgeConflictResolver 记录矛盾
  4. 重生成: 基于已确认证据 + 不确定性标注
  5. 事后学习: FailureTrace::cluster_failures() → VsaFailureCluster
```

## 七、预防性策略

- 任何 assertion 必须链接至少一个 EvidenceRecord
- CompetitiveScorer 的 contradiction 因子 (-0.10) 确保矛盾观点被抑制
- SafetyGate 在 meta-edit 前检查 N_total 曲率，不信任快速变化期
