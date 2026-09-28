# 外部项目架构吸收 — 2026-09-28

> 调研 8 个项目，提取可吸收模式。**本文件只记结论与裁决，不记调研过程。**
> 口径：`核心代码` = 已接线并在生产生效；`门` = 已接线但 advisory；
> `未落地` = 已取证待做。**「导出 ≠ 调用」已错过多次，故每条都标状态。**

## 一、已落地（本会话接线）

| 来源 | 模式 | 落点 | 状态 |
|---|---|---|---|
| 官方 Agent Skills 规范 | 路径式 key 解析（`<cat>/<skill>` 优先，裸名兜底） | `skill_loader.rs:resolve_from_index` | **核心代码** 13/58→58/58 |
| 官方 Agent Skills 规范 | 调用策略 `disable-model-invocation` / `user-invocable` 正交两维 | `SkillInvocationPolicy` + `SkillEntry` 两字段 | **核心代码** 4 测试 |
| 官方 Agent Skills 规范 | frontmatter 是自动发现依据 | `check-skill-gate.sh` 文件侧校验 | **门**（41/58 欠账，advisory） |
| Understand-Anything | worktree 数据目录重定向（数据别写在会被销毁的 worktree） | 待接 `nt_worktree_gate.sh --detect` | **未落地** |

## 二、核心结论：不要「把 sh/py 改写成 Rust」

调研的核心发现是**病根不在语言，在接线**：

- `skill_validator`（L6，789 行，30+ 规则）**零消费者** —— 真需要的是接线
- `check-skill-gate.sh` **从未接 CI 也不在 Makefile**
- `nt_locate.py` / `nt_mapgen.py` 与 `nt_core_code_search.rs`（670 行，含 RRF 融合）
  **强重复且 Rust 更强** —— 这个才该合并
- 但 `nt_lock_audit.py` **零对应物**（Rust 生态的 clippy lint 覆盖不到
  块作用域自死锁），**不该重写**

`scripts/ops/` 里 10 个 playwright/faster-whisper 脚本（douyin/deepseek/doubao）
在 Rust 里**没有等价形态**（需 headless Chrome + CDP 拦截 / CTranslate2），
改写成本远高于收益。

## 三、明确不吸收（会冲突或重复）

| 模式 | 来源 | 不吸收的理由 |
|---|---|---|
| 自动架构层推断 | Understand-Anything `architecture-analyzer` | **冲突**。`.neotrix/layer-map.json` 是层归属显式真源，让 LLM 猜层会覆盖人工裁决。只取「注入 previous layers 保命名一致」 |
| logo skill 几何规范 | logo-design-skill | **冲突**。`viewBox 256` / 精确角度 与 `des/branches/icon-design` 的 24×24 网格 / 1.6px 笔画 / 浅金渐变不同源 |
| BM25 `history_search` | Horizon | **重复**。`nt_crystal_core` 是更强机制；Horizon 的价值只是「零依赖」这个约束 |
| 「AGENTS.md 只写操作、~60 行」 | ralph-playbook | **冲突**。门记录内联在 AGENTS.md 里**正是 R-SCAN-3 生效的原因**。正确做法是拆分而非清空 |
| 策展门「策展≠背书」 | awesome-autoresearch | **已吸收**（`check-skill-gate.sh:3-6`），不要重复 |
| 「模型可见 ⇒ 必须已记日志」 | deepseek-harness | **已吸收**（`EVOLUTION-ROADMAP:322` §4.4） |

## 四、待落地（按价值排序，均已取证）

| 优先 | 模式 | 来源 | 建议落点 |
|---|---|---|---|
| 高 | **单调 guard**（guard 返回类型无 allow ⇒ 装载顺序无法把拒绝翻回允许）+ `ToolRestriction` **求交** + 模型可见字段**显式白名单** | deepseek-harness `docs/subsystems/tools.md` | `l5_cognition/nt_core_gate/nt_tool_registry.rs` |
| 高 | **Context Manifest**（`context_fingerprint` + `resolved_endpoints` + 被排除候选的原因） | aliyun §15.9.5 | `l5_cognition/nt_core_context/` + `.neotrix/context-manifest.json` |
| 高 | **Skill 六阶段准入**（来源核实/完整性/静态检查/隔离验证/人工评审/内部发布）+ `draft→reviewing→online→offline` 且 **online 版本不可变** | aliyun §15.2.4-6 | 扩展 `check-skill-gate.sh` + `index.json` 加 `status` |
| 中 | **权威任务状态机**（10 态，每个态显式允许的下一步；WAITING 不是失败、PAUSED 不是结束）+ 预算耗尽要**明确终态**而非悄然截断 | aliyun §4.1.2 | `l5_cognition/nt_core_gate/nt_types.rs` |
| 中 | **五级验证阶梯** + 「模型只能申请完成，Harness 才能提交完成」提交权分离 | aliyun §4.6.2 | `l5_cognition/nt_core_gate/nt_judge.rs` |
| 中 | **阶段门禁**：每阶段必须回答「输出是什么、证据在哪里、谁来确认」 | aliyun §4.2.2 | `AGENTS.md` 三道闸的升级形态 |
| 中 | **相关性 ≠ 治理资格**；过滤在分页之前（避免页数泄露不可见资源） | aliyun §15.8.3 | KB 检索与能力发现的排序位约定 |
| 中 | **增量 4-action 状态机**（SKIP / PARTIAL_UPDATE / ARCHITECTURE_UPDATE / FULL_UPDATE）+ 结构指纹 + 符号丢失门（只允许 1 次定向重试） | Understand-Anything | `.neotrix/graph/` + 增量脚本 |
| 中 | **有界 handoff 的 fresh-agent 循环**（模型只供数据，脚本由部署拥有；非法报告**让 workflow 失败**而非截断） | deepseek-harness `ralph` | `l5_cognition/nt_agent/` |
| 中 | **rank + scope 层的 skill 注册表**（nearest layer wins outright，rank 只做层内 tiebreak） | deepseek-harness `docs/subsystems/skills.md` | `skill_registry.rs` + `skill_loader.rs` |
| 中 | **Checkpoint 协议**：重活在人工确认之后（logo skill Phase 6 强制停，Phase 7 占 90% 工作量） | logo-design-skill | `skills/SKILL-SPEC.md` |
| 低 | **Profile 四文件包** + strict schema + prompt 路径不得逃逸目录 + 策略/偏好正交分离 | Horizon | `skills/*/profiles/<domain>/` |
| 低 | **规则是循环不是标签** —— 结构性证据探针（词在树里但不在 README；引用也算证据） | awesome-autoresearch | 增补 `nt_locate.py` 的 grep 定点清单 |
| 低 | **反脚手架审查**（一个模板套多个仓库不会让任何一个更完整；共享发布日期是**风险信号不是势头**） | awesome-autoresearch `CONTRIBUTING.md` | `AGENTS.md` 并行公约 |
| 低 | **「画了必须看」**（在 SVG 里画就是盲画；跑没跑的测试要诚实声明） | logo-design-skill | `SKILL-SPEC.md` Failure Modes |

## 五、两条可直接抄的纪律

- **相关性分数不是安全评分**（aliyun §15.8.2）：检索得分只表达「与任务匹配程度」，
  不代表质量保证。过滤（治理资格）应在**分页之前**。
- **Prompt 不能替代权限模式**（aliyun §4.2.3）：只有权限模式、工具白名单、持久
  状态与 HITL 共同生效，系统才真正具备「计划获批前不可写」的约束。

## 六、调研中被证伪的结论（留档防重犯）

调研报告称「skills/ 下所有 skill 因文件名不是 `SKILL.md` 而在所有兼容 agent
里不可发现」。**实测证伪**：`find skills -name SKILL.md` = 67 个，
`index.json` 的 59 条路径**零缺失**。真实的问题是 **frontmatter 只有 24/67**
——那才是自动发现的判据，已接进门。
