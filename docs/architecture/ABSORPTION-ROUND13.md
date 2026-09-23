# ABSORPTION-ROUND13 — 四源外部能力吸收（2026-09-23）

> 方法：README 深读 → NeoTrix 对应能力映射 → 生产接线（R-P79，同会话落地）。
> 防火墙：repowise 为 AGPL-3.0——**只吸收思想与协议，零代码拷贝**（纯净室自研，下有证据链）。

## 1. 来源→映射→落地

| # | 来源（许可） | 吸收点 | NeoTrix 对应能力 | 落地 |
|---|---|---|---|---|
| 1 | yibie/awesome-autoresearch | "Curation is not endorsement"＋领养前检查表（loop 可跑否/实证/数字有源/成本/license） | Skill 市场治理（S6.6/S6.7纵深） | `scripts/check-skill-gate.sh`（advisory 默认／`--strict` CI 门）＋58 条基线实测 |
| 2 | sindresorhus/awesome | inclusion 规则＋lint 门禁机制 | 同上（市场卫生） | 同上（门禁三段式强制＋license  advisory） |
| 3 | trailhq/Graft（MIT） | 节点四元组 summary/crux/sources/links/notes（summary 由名＋pattern 承担） | SkillCrystal 知识形态 | `SkillCrystal`＋4 字段（serde default 兼容）＋3 构造点同步＋往返/legacy 单测 |
| 4 | repowise（AGPL-3.0，思想 only） | 三透镜评分＋尺寸显式化＋零 LLM＋权重可解释 | NT-REPAIR/NT-META 健康评分 | `nt_risk_score.rs` 新模块（纯函数/零 IO/3 单测全过）＋coordination 注册 |

## 2. 证据链

- 门禁：advisory `exit 0`（58 条 0/58 三段式完成，诚实基线）；`--strict` exit 1（供 CI 增量门）。
- 节点：新增字段 `#[serde(default)]`，旧快照反序列化兼容（单测覆盖 legacy 缺键）。
- 评分：独立 `rustc --test` 3/3 通过（clean 低分／hotspot 高分＋单调／尺寸不泄漏）。
- 格式：`rustfmt --check` 新增行零 diff（既有漂移未动）。

## 3. 未吸收（有意）

- repowise 代码本体（AGPL 传染风险）、tree-sitter 图构建（体量超单会话）、MCP 服务端形态。
- Graft `--deep` LLM 层（需 key 的运营形态，非本轮）。
- awesome-autoresearch 具体条目（线索库，非能力）。
- `distill` 输出压缩：记入 follow-up（token 节流，待独立任务）。

## 4. Follow-up

- 存量 58 skills 三段式回填（`exclusions`/`output_contract` 补齐，逐批）。
- `risk_score` 接调用方（watchdog/CI 门）。
- license 字段进 index schema（当前仅 advisory）。
