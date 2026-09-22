# NeoTrix Absorption Round 10: Compaction / Routing / Review / Skills

> **Absorption Date**: 2026-09-22 | **SIM**: SIM-44 | **Status**: Landed
> **Upstream**: R10-TRIAGE P0 短名单 8/8 已读。
> **This round: NTS-F12 (agentic workflow discipline)**. 条款预算 ≤2，用 1。

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-66 | fast-jev-compaction 6k★ (never-rewrite compaction; Jev-scored keep/drop vs τ=0.5; pin 首＋近 6 条；staged fitting 1000→200→60/head+tail/单行折叠；fallback 内建摘要；reduction<0.25 判不值；fake-Jev 单测) | Library | F03 overflow playbook 注 |
| S-67 | jev-ultrafast 16.2k★ (单请求/周期；结构化状态零截图；模型输出永不直达执行——非 selector/坐标/shell/JS；DONE 需独立验证；测量边界诚实声明) | Agent | E12 佐证＋eval 诚实注 |
| S-68 | jev-codex-router 181★ (per-turn 模型＋深度路由；bounded dossier vs canonical replay；fail-open＋kill-switch＋quota fallback；本地 JSONL＋反事实报告；phase-scoped lease；confidence≠成功率；凭据 hygien) | Router | SIM-43 D-3 佐证＋Confidence 注 |
| S-69 | jev-security-scan 10★ (双轮阈值 0.35 初筛→双 0.85＋conf 0.6＋active 0.7；永不执行目标 `-I -B`；覆盖缺口显式＋exit 1/2/3；阈值未校准声明；报告 0600) | Scanner | SEAL 对子候选值＋供应链注 |
| S-70 | awesome-hermes-agent 5.7k★ (maturity 标签 production/beta/experimental 快照＋复核；trust-boundary 五问；skills 即程序性记忆；meta-skills factory/motif/eval/curator 带 rollback；eagle-eye 五层路由＋"no match" 合法；RTK 压缩；DashClaw 审批层) | Directory | G10 佐证＋SEAL 输入＋triage 注 |
| S-71 | KaLM-Jev 28★ (本地 Choice/Score/Noul；reranker 无文本生成；127.0.0.1 无鉴权；输出未校准；checkpoint 含可执行 .py——仅信受信仓库；严格入参校验；LRU cache) | Engine | 本地 judge P-task＋供应链警告 |
| S-72 | Agent-Reach 84.4k★ (capability layer 非又一工具；每平台 首选＋备选 有序后端；真实探测非命令存在性；doctor 诊断＋修复处方；默认安全 install／dry-run／600 凭据；换代＝调序非重写；小号 Cookie 卫生) | Capability layer | 集成惯例注 |
| S-73 | jev-review 499★ (编排在代码，模型只做有界判断；Noul→画像→证据→机理→severity→条件路由；阈值＋策略即代码常量；层只向下依赖＋check 脚本；dashboard 127.0.0.1 永不 serving env；finding＝提示非定罪) | Workflow | NTS-F12 |

Rejected: 云端 judge 依赖（本地优先 P-task）；全量翻转（ADR-0004 三阶段 stands）；
H 类 14 项（triage 折叠）；新 vendor SaaS（无）。

---

## 2. Pattern → NeoTrix Mapping

### 2.1 Agentic workflow discipline (S-73＋S-68) → NTS-F12
编排在代码、模型只做有界类型化判断；阈值与路由策略是代码常量非提示词散文；
工作流层只向下依赖（脚本检查）；finding 永远是提示，晋升需独立验证。
S-68 的 dossier/replay 分离是同一原则的路由形态。

### 2.2 Bake notes（无条款）
- Compaction（S-66）→ F03 overflow playbook：never-rewrite＋pin＋阈值 keep/drop＋
  reduction 门（<0.25 不值）＋fallback 链；dispatch 日志体积控制用此形。
- Confinement（S-67）→ E12 佐证：模型输出永不直达执行层；DONE 必须独立验证
  （SDB C 列语义）；测量边界诚实声明并入 eval 惯例（AERS/R9）。
- Degraded mode（S-68）→ SIM-43 D-3 佐证：fail-open＋kill-switch 哨兵＋quota fallback；
  confidence 明确≠成功率（Confidence 枚举文档注）。
- SEAL 对子（S-69）：双轮阈值 0.35/0.85/0.6/0.7 列为 SEAL pair ledger 候选初值
  （未校准声明同样入库）；永不执行目标并入 R-P170/R9 rizin 注。
- 自进化闭环（S-70）→ G10 佐证：meta-skill＋evidence-backed＋rollback manifests
  即 rejected-edit buffer 的生产形态；trust-boundary 五问并入 SEAL 威胁模型输入；
  eagle-eye "no match 合法"并入 triage UNKNOWN 纪律。
- 本地 judge（S-71）→ P-task（成本/延迟/隐私三算 vs 云 Jev）；checkpoint 可执行代码
  警告并入供应链巡检（supply-iocs 语义扩展，P-task）。
- Capability layer（S-72）→ 集成惯例：外部工具集成声明有序后端＋真实探测＋doctor；
  默认安全 install＋dry-run；换代调序不重写。

---

## 3. Landings

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R10-1 | NTS-F12 in NT-STD 1.0.8 | clause present with Verify |
| L-R10-2 | SIM-44 record | row + §43 present |
| L-R10-3 | BLUEPRINT v1.6.9 changelog | line present |

Deferred: 本地 judge 引擎（P-task，成本/隐私评估）；supply-iocs 语义扩展（P-task）；
SEAL ledger 阈值校准（P-task L5）；R10 A–G 余项＋H 折叠（R11 候选）。

---

*End of Absorption Round 10*
