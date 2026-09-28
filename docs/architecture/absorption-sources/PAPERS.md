# 吸收的 arXiv 论文（2026-09-28 实测存在）

> ⚠️ **arXiv API 当日被封**（对照测试：已知真实 ID `1706.03762` 同样返回空），
> 故标题/摘要取自 `arxiv.org/abs/` 页面。**这是环境限制，不是论文不存在。**

| arXiv ID | 标题 | 与 NeoTrix 的关系 |
|---|---|---|
| `2609.30199` | **ExplorationBench: Measuring AI Systems' Exploration in Verifiable Alien Worlds** | ⭐ **最有价值**。AlienCode(31 发现目标/70 任务) + AlienLogic(24/70)。**规则可执行 ⇒ 每个答案可精确校验；且与既有知识冲突 ⇒ 纯回忆无法解题** —— 正好解决「怎么证明能力真的学会了」。10 个系统评测：最强系统能获取并应用陌生规则，但**跨轨迹方差大，持续探索会停滞甚至逆转早期增益** |
| `2609.19644` | **ScientistTwo: Pioneering the Human Knowledge Frontier with Autonomous AI** | 全自主多 agent 科学发现框架：建 SOTA 基线 → 提假设 → 编排专职 agent → **闭环模拟同行评审反驳引擎** → 自动化消融。对 `l6_meta/evolution` 的「autoresearch 式循环 + 审稿门」有直接参考 |
| `2609.22175` | **Contrastive World Models** | 基于 Dreamer，用 Deep InfoMax 类下界**取代像素重建**目标 —— 状态表征保留「对未来可预测」的信息，**不重建视觉无关细节**。带干扰物/自然视频背景时显著优于 Dreamer。⇒ 对 `l5_cognition` 世界模型降噪有方法论价值 |
| `2606.19357` | **Physical Atari**（Robotroller + Atari Devbox） | 实体 RL 平台，<$1000 3D 打印件。关键结论：**学习与部署间小分布偏移就会显著降低策略性能** ⇒ 支撑「必须做 on-device 适应」。对 `crates/neotrix-game` 的 sim2real 有参考 |
| `2609.29098` | Evaluation-efficient quantum architecture search with ZX-calculus | ⚠️ **量子架构搜索，与 NeoTrix 无直接关系** —— 用户清单里混进的无关项，仅存档 |

## 可落地的两条

1. **E-1 证伪门可直接借 ExplorationBench 的思路**：让「能力真的学会了」变成可机器校验的
   —— 不是让 agent 自我声明（prime-agent 的 `RefinementEvent.outcome` 就是自由文本），
   而是**规则可执行 + 与既有知识冲突**，使纯回忆无法蒙对。
2. **持续探索会停滞甚至逆转** ⇒ `l6_meta/evolution` 的自进化循环必须有**棘轮**：
   变差即 `git reset`（`autoresearch` 的做法），且**负面结果也要提交入库**作为信号。
