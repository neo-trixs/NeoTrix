# SIM-first 协议：先模拟推演、补齐漏洞、再执行

> **版本**: 1.0.0 | **日期**: 2026-09-21 | **状态**: 强制 (Mandatory for P1+)
> **血统**: Gary Klein premortem（前视性后见，HBR 2007）+ XP Architectural Spike
> （定框调查，证据收尾）+ Spike-first-ADR-second（Question→Spike→Evidence→Decision→ADR）。
> NeoTrix SIM = 三者合一：以 premortem 姿态提问，以 spike 手段探查，以 ADR 落点关闭。
> **配套**: BLUEPRINT D-14（本协议的图）、D-03（SIM-preview 状态）、dev-rules R-P241–R-P245。

---

## §1 原则

1. 不确定性用证据购买，不用决心硬扛。凡跨层、新门禁、新依赖、新工具，先 SIM 后动手。
2. SIM 只读不写：探针限 rg/读码/跑只读脚本（check 类、--dry-run 类），不动生产代码。
3. Spike 产出即抛：SIM 中的临时命令、草稿计数不入库，入库的只有登记表行项与落点。
4. 无 tripwire 的缺口等于没发现：每个缺口必须有 Owner + 可观测信号 + 日期。
5. SIM 不关闭任务，只决定任务能不能开：放行三态 GO / GO-WITH-MITIGATION / NO-GO。

## §2 何时必须 SIM（触发器）

| 触发 | 示例 | 豁免 |
|------|------|------|
| 跨层改动 | L1 消减越层、类型下沉 L0 | 纯本层内部重命名 |
| 新增门禁/适应度函数 | P1-04 五函数、覆盖率阈值 | 现有门阈值微调（仍需记录） |
| 新增依赖/工具 | geiger/machete/benchmark-action | patch 级小版本升级 |
| 新流水线/新 Crate | 搜索统一、新 bounded context | 文档 typo、注释补齐 |
| SDB 相关 | 新 LLM→动作点、verifier 变更 | prompt 文案微调（仍走 SDB 登记） |

## §3 SIM 记录格式（九格，缺一格不算完成）

```
SIM-ID:    SIM-NN（顺序分配，登记表唯一）
任务:      ROADMAP/BLUEPRINT 中的任务号 + 一句话
认领/时间盒: Owner + ≤4h（任务级）；超时必须拆 SIM，不延长
探针:      跑了哪几条只读命令 / 读了哪几个文件行段
证据:      行号 + 数字 + 文件路径（无"我觉得"）
缺口:      按影响排序，每条 G-NN
外部方案:  先搜再造；搜不到注明"自研"，给出否决掉的备选
落点:      脚本/登记表/门禁/ADR，精确到路径
蓝图回写:  改了哪张图哪张表（D-NN + §）
放行判定:  GO / GO-WITH-MITIGATION（附条件）/ NO-GO（回 Design）
tripwire:  每缺口一行：信号 + 日期 + Owner
```

## §4 时间盒与纪律

- 任务级 SIM ≤ 4h；到点即停，未探明记为缺口（UNKNOWN 也是有效输出）。
- 探针白名单：rg/read 脚本、check 类脚本只读运行、cargo metadata/check -n 类；
  黑名单：写代码、改 CI 执行态、装生产依赖。
- 连续两个 NO-GO 的任务升级：整组 spike（半天）或请外部输入，不许第三次裸冲。

## §5 登记表

> 登记铁律：新行一律末尾追加，禁止锚点前插（前插已造成三次错序：SIM-13/14/15/17）。
> 每次追加后用序号排序脚本机验。

| ID | 任务 | 判定 | 落点 |
|----|------|------|------|
| SIM-01~08 | 蓝图 v1.1.0 §15 批量模拟 | GO-WITH-MITIGATION | BLUEPRINT §15-§17 + Makefile + 双脚本 + SDB-REGISTRY |
| SIM-09 | P1-04 五个新适应度函数编码（本文件 §6 试跑） | GO-WITH-MITIGATION | 见 §6 落点行 |
| SIM-10 | 机制能力第二轮补强（本文件 §8） | GO | ABSORPTION-ROUND2 + build-surface 脚本 + ADR 模板 + SDB v0.2 + R-P246-249 |
| SIM-11 | 机制能力第三轮补强（本文件 §9） | GO | ABSORPTION-ROUND3 + fuzz-ready 脚本 + R-P250-253 |
| SIM-12 | 机制能力第四轮补强（本文件 §10） | GO | ABSORPTION-ROUND4 + release.yml 路径修复 + R-P254-257 |
| SIM-13 | 规则熔炼为 NT-STD 1.0 标准版（本文件 §11） | GO | docs/standards/NEOTRIX-STD-1.0.md + Annex A/B/C |
| SIM-14 | 旧规则归档+新模板套件切换（本文件 §12） | GO | archive + templates + SKILL 指针 |
| SIM-15 | 登记表时序重排+Dependabot补cargo+deny三重奏复核（本文件 §13） | GO | dependabot.yml cargo 项 |
| SIM-16 | 意识指导为先：宪法加载器接正典+旧引用替换+优先级教义（本文件 §14） | GO | constitution 代码 + NT-STD 1.0.1 |
| SIM-17 | 旧口径清零：宪法注释/SKILL教学换正典（本文件 §15） | GO | 注释换装 + 11/11 单测 |
| SIM-18 | 全图审计重绘 D-00/D-02/D-04/D-05＋新增 D-15（本文件 §16） | GO | BLUEPRINT v1.3.0 |
| SIM-19 | 过期信息清理：TODO 基线挂 STALE＋占位符盘点（本文件 §17） | GO | TODO 横幅（删 0 行代码） |
| SIM-20 | P0 门 override 提交（本文件 §18，用户已批复） | GO-WITH-OVERRIDE | ADR-0001 + 双 commit |
| SIM-21 | cli升级残留审计：零悬空＋他人4错归属＋脚本位修复（本文件 §19） | GO | chmod×2（代码改动0行） |
| SIM-22 | 经验吸收＋全量总验＋收尾（本文件 §20） | GO | SESSION-ABSORPTION + sweep |
| SIM-23 | 核心建议修复轮：R-P打捞19+1＋4错证灭＋hook -j4（本文件 §22） | GO | RECOVERY annex + check 绿 |
| SIM-24 | 全域审计＋第五轮吸收＋蓝图 v1.4.0（本文件 §23） | GO | ROUND5 + NTS 1.0.3 |
| SIM-25 | 第六轮吸收：API/文档/Judge/混沌（本文件 §24） | GO | ROUND6 + NTS 1.0.4 |
| SIM-26 | 蓝图实施影响面分析（本文件 §25） | GO | IMPACT-ANALYSIS + P1-03 修订 |
| SIM-27 | A组收尾＋B1合入CI见证（本文件 §26） | GO-WITH-CI-BACKSTOP | ADR-0002 |
| SIM-28 | SDB调用链追踪收敛＋B1验证环境战（本文件 §27） | GO-WITH-CI-BACKSTOP | SDB v0.4 |
| SIM-29 | 提交清场＋SDB-06/07定级＋R-P111指针落地（本文件 §28） | GO | SDB v0.5 |
| SIM-30 | 门禁复测零进展＋audit-all聚合（本文件 §29） | GO | Makefile |
| SIM-31 | B2守卫编码＋14/14实跑绿＋死守卫发现（本文件 §30） | GO | fitness.rs |
| SIM-32 | 有序执行清单 EQ-01~20（本文件 §31） | GO | EXECUTION-QUEUE |
| SIM-33 | 提交清场＋暂存污染排查＋标记清扫（本文件 §32） | GO | 7b044f42 |
| SIM-34 | EQ-01/02/03闭环＋ADR-0003＋死守卫退役决议（本文件 §33） | GO | ADR-0003 |
| SIM-35 | EQ-04执行：死守卫删除＋13/13实跑绿（本文件 §34） | GO | fitness.rs |
| SIM-36 | EQ-05执行：H-08改直引＋覆盖丢失事故＋即交原则（本文件 §35） | GO | search.rs |
| SIM-37 | 第七轮吸收：旗标/签名/行为扫描（本文件 §36） | GO | ROUND7 + NTS 1.0.5 |
| SIM-38 | EQ-06执行：TaskType改指正典＋playback递延＋lib门绿（本文件 §37） | GO | bank |
| SIM-39 | 第八轮吸收：策略/持久化/WASM（本文件 §38） | GO | ROUND8 + NTS 1.0.6 |
| SIM-40 | EQ-07部分执行：门面/正典改指5处＋结构项递延（本文件 §39） | GO | bank/hive |
| SIM-41 | 第九轮吸收：技能/审议/防御/路由（本文件 §40） | GO | ROUND9 + NTS 1.0.7 |
| SIM-42 | EQ-08~12＋T3 并行执行轮（本文件 §41） | GO-WITH-MITIGATION | SDB v0.6 + hook |
| SIM-43 | P1-02 门面 API 设计：dispatcher 注入缝（本文件 §42） | GO (设计) | API 草案 |
| SIM-44 | 第十轮吸收：compaction/路由/安全/技能（本文件 §43） | GO | ROUND10 + NTS 1.0.8 |
| SIM-45 | 第十一轮吸收：meta-skill/编排/MCP/桌面（本文件 §44） | GO | ROUND11 + BP v1.6.10 |
| SIM-46 | 第十二轮吸收：余项扫尾（本文件 §45） | GO | ROUND12 + BP v1.6.11 |
| SIM-47 | Phase 2 dispatcher 代码实施（本文件 §46） | GO-WITH-CI-BACKSTOP | dispatcher E1a-E6 |
| SIM-48 | 调用方补齐＋E1b 证伪＋Phase 3 探针（本文件 §47） | GO-WITH-CI-BACKSTOP | entry 注入＋下沉位 |
| SIM-49 | 并行收口：双路探针＋点火清单（本文件 §48） | GO | IGNITION-CHECKLIST |
| SIM-50 | CI 侦察：公开 API 归因（本文件 §49） | GO | 红基线清单 |
| SIM-51 | Phase 3 类型迁移提案（本文件 §50） | GO (proposed，待 Architect 签) | 下沉三决 |
| SIM-52 | M1–M7 执行（ADR-0005/SEAL/F03/Footer/iocs/H抽验/fuzz/复验） | GO | ADR-0005＋fuzz 骨架 |

## §6 SIM-09 试跑：P1-04 五个新适应度函数编码

> 目的：验证本协议可执行。只预演，不编码（编码是 Implement 态的事）。

**探针（只读，已跑）**：
- P-1：读 `nt_core_arch_fitness.rs:1-100`（守卫模式：struct + `impl SelfTest` + `name()/self_test()`，纯只读扫描，违规返 Err 明细）。
- P-2：读同文件 `:640-732`（`arch_fitness_tests()` 返回 `Vec<Box<dyn SelfTest>>`，
  注册即生效；tests mod 有 6+ 单测；先例
  `test_core_boundary_allowlist_covers_all_reverse_deps`——守卫必须在现状下通过，
  否则接线自身违规）。
- P-3：读 `l0_substrate/nt_core_self_test.rs`（SelfTest trait 位置，P1-05 的 Confidence 类型尚不存在——ABSORPTION 4.4 为设计稿）。
- P-4：跑 `check-doc-drift.sh`（基线 111，DocDriftFitness 若按"零缺文档"写必红）。

**证据**：
- E-1：7 守卫全部经 `arch_fitness_tests()` 一个 vec 注册（645-655 行），新增=加 struct + push 一行。
- E-2：PanicDensity 有阈值先例（3000 绝对 / 12.0 每千行），其余守卫多为 0 容忍 + allowlist。
- E-3：Confidence 类型零代码（仅设计稿），doc-drift 缺口 111 为实数。

**缺口**：
- G-1（BLOCKER）：ConfidenceLabelFitness 依赖 P1-05 的 Confidence 类型——顺序反了，先 P1-05 后编码。
- G-2（HIGH）：Pipeline/DocDrift/Security/Responsibility 四函数缺可操作定义
  （何为"阶段"？何为"职责"？阈值多少？）。
- G-3（HIGH）：新守卫在现状下必红（doc 111 precedent）→ 必须沿用 CoreBoundary allowlist 模式
  （allowlist + 过期日），否则 CI 自杀。
- G-4（MED）：SelfTest 需引 `l0_substrate::nt_core_self_test` + log 宏，跨层引用合规（L5→L0 允许）。

**外部方案**：Spike-first-ADR-second（本次 SIM 即 spike）；allowlist 模式内部已有先例，
不另搜；阈值定义参考 PanicDensity（绝对数 + 密度双轨）。

**落点**：
- L-1：顺序修正案——P1-05（Confidence 类型 + ADR）先行，ConfidenceLabelFitness 跟随。
- L-2：四函数与阈值同 PR 定义（Pipeline：阶段=独立模块+plain 通信，违例数阈值 0；
  DocDrift：复用脚本计数，阈值=基线递减；
  SecurityMitigation：威胁向量有测试配对率 100%；
  Responsibility：职责陈述缺失数阈值 0，新模块起）。
- L-3：全部新守卫带 allowlist + 过期日（仿 CoreBoundary），过期即红。
- L-4：单测仿 tests mod 风格（纯函数单测 + 全仓守卫冒烟）。

**蓝图回写**：BLUEPRINT D-08 P1-04 行加"顺序：P1-05 先行"；ROADMAP §5 P1-04 加 allowlist 要求。

**放行判定**：GO-WITH-MITIGATION（条件：L-1 顺序 + L-3 allowlist，缺一则执行中停）。

**tripwire**：
- W4 末 Confidence 类型 ADR 未签 → P1-04 自动顺延，Owner 上报。
- 任一新守卫红超 1 周 → 拆任务，不许带红合入。

## §7 关闭条件

SIM 关闭 = 九格填满 + 蓝图回写合并 + 放行判定非空。
SIM 记录永不删除；NO-GO 的 SIM 同样归档（失败证据比成功经验值钱）。

---


---

## §8 SIM-10：机制能力第二轮补强（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针全只读（4 组 websearch + rg 实测），
> 零生产代码改动（仅新增脚本/文档/Makefile targets）。

**探针**：P-1 四组外部搜索（适应度/验证器/供应链/ADR，ADR 组重试一次 429 后成功）；
P-2 构建面实测（2 build.rs 良性、0 自有 proc-macro、2 处 build-deps）；
P-3 现有 7 守卫注册模式复核（arch_fitness_tests vec + 现状必过先例）。

**证据**：E-1 Ford 分类学证明我方守卫全是 atomic+triggered（缺口具名化）；
E-2 Reflect/PSR/NeurIPS25 给出可直接抄的评分制四条；
E-3 构建面基线干净（2 良性 + 0），规则现在立代价最低；
E-4 madr-lint 7 规则即 ADR lint bar，无需新运行时。

**缺口**：G-1 新守卫阈值未定义（转 P1-04 落点 L-2）；G-2 vet/Scorecard/SLSA 需 release 级改动
（defer P3，留烘焙计划）；G-3 Log4Brains web UI 无受众（否决，留手卷模板）。

**外部方案**：见 ABSORPTION-ROUND2.md §1（S-1–S-10）+ 否决项记录。

**落点**：L-R2-1–L-R2-6（吸收文档 §3 全绿，脚本双跑验证，Makefile `make -n` 验证）。

**蓝图回写**：BLUEPRINT v1.2.1 changelog + D-04 索引增评分制行；本文件 §5 登记。

**放行判定**：GO（全部落点已验证，无遗留 UNKNOWN）。

**tripwire**：P1-04 开工时复核 G-1 阈值是否已定义（Owner：L5 owner，日期：P1 启动日）；
P3 启动时复核 G-2 vet 烘焙是否排期（Owner：Architect，日期：M2 评审日）。

---


---

## §9 SIM-11：机制能力第三轮补强（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：4 组 websearch（RouteLLM 组重试一次
> 429 后成功）+ rg 实测 4 处基线。零生产代码改动（仅新增脚本/文档/Makefile target）。

**探针**：P-1 四组外部搜索（路由/OTEL/记忆/fuzz）；P-2 网关实测（FallbackChain 存在，
cascade/score 零命中）；P-3 遥测实测（nt_core_span.rs 7 个 gen_ai 字段已存在）；
P-4 记忆实测（3-tier 结构在，Letta 控制全缺）；P-5 fuzz 实测（无 fuzz 目录）。

**证据**：E-1 路由：有可用性 fallback，无质量评分/预测路由/成本度量；
E-2 遥测：7 字段部分对齐，缺 request.model/temperature/TTFC/tool span/conversation.id/内容策略；
E-3 记忆：结构镜像 Letta，控制（limit/description/并发/溢出/80%）全缺；
E-4 fuzz：readiness 1/4（仅 nightly）。

**缺口**：G-1 网关级联需校准数据（转 P-task）；G-2 遥测补完是 P-task；
G-3 记忆清单 enforcement 绑 P2；G-4 vet/Scorecard 仍 P3（R2 延续）。

**外部方案**：ABSORPTION-ROUND3.md §1（S-11–S-20）+ 否决项（因果 LLM 路由/AFL++/全量 SLSA）。

**落点**：L-R3-1–L-R3-5（脚本双跑验证，`make -n fuzz` 验证，规则顺序已修复）。

**蓝图回写**：BLUEPRINT v1.2.2 changelog；D-04/D-09 相关行已覆盖路由与遥测要求，无新图。

**放行判定**：GO（全部落点已验证；dev-rules 顺序事故已修复并复核）。

**tripwire**：网关 P-task 开工时复核校准数据集是否存在（Owner：L5 owner，日期：P2 启动日）；
fuzz 首 harness 落地时复核语料种子已提交（Owner：L3 owner，日期：P3 启动日）。

---


---

## §10 SIM-12：机制能力第四轮补强（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：4 组 websearch（promptfoo 组 429，
> 以 DeepEval 原生 CI 为主方案并诚实标注）+ rg 实测 6 处基线。生产代码改动仅 1 行
> （release.yml 路径修复），其余为文档/规则。

**探针**：P-1 四组外部搜索（A2A/MCP/评估/发布；promptfoo 429 未果）；
P-2 A2A 实测（AgentCard+注册+3 终态存在；REJECTED/中断态/流推送待验）；
P-3 评估实测（golden+Criterion+G-Eval 形 judge+FaithfulnessReport 存在； relevancy/
precision/recall/门缺失）；P-4 发布实测（git-cliff 在用但 config 路径悬空）；
P-5 Prompt 实测（registry+version+eval 脚手架在，无版本晋升门）。

**证据**：E-1 A2A 骨架 60%（卡+注册+3 终态）；E-2 评估"有分无门"；
E-3 release.yml:25 路径指向不存在的根 cliff.toml（真缺陷）；
E-4 prompt"有版无门"（同构 E-2）。

**缺口**：G-1 A2A 3–5 档（转 P-task 共形测试）；G-2 quartet 补完+门（P2）；
G-3 dist/plz 初始化（P3，需 secrets+tag 纪律）；G-4 Dependabot 存在性待验（快 P-task）。

**外部方案**：ABSORPTION-ROUND4.md §1（S-21–S-27，promptfoo 标注未 live 验证）。

**落点**：L-R4-1–L-R4-4（release 路径修复已读回验证；规则顺序事故同 R3 修法复核）。

**蓝图回写**：BLUEPRINT v1.2.3 changelog；无新图（A2A/评估/发布/Prompt 均有归属图位：
D-08 节点卡、D-05 门、D-10 追溯）。

**放行判定**：GO（全部落点已验证；dev-rules 顺序事故已修复并复核全序号）。

**tripwire**：下次 tag 推送时观察 changelog job 是否绿（Owner：Release owner，日期：下个 tag 日）；
P2 启动时复核 quartet 缺口是否进 gate（Owner：QA，日期：M2 评审日）。

---


---

## §11 SIM-13：规则熔炼为 NT-STD 1.0 标准版（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针全只读（资产盘点 rg + 文件头读）；
> 产出 1 个新文件 + 2 处回写，零生产代码改动。

**探针**：P-1 规则资产盘点（根 dev-rules 210 行 R-P1–110 + docs/dev-rules 88 条 R-P161–257
+ RUST-STANDARDS + SKILL 指针）；P-2 缺口盘点（R-P111–160 无正本、R-P221–229 空号）；
P-3 经验资产盘点（SIM-01–12、ABSORPTION R1–4、5 脚本、fitness 注册模式）。

**证据**：E-1 同一主题散落 3–4 处（如 unsafe→公理+clippy+geiger；allowlist→R-P246+SIM-09+vet）；
E-2 条款无统一验证器（仅新规则有 Implementation 段，老规则多为叙述）；
E-3 R-P111–160 被引用 4 处但无源文件。

**缺口**：G-1 R-P111–160 正本缺失（转 NTS-G08 + Annex B SUSPENDED，附locate-or-reratify tripwire）；
G-2 OSINT 规则与核心版范围不合（转 Annex B PARKED，NT-STD-1.1 领域附录）；
G-3 deny 双工作流重复（FLAG 入 Annex B，P3 合并）。

**外部方案**：ISO/IEC/IEEE 标准文档体例（范围/引用/术语/RFC2119/附录）+ Ford 分类学已内化。

**落点**：`docs/standards/NEOTRIX-STD-1.0.md`（64 条款 A8/B12/C12/D8/E8/F8/G8，全带 Verify；
Annex A 映射、Annex B 废止/保留/暂停、Annex C 符合性三级）+ dev-rules 正典指针 + 蓝图 v1.2.4。

**蓝图回写**：BLUEPRINT v1.2.4 changelog；D-图无新增（标准版是规则层收敛，非新维度）。

**放行判定**：GO（条款计数机验 64/64 带验证器；映射覆盖 R-P1–257 无遗漏，缺号段诚实标注）。

**tripwire**：R-P111–160 locate-or-reratify（Owner：Architect，日期：下一周期评审日）；
NT-STD-1.1 OSINT 附录（Owner：Security，日期：M3 评审日）。

---


---

## §12 SIM-14：旧规则归档＋新模板切换（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：引用链 rg 三组 + 体量统计；
> 生产代码零改动（2 次搬家 + 桩 + 模板 + 指针）。

**探针**：P-1 引用盘点（docs/dev-rules 被 3 处引、根 dev-rules 被 3 处含代码 1 处引、
RUST-STANDARDS 被 5 处引）；P-2 跟踪状态（根文件 untracked，直 mv；
docs 版 tracked，直 git mv）；P-3 模板缺口（SIM 记录格式只在协议 §3、
模块文档模板只在吸收文档节、无适应度模板）。

**证据**：E-1 搬家前后 `git status` 对照（RM + ??各一）；E-2 四模板文件落盘；
E-3 SKILL 指针已改；E-4 RUST-STANDARDS 未搬（5 处引用 + 现行编码标准，搬家收益为负）。

**缺口**：G-1 RUST-STANDARDS 留守（有意，见证据 E-4，STD Annex A 已映射）；
G-2 DOCUMENTATION-MAP 仍指向旧路径（桩保活，内容更新转 P-task，不阻塞）。

**外部方案**：归档惯例（archive/ + 桩 + README 冻结声明）；模板套件防格式增殖
（新模板需 ADR，见 templates/README）。

**落点**：archive 2 文件 + README + 双桩 + SKILL 指针 + templates 4 文件 +
ROADMAP 附录 4 行。

**蓝图回写**：BLUEPRINT v1.2.5 changelog；无新图（归档是治理动作，非架构维度）。

**放行判定**：GO（引用链不断：桩保活全部旧路径；新工作流模板齐备）。

**tripwire**：DOCUMENTATION-MAP 旧路径内容更新（Owner：Docs，日期：P2 文档阶段）；
RUST-STANDARDS 并入 STD 的时机评估（Owner：Architect，日期：NT-STD-1.1 规划日）。

---


---

## §13 SIM-15：登记表时序＋依赖更新缺口＋deny 三重奏（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针全只读 + 1 处 YAML 加项
> （只开 PR，不阻塞构建）。生产代码零改动。

**探针**：P-1 登记表顺序（SIM-13/14 插到了 12/11/10 前面——prepend 锚点副作用）；
P-2 Dependabot（`.github/dependabot.yml` 仅 github-actions，无 cargo）；
P-3 deny 三重奏（deny.yml 路径门禁/advisories 不阻塞 + security-audit.yml 周扫全量阻塞
+ audit.yml rustsec 周扫 warn）；P-4 遗留 tripwire 到期检查（SIM-09 W4、SIM-10/11 P-启动、
SIM-12 tag 日——同日会话，均未到期）。

**证据**：E-1 登记表 7 行乱序（机验）；E-2 dependabot 无 cargo 段；
E-3 三工作流触发器与阻塞性各不相同（deny 快门禁周扫 + audit 第三方库）；
E-4 tripwire 无到期项。

**缺口**：G-1 登记表时序（已修）；G-2 cargo 无自动更新（已补）；
G-3 deny.yml advisories `continue-on-error` 与 NTS-D02 "MUST pass" 张力
（有意 defer：快门禁防噪音，周扫兜底；P3 统决定夺，不碰执行态）。

**外部方案**：Dependabot cargo 生态为标准能力（Orhun R4 管道本就假设它存在）；
deny 合并无外部强制要求——分工不同即保留。

**落点**：登记表按 SIM 号重排（机验 01~08/09/10/11/12/13/14）；
dependabot.yml 加 cargo 段（YAML 机验通过）；Annex B FLAG 维持（加注分工说明见本节）。

**蓝图回写**：BLUEPRINT v1.2.6 changelog；无新图。

**放行判定**：GO（登记表机验有序；YAML 机验有效；CI 执行态未动）。

**tripwire**：首个 Dependabot cargo PR 合并时观察 Cargo.lock 漂移（Owner：Deps，
日期：首 PR 日）；P3 deny 整合决策（Owner：Architect，日期：M3 评审日）。

---


---

## §14 SIM-16：意识指导为先（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：引用链 + 加载器三段精读 +
> 正则烟测 + 11/11 单测实跑。生产代码改动 1 文件（+120 行），全有测试覆盖。

**探针**：P-1 引用盘点（MAP×2、AGENTS×2、constitution×12、TODO×1、ROADMAP×3）；
P-2 加载器精读（392 伴生合并、437 正则、665 GLOBAL、762 测试）；
P-3 定损三问（legacy 84 子弹可解析 / AGENTS 无 Dev Rules 节 / STD 无节需免门解析）；
P-4 正则烟测（NTS 64/64 唯一、双向零串扰、R-P 84）；
P-5 `cargo test -p neotrix --lib nt_core_self_constitution` 实跑 11/11 绿。

**证据**：E-1 SIM-14 归档引入回归（伴生变桩→解析 Err→整 load 失败；GLOBAL 回空宪法）；
E-2 AGENTS 无节问题先前已红（指针守恒的代价，注释与代码自相矛盾）；
E-3 categorize 非幂等（append），vectorize/index 幂等；
E-4 AGENTS/TODO 的旧引用为磁盘实况/历史记录，不动。

**缺口**：G-1 加载器三处硬错（伴生 Err 传染/无节 Err/正则不认 NTS）——已修；
G-2 R-P42/48 测试依赖归档文件——已转 archive 直读；
G-3 优先级无文字——已立 §0.1；
G-4 AGENTS/TODO 引用——有意不动（E-4）。

**外部方案**：无（内部一致性修复）；正则写法遵循 regex crate 语义，fmt 机验。

**落点**：constitution 四编辑（NTS 分类臂/免门节/四源宽容合并/双新解析函数）
+ 派生清零 + 注释更新；MAP×2/ROADMAP×3 换正典；NT-STD 1.0.1 §0.1；
11/11 单测绿（含先前已红的 load_real_agents_md）。

**蓝图回写**：BLUEPRINT v1.2.7 changelog；无新图（治理接线属 NODE-L6 卡内工作）。

**放行判定**：GO（单测实跑为证；fmt 仅剩历史漂移一行，非我引入）。

**tripwire**：下次全量 `cargo test --lib` 时观察 constitution 11 项是否保持绿
（Owner：QA，日期：P0 全量门日）；NT-STD-1.1 规划日复核 R-P111–160（NTS-G08 延续）。

---


---

## §15 SIM-17：旧口径清零（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：引用全量 rg + 注释精读；
> 生产代码改动仅注释与 expect 文案（零逻辑变更）；单测实跑 11/11。

**探针**：P-1 残留定位（R-P 范围串 4 处：STD 自述 2、宪法 enum 文档 1、桩文件 1——均为
自述/历史/桩，定位后判定）；P-2 宪法 stale 注释 8 处（伴生/R-P101/测试文案）；
P-3 SKILL 教学 6 条（2 条可精确映射 NTS，4 条留 legacy 括号）；
P-4 MAP/ROADMAP 余量（桩与历史指向，意图保留）。

**证据**：E-1 11/11 单测绿（含注释换装后重编）；E-2 fmt 仅剩历史 :341；
E-3 中途 E0689 误报：系 30min 超时 kill 残留增量缓存污染，重编即消（5m32s 全绿）；
E-4 AGENTS/TODO 不动之理由成立（磁盘实况/历史），复核无新增引用。

**缺口**：G-1 kill 残留污染无防护——转教训（不用 SIGTERM 杀 cargo；清缓存走脚本）；
G-2 enum 变体文档仍带 R-P 出处——有意保留（历史溯源，NTS 映射已在 from_rule_id）。

**外部方案**：无（内部口径统一）。

**落点**：宪法注释×8（R-P101 同步契约标退役、测试文案指 archive/NTS）；
SKILL 教学 NTS 优先括号留 legacy；MAP×0/ROADMAP×0（已无活引用）。

**蓝图回写**：BLUEPRINT v1.2.8 changelog；无新图。

**放行判定**：GO（单测实跑 + fmt 对照 + 引用复核三证）。

**tripwire**：CI 全量门若报 E0689 类幽灵错误，先清增量缓存再定罪（Owner：Infra，
长期）；NT-STD-1.1 评估 enum 文档是否同步 NTS（Owner：Architect，日期：1.1 规划日）。

---


---

## §16 SIM-18：全图审计重绘（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：围栏机验 + 15 图类型声明 +
> D 引用计数 + D-00 文本审计。零生产代码改动（纯文档手术 8 处）。

**探针**：P-1 围栏机验（15 mermaid 块，32 fence 标记，偶数平衡）；
P-2 类型声明（graph/state/sequence/gantt 全合法）；P-3 D-00 文本审计
（D-13/D-14 零提及——总图先于两图出生，从未回写）；P-4 逐图核现实
（D-02 缺 H-08 行、D-04 缺评分步、D-05 缺 P3/P4 标注、治理接线无图）。

**证据**：E-1 D-00 图内 D-13/D-14 缺席（机验 False/False）；
E-2 D-02 索引表止于 H-07（H-08 只在 §17）；E-3 D-04 时序无评分语义；
E-4 其余 10 图与现实一致（D-01 目录/D-03 状态/D-06~D-14 内容复核）。

**缺口**：G-1 总图滞后两版本（已重绘，纳入 D-13/D-14/正典轨/STD 虚线）；
G-2 治理接线无图（新增 D-15，四源＋豁免语义＋11/11 实证）；
G-3 尾部双结尾标记（v1.2.4 残留——版本只升头不升尾的纪律漏洞，已清）。

**外部方案**：无（内部一致性）。

**落点**：D-00 重绘＋索引 2 行；D-02 加 H-08 行＋归边注记；D-04 加评分步＋阈值分支；
D-05 加 P3/P4 标注；§19 D-15＋索引/附录 2 行；v1.3.0 changelog。

**蓝图回写**：即本轮（BLUEPRINT v1.3.0，16 图）。

**放行判定**：GO（围栏平衡复验 + D 引用全覆盖 D-00~D-15 + 单结尾标记）。

**tripwire**：每次加新图必须同步改 D-00（Owner：作图人，即时）；
每次版本升级必须头尾标记同升（Owner：作图人，即时；本次 G-3 即判例）。

---


---

## §17 SIM-19：过期信息清理（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：占位符三组 + 死引用两组 +
> 候选删除项逐项定性。原则：删文档过期不断言，删代码占位不动行为——
> 本轮代码删除行数为 0 是有意结论，不是没干活。

**探针**：P-1 `todo!/unimplemented!/unreachable!` 全仓计数（命中多为扫描器字串、
防御性穷举臂、测试替身）；P-2 精确 `todo!\s*\(` 宏调用（生产代码 0，真空）；
P-3 `.disabled`×2 内容（555+1035 行真集成测试，头注写明阻塞模块）；
P-4 活指令式 dev-rules 引用（0 处）；P-5 docs/脚本 TODO 标记（全为合法散文）；
P-6 e2e_scaffold（462 行真 harness）/游戏 todo（字串，非调用）。

**证据**：E-1 真占位为零——删除任何候选都等于删功能或删覆盖；
E-2 TODO.md 基线表过期（SIM-01 已证）；E-3 其余候选均为合法资产。

**缺口**：G-1 TODO.md 过期表未标记——已挂 STALE 横幅（删 0 行，史实保留）；
G-2 `.disabled` 两文件重启用时机未定——转 tripwire（P0 复检阻塞模块后决定）；
G-3 游戏域 todo! 字串——域外，不管（记录）。

**外部方案**：无（内部卫生）。

**落点**：TODO.md STALE 横幅 6 行；其余 0 改动（有意）。

**蓝图回写**：BLUEPRINT v1.3.1 changelog；无新图（卫生轮）。[SIM-20 修正：原记录误写 v1.2.9，该版本从未存在]

**放行判定**：GO（"无可删"是探针结论，不是不作为；删错的代价 >> 留对的代价）。

**tripwire**：P0-01 重跑 check 后刷新 TODO.md 并摘横幅（Owner：P0 owner，日期：M0 日）；
阻塞模块修复后评估 `.disabled` 重启用（Owner：QA，日期：M1 评审日）。

---


---

## §18 SIM-20：P0 门 override 提交（GO-WITH-OVERRIDE，用户已批复）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：hook 日志全文 + 错误定位到文件 +
> OOM 证据 + 自文件清单。§0.1 conscious override 程序首次实战。

**探针**：P-1 hook 日志 4×`^error`（E0277 nt_crystal_task_fusion.rs:687 测试替身
RefCell 非 Sync；E0004×3 factory.rs:167/474/593 新增 Shimmy 变体未覆盖）；
P-2 `Killed: 9`（OOM，本机跑不完 check --tests）；P-3 自文件清单（33 文件：
文档/脚本/配置＋模板 `.rs`（crate 外不可编译）＋宪法 1 文件已 11/11）；
P-4 用户批复（question 工具，2026-09-21，选推荐项）。

**证据**：E-1 4 错文件均非本次触碰（staged 集无此二者之修改——constitution 除外，
其已 11/11）；E-2 模板 `.rs` 在 docs/ 下，cargo 不可见；
E-3 两次标准提交均被同一环境门拦下（非代码原因）。

**缺口**：G-1 他人 4 错（转 P0-01，不在本 commit 修——越权＋超验证能力）；
G-2 本机 OOM 跑不完门（转 infra：CI 大 runner 兜底）。

**外部方案**：NT-STD §0.1 override 程序（SIM-ID＋ADR＋最小范围＋tripwire）。

**落点**：ADR-0001（双证之文）＋ commit trailer 注明（双证之链）＋ CI tripwire。

**蓝图回写**：BLUEPRINT v1.3.2 changelog（override 事件记入版本史）；无新图。

**放行判定**：GO-WITH-OVERRIDE（用户批复＋双证＋CI 兜底；缺一不可，今次三者齐）。

**tripwire**：push 后 CI 同门（check --tests）变绿前，ADR-0001 视为 pending-CI
（Owner：CI，日期：push 当日）；P0-01 修 4 错（Owner：P0 owner，日期：M0 日）。

---


---

## §19 SIM-21：cli 升级残留审计（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针全只读 + chmod×2。
> 结论先行：升级本身干净，残留只剩执行位；4 个编译错是他人进行中改动，不归本轮。

**探针**：P-1 删除清单（100 文件：cli/commands 52＋tui 11＋cli 根 9＋tui/app 11＋
src-tauri 9＋2＋1＋skills 文档 1）；P-2 `cli::` 残留 10 文件逐条定性；
P-3 entry/mod.rs:1968 真调用归属（沙盒自有 cli，非已删 src/cli）；
P-4 4 错归属（factory.rs 已改态 Shimmy 未覆盖×3＋nt_crystal_task_fusion 全新 untracked×1）；
P-5 活路径引用（2 注记＋1 测试固件，全 benign）；P-6 沙盒 cli.rs 在盘＋集成测试零引用。

**证据**：E-1 lib 11/11 绿即含删除态（删除与绿单测同树）；E-2 真调用全解析到存活模块；
E-3 capability_tree 自有 cli.rs 在盘；E-4 4 错文件皆非本会话触碰；
E-5 docs 零活引用。

**缺口**：G-1 他人 4 错（转 P0-01，与 SIM-20 G-1 同源——仍未修）；
G-2 src-tauri 11+3 删除需独立审计（另 crate，Rust 门覆盖不到）；
G-3 FULL-ARCHITECTURE untracked（非我文件，等主人）；
G-4 脚本执行位 2 缺（已 chmod，机验 4/4 +x）。

**外部方案**：无（内部审计）。

**落点**：chmod×2；其余 0 改动（升级注记齐全：eprintln 降级通知＋迁移注释链完整，
删得有交代——反例是静默删除，本轮证实非反例）。

**蓝图回写**：BLUEPRINT v1.3.3 changelog；无新图（审计轮）。

**放行判定**：GO（零悬空已证；越界修复已拒——他人进行中代码不动）。

**tripwire**：P0-01 修 4 错后重跑 check --tests，届时 bins（含 entry 沙盒调用链）
首次被编译验证（Owner：P0 owner，日期：M0 日）；tauri 删除审计（Owner：Desktop，
日期：M2 评审日）。

---


---

## §20 SIM-22：经验吸收＋全量总验＋收尾（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：12 教训成文＋三文档围栏机验
> ＋ARCHITECTURE.md 异常定位。零生产代码改动。

**探针**：P-1 会话复盘（SIM-01–21 + 4 吸收轮 + 熔炼 + 归档 + 3 提交）→ 12 条教训；
P-2 围栏总验（arch/standards/adr 全文件偶平衡，唯一例外 ARCHITECTURE.md 27 奇）；
P-3 异常定位（疑似 161/176/191 处标题吞入代码块＋内容实质过时 L4-L5 写法）。

**证据**：E-1 经验文档 12 行表落盘；E-2 ARCHITECTURE.md 非我文件、内容涉他人架构主张；
E-3 其余我方文件全平衡、单结尾、版本头尾一致（机验）。

**缺口**：G-1 ARCHITECTURE.md 过时＋疑似误渲染——不修（他人正文，重写越权；
降级为 finding＋tripwire）；G-2 本轮无新图（收尾轮）。

**外部方案**：无。

**落点**：SESSION-ABSORPTION-2026-09-21.md；BLUEPRINT v1.3.4 changelog；
7 条核心建议（见 §21）。

**蓝图回写**：即本轮（v1.3.4）。

**放行判定**：GO（总验机验全过；唯一例外有主、有据、有去处）。

**tripwire**：ARCHITECTURE.md 重写或归档（Owner：Architect，日期：M2 评审日）；
全仓库 markdown 围栏门（Owner：Docs，日期：P2 文档阶段，`find docs -name *.md` 计数偶校验）。

---

## §21 核心建议（7 条，按优先级）

1. **先修 P0-01 的 4 个他人错误**（factory Shimmy×3＋crystal RefCell）：门禁、CI、bins 全卡于此，
   它是当前唯一的真阻塞。工作量小（补臂＋换 Mutex），但必须配完整验证周期。
2. **给 CI 换大 runner 或拆分 check**（lib/bins/tests 分 job）：本机 OOM 证明全量
   check --tests 的资源需求超配，CI 同配会 flaky，先行验证再谈门禁可信。
3. **找回 R-P111–R-P160 正本**（NTS-G08）：唯一 SUSPENDED 大项，找不到就重议，
   不能无限期悬空——悬空规则比无规则更腐蚀信任。
4. **P1-05 Confidence 类型先行**（SIM-09 顺序结论）：P1-04 五适应度函数编码
   在此之前开工必然撞墙。
5. **SDB 登记提到 100%**（SDB-REGISTRY 7 候选全 UNKNOWN）：M1 放行条件；
   每新增 LLM→动作点先登记再编码的纪律从下个 PR 开始执行。
6. **覆盖率/基准门现在就 bake**（P3 前置 1 周＋）：gh-pages 基线、llvm-cov 地板 70、
   critcmp 本地基线，三件事都不需要等 P3 开工。
7. **Tauri 删除独立审计＋FULL-ARCHITECTURE 认领**：两轮点名、无人认领的悬空项，
   M2 前必须有主，否则它们会变成下一个"12 编译错误"式惊喜。

---


---

## §22 SIM-23：核心建议修复轮（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：打捞脚本＋三处上下文精读＋
> 两次 cargo 实跑。生产代码改动 1 行（hook -j4），其余全为验证与文档。

**探针**：P-1 R-P 打捞（20 ID 命中，正文多可读）；P-2 factory 六 match 逐一核
（Shimmy 臂已齐，协作者补完）；P-3 crystal cfg 界定（cfg(test) 内，lib check 覆盖不到）；
P-4 机器规格（10c，空闲内存 ~72MB——OOM 根因实锤）；
P-5 `cargo check --lib -j4`（5m18s 绿）。

**证据**：E-1 `cargo check -p neotrix --lib --tests -j2` 6m44s 全绿——crystal Mutex、
Shimmy 三臂、宪法改动一次证完，他人 4 错已灭（非我所修，归属协作者）；
E-2 R-P128 三义并存（cost-aware×2 / no-raw-pointers / config-driven 实为 R-P124 复述）；
E-3 hook `-j4` 已加注（同命令，1/3 并行峰值）。

**缺口**：G-1 全量 check --tests（bins/examples/integration）在本机仍有 OOM 风险
（-j4 缓解未根除，转 CI 大 runner 验证）；G-2 R-P128b/c 改号（转 owner）；
G-3 31 STILL-MISSING（转 NT-STD-1.1）。

**外部方案**：cargo `-j` 并行上限（官方 flag，非偏方）。

**落点**：RECOVERY annex（19+1+31）＋ hook -j4 ＋ NT-STD 1.0.2（Annex B/G08 改写）
＋ BLUEPRINT v1.3.5。

**蓝图回写**：即本轮（v1.3.5）。

**放行判定**：GO（两次实跑绿＋打捞机验＋改动 1 行可 review）。

**tripwire**：CI 全量 check 首绿确认（Owner：CI，日期：push 当日，承接 SIM-20）；
R-P128b/c 改号（Owner：模块 owner，日期：M1）；31 缺失（Owner：Architect，1.1）。

---


---

## §23 SIM-24：全域审计＋第五轮吸收（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：机测规模三组＋外部三组搜索＋
> unsafe 逐一定性。生产代码零改动（Largest diff：Makefile 1 target＋hook 已在 SIM-23）。

**探针**：P-1 规模机测（2191 文件/~719k 行分层＋9 crates＋26 集成测试）；
P-2 违规/漂移机测（12 组越层，111 漂移零进展）；P-3 unsafe 逐一定性
（sysctl 真 FFI＋文书 vs 其余全字串/固件）；P-4 外部三组
（AgentHarm 110×11＋τ-bench＋DORA/SPACE＋cargo-modules 家族）。

**证据**：E-1 sysctl `#![allow]`＋ justification 头＋ core forbid 不受影响（三段式齐全）；
E-2 DORA/SPACE 共识序列（先 DORA 后 SPACE，团队级禁个人排名）；
E-3 cargo-modules orphans/--acyclic/cargoscope 免编译三件套定位完成。

**缺口**：G-1 111 漂移零进展（转 P2，与覆盖率门同批）；G-2 12 越层组未消（P1-02）；
G-3 AgentHarm 落地需 P3 评估管线（设计已定 NTS-E09）。

**外部方案**：见 ABSORPTION-ROUND5.md §1（S-28–S-32）＋否决项。

**落点**：ROUND5＋NTS-E09/D09/B13（STD 1.0.3）＋sysctl 特许登记＋ROADMAP 快照＋
`make arch-acyclic`＋BLUEPRINT v1.4.0（D-05 G6 行＋changelog）。

**蓝图回写**：即本轮。

**放行判定**：GO（机验数字全出自实跑；条款计数待下轮复核）。

**tripwire**：111 漂移到 0 才进 `--strict`（Owner：Docs，日期：P2）；
AgentHarm 对子首跑（Owner：L3，日期：P3）；DORA 四格首填（Owner：QA，日期：M2）。

---


---

## §24 SIM-25：第六轮吸收（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：4 组搜索（Judge 组 429，
> 以既定知识＋R4 实证推进并标注）＋ API/文档双盘点。生产代码零改动。

**探针**：P-1 四组外部搜索（oasdiff 全家桶＋Diátaxis 全套＋Toxiproxy 全家桶）；
P-2 API 盘点（19 tauri＋267 routes 上限＋0 spec）；P-3 文档盘点（70 文件无象限）；
P-4 Judge 复核（Criterion 有，bias 处理零）；P-5 混沌复核（零引用）。

**证据**：E-1 无 spec 即无契约（lint 单文档，diff 需双文档——缺第一份）；
E-2 Diátaxis 警告禁空目录（只分类不新建）；E-3 bias 清单皆有标准缓解；
E-4 toxiproxy `/reset` 强制 teardown 是可靠性关键。

**缺口**：G-1 spec 导出（P-task，门之门）；G-2 bias 代码（P2）；G-3 toxics 套件（P3）；
G-4 本轮无 Makefile 新增（无 spec/工具前加 target 即空转，诚实省略）。

**外部方案**：见 ABSORPTION-ROUND6.md §1（S-33–S-38）＋否决项（Pro 版 approve 门等）。

**落点**：ROUND6＋api-surface 脚本（19/267/0 机验）＋MAP 象限节＋NTS-D10/F09/E10
（STD 1.0.4）＋BLUEPRINT v1.5.0。

**蓝图回写**：即本轮（无新图——四机制归属既有图位：D-05 门/G8 文档/MAP）。

**放行判定**：GO（探针数字全机验；省略项有书面理由）。

**tripwire**：spec 导出首版（Owner：API，日期：P2 启动日）；bias 代码（Owner：ML，日期：P2）；
toxics 首测（Owner：SRE，日期：P3）。

---


---

## §25 SIM-26：蓝图实施影响面分析（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：blast 机测四组。
> 生产代码零改动（1 行计划修订）。

**探针**：P-1 越层 153 行；P-2 SearchResult 109 引用文件；P-3 热点 5 文件 2981 行；
P-4 SDB 核心 4 文件 1090 行；P-5 集成测试 26 文件。

**证据**：E-1 广度 5–8% 文件，风险集中 <10 行为点；E-2 SDB fail-closed 是全计划
最高行为风险（ verifier 误拒＝活功能变砖）；E-3 本机 OOM 使 P2 级改动本地不可全验。

**缺口**：G-1 P1-03 缺烘焙环节——已修订（log-only＋逐站点 ADR 翻转）；
G-2 P0-01 数量未知（重跑前）。

**外部方案**：无（内部评估；数字全部机测）。

**落点**：IMPACT-ANALYSIS.md＋ROADMAP P1-03 修订＋BLUEPRINT v1.5.1。

**蓝图回写**：即本轮。

**放行判定**：GO（评估类产出；执行按 §Go/No-Go 分级）。

**tripwire**：SDB 首站点翻转时复核误拒率（Owner：L5，日期：翻转日）；
P0-01 数量出炉后刷新本分析 §1（Owner：P0，日期：M0 日）。

---


---

## §26 SIM-27：A组收尾＋B1合入CI见证（GO-WITH-CI-BACKSTOP，用户已批复甲）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：zsh 分词坑定位＋SDB 调用链
> 精查（退役 04 登记 05）＋静态复核。生产代码增量：L0 纯新增类型＋3 单测。

**探针**：P-1 构建排队根因（活锁持有：协作者全量编＋本机 30min 上限＋nohup 不存活）；
P-2 zsh 未引用变量不分词（真实 RAW 216／注释 19／真违规 197，替代 153/150 显示数）；
P-3 SDB 调用链（dispatcher 442/890/1001 实点；trade 系误报；loops 需 call-graph）；
P-4 B1 静态复核（类型闭合＋serde 先例＋fmt-clean）。

**证据**：E-1 A1：脚本＋真数（197 分组明细）；E-2 A2：SDB-REGISTRY v0.3；
E-3 A3：规则不动只标现状（D-04 现状行）；E-4 A4：阅读索引；
E-5 B1：静态全绿（fmt＋复核），执行待 CI。

**缺口**：G-1 B1 执行确认（转 CI 首绿 tripwire）；G-2 B2 待 B1 绿后开工；
G-3 全量 --tests 本机不可复现（转 CI 大 runner，承接 SIM-23）。

**外部方案**：NT-STD §0.1 override（SIM＋ADR＋批复＋trailer＋tripwire，五件齐）。

**落点**：4 脚本/文档项＋B1 代码＋ADR-0002＋BLUEPRINT v1.5.2。

**蓝图回写**：即本轮。

**放行判定**：GO-WITH-CI-BACKSTOP（用户批复甲；纯新增零行为面；ADR-0002 双证）。

**tripwire**：CI 首绿/首红（Owner：CI，日期：push 当日）；B2 开工门为 B1 绿
（Owner：P1，日期：首绿日）；全量 --tests CI 化（Owner：Infra，日期：M0）。

---


---

## §27 SIM-28：SDB 调用链追踪收敛（GO-WITH-CI-BACKSTOP）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针全只读（4 文件精读＋调用链两跳）。
> B1 验证战：nohup 不存活、锁被协作者构建持有、30min 上限三连——转 CI 兜底（甲决议已覆盖）。

**探针**：P-1 executor 解剖（strategy 分发＋trait 对象，无 LLM 句柄）；
P-2 planner 精读（decompose 硬编码模板＋execute_step 纯 format!）；
P-3 oracle_gate 精读（确定性 Gap 枚举）；P-4 hive（mock 注释 only）。

**证据**：E-1 loops 三处皆无 LLM 调用——SDB-01/02/03 从 UNTRACED 改判 SCAFFOLD；
E-2 全仓唯一活 P 点组：dispatcher 442/890/1001；
E-3 B1 验证在本机不可完成（锁＋时长＋nohup 三重门，实测三轮）。

**缺口**：G-1 B1 执行确认（CI，ADR-0002 不变）；G-2 loops 接 LLM 日重登记（tripwire，
Owner 为接线 PR 作者）；G-3 SDB-06/07 构件是否为真 verifier（P1-03 内审）。

**外部方案**：无（内部代码考古）。

**落点**：SDB-REGISTRY v0.4（收敛结论＋SCAFFOLD 态）＋BLUEPRINT v1.5.3。

**蓝图回写**：即本轮。

**放行判定**：GO-WITH-CI-BACKSTOP（同甲；追踪结论本身零风险）。

**tripwire**：loops 任一文件出现 LLM 调用即重登记（Owner：接线作者，即时）；
B1 CI 首绿（Owner：CI，承接 ADR-0002）。

---


---

## §28 SIM-29：提交清场＋V 构件定级＋R-P111 指针落地（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：暂存污染排查＋双 verifier 精读
> ＋R-P111 全源搜索。生产代码零改动（只读＋文档）。

**探针**：P-1 暂存区审计（揪出协作者 nt_dialogue 重命名已 staged→剥离，内容未动）；
P-2 prm/verifier.rs 精读（6 维 VerificationAction＋label 全覆盖）；
P-3 behavioral_verifier 精读（编译＋测试＋属性＋run_bounded＋RL 信号）；
P-4 R-P111 全源搜索（唯一命中 bend 分析指针行；112–115 全仓零命中）；
P-5 nt_laws.rs 确认（LawViolation 引擎本体存在）。

**证据**：E-1 SDB-06/07 均为真 verifier 构件（V-READY，接线待定）——SDB 首次拥有 V 库存；
E-2 R-P111 意图可落地（law 进治理已有 nt_laws 实现承载）；
E-3 R-P112–115 确认佚失（非暂置，是真丢）。

**缺口**：G-1 V 构件与 dispatcher 三点位的接线设计（P1-03）；
G-2 R-P112–115 重议（无 seed，需 architect 主持，NT-STD-1.1）。

**外部方案**：无（内部考古＋清场）。

**落点**：7b9e0777 提交（4 文件干净）＋ SDB-REGISTRY v0.5 ＋ BLUEPRINT v1.5.4。

**蓝图回写**：即本轮。

**放行判定**：GO（只读审计＋干净提交；接线与重议转 P-task）。

**tripwire**：V 接线设计（Owner：L5，日期：P1-03 启动日）；
R-P112–115 重议会（Owner：Architect，日期：NT-STD-1.1 规划日）。

---


---

## §29 SIM-30：门禁复测＋聚合器（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：四门禁重跑（秒级）。
> 生产代码零改动（Makefile 1 target）。

**探针**：P-1 层违规 12 组（持平）；P-2 漂移 111（持平，零进展）；
P-3 构建面 clean；P-4 fuzz 1/4（持平）。

**证据**：E-1 三轮门禁数字全部原地踏步——说明 P1-02/P2 尚未开工，
门禁体系在"看守"而非"推进"状态；E-2 聚合器一次跑完五门（`make -n` 验证）。

**缺口**：G-1 门禁只看守不推进（转 P1-02 开工）；G-2 111 漂移需内容劳动（转 P2）。

**外部方案**：无。

**落点**：`make audit-all`＋BLUEPRINT v1.5.5。

**蓝图回写**：即本轮。

**放行判定**：GO（聚合器机验；数字持平如实记录）。

**tripwire**：任一门禁数字变动即更新 IMPACT 快照（Owner：QA，持续）。

---


---

## §30 SIM-31：B2 守卫编码＋实跑绿（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：守卫写法精读＋目录存在性机验。
> 生产代码增量：1 guard＋6 单测＋注册（约 150 行， additive only）。

**探针**：P-1 现有 7 守卫写法（helpers/allowlist/注册/测试四范式）；
P-2 目录存在性（`src/core`、`l1_body_impl` 双双不存在——Layer/Core 二守卫为 theater）；
P-3 锁门控（wait 0s 即空闲，窗口难得，立即开编）。

**证据**：E-1 `cargo test --lib nt_core_arch_fitness` **14/14 绿**（8 旧＋6 新），
新守卫真实仓库实跑 60s＋零干净对违规——校准与实现一次证完；
E-2 fmt 仅剩历史漂移；E-3 死守卫发现（见缺口）。

**缺口**：G-1 Layer/Core 二 theater 守卫修退役（转 P1 tripwire，不在本轮动——改他人守卫语义需独立 SIM）；
G-2 其余 4 fitness 阈值未定义（SIM-09 G-2 延续）；
G-3 B1 CI 首绿仍挂（ADR-0002 tripwire 不变）。

**外部方案**：无（内部实现；Ford 分类学已在 R-P246）。

**落点**：nt_core_arch_fitness.rs（守卫＋注册＋6 测）＋BLUEPRINT v1.5.6。

**蓝图回写**：即本轮。

**放行判定**：GO（实跑绿为证；死守卫发现如实记录未私刑）。

**tripwire**：死守卫修退役（Owner：Architect，日期：P1 启动日）；
其余 fitness 阈值（Owner：L5，日期：P1-04 开工日）。

---


---

## §31 SIM-32：有序执行清单（GO）

> 认领/时间盒：NeoTrix agent，单会话内完成。探针：既有决议收敛（无新探针，
> 全部引用已验证结论）。生产代码零改动（纯计划文档）。

**探针**：P-1 前 31 个 SIM 的决议＋tripwire＋defer 项收敛（去重、排序、定完成定义）。

**证据**：E-1 每行 EQ 均有出处（SIM 号／门禁数／审计结论），无拍脑袋项；
E-2 阻塞边全部显式化（B1→B2，基线→P1-02，设计→实现）。

**缺口**：G-1 EQ-01/02 执行（本轮只建表不执行——建表与执行分开，避免计划即开工的幻觉）。

**外部方案**：无。

**落点**：EXECUTION-QUEUE.md（EQ-01~20＋状态机）＋BLUEPRINT v1.5.7。

**蓝图回写**：即本轮。

**放行判定**：GO（计划文档；执行按表逐项开 SIM）。

**tripwire**：首个 EQ 状态翻绿时复核完成定义是否被绕过（Owner：QA，持续）。

---

## §32 SIM-33：提交清场（GO）

> 认领：单会话内完成。探针：暂存名录审计＋标记计数机验。

**探针**：P-1 暂存审计（两度混入协作者文件：ntcode 重命名＋cli_free/opencode 新文件）；
P-2 标记计数（24→1，节/行完好性机验）；P-3 §31 虚惊复核（rg 证实在场）。

**证据**：E-1 剥离后提交名录干净（7b044f42，3 文件）；E-2 31 节＋25 登记行机验完好。

**缺口**：G-1 共享分支暂存竞态（转工作方式：提交前必看名录，L-13）。

**落点**：7b044f42＋L-13/14/15＋BLUEPRINT v1.5.8。

**放行判定**：GO。

**tripwire**：暂存再污染即停手排查（Owner：提交人，即时）。

---


---

## §33 SIM-34：EQ-01/02/03 闭环（GO）

> 认领：单会话内完成。探针：B1 实跑＋allowlist 路径存活检查＋调用方枚举。

**探针**：P-1 B1 三单测本地 3/3（EQ-01 关）；P-2 allowlist 路径半数已不在
（consciousness_core/second_brain 在，forecast/chain/bridge 不在）；
P-3 调用方枚举（self_test_integration＋SEAL pipeline——注册表是活的）。

**证据**：E-1 退役不改变活行为（删恒 Ok 项）；E-2 后继覆盖已实跑（B2＋脚本）。

**缺口**：G-1 EQ-04 代码删除＋全量验证（下轮，需独占窗口）。

**落点**：ADR-0002 关闭＋ADR-0003＋EQ 状态三翻绿＋BLUEPRINT v1.5.9。

**放行判定**：GO（决议类产出；执行转 EQ-04）。

**tripwire**：EQ-04 执行时复核调用方无行为依赖（Owner：L5，日期：执行日）。

---


---

## §34 SIM-35：EQ-04 执行（GO）

> 认领：单会话内完成。探针：调用方枚举＋共享 helper 存活确认＋精确行锚。
> 生产代码：删 175 行（2 守卫＋1 helper＋1 allowlist＋注册 2 行＋专属单测），零新增逻辑。

**探针**：P-1 外部引用（仅本文件内：注册＋专属单测）；P-2 共享 helper 存活
（repo_root/src_root/rs_files/in_test_context/regex/HashSet 全被幸存者使用）；
P-3 精确行锚（l1_root 42–45，Layer 58–102，Core 104–211，测试 843–854）。

**证据**：E-1 `cargo test --lib nt_core_arch_fitness` **13/13 绿**（213s）：
7 旧幸存者（core 专属单测已随葬）＋6 新全过，含真实仓库 60s＋实跑；
E-2 fmt 全文件零 diff（含历史漂移被协作者顺手清零）；
E-3 计数机验：13＝7＋6，删除前后自洽。

**缺口**：G-1 其余 4 fitness 阈值（EQ-08，不变）；G-2 B1 CI 首绿（ADR-0002，不变）。

**落点**：fitness.rs（-175 行）＋BLUEPRINT v1.6.0。

**放行判定**：GO（实跑绿为证；删的是恒 Ok 项，行为面零变化）。

**tripwire**：注册表再有人加 theater 守卫（扫描不存在目录）即按本案重演
（Owner： reviewer，持续；判据：守卫必须有一次红过或对应真实目录）。

---


---

## §35 SIM-36：EQ-05 执行（GO）

> 认领：单会话内完成。探针：双 math 溯源＋全仓路径普查＋格式三轮收敛。
> 生产代码：3 行改路径（同项 item，零行为）＋删 14 行过期注释。

**探针**：P-1 L0/L5 math 关系（L0 实现＋L5 `pub use` 重导出，mod.rs:81）；
P-2 全仓 L5 路径普查（bank×3 属 EQ-05；svaf f32/bin/diverse 属他域，不管）；
P-3 L0 四函数齐备（f32/f32_f32/f64/normalize_url）。

**证据**：E-1 改路径前后同一 item（重导出恒等式），语义零变化；
E-2 rustfmt 全文件零 diff；E-3 L1×l5 组数待门禁复核（预期 26→23）。

**缺口**：G-1 全量门禁确认（本机排队，见 tripwire）；
G-2 **覆盖丢失事故**：首轮改写被外部 checkout 式覆盖还原（09:03 距检出 60 秒），
根因是改完未即交、暴露窗口 40 分钟。教训：改验证交必须同轮闭环，
本轮已做到（改→验→交同一 turn 内）。

**落点**：search.rs（+3/-14 行）＋BLUEPRINT v1.6.1。

**放行判定**：GO（改动机械可审；门禁复核转 tripwire）。

**tripwire**：门禁确认 L1×l5 计数 26→23（Owner：CI/下次全量门，日期：即时）；
svaf/bin 的 L5 路径改直引（Owner：L4/Desktop，日期：P1-02 内，不属 EQ-05）。

---


---

## §36 SIM-37：第七轮吸收（GO）

> 认领：单会话内完成。探针：3 组外部搜索（全命中）＋版本号三处对账。
> 生产代码零改动（脚本＋文档＋Makefile target）。

**探针**：P-1 OpenFeature/Flagd/TTL/杀伤开关/渐进发布；P-2 sigstore keyless
（Fulcio+Rekor）＋ Rust 原生验证库（sigstore-rust/jdx）；P-3 Socket 2026-08
proc-macro1 构建期攻击＋time/finch 战役＋Rust GA；
P-4 版本对账（蓝图头 1.5.9／志 1.6.1——头滞后，再犯必查三处）。

**证据**：E-1 自家 Cargo.lock 零命中（机验）；E-2 IOC 脚本双向验证
（clean 0／投毒 1，参数化后不再碰真文件）；E-3 `make -n` 全 target 可解析。

**缺口**：G-1 Flagd 后端＋渐进发布接线（P-task，SDB log-only 翻转为天然首用）；
G-2 cosign CI 作业（P3）；G-3 Socket 全套（P3）。

**外部方案**：见 ABSORPTION-ROUND7.md §1（S-39–S-43）＋否决项（vendor 锁／管制 key／企业版）。

**落点**：ROUND7＋supply-iocs 脚本＋Makefile target＋NTS-G09（STD 1.0.5）＋BLUEPRINT v1.6.2。

**蓝图回写**：即本轮（头/志/尾三处同升，治头滞后旧病）。

**放行判定**：GO（双向验证＋版本三处机验）。

**tripwire**：IOC 名单复核（Owner：Security，日期：新 advisory 即时）；
Flagd 首旗（Owner：L5，日期：SDB 翻转日）。

---


---

## §37 SIM-38：EQ-06 执行（GO）

> 认领：单会话内完成。探针：15 同名 TaskType 普查＋正典比对＋跨界读写审计。
> 生产代码：8 文件 import 改指（7 bank＋1 shield），零逻辑变更。

**探针**：P-1 TaskType×15 同名普查（types-crate 正典版为合并超集，12 原生全在）；
P-2 RewardSource 双版本逐字同一（变体/方法/值）；P-3 跨界审计
（field 读零、task_type 读全系他家字段、构造点全在 bank 内＋shield 一处同改）；
P-4 playback 判定（trait＋6 类型＋L2 实现者不可分拆→递延，非沉默跳过）。

**证据**：E-1 L1×l2 真数 34→27（恰为 7 改道行，一一对应）；
E-2 `cargo check --lib -j4` exit 0（1m42s）；E-3 fmt 全文件零 diff；
E-4 H-02 原"下沉"方案证伪两次（math 重导出案＋本次正典已存案）。

**缺口**：G-1 全量 --tests 门（转 CI，本地 OOM 旧疾）；
G-2 playback trait 拆分设计（P-task，需 L2 owner）。

**落点**：8 文件改指＋BLUEPRINT v1.6.3。

**放行判定**：GO（计数＋编译双证；行为面零变化）。

**tripwire**：CI 全量门首红先查他人文件（Owner：CI，日期：push 当日）；
playback 设计启动（Owner：L2，日期：P1-02 内）。

---


---

## §38 SIM-39：第八轮吸收（GO）

> 认领：单会话内完成。探针：3 组外部搜索（1 组传输失败转既定知识＋标注）＋
> 本地三处机验。生产代码零改动（文档＋2 条款）。

**探针**：P-1 Cedar 全家桶（语言＋Rust crates＋Atlas 2025＋42–60x）；
P-2 WASM 现状（wasmtime 已用＋registry 兼收原生 .so＋零加固机验）；
P-3 policy 单文件 935 行；P-4 版本三处对账（头 1.6.3／志 1.6.3／尾 1.6.3 一致）。

**证据**：E-1 WASM 零加固是机验事实（StoreLimits/fuel/WasiCtx/provenance 全缺）；
E-2 原生 `.so` 加载路径存在——即 tartanllama 警告的精确形态；
E-3 Durable 组搜索失败两次，已转既定知识＋诚实标注（promptfoo先例）。

**缺口**：G-1 Cedar crate 采用（需依赖评审 SIM）；G-2 WASM 加固代码（P-task）；
G-3 原生插件 verdict（dedicated SIM，否则维持 deny）。

**外部方案**：见 ABSORPTION-ROUND8.md §1（S-44–S-47）＋否决项（托管服务/原生默认/全文 SLSA）。

**落点**：ROUND8＋NTS-F10/E10（STD 1.0.6）＋BLUEPRINT v1.6.4。

**蓝图回写**：即本轮（头/志/尾三处同升，机验）。

**放行判定**：GO（版本号三处机验一致；条款计数下轮复核）。

**tripwire**：WASM 加固代码落地（Owner：L1/L3，日期：P-task 启动日）；
Cedar 依赖评审（Owner：L5，日期：P1-03 内）。

---


---

## §39 SIM-40：EQ-07 部分执行（GO）

> 认领：单会话内完成。探针：门面库存普查＋正典复用核查＋fmt 归属鉴定。
> 生产代码：5 文件改指（facade+1 别名、io/hive/svaf/bin 路径），零逻辑变更。

**探针**：P-1 门面库存（emotion 空、feel 有数字人、L3/L5 l1_facade 系 L1 向上便利贴）；
P-2 正典复用（multi_agent crate 含全部 6 hive 项；L5 系重导出）；
P-3 Lead 无正典（types-crate 零命中→递延）；P-4 MediaSource 不可分拆
（trait＋6 类型＋L2 实现者→递延）；P-5 fmt 归属（我行全洁，余皆历史）。

**证据**：E-1 旧路径归零 4 处（hive×3 位点、digital_human、svaf、bin）；
E-2 别名回绕保本地名不变（DigitalHumanEmotion as Emotion）；
E-3 改动文件 fmt 增量为零（新增 hunk 全归历史）。

**缺口**：G-1 server 路由器/state（门面 API 设计，P-task）；
G-2 hotreload 搬迁 vs 门面（P-task）；G-3 lead 正典（L4 owner）；
G-4 全量门确认（转 CI）。

**落点**：5 文件＋BLUEPRINT v1.6.4。

**放行判定**：GO（机械改指＋证据闭环；结构项有据递延非跳过）。

**tripwire**：全量门首红先查他人文件（Owner：CI，持续）；
门面 API 设计启动（Owner：L1/L3/L4，日期：P1-02 内）。

---

## §40 SIM-41：第九轮吸收（GO）

> 认领：单会话内完成。探针：全单去重 ~95（8 先读＋8 新读＋2 摘要）＋
> 本地三处机验（SIM 下号 41／STD 现版 1.0.6／蓝图现版 1.6.5 一致）。
> 生产代码零改动（文档＋4 条款）。

**探针**：P-1 去重计数（council×4 等 15 组重复，净 ~95）；
P-2 P0 新读 8（council/AERS/defense-harness/SkillOpt/deepteam/codex-security/LLMRouter/OpenResearch）；
P-3 摘要 2（2609.10883 imprinting／2609.20800 JEPA）；P-4 版本三处对账一致。

**证据**：E-1 PoC-only 跨越＋四条件补丁验收（defense-harness 实测形态）；
E-2 饱和停止三元组（codex-security 默认 stopAfterNoNew=3/maxRuns=10/maxTime=1.5h）；
E-3 held-out 门＋拒收缓冲（SkillOpt 17.3k★，52 格全优）；
E-4 工作树隔离＋不可变运行归档（OpenResearch，与 .worktrees/ 同构）。

**缺口**：G-1 SEAL 威胁模型＋奖励设计（L5，P-task）；G-2 EQ-08 分发器路由记忆
（L1，P-task）；G-3 运行归档不可变（P-task）；G-4 R10 P1 批次。

**外部方案**：见 ABSORPTION-ROUND9.md §2–§3（S-48–S-65）＋否决项（托管安全 SaaS/
原生默认/JEPA 运行时/无门自变异）。

**落点**：ROUND9＋NTS-F11/E12/E13/G10（STD 1.0.7）＋BLUEPRINT v1.6.6。

**蓝图回写**：即本轮（头/志/尾三处同升，机验）。

**放行判定**：GO（版本号三处机验一致；条款计数下轮复核）。

**tripwire**：SEAL 威胁模型启动（Owner：L5，日期：P1-03 内）；
EQ-08 路由记忆评估（Owner：L1，日期：P1-02 内）；
R10 P1 批次 triage（Owner：全组，日期：下轮吸收前）。

---

## §41 SIM-42：EQ-08~12＋T3 并行执行轮（GO-WITH-MITIGATION）

> 认领：单会话内完成。探针：dispatcher 只读扫描（1494 行；L5 用点全枚举）＋
> 门面库存（l1_facade 系向下便利贴，无向上 CoT/Crt/E8 出口）＋守卫阈值提取＋hook 现状。
> 生产代码：hook advisory 9 行（`bash -n` 过）；dispatcher 零改动（结构项，有据递延）。

**探针**：P-1 dispatcher L5 实结构引用 5 路（antidistil-decompose／CoTGenerator 实例化／
CrtPlan::new+decompose／E8Policy 持有／TraceSource+ReasoningTrace 构造）＋L2 predictor
load/persist（L687 函数内）；P-2 `l5_cognition/l1_facade.rs` 系向下重导出（L1/L3/L4），
无 CoT/Crt/E8 出口——改指门面不可行；P-3 三谓词代码实证（L700 快路径门／τ=0.65／&0x3f）；
P-4 六守卫阈值（0／0／≤1／0／3000·12.0／B2 零新增干净对）；P-5 pre-commit 现有 2 门，
doc-drift 基线 111 可跑。

**证据**：E-1 构造性使用（`DefaultCoTGenerator::new`、`CrtPlan::new`）≠ 剧场重导出，
下沉 L0 违层语义；E-2 hook advisory `bash -n` 过；E-3 drift 门 exit=0 持平 111。

**缺口**：G-1 dispatcher 构造注入（P1-02 门面 API 设计，L1/L5）；
G-2 三谓词语 Likert 化＋阻断翻转（P1-03＋ADR）；G-3 C 列 outbox 推广（P-task）；
G-4 EQ-11 四阈值 L5 书面签字（提案已备，待签）；G-5 全量门／P0 独占／P3 件转 CI。

**外部方案**：NTS-B06 时限 allowlist（到期 2026-10-31，过期即红）；
defense-harness/SIM-41 证据交接（finder/grader 分离，log-only 先行）；
AERS 同一评分器（首填低分是基线）。

**落点**：SDB v0.6（VP-1~3＋τ＋log 格式＋评分首填 1/2/2/1/1＋06/07 接线确定）＋
hook advisory＋EQ-08~12 状态推进＋BLUEPRINT v1.6.7。

**蓝图回写**：即本轮（头/志/尾三处同升，机验）。

**放行判定**：GO-WITH-MITIGATION（条件：allowlist 到期前 P1-02 启动；缺则 EQ-08 转红）。

**tripwire**：allowlist 到期 2026-10-31（Owner：L1/L5，每周复核）；
EQ-11 签字追踪（Owner：L5，日期：P1-03 内）；
全量门首红先查他人文件（Owner：CI，持续）。

---

## §42 SIM-43：P1-02 门面 API 设计（GO，设计态）

> 认领：单会话内完成。探针：构造位全枚举（只读）＋既有缝盘点＋阈值配置化。
> 生产代码零改动（设计先行，实现待可构建窗口＋独立 SIM）。

**探针**：P-1 构造位 4 处：`new()` L183 直构 `DefaultCoTGenerator::new`；
`decompose_task` L271 直构 `CrtPlan::new`＋`decompose()`；
`with_kernel/with_e8_policy` L211/217（缝已存在，仅持有）；
`execute_single_sub_task` L687 直调 L2 `predictor_load/persist`。
P-2 既有缝：`reasoning_engine: Option<Box<dyn ReasoningEngineProvider>>`＋
`with_reasoning_engine` L205，文件内注释已明示“L1 不直接依赖 L5 具体类型”——
作者留过门，只是 CoT/Crt/ predictor 三处没走。
P-3 配置化点：`DispatcherConfig` 已有 from_env 七键，τ=0.65 系字面量（L700 实证）。

**证据**：E-1 注入缝 precedent 在文件内（trait object builder）；
E-2 `kernel.is_some()` 已是 VP-1 合取项——缺失注入天然可观测；
E-3 τ 环境键缺失是 V-3 可调阈值要求的具体缺口。

**API 草案（Phase 2 实现契约）**：
D-1 `with_cot_generator()` 新增；`new()` 不再直构（None＋log 记录缺席进 dispatch 日志）。
D-2 Crt 规划走 `ReasoningEngineProvider` 扩展方法；`CrtTimeScale→hexagram` 映射留 L1
（确定性域逻辑，非认知构造）。
D-3 E8/Kernel 保持持有＋builder；缺席＝降级模式（VP 谓词记 false＋reason，可见性代替阻断）。
D-4 L2 predictor 抽 `PredictorStore {load,persist}` 缝注入；缺席则 VP-2 跳过＋记录。
D-5 `DispatcherConfig.confidence_threshold` 默认 0.65＋`NEOTRIX_DISPATCH_CONFIDENCE` 键，
钳位 [0.65, 1.0]（V-3：只许调严，调松需 ADR＋安全签字）。
D-6 路由记忆（R9 tripwire 收敛）：dispatch 日志累积 `{task_sig, confidence, class, decision}`；
top-k 相似历史调制单任务 aggression；与不可变运行归档互引（LLMRouter＋OpenResearch 共振）。

**缺口**：G-1 Phase 2 实现＋layer-deps dispatcher 计数归零＋单测绿（可构建窗口＋独立 SIM）；
G-2 Phase 3 allowlist 退役（2026-10-31 前）。

**落点**：本草案＋EQ-08 追记＋BLUEPRINT v1.6.8。

**放行判定**：GO（设计态；任何代码实现另立 SIM，否则按无 SIM 编码论处）。

**tripwire**：Phase 2 启动（Owner：L1/L5，条件：可构建窗口）；
allowlist 倒计时延续（Owner：L1/L5，每周复核）。

---

## §43 SIM-44：第十轮吸收（GO）

> 认领：单会话内完成。探针：R10 triage P0 短名单 8/8 深读（README＋关键行为摘录）＋
> 本地三处机验（SIM 下号 44／STD 现版 1.0.7／蓝图现版 1.6.8 一致）。
> 生产代码零改动（文档＋1 条款，预算 ≤2 用 1）。

**探针**：P-1 compaction 双子（never-rewrite＋τ=0.5 keep/drop／reduction 门／模型输出永不执行）；
P-2 路由＋扫描（dossier/replay 分离／fail-open＋kill-switch／双轮 0.35-0.85-0.6-0.7／永不执行目标）；
P-3 技能＋本地 judge（maturity 标签／trust 五问／meta-skill 闭环／未校准声明／checkpoint 信任警告）；
P-4 能力层＋评审流（有序后端＋真实探测＋doctor／编排在代码＋层向下＋finding 非定罪）；
P-5 版本三处对账一致。

**证据**：E-1 编排在代码＋阈值为常量（jev-review 可机验形态：check-dependencies 脚本）；
E-2 dossier/replay 分离与 SIM-43 D-1~D-6 同构（独立佐证）；
E-3 SEAL 候选阈值四数＋未校准声明（诚实口径，可直接入 ledger）。

**缺口**：G-1 本地 judge 引擎评估（P-task）；G-2 supply-iocs 语义扩展（P-task）；
G-3 SEAL ledger 阈值校准（P-task L5）；G-4 R10 A–G 余项＋R11（下轮 SIM）。

**外部方案**：见 ABSORPTION-ROUND10.md §2（S-66–S-73）＋否决项（云 judge 依赖/
全量翻转/H 类/新 SaaS）。

**落点**：ROUND10＋NTS-F12（STD 1.0.8）＋BLUEPRINT v1.6.9。

**蓝图回写**：即本轮（头/志/尾三处同升，机验）。

**放行判定**：GO（版本号三处机验一致；条款计数下轮复核）。

**tripwire**：本地 judge 评估启动（Owner：L5，日期：P1-03 内）；
R11 立项（Owner：全组，日期：下轮吸收前）。

---

## §44 SIM-45：第十一轮吸收（GO）

> 认领：单会话内完成。探针：A–G 余项 6 读封顶（5 实读＋Exegol 正文传输失败诚实记未读）＋
> 本地三处机验（SIM 下号 45／STD 现版 1.0.8／蓝图现版 1.6.9 一致）。
> 生产代码零改动（文档＋0 条款；零条款是决议，STD 停 1.0.8）。

**探针**：P-1 meta-skill（自写 lens／constraint footer／conservation law／增长环三局限）；
P-2 编排＋MCP（ScopeGuard 双层／dry-run／HMAC＋taint／10 工具逐 claim fail-closed／
风险轴→decide／模糊永不重试）；P-3 桌面双子（ref 稳定永不擅选／headless 默认／
权限层／Keychain／零遥测）；P-4 Exegol 传输失败（仅目录机验，不判内容）；
P-5 版本三处对账一致。

**证据**：E-1 jev_gate 双真才 auto 即 ADR-0004 Stage 门同构（独立佐证）；
E-2 ARES 六件套即 SEAL 骨架（可直接取用）；E-3 桌面 checklist 增补六条均有对照源。

**缺口**：G-1 Exegol 正文（R12）；G-2 本地 judge＋supply-iocs 扩展（P-task 延续）；
G-3 SEAL 骨架取用（P-task L5）；G-4 R12 A–G 余项＋H（下轮 SIM）。

**外部方案**：见 ABSORPTION-ROUND11.md §2（S-74–S-79）＋否决项（云 judge/新 SaaS/H 类/未读不判）。

**落点**：ROUND11＋BLUEPRINT v1.6.10（STD 停 1.0.8，决议记录在案）。

**蓝图回写**：即本轮（头/志/尾三处同升，机验）。

**放行判定**：GO（版本号三处机验一致；条款计数下轮复核）。

**tripwire**：SEAL 骨架取用（Owner：L5，日期：P1-03 内）；
R12 立项（Owner：全组，日期：下轮吸收前）。

---

## §45 SIM-46：第十二轮吸收（GO）

> 认领：单会话内完成。探针：余项 6 读封顶（5 实读＋Exegol 双失败永久折叠）＋
> 本地三处机验（SIM 下号 46／STD 现版 1.0.8／蓝图现版 1.6.10 一致）。
> 生产代码零改动（文档＋0 条款；STD 停 1.0.8 三轮决议）。

**探针**：P-1 RE IDE（Engine trait／grounded 引用／eval 独立二进制／懒分析＋降级）；
P-2 安全全景（扫描器矩阵／pinning／memory guard／flight recorder／凭据占位）；
P-3 具身＋抓取＋工作区（模块切换／独白透明／strip 注入／AutoThrottle／issue 钉证据／license 警告）；
P-4 Exegol 双失败折叠；P-5 版本三处对账一致。

**证据**：E-1 Engine trait 与 SIM-43 D-2 同构（独立佐证）；
E-2 eval-binary 形态可直接写入 EQ-17 工单（成本纪律）；
E-3 Exegol 两次传输失败记录在案（未读不判）。

**缺口**：G-1 H 类维持折叠；G-2 R13（需新 P0 出现，否则不开轮）；
G-3 SEAL 采购单取用（P-task L5）；G-4 本地 judge（P-task 延续）。

**外部方案**：见 ABSORPTION-ROUND12.md §2（S-77b–S-84）＋否决项（未读不判/云 judge/新 SaaS/H 类）。

**落点**：ROUND12＋BLUEPRINT v1.6.11（STD 停 1.0.8）。

**蓝图回写**：即本轮（头/志/尾三处同升，机验）。

**放行判定**：GO（版本号三处机验一致；条款计数下轮复核）。

**tripwire**：SEAL 采购单取用（Owner：L5，日期：P1-03 内）；
R13 门禁：新 P0 出现才开轮（Owner：全组，持续）。

---

## §46 SIM-47：Phase 2 dispatcher 代码实施（GO-WITH-CI-BACKSTOP）

> 认领：单会话内完成。Spike＝SIM-43 D-1~D-6＋PHASE2 施工单＋本轮锚点机验
> （hexagram_bias: Option<u8>／SubTaskClass: Copy／estimate_time_budget→f64／
> CrtTimeScale: Copy／serde_json 在 dependencies／tests mod L1306 起）。
> 生产代码：单文件 `l1_action/nt_core_task_dispatcher.rs`，E1a/E2/E3/E4/E5（record＋
> retrieval，调制接线 Phase 2b 不碰行为）＋E6 四纯测。本地不可构建（OOM），
> 验证＝重读＋grep＋rustfmt（如有）＋CI backstop（SIM-27/28 先例）。

**探针**：P-1 爆炸半径 2 调用方（entry/mod.rs:677 全注入／seal_loop.rs:1069 kernel+e8）；
P-2 `CoTGenerator` trait 零使用（仅 DefaultCoTGenerator）→ E1a 去 CoTConfig 导入；
P-3 `log::debug!` 既有可用；P-4 `unwrap_or` 存量（L733）不动，新增零 unwrap。

**行为增量（诚实）**：`new()` 不再按 `enable_cot` 自构 CoT——2 调用方 CoT 路径降级
（有日志），其余路径照常；组合根补 `with_cot_generator` 是 follow-up（本轮不动他文件）。

**缺口**：G-1 CI 绿（check --tests -j4＋dispatcher 单测＋layer-deps 计数记录）；
G-2 E1b 对象安全确认；G-3 调用方注入补齐；G-4 Phase 3 类型迁移（另 SIM）。

**放行判定**：GO-WITH-CI-BACKSTOP（首红先查他人文件；本文件红则单 commit 还原；
allowlist 与 EQ-08 🟨 不动）。

**tripwire**：CI 首跑结果（Owner：CI，PR 即验）；
调用方注入补齐（Owner：L1/L5，日期：Phase 2b 内）。

---

## §47 SIM-48：调用方补齐＋E1b 证伪＋Phase 3 探针（GO-WITH-CI-BACKSTOP）

> 认领：单会话内完成。Spike＝SIM-43/47＋调用位实测。
> 生产代码：`entry/mod.rs` 2 处（import＋with_cot 链）；seal_loop 零改（有据不补）。

**探针**：P-1 entry/mod.rs:677 调用方 gateway 为 Arc（clone 廉价），同文件已有
ReasoningKernel/E8Policy 同类导入——注入不新增违层类；
P-2 seal_loop:1069 已独立持有 `self.cot_generator`（`DefaultCoTGenerator::new(gw,
CoTConfig::from_env())`）＋dispatcher 侧 kernel+e8 双注入——重复构造浪费，
且 kernel 主路径完好，决议不补（非跳过）；
P-3 E1b：`execute_with_cot` 仅调 trait 方法 `generate_cot`（✓），但 trait 系原生
`async fn`（L115，无 async_trait）→ `Box<dyn CoTGenerator>` 非对象安全，
E1b **证伪**（不做；trait-boxing 需 async_trait 重构，另议）；
P-4 Phase 3 下沉位：L0 有 `nt_core_shared_types / nt_core_capability_types /
nt_core_traits` 三候选；E8Policy（纯数据＋default）可整体下沉；
CoTGenerator trait 可下沉（用点不变）；CrtTimeScale 枚举下沉／CrtPlan 留 L5；
Trace 系贴遥测（`nt_core_telemetry` 相邻）。

**行为增量**：entry 恢复 `enable_cot` 默认语义（与 E1a 前一致）；seal_loop 行为零变。

**缺口**：G-1 CI 绿（两文件）；G-2 Phase 3 下沉 SIM（Architect，P1-02 内）。

**放行判定**：GO-WITH-CI-BACKSTOP（同 SIM-47 条件）。

**tripwire**：CI 首跑（Owner：CI，PR 即验）；Phase 3 SIM（Owner：Architect，P1-02 内）。

---

## §48 SIM-49：并行收口（GO）

> 双路子代理（只读）＋主收口。生产代码零改动（文档＋清单＋修订）。

**A路（Phase 3 深探）**：E8Policy 字段（epsilon/decay/lr/discount/mode[64] 系＋select/update/best/decay/learn）；
CoT trait（generate_cot＋默认 batch）／Default{provider,config}；
CRT 变体＋helper 全／CrtPlan{scale/budget/max_ticks/sub_plans/parent}；
L0 shared 纯数据／traits 可容纳；引用计数 E8Policy 83／CrtPlan::new 13／ReasoningTrace{ 18。
**结论反转 SIM-48 P-4**：E8 留守（83 引用成本）／CRT 整体搬／CoT 拆分。

**B路（代码终审）**：dispatcher 括号差 0；with_crt_factory/with_predictor_store 仓外零调用
（预期内，pub 无警告，激活待 2b）；测试 helper 全在；entry 导入风格一致、
gw.clone＋复用确认；零残留（CoTConfig/l2/0.65 字面量全清）。

**CI 认证实测**：`git ls-remote` 读 OK；写 main 禁区；无 `gh`；
CI 仅 push-main/PR 触发——点火只能人来（已入清单 #1）。

**落点**：IGNITION-CHECKLIST（6 项人侧）＋PHASE2 SIM-49 修订＋BLUEPRINT v1.6.12。

**放行判定**：GO（纯文档；点火清单即交付物）。

**tripwire**：6 项逐项关（Owner：见清单，日期：见清单）。

---

## §49 SIM-50：CI 侦察（GO）

> 手段：公开 Actions API（匿名可读）＋jobs 下钻。零推送，零写操作。

**证据**：E-1 main 全红：Security Scan（09-21：sbom✗／security-scan✗／secret-scan✓）、
Security Audit（audit✗，09-20）、evolution-release（自 09-10 每日✗，慢性）；
E-2 本分支末跑 09-17（Security Audit✗）；E-3 本地 12 commit 从未推送——CI 没见过我的代码；
E-4 可观测窗内无 ci.yml（check/test/layer-deps）运行记录。

**结论**：点火≠门绿。PR 落上已红主干，首红归因先行（SIM-40 纪律）：
sbom/security-scan/audit/evolution 四红属既有，他人先修或注脚隔离；
我的两批代码只认 ci.yml check/test 两门。工单升级为：push→归因→分门裁决。

**落点**：EQ-13＋IGNITION #1 追记＋BLUEPRINT v1.6.13。

**放行判定**：GO（侦察即交付；点火仍待人）。

**tripwire**：PR 创建（Owner：人，即刻）；首红归因（Owner：CI，PR 即验）。

---

## §50 SIM-51：Phase 3 类型迁移提案（GO，proposed）

> 认领：单会话内完成。输入：SIM-48 P-4＋SIM-49 A路反转（E8 83 引用／Crt new 13／
> Trace 18）＋E1b 证伪。零代码（提案态；实现另立执行 SIM）。

**三决**：
D-3.1 E8Policy **留守 L5**（83 引用，下沉成本＞收益；改走 L0 trait 抽象新接口，
dispatcher 只依赖 trait——E1b 教训：trait 方法禁原生 async，签名前先定 async 形态）。
D-3.2 CRT **整体搬 L0**（`CrtPlan::new` 13 处；枚举＋planner＋helper 同迁；
`nt_core_shared_types` 相邻；迁移后 dispatcher 用线归零一半）。
D-3.3 CoT **拆分**（`CoTGenerator` trait 下沉 `nt_core_traits`；`DefaultCoTGenerator`
实现留 L5；用点签名不变）。

**顺序**：D-3.2 → D-3.3 → D-3.1（由易到难；每步独立 commit＋CI 门）。
**allowlist 退役计划**：Phase 2b builder 激活（with_crt/factory、predictor）与 D-3.2 同车；
退役条件＝dispatcher L1×L5/L2 计数归零＋全量门绿；deadline 2026-10-31 不动。
E1b async_trait 重构：否决（引入成本＞单文件收益；trait 下沉时统一 async 形态）。

**缺口**：G-1 Architect 签字（本提案→accepted）；G-2 可构建窗口（实现）。

**落点**：本提案＋BLUEPRINT v1.6.14。

**放行判定**：GO（提案态；签字前任何实现按无 SIM 编码论处）。

**tripwire**：Architect 签字（Owner：Architect，日期：P1-02 内）；
allowlist 倒计时延续（Owner：L1/L5，每周复核）。

---

## §51 SIM-52：M1–M7 批量执行（GO）

> 手段：ADR-0005（M1）＋F03 playbook（M3）＋Footer 惯例（M5）＋supply-iocs 扩展
> （M2，实跑通过）＋H 抽验（M6，H-01/H-02 归档 H-05 保留）＋fuzz 骨架（M4）＋
> 蓝图/复验（M7）。drift 111 全程持平。

**证据**：
M1: `docs/adr/0005-seal-threat-model-skeleton.md`（proposed，三层骨架：L1 门/L2 表/L3 纲）。
M2: `scripts/check-supply-iocs.sh`（+38 行，advisory A/B 双检查，实跑 IOC 全绿，advisory 三告警均为已知）。
M3: `docs/architecture/MEMORY-OVERFLOW-PLAYBOOK.md`（compaction 四件＋volume 纪律）。
M4: `fuzz/`（独立 crate，3 harness：confidence/compactor/dispatch_plan；nightly only，cargo test 零影响）。
M5: `docs/architecture/REVIEW-FOOTER-CONVENTION.md`（三行 footer 惯例）。
M6: `docs/architecture/ABSORPTION-R10-TRIAGE.md` H 抽验结论＋蓝图 §15 D-02 H-01/H-02 归档行。
M7: drift 111 持平，fuzz 骨架不入 workspace（零回归面）。

**结论**：M1–M7 全部完成。SIM-52 作为批量执行 SIM 收口本轮全部小任务。

**放行判定**：GO（纯文档/脚本/骨架，零运行时行为变更）。

**tripwire**：无（本轮无 P0 门依赖）。

---

*End of SIM Protocol v1.0.0 —— 下一编号 SIM-53.*

<!-- 路径说明（2026-09-28 追加，**不重写上文**）：上文出现的 `src-tauri` 与
     `apps/neobot-desktop` 均为**当时的历史路径**。`src-tauri` 于 `5c02e738` 归档、
     `apps/neobot-desktop` 于 `d5413335` 删除，桌面 App 统一到独立仓
     `~/Downloads/Neo/neobot`。改写历史记录等于伪造当时的事实，故只加此注记。 -->
