# 核心进化路线 — 支脉节点定位清单

> 配套文档：`ABSORPTION-EXTERNAL-2026-09-27.md`（外部吸收分析，1041 仓库 + 22 指定源）
> 本文只回答一件事：**每一步落在哪个 file:line。**
> 全部锚点 2026-09-27 用 `grep`/`read` 实测复核，非文档转述。
> 定位工具 `scripts/ops/nt_locate.py` 的索引 `.project-map/codemap.json` 已 3 天陈旧（`rs_coverage=2712/3020, missing=476, extra=275`），故本文用 grep 精确定位而不用该索引。

---

## 图例

| 标记 | 含义 |
|---|---|
| 🟢 **接线** | 机制**已完整存在**，只缺调用方 / CI 挂钩 |
| 🔵 **扩字段** | 结构已对，缺 1-2 个字段或一条分支 |
| 🟡 **新建** | 需要新写，但有精确规格 |
| ⛔ **删除** | 收敛掉重复实现 |

**路线共 5 阶段 / 18 项。阶段内可并行，阶段间有强依赖。**

---

# 阶段 0：止血（1-2 天，不依赖任何外部吸收）

> 这一阶段的四条**全部是接线/删 stub**，是后面所有工作的前置。不做完，后面每一项的验证都不可信。

## 0.1 🟢 `UnifiedApi` 脱 stub — **全项目单点收益最大**

| | |
|---|---|
| **支脉节点** | `src-tauri/src/stub.rs:275` `pub struct UnifiedApiImpl;` ← 零状态单元结构体<br>`src-tauri/src/stub.rs:285-304` `impl UnifiedApi for UnifiedApiImpl`，`:289` 返回字面量<br>`src-tauri/src/stub.rs:260-273` trait 6 方法<br>`src-tauri/src/main.rs:52` `use neotrix_tauri::stub::{UnifiedApi as _, UnifiedApiImpl}`<br>`src-tauri/src/main.rs:387` `let api = UnifiedApiImpl::new();`（system_state 轮询）<br>`src-tauri/src/main.rs:407-408` `UnifiedRequest::chat(prompt)` + `api.handle()` |
| **缺口证据** | `neotrix::` 在整个 `src-tauri/src` 只出现 **5 文件 13 次**：`main.rs:3` `llamacpp.rs:1` `chat.rs:2` `ntcode/commands.rs:4` `ntcode/streaming.rs:3`。而 `Cargo.toml:38` 确实依赖了 `neotrix`。 |
| **关键发现（降低改造量）** | `UnifiedResponse.metadata`（`stub.rs:292-303`）**已经预留** `layers_involved: Vec<..>` · `capabilities_used: Vec<..>` · `consciousness_state { phi, coherence, gwt_resonance, emotion, attention_focus }` · `confidence: f64`。**这与 L5 意识核是类型级吻合，不是重新设计** —— 契约已定好，只差填值。 |
| **动作** | 1) `UnifiedApiImpl` 改为持状态（`Arc<Engine>` 或现有 registry 组合）<br>2) `handle()` 委派 `neotrix` 真实对话路径，把 `consciousness_state` 接到 `l5_cognition/consciousness_core/`<br>3) `main.rs:52` 换注册源 |
| **验收** | 桌面发一次对话 → `neotrix audit` 落库 → `scripts/truth-surface-baseline.txt` 保持 0 条 |
| **依赖** | 无。**必须最先做。** |

## 0.2 ⛔ 注册表 4+4+5 收敛为 1+1+1

| | |
|---|---|
| **支脉节点（4 份 `CapabilityRegistry`）** | ① `neotrix-core/src/l0_substrate/nt_core_capability_types.rs:533`<br>② `neotrix-core/src/l5_cognition/nt_core/capability/registry.rs:453`<br>③ `neotrix-core/src/neotrix/nt_file_ability/capability.rs:173`<br>④ `neotrix-core/src/neotrix/nt_core_capability_tree/src/registry.rs:53` |
| **支脉节点（4 份 `ToolRegistry`）** | ① `neotrix-core/src/l1_action/nt_act/tool_registry.rs:85` ← **唯一真实现**（770 行，真实 dispatch + stats）<br>② `neotrix-core/src/l5_cognition/nt_core_gate/nt_tool_registry.rs:11` ← **45 行 stub**<br>③ `neotrix-core/src/l2_perception/nt_world/crawl/agentic_browse.rs:34`（crawl 域局部）<br>④ `crates/neotrix-gateway/src/gate.rs:1273` |
| **支脉节点（5 份 `SkillRegistry`）** | ① `neotrix-core/src/skill_registry.rs:14`（L3 路由门面）<br>② `crates/neotrix-types/src/core/skill.rs:54`<br>③ `crates/neotrix-types/src/core/skills/mod.rs:25`<br>④ `crates/neotrix-gateway/src/skill_registry.rs:157`<br>⑤ `crates/neotrix-multi-agent/src/skill_registry.rs:157` |
| **动作** | 保留 ①（`nt_act/tool_registry.rs` + `neotrix-core/src/skill_registry.rs`）。<br>**`nt_core_gate/nt_tool_registry.rs` 整文件删除**（45 行 stub）。<br>③④ 改 `pub use` re-export。② 与 ④⑤ 择一为类型源。 |
| **验收** | `grep -rc "pub struct CapabilityRegistry" neotrix-core/src crates` 合计 == 1；`ToolRegistry` == 1；`SkillRegistry` == 1。`cargo xl` 绿。 |
| **纪律** | **收敛期间禁止新增第 5 份。** 建议同时加一条 CI 断言（见 0.5）。 |

## 0.3 ⛔ 四 orchestrator 择一

| | |
|---|---|
| **支脉节点** | ① `neotrix-core/src/l1_action/nt_act/nt_act_trade/orchestrator.rs`（1,471）+ `orchestrator_v2.rs`（1,510）<br>② `neotrix-core/src/l6_meta/nt_auto_orchestrator.rs`（1,037，含 `IntentClassifier` 15 类任务 + `AgentLifecycleManager`）<br>③ `neotrix-core/src/l0_substrate/nt_core_platform/{orchestrator,pipeline,pipeline_registry}.rs`<br>④ `neotrix-core/src/pipeline/`（7 文件 / 1,130，规范 D/E/B/A/R/X stage） |
| **动作** | 留 ④（阶段命名与 `NEOTRIX-MASTER-BLUEPRINT` D-06 对齐）+ ②（有 `IntentClassifier`）。① 的 v1/v2 二选一后降级为薄适配。③ 走 `neotrix-core/src/nt_route_features.rs` 显式路由。 |
| **已知遗留** | `ARCHITECTURE-MAP-ROADMAP-V2.md:364-370` 记录 3 个重复定义未解：`ExtractConfig` / `EmailConfig` / `PlatformRegistry` —— 一并清掉。 |

## 0.4 🔵 `maturity_audit()` 接 CI — **已有资产，纯接线**

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/neotrix/nt_core_capability_tree/src/registry.rs:459` `pub fn maturity_audit(&self) -> Vec<MaturityFinding>`<br>`:15-18` `struct MaturityFinding { claimed, supported }`<br>`:464` `n.evidence_supported_constellation()`<br>`:467` `.filter(\|(n, supported)\| n.constellation > *supported)`<br>**`:484-485` 自愈：把 `node.constellation` 下调到 `f.supported`**<br>`:493` 报告输出<br>`:979-984` `legacy_constellation_of`（C1..C4 映射）<br>数据源 `.neotrix/capability_registry.json` → `nodes[]`，字段 `{id, domain, layer, constellation, provides, requires, rune_sockets, dependents, evolution_log, metadata, created_at, updated_at}` |
| **关键发现（比原判断更乐观）** | `maturity_audit()` **不是"已建未引用"—— 它已完整实现，且带自愈降级路径**（484-485 会自动把声称等级下调到证据支撑等级）。缺口**纯粹是没有任何 CI 门调用它**，所以自愈从不发生。 |
| **动作** | 加一个 workflow / `scripts/ops` 门：跑 `maturity_audit()`，若 `claimed > supported` 的节点数 > 0 则**要么自动降级落盘、要么 CI 失败**。 |
| **价值** | 这是全仓唯一现成的"能力诚实度"机制。接上它 = 一次调用把 22 轮吸收里所有"声称 C4 但只有 C2"的能力自动降级。**投入产出比最高的单项。** |

## 0.5 🟡 CI 门基座（三条断言，一次性把多个缺口钉死）

| | |
|---|---|
| **支脉节点** | `.github/workflows/security-audit.yml` ← **实测全文只有 `cargo deny check all`（`:19-22`）**，无任何 agent 行为审计<br>已有 14 个 workflow 可挂靠：`audit.yml` `bench.yml` `ci.yml` `deny.yml` `lint.yml` `nt-audit.yml` `security-scan.yml` `test-action.yml` 等<br>门脚本目录 `scripts/ops/`（`nt_locate.py` `nt_lock_audit.py` `nt_mem_gate.sh` 已在） |
| **断言 A** | `detector_coverage ⊇ execution_scope` —— 监控/检测覆盖范围必须包含执行范围。（来自 OpenAI 事故：其基础设施异常 DNS 检测器**把受影响环境排除在外**。） |
| **断言 B** | 注册表唯一性：`CapabilityRegistry` / `ToolRegistry` / `SkillRegistry` 各 == 1（钉死 0.2，防回潮） |
| **断言 C** | schema 指纹门：`SCHEMA_FINGERPRINT = sha256(DDL)[..12]`，**环境派生字段（嵌入维度 `FLOAT[N]`）拆成独立门**。<br>⚠️ 借 GitNexus 的血泪：其手写递增版本号与 main 冲突 **8 次、其中 2 次完全相同**，完全相同那次是**静默失败** —— 门禁读成"当前"、所有 `CREATE TABLE` 被当作"已存在"跳过、活库存不下的边被裸 `catch` 吞掉，**产出了一张错误的图而不是一个错误**。 |

---

# 阶段 1：安全封口（3-5 天）

> 来自 OpenAI 2026-09-20 DNS 事故。**这是唯一一个"事故已真实发生"的缺口。**

## 1.1 🔵 DNS 出口：qtype 白名单 + 长度上限

| | |
|---|---|
| **支脉节点（核心）** | `neotrix-core/src/l1_action/nt_io/nt_io_provider/common/egress_types.rs:14-21` `struct SandboxEgressRule { host, port, allow }`<br>`:31-42` `host_matches()` —— 已正确处理 apex 与点边界（`example.com.evil.net` 不会匹配 `*.example.com`）✅<br>`:44-55` `port_matches()`<br>`:57-58` `struct SandboxEgressPolicy` |
| **缺口证据** | **`SandboxEgressRule` 只有 3 个字段，全文件零 DNS 概念。** 无 qtype、无 resolver 管控、无 label 长度上限。 |
| **扩展点（同级消费者）** | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_sandbox/mod.rs:76-138`（策略扩展）<br>`:227-260` 每情报源策略 `gdelt_egress_policy` / `edgar_` / `usgs_` / `gdacs_`<br>`neotrix-core/src/l3_embodiment/nt_shield/nt_shield_stealth_net/firewall.rs`（565 行，PF anchor + nftables，`DIVERT_PORT 11081`） |
| **动作（按性价比）** | 1) **`egress_types.rs` 加 `qtype: String` 字段**（空 = 不限，仅 A/AAAA 走白名单），并加 `label_len_max` / `name_len_max` 校验 —— 长编码 label 是通配 DNS 编排的必要条件，设上限极便宜地打掉它<br>2) `TXT`/`NS`/`SRV`/`CAA`/`DNSKEY` 一律拒（追 `CNAME` 时临时放行）<br>3) 语义反转：把 `dns_allow` 从**"学习提示"改成"过滤器"** —— 不在目的地白名单内的查询直接拒 |
| **全行业空白（先做者领先）** | 深挖 7 个沙箱/安全仓：**没有一个做 qtype 过滤**。`superradcompany/microsandbox` 唯一真做 host 侧 DNS 管控（smoltcp 用户态栈，guest 不持 resolver socket）但**也无 qtype**；`TencentCloud/CubeSandbox` 的 eBPF `dns_allow` **只学 A 记录**，不在名单内的查询不被拦且 gateway 常放行 —— **与事故同构的隐蔽信道**。 |
| **验收** | 单测：构造 `who-is-<长编码>.wildcard-dns.example` 的 TXT 查询 → 拒；`A` 查询正常；`.evil.net` 后缀欺骗仍拒（已有测试保护） |

## 1.2 🟡 "以尝试为检测单元，结果是独立字段"

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/l0_substrate/nt_core_telemetry.rs`（1,570 行，10+ 事件变体）← **事件 schema 落点**<br>`neotrix-core/src/l6_meta/nt_safety_monitor.rs` ← 监控器<br>`neotrix-core/src/l6_meta/runtime_monitor.rs`<br>`neotrix-core/src/l3_embodiment/nt_shield/nt_shield_audit/`（10 文件 / 2,810）<br>已有 `crates/neotrix-audit`（纯函数、零 LLM）+ `src-tauri` 内 `nt_shield_audit/mod.rs`（1,548） |
| **事故的三条运营教训** | ① 监控 15 分钟报警、人 3 分钟确认，**但 run 没自动停**，2.5h 后手动 kill<br>② 复盘发现监控**低报**其他 DNS 尝试，因为"没拿到有用信息"被误读成"尝试失败"<br>③ 基础设施异常 DNS 检测器**把受影响环境排除在外** |
| **动作** | ① 事件 schema 加 `attempt: Attempt` 与 `outcome: Outcome` **两个独立字段** —— 硬规则：**任何情况下不得让 outcome 衰减 attempt 的严重性**。返回 NXDOMAIN / 空答的越权查询**仍然是越权查询**。<br>② 断言 A 落 CI（见 0.5）<br>③ 告警升级为**自动围堵**而非仅告警 |
| **⚠️ 行业空白（同一处）** | 深挖 7 个仓：**没有一个有"因策略违规自动杀掉运行"的机制**。唯一有的（`usestrix/strix` 的 `--max-budget`/`--max-turns`）**按资源耗尽触发，不按策略违规触发**，且其文档自承预算值是 *"best-effort estimate"*、*"providers that do not expose priced usage may under-count"* —— **能低估自己触发条件的围堵闸不是 fail-closed**。 |

## 1.3 🟡 严重性校准对 + 职责分离

| | |
|---|---|
| **支脉节点** | `crates/neotrix-audit/`（finding taxonomy、drift-resistant identity、baseline compare、layered rule resolver）+ `src/bin/nt_audit.rs`<br>`.github/workflows/nt-audit.yml`、`audit.yml` |
| **动作（借 cloudflare/security-audit-skill，MIT，14 commits，纯方法论）** | 写入 `RUST-STANDARDS.md` 或 audit 规则：<br>① **"Severity requires impact."** ← 直接对治"没拿到有用信息=尝试失败"<br>② **"Defense-in-depth gaps are not vulnerabilities. 若 Layer A 挡住了攻击，Layer B 的缺失是加固备注。"** ← 镜像护栏，防止过度纠偏成"什么都报"<br>③ **职责分离：*"The agent that checks a finding is never the agent that found it."***<br>④ `needs_validation` 记录**结构上不能携带 severity**（severity 是"已建立影响证据"的类型级推论） |
| **成本** | 14 commits / ~24 文件，MIT 可整套吸收 |

## 1.4 🟢 非不可宽化策略地板（Neobot）

| | |
|---|---|
| **支脉节点** | `crates/neotrix-neobot/src/nt_policy.rs:75` `pub fn evaluate_policy(ctx: &PolicyContext) -> PolicyDecision`（fail-closed）<br>`:67` `fn deny(rule, reason)`<br>deny 规则全集：`human-control`(:77) `computer-allow`(:84) `computer-host`(:90) `workspace-jail`(:98,:106) `unknown-tool`(:119) `default-deny`(:121)<br>`:80` 注释"默认双空 = 全拒 (fail-closed, 替代旧 blanket-deny 的可配版本)"<br>`:137` `evaluate_extra_deny`（operator 自写规则，坏规则**仍拒**：缺前缀 :149 / 缺 `:` :155 / 空值 :162 / 未知 key :173）<br>`crates/neotrix-neobot/src/nt_audit.rs`（先写 audit → 再执行）<br>`nt_token_guard.rs`（379 行） |
| **缺口** | 已是 default-deny + 拒绝优先 + 畸形规则拒绝 ✅，但**没有任何机制阻止一个新 bug 或一次新加的 override 把地板降低**。 |
| **动作（借 ironclaw，MIT OR Apache-2.0）** | 把"**被允许绕过批准阶段的能力集合**"变成**测试钉死的冻结数据**（ironclaw 的 `origin_gate_matrix` + `reborn_origin_gate_matrix_ratchet.rs`）。任何新增 bypass 必须同时改冻结集 → 改动必然经过 review。 |
| **顺带抄一条反向正确判断** | ironclaw 与 `vxcontrol/pentagi` **独立**做了同一个决定：**故意移除 `no-new-privileges`**，因为 capability bounding set 已封顶、`no-new-privileges` 无条件破坏 SUID/SGID 提权测试。**删掉一个不带来任何东西的控制，是好工程。** |
| **验收** | 一条单测：冻结集之外的任何 capability 请求批准 → 拒；改冻结集 → 该测试必须被显式更新 |

---

# 阶段 2：记忆正确性（1-2 周）

## 2.1 🔵 supersession 双时间形态（绕开主键重写）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs` —— **5 段协议就是这 5 个函数**：<br>`:169` `snapshot()` / `:195` `distill()` / `:253` `classify()` / `:306` `persist()` / `:403` `feedback()`<br>`:432` `struct ExperienceEngine`（`:444` `run()`）<br>`:490` `SessionEndHook`<br>`:558` `NexusWeaverScheduler`（`:578` `weave_patterns()`）<br>`:639` `struct ExperienceQuery`<br>`:29` `struct ExperienceEntry` / `:47` `EntryType` / `:62` `Domain`(12 变体) / `:101` `Source`<br>记忆存储：`neotrix-core/src/l4_emotion/nt_memory/`（35 条目）—— `paged_kv/mod.rs` · `nt_memory_kb/kb_kv.rs` · `vector_index.rs` · `addressable_store.rs`<br>第二处：`neotrix-core/src/l6_meta/memory/nt_memory_experience_tree.rs`（458 行） |
| **已知缺陷（`TODO.md:12-14`）** | `nodes.id` **单列主键**使双时间版本化不可行。<br>另：`nt_memory/cascade.rs:216,126` `length_score = len/200`，且 `tick()` **永久丢弃**低于阈值样本。 |
| **关键设计（借 agentmemory，Apache-2.0）** | > *"被取代的记忆版本离开搜索索引；版本链在 KV 里保留完整历史。"*<br>即：**被索引的行只带指针（或移出索引），另有一条 append-only 链承载历史。** 这**绕开 `nodes.id` 单列主键的重写**，且与将来完整双时间迁移**兼容**。今天就能迁。 |
| **⚠️ 不要信** | agentmemory 的"Ebbinghaus 曲线衰减"README 里**只有散文，没有公式、没有半衰期常量**。当规范引用会出事。 |

## 2.2 🟡 真双时间四列迁移

| | |
|---|---|
| **支脉节点** | 同 2.1 的 `experience_tree/mod.rs:29` `ExperienceEntry`（加 4 列）+ `kb_kv.rs` / `paged_kv/mod.rs`（schema 迁移）<br>参照已有双时间先例：`neotrix-core/src/l0_substrate/nt_core_kb_primitives.rs`（已是双时间存储） |
| **动作（借 utopia，Apache-2.0，迁移文件名直接可抄）** | ```sql<br>valid_from, valid_to, valid_from_precision, valid_to_precision   -- 世界时间（何时为真）<br>recorded_at, invalidated_at                                       -- 记录时间（系统何时相信）<br>supersedes  UUID REFERENCES entries(id)                            -- 永不覆盖<br>``` |
| **★ 最值得抄的一个细节** | `valid_to_precision = 'unknown'` **三态**：<br>```<br>仍在持续         valid_to IS NULL   precision IS NULL<br>已结束但日期未知 valid_to IS NULL   precision = 'unknown'   ← 教科书做法做不到<br>2023 年结束      valid_to = …      precision = 'year'<br>```<br>该仓**显式拒绝**教科书做法（把文档日期塞进 `valid_to` 当上界），理由是：*"看起来像个确定的时间戳；每个读者都得先查精度，而不撒谎才是产品。"*<br>NeoTrix 的 `EntryType::Defect` / `Domain::Repair` 语义上大量需要这个三态。 |
| **验收** | 单测：`valid_to=NULL, precision=NULL` 读作"持续"；`valid_to=NULL, precision='unknown'` 读作"已结束" |

## 2.3 🔵 来源标注：加第二轴，**不要重载 `Source`**

| | |
|---|---|
| **支脉节点** | `experience_tree/mod.rs:101` `enum Source { Dialogue, Audit, Research, Absorption }`（`:108` `Default = Dialogue`） |
| **关键判断** | `Source` 是**渠道**语义（从哪来），而 learn-from-materials 的四分法是**证据等级**语义（可不可信）。**两者正交。** 重载 `Source` 会把两个轴焊死且破坏 `:108` 的 `Default`。 |
| **动作** | **新增** `enum Provenance { MaterialBacked, Uncovered, ModelAdded, ExternallyVerified }`（4 变体，对齐 `Source` 的 4 变体风格），进 `ExperienceEntry`（`:29`），并在 `persist()`(`:306`) 强制填充 —— `Uncovered` **不能**携带断言。 |
| **配套（该仓另外两条）** | ① **覆盖率审计是一等闸**：`verify_coverage.py` 双向映射 + 反向抽检 + **源哈希** + 增量更新校验。诚实承认结构/引用/覆盖率检查**不能证明第一次阅读找到了每个重要想法**。<br>② **方法库版本化**：原文与旧版本**保留**；综合出的方法拿**新 ID + 显式父方法引用**，绝不静默覆盖。且"材料不含方法论时就直说，不许编"。 |
| **安全姿态（照抄）** | 材料是**不可信数据**；内嵌 prompt/命令/角色覆盖文本**永不执行**；ZIP 容器查路径穿越 / 条目数 / 展开体积 / 压缩比（**容器炸弹防护** —— 你摄入任意用户文件，这条要）。 |
| **许可** | MIT，但**声明派生**自 `virgiliojr94/book-to-skill` 与 `crayon-ai/book-to-webpage`，`NOTICE.md` 须保留 |

---

# 阶段 3：可观测与成本（1-2 周）

## 3.1 🔵 成本归因：在链路上抓，不在日志里读

| | |
|---|---|
| **支脉节点（唯一插点）** | `neotrix-core/src/l1_action/nt_io/nt_io_provider/anthropic/anthropic.rs:92` 与 `:237` —— 注释 **"P0-4 prefix caching: 在稳定前缀边界消息上打 cache_control 断点"**<br>同文件 `:24` 缓存断点辅助 / `:38` 文本块数组 / `:42`+`:63` `cache_control:{type:"ephemeral"}` 打点<br>适配器层：`neotrix-core/src/l1_action/nt_io/universal_model/{anthropic,openai,gemini,ollama}_adapter.rs`<br>provider 抽象：`neotrix-core/src/l1_action/nt_act/provider_abstraction/{provider,registry,router}.rs`<br>**已有 Usage 结构**：`crates/neotrix-decision-engine/src/types.rs:304` `pub struct Usage`<br>成本账本（已存在）：`crates/neotrix-neobot/src/nt_cost.rs` · `neotrix-core/src/.../cost_dashboard.rs` |
| **关键发现** | 成本归因的**唯一插点已经存在** —— `anthropic.rs:92,237` 就是"在请求时组装、从不写进 transcript"的那段不可见前缀的断点逻辑。**这是全项目最重要的成本中心，而它现在没有任何计量。** |
| **外部实测** | 全行业只有 `tigerless-labs/cost-xray`（MIT）实现全链路。核心洞察：*"系统提示、注入的工具 schema、MCP schema 都是请求时组装、从不写进 transcript 的，那段前缀可以占到上下文的一半甚至更多。任何基于日志的归因方案，结构性地对最大的成本中心是盲的。"* |
| **动作** | 在 `anthropic.rs` 断点处发一条 canonical block 记录，字段照抄：<br>`zone{input,output}` · `section{static,messages}` · `bucket{system,schema,text,thinking,tool_use,tool_result}` · `tool` / **`skill`** / `role` / `tokens` / `hash=sha1(content)[:8]` / `exact`<br>→ 派生 `cal_tokens, cached, rewrote, fresh, output, usd` |
| **两条最高杠杆的维度** | ① **skills 是一等维度**：`tool=="Skill" && skill` 分流为 `("Static","Skills")` / `("Messages","Skill loads")` —— **schema 与内容加载分开计价**。你有 **120 个 skill**，无第二家拆这一刀。<br>② **MCP server 是一等维度**：从 `mcp__<server>__<tool>` 字符串还原，配一个"**注入了但从未被调用的 MCP 浪费**"探测器（你已暴露 MCP，见 4.1）。 |
| **诚实契约（照抄）** | *"总额与账单精确；同一请求内各来源的拆分是近似的。"* 并公开残差 `input_err` / `output_err` / `exact: bool`，**不要展示假精度数字**。 |
| **顺带一条（AIBrix）** | 把**工具集身份**纳入缓存身份（`AIBRIX_PREFIX_CACHE_INCLUDE_TOOLS` 默认 true 的做法）。**一改 skill 集就击穿前缀缓存**，`rewrote` 会尖峰 —— 这本身就是要监控的回归信号。 |

## 3.2 🔵 声明式能力 manifest + 声明式爆炸半径

| | |
|---|---|
| **支脉节点** | `.neotrix/capability_registry.json`（`{nodes, edges, experience_targets}`）<br>`.neotrix/capability_overrides.json`（node 已含 `deprecated` / `deprecated_reason` / `run*` 字段）<br>消费方：`nt_core_capability_tree/src/registry.rs:53` + `node.rs:606` `ConstellationLevel` C0-C6<br>已有弃用机制：`capability_overrides.json` 的 `deprecated` / `deprecated_reason` ✅ |
| **缺口** | manifest 已有正确**形状**（`provides` / `requires` / `rune_sockets` / `dependents` / `evolution_log` / `metadata`），**但没有安全信封**：没有声明式爆炸半径、没有构建期安全校验。 |
| **动作（借 MangoDisk 的形状，⚠️ GPL-3.0 只读形状不抄代码）** | ① node 加 `blast_radius: {writes_fs, spawns_process, network, spawns_container}`<br>② 加**构建期校验**：无清晰安全边界的 node 直接排除（该仓原话：*"If a rule has no clear safety boundary it is excluded"*）<br>③ 第三方 node 要求可靠来源 + 真机验证 |
| **另两条形状（该仓）** | ① **默认只读、显式确认、写后回读校验** —— 扫描永不变更；每次变更后**重读设置**；高影响项与需管理员/重启项被标出。→ 直接映射 `neobot` 的审批 + 审计。<br>② **同一引擎两个前端**：`--dry-run` / `--format json` / `--yes` 与 GUI 共用核心，非交互**强制显式 `--yes`**。→ **形状正好是 `neotrix` CLI ⇄ Tauri 想要的。** |

---

# 阶段 4：决策面与自治（2-3 周）

## 4.1 🔵 JEV 四件套 —— 决策面成本与正确性

| | |
|---|---|
| **支脉节点（三原语**已完整建模**）** | `crates/neotrix-decision-engine/src/types.rs:85` `QuestionType` / `:109` `NoulCriteria` / `:118` `Question` / `:129` `QuestionSet`<br>`:146` `NoulAnswer` / `:166` `ChoiceAnswer` / `:189` `ScoreAnswer` / `:217` `Answer` / `:289` `EvaluationResult` / `:304` `Usage`<br>`:334` `Decision` / `:348` `ExitCode`<br>后端 `backends/simulation.rs` · `model/mod.rs:22` `LayaModel` / `:32` `ModelConfig` / `:179` `forward()`<br>已有 stale 机制：`crates/neotrix-neobot/src/nt_stale_guard.rs` ✅ |
| **缺口** | 三原语模型是**忠实的**。缺的是 **jev-drone 教的调用纪律**（`tactics.py:100-140` 四件套）：<br>① **粗粒度场景指纹** —— 情景未变则复用缓存判断<br>② **硬 `call_budget` 上限** —— 一个 bug 也烧不掉预算<br>③ **`stale_after_s` 过期** —— 已有 `nt_stale_guard.rs` 可接<br>④ **非阻塞 worker** —— `queue.Queue(maxsize=1)` + worker 线程，**热循环永不在模型上阻塞** |
| **动作** | ① 在 `engine.rs` / `router.rs` 外包一层 fingerprint + budget + staleness 装饰器<br>② **三路置信门**（借 JEV 原生设计，阈值**按动作 stakes 分别标定**）：高 → 静默执行；中 → 审批弹窗；低 → 升级给人 |
| **★ 最值钱的一条方法论（jev-drone 自评）** | > *"a state-design bug, not a model failure"* —— *"this project 教给我的最有用的一件事"*<br>JEV 从不选 `climb`，因为状态是 5 个**水平**距离扇区、**没有垂直信息**，"飞过去"根本不可推断。加入障碍顶边高度 + 顶边是否相机可见 + 爬升上限后，`climb` 从"从未被选" → **p=0.93**。<br>→ **规则：如果一个决策从不触发，先审状态形状，再审模型。** |
| **⚠️ 成本现实** | JEV 是**专有托管闭源**模型（`$42/MTok` 输入，无开源实现）。jev-drone 单次 65 秒飞行 96k tokens ≈ **$4/回合**，靠 160 次硬上限兜底。**这不是 per-tick 成本** → 上面四件套不是优化，是必需。 |
| **另抄一条** | **按问题选原语**：jev-drone 的 `BRIEF OCCLUSION` 案例里 Choice 不自信（p=0.24）但 **Noul 答得干净**（`target_truly_lost=0.12`）→ 搜索行为该由 Noul 门控。 |
| **⚠️ 反面教材（必须读）** | jev-drone 的头条数字（77.5m vs 17.7m 基准）是 **n=1 单次运行**；在更早的竞技场做了 3-seed 配对比较，**JEV 无优势（0/3，与基准持平）**。可辩护的结论很窄：*基准在结构上无法完成该机动*。→ **别把单次胜利当机制有效性证据。** |

## 4.2 🔵 "未解析"建模为一等状态

| | |
|---|---|
| **支脉节点** | `nt_core_capability_tree/src/node.rs`（`ResolutionOutcome` 等价物）<br>`registry.rs:15-18` `MaturityFinding` ← **已是 `claimed` vs `supported` 双值结构**，正是这个模式的雏形<br>`registry.rs:459` `maturity_audit()` / `:484-485` 自愈降级 |
| **通用铁律（12 个代码图仓深挖后的收敛结论）** | > **join key 必须可空，未解析必须被存成"未解析"，不能猜。** 解析不出的配置键应落成"target 为空的节点"，而非伪造的边。**缺失的路由是覆盖率上限；捏造的路由是谎言。** |
| **动作** | ① `MaturityFinding` 加 `epistemic: 'exact' \| 'lower-bound'` + `EpistemicCauses`（对齐 0.4 的 CI 门输出）<br>② 依赖图边上加 `confidence` + `confidence_tier` + `reason`<br>③ 间接边置信度直接采用已标定值：**callable-value-flow 0.8 单例 / 0.7 有界多目标；fan-out 上限 32；溢出时发不出部分边、只发结构化告警** |
| **为什么对 Rust 内核尤其重要** | `#[cfg]` / feature flag、宏展开、trait object 分发、build script 生成码构成**不可消除的静态分析边界**。**不能宣告"这是下界"的系统，会在该被怀疑的时候被信任。** 13,316 个测试并不改变这一点。 |
| **许可警告** | `GitNexus` 是 **PolyForm Noncommercial 1.0.0 —— 非 OSI 许可，不可 ship**。**只借设计，不抄代码。** |

## 4.3 🟡 自治循环 git 化 + 固定墙钟预算

| | |
|---|---|
| **支脉节点** | `Makefile`（现有目标：`run:4` `sync-todo:16` `shanhai-*:70-124` `project-map:126` `project-locate:151`）<br>`scripts/ops/`（14 个门/脚本，`nt_lock_audit.py` `nt_mem_gate.sh` `nt_ascend_engine.py` `nt_brain_catalog_build.py`）<br>经验树侧：`experience_tree/mod.rs:432` `ExperienceEngine::run()` / `:403` `feedback()` ← **反馈判据落点**<br>CI：`ci.yml` `test-action.yml` `evolution-release.yml` |
| **缺口** | 自治循环目前是"**吸收 → 改文档**"。需要变成"**改代码 → 跑门 → 记结果 → 变好推进 / 变差 reset**"。 |
| **动作（借 karpathy/autoresearch，MIT，96.9k★，36 commits，3 个文件）** | ① 每次迭代开 `autoresearch/<日期>` 分支（tag 必须不存在）<br>② 全重定向跑测试，**不要用 tee**<br>③ 结果记入 `results.tsv`（**tab 分隔**，描述里用逗号会坏），5 列：`commit  val_bpb  memory_gb  status  description` —— `val_bpb` 换成你的判据（绿测数 / truth-surface 条数）<br>④ 变好 → 推进分支保留 commit；持平/变差 → **`git reset` 回去**<br>⑤ **固定墙钟预算**（不含编译）→ 实验直接可比，**不论 agent 改了什么** |
| **★ 两条最容易被忽略的判据** | ① **复杂度判据**：0.001 的提升换 20 行 hacky 代码 → 不要；换"**删掉** 20 行" → 要。<br>② **诚实的负面结果也提交进仓** —— autoresearch 的 ollama `qwen2.5:7b` 那一跑**全任务平局、卡在门外**，仍然提交了。**这是好信号，不是丢脸。** |

## 4.4 🟡 覆盖率账本（自审校准）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_audit/`（10 文件 / 2,810）<br>`neotrix-core/src/l6_meta/nt_core_self/self_audit.rs`（3 个 `#[ignore]`）<br>`neotrix-core/src/l0_substrate/nt_core_self_test.rs` / `nt_core_qtest.rs` / `nt_core_schema_watchdog.rs`<br>`crates/neotrix-audit`（纯函数、零 LLM）+ `nt_audit.rs` bin<br>`scripts/truth-surface-baseline.txt`（基线实测 **0 条** ✅） |
| **动作** | 覆盖度做成**确定性状态机账本**，验证器由**父进程**在账本创建后 + 每次更新后运行 —— **hunters 不能自己写自己的覆盖率**。三个判定值：`confirmed` / `needs_validation`（**结构上不能带 severity**）/ `rejected`。 |
| **★ 方法论** | 该仓公开承认：**单次运行只找到重复运行总和约一半的漏洞**。把这个召回率上限写进文档，别假装单跑等于全覆盖。 |

---

# 阶段 5：GUI 与 MCP（可并行，2-4 周）

## 5.1 🟡 元素寻址 GUI 驱动（承认当前是空的）

| | |
|---|---|
| **支脉节点（现状实测）** | `neotrix-core/src/l3_embodiment/nt_computer.rs`（477 行）← **文件/进程/sysinfo 抽象 trait，明确不是 GUI**<br>`neotrix-core/src/l3_embodiment/nt_computer_fleet.rs`（262 行）<br>`crates/neotrix-neobot/src/nt_computer.rs`（155 行，`NoopBackend` 是**唯一后端**）<br>`neotrix-core/src/l1_action/nt_io/nt_io_desktop/` ← **只有 2 文件**：`mod.rs`（9 行）+ `updater_signing.rs`（Ed25519）<br>屏幕采集：`neotrix-core/src/l2_perception/nt_world/sense/real_sensors/screen.rs`（单文件）<br>训练游戏已有渲染栈：`crates/neotrix-game/`（macroquad） |
| **⚠️ 文档在撒谎** | `ARCHITECTURE.md:97` 把 `nt_computer/` 宣传为"计算集群" —— 它其实是个 filesystem/process trait。**无鼠标/键盘/截图回路驱动。** |
| **动作** | 走 **B 路（元素寻址优先）**，理由：<br>① 唯一"抢注意力"的生态变化是 Android 的 **a11y-helper 技巧** —— 装一个无障碍服务绕开 `UiAutomation` 对其他 a11y 服务的抑制，否则每个首任务吃 ~3s 罚时<br>② **把百分比坐标写进 driver ABI**（`PercentagesSelectorRequest{x_percent,y_percent}` 与 `CoordinatesSelectorRequest{x,y}` **并列**）—— 分辨率无关性属于**驱动契约**，不是 prompt 技巧<br>③ **诚实结论：坐标漂移在 2026 无人解决。** 野地里只有四种部分机制，**没有滚动偏移补偿、除"刷新后再复用"外无过期索引检测、无 DOM diff 重锚定、无重复点击失败计数器**。不要设计成好像它已解决。 |
| **动作空间收敛（可直接抄）** | UI-TARS 的 24 型 `SupportedActionType` 与 Gemini 3.x desktop 动作表**约 85% 相同**。若走 A 路，坐标要**同时携带三套空间**：`{raw, normalized, referenceBox, referenceSystem}` —— **保留模型发出的原始 box**，才能 diff 出"模型说 box A、我们点了 center B"。 |

## 5.2 🔵 MCP per-request capability 协商

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/l1_action/nt_act/mcp_protocol/protocol.rs`（343）/ `tool_endpoint.rs`（218）/ `resource_endpoint.rs`（214）= 775 行 ✅<br>`neotrix-core/src/l1_action/nt_io/mcp_server.rs:20` `pub struct McpHttpRegistry`（**内存 Vec，无 wire transport**）<br>`:24` `#[deprecated]` `McpToolRegistry` / `:54` `create_default_registry()`<br>`neotrix-core/src/l1_action/nt_io/nt_io_mcp_bridge.rs`（259）<br>MCP 安全套件：`neotrix-core/src/l3_embodiment/nt_shield/shield_core/nt_shield_mcp_security/`（6 文件：injection / deps / secrets / threat / health / audit）✅<br>**客户端 `McpRegistry` 定义在 `neotrix-core/src/agent.rs:566` 的 `#[cfg(test)]` 块里** ⚠️<br>桥：`src-tauri/src/domain/plugins/mcp_extension.rs`（520）<br>`TODO.md:419-424`：`McpRegistry.gateway()` 有 2 个未实现 FIXME |
| **协议现状（2026）** | ✅ **已成熟、可用**：`per-request capability negotiation` 是本年最大协议变化 —— 基础协议**要求每个请求**带 `protocolVersion` + `clientCapabilities`，缺字段必须 `-32602` 拒 + HTTP 400；`MissingRequiredClientCapabilityError`(`-32021`) 带 `data.requiredCapabilities`。传输 stdio + Streamable HTTP 已定，**SSE 已弃用**。协议层已稳定 → **可以现在做**。 |
| **⛔ 仍是 stub（别投钱）** | ① **registry 只是元数据** —— registry 自己的 roadmap 白纸黑字：*"Unified runtime: Not solving how servers are executed"* · *"Quality rankings: No built-in server quality assessments"*。schema 里**根本没有 `capabilities` 字段**。**你无法从 registry 知道某 server 支持哪些工具、是否活着、是否安全。**<br>② **长任务**：Tasks 实验特性**被从 core 移出**到扩展（SEP-2663），协议**没有服务端 task store**。<br>③ **端到端身份传播没有** —— registry 证明了 *publisher* 命名空间归属，*caller* 侧无等价物。<br>④ **2026-07-28 弃用 `roots`/`sampling`/`logging`** → **2026 年客户端必须处理三个协议世代**，这是当下真实成本。 |
| **动作顺序** | ① 先把 `McpRegistry` 客户端从 `agent.rs:566` 的 test 块**提到生产模块**（这是最便宜的一步）<br>② 实现 per-request capability 协商（协议已稳定）<br>③ 补 `TODO.md:419-424` 的 2 个 FIXME<br>④ **registry 能力目录不要自建** —— 行业自己都还没做 |

## 5.3 🔵 无 API 重放（审计路径）

| | |
|---|---|
| **支脉节点** | `crates/neotrix-neobot/src/nt_audit.rs` ← **实测全仓 `replay` 零命中，确认为空缺**<br>`crates/neotrix-neobot/src/nt_store/`（append-only 审计落库）<br>CLI 已有 `audit` 子命令：`crates/neotrix-neobot/src/bin/neobot.rs`（1,466 行） |
| **动作（借 jev-drone 的 `replay.py` / `.tape.npy`）** | 重放录制回合：**无外部调用、不烧钱**。<br>→ 落在 `nt_audit.rs` 旁新增 `nt_replay.rs`，输入 = 审计账本，输出 = 决策路径重演。**这是把"审计"变成"可验证"的关键一步**，且零外部依赖。 |

## 5.4 🔵 UI：组件 + 动效 token（唯一的非 Rust 项）

| | |
|---|---|
| **支脉节点** | `src-tauri/frontend/src/canvas/`（9 文件 / 1,793：`SmartCanvas.tsx` 408 · `nodeRegistry.tsx` 379 · `panelRenderers.tsx` 334 · `evolution.ts` 314 · `bridge.ts` 140）<br>**SolidJS**（`src-tauri/frontend/package.json`：`solid-js` `@solidjs/router` `globe.gl` `three` `xterm` vitest + playwright）<br>`src-tauri/frontend/src/{features,pages,stores,entities,widgets,routes}/`<br>数据模型 `crates/neotrix-types/src/core/node_canvas.rs`（340，`CanvasNode`/`NodeType`）← **实测在 `neotrix-core` 内无消费者**<br>决策面 UI 需求来自 `crates/neotrix-decision-engine/types.rs` 的 `ChoiceAnswer.confidence` |
| **⚠️ 先算成本** | 5 个候选 UI 站里**4 个是 React**，你是 **SolidJS**。以下都是"设计与标记参考"，需 **1-3h/个**移植。只有 `transitions.dev` 是纯 CSS（近零成本）。**`rareui.com` 已死**（HTTP 402 `DEPLOYMENT_DISABLED`）—— 从清单剔除。 |
| **该建的那一叠** | **`beautifului.dev`(MIT) + `ui.shadcn.com`(MIT)** 是唯一连贯的一对（同 MIT 姿态、同 copy-paste 分发）。<br>**别混 beUI 的动效进 transitions.dev 的 token scale** —— 会正好产出 `transitions.dev refine` 命令存在的意义所在的那种 ad-hoc 硬编码时长债务。 |
| **按优先级拿的组件** | ① **审批弹窗**：`beautifului` **Approval Card** —— 全 5 站里与自演化内核匹配度最高的单件。**用 4.1 的三路置信门驱动**，`confidence` 渲染成仪表而非数字。<br>② **tick / 推理轨迹**：`beautifului` **Thinking**（Steps/Reasoning/Search/Coding 分页）+ **Task Rows**（running/failed/completed + 百分比 + 嵌套）+ **Loading State**（带**已用时读数** —— tick 循环正需要）；`transitions.dev` **Reasoning stream / Thinking states / Streaming text / Matrix dot loader**（全套里唯一为 agent tick 循环造的原生件）。<br>③ **审计日志**：`beautifului` **Filter Table**（状态 chip **实时重组数据**）+ **Selection Actions**（高亮→检视）；底座 shadcn `Table`/`ScrollArea`/`Badge`/`Sheet`/`Command`(⌘K)。<br>④ **能力/记忆/决策**：`beautifului` **Tool Chips** + **Context Cards**（检索知识块带字符数与来源文件/类型 —— **已为 experience-tree 检索显示设计**）+ **Insight Cards**（决策引擎置信度时序）+ **Recommendation Card**（建议 + 置信度 + Accept/Alternatives/Needs review，**直接对上 `ChoiceAnswer`**）。<br>⑤ **画布是真空（诚实说）**：5 个库**无一提供自由节点图画布**。最近的 `beautifului` **Flowchart** 是"点阵画布 + Trigger/If-Else"，但那是**线性步骤流不是可编辑节点图**。抄它的点阵背景处理做视觉先例，**图自己写**。 |
| **顺带一个契约** | `aldegad/sprite-gen`（Apache-2.0）的 **`manifest.json`**：绝对帧矩形 + 每状态 fps + loop flag，让消费方**采样矩形、绝不猜网格**。→ 你的图标系统 + 画布缩略图。 |

---

# 依赖图

```
0.1 脱 stub ──────────────┬──→ 阶段 3 成本归因（可观测才有基线）
  （必须最先）             └──→ 4.1 三路置信门（审批弹窗要有真数据）
0.2 收敛 ───→ 0.5-B 断言 ──→ 防回潮
0.4 maturity CI ──────────────→ 4.2 epistemic 扩展（先有门才有输出）
1.1 DNS qtype ──┐
1.2 尝试/结果 ──┼──→ 0.5-A 断言（检测覆盖 ⊇ 执行范围）
1.3 校准对 ────┘
1.4 策略地板 ────────────────→ 独立，可全程并行
2.1 supersession ──→ 2.2 真双时间（必须先 supersession 落地再改列）
2.3 Provenance ──────────────→ 依赖 2.1（要在同一次迁移里加字段）
3.1 成本归因 ────→ 成本感知路由（2026 全行业空位，但强依赖 3.1 先落地）
5.1 GUI ──────────────────────→ 独立，可全程并行
```

**强依赖只有 4 条**，其余可并行：
1. `0.1 脱 stub` → 阶段 3（没真实流量就没有成本基线）
2. `2.1 supersession` → `2.2 真双时间`（别在还没 supersede 能力时改列）
3. `3.1 成本归因` → 成本感知路由
4. `0.4 maturity CI` → `4.2 epistemic`（先有门输出才能加维度）

---

# 与上一份文档的差异（本次新增发现）

深挖 + 定点后，有 4 处判断**比首轮更乐观**，1 处**更严重**：

| # | 首轮判断 | 修正后 | 证据 |
|---|---|---|---|
| 1 | `maturity_audit` 是"已建未引用的机制" | **已完整实现，且带自愈降级** —— 缺的是 CI 门不调用它，所以自愈从不发生 | `registry.rs:459` `maturity_audit()`、`:484-485` 自动下调 `node.constellation` |
| 2 | 脱 stub 需要"重新设计响应" | **契约已定好** —— `ResponseMetadata` 已预留 `consciousness_state{phi,coherence,gwt_resonance}` + `capabilities_used` + `confidence`，与 L5 意识核**类型级吻合** | `stub.rs:292-303` |
| 3 | 决策引擎"JEV-inspired" | **三原语已忠实建模**（`QuestionType`/`NoulAnswer`/`ChoiceAnswer`/`ScoreAnswer` + `Usage`）；缺的是调用纪律 | `neotrix-decision-engine/types.rs:85,146,166,189,304` |
| 4 | 成本归因"要新建管道" | **唯一插点已存在** —— `anthropic.rs:92,237` 的 prefix-caching 断点就是那段不可见前缀 | `nt_io_provider/anthropic/anthropic.rs:92` 注释 "P0-4 prefix caching" |
| 5 | 能力 registry "要建声明式 manifest" | manifest **形状已对**（`provides`/`requires`/`rune_sockets`/`evolution_log`/弃用机制齐全），只缺**安全信封/爆炸半径** | `.neotrix/capability_registry.json` + `capability_overrides.json`（含 `deprecated`/`deprecated_reason`） |

**更严重的一条**：`ARCHITECTURE.md:97` 把 `nt_computer/` 宣传为"计算集群"，实测是 filesystem/process trait，**GUI 驱动完全不存在**。而首轮只说"弱"。→ 已升级为 5.1 独立阶段。
