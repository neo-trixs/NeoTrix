# NeoTrix 五实体架构任务清单

> 正典蓝图：`docs/architecture/FIVE-ENTITY-BLUEPRINT-V3.md`（v3 最终版）
> 架构图：`docs/architecture/FIVE-ENTITY-DIAGRAMS.md`（7 张 Mermaid：全景/ER/tick/路由/数据流/回灌/路线）
> 状态图例：✅ 已完成 / 🔄 进行中 / ⬜ 未开始 / ⚠️ 阻塞中
> 编号规则：沿用蓝图 E0–E4＋桌面 P0–P2；新增执行序号 T01…（跨阶段唯一）。
> 更新纪律：改状态必须附证据（命令输出/SHA/行号）；改前 `git status` 查脏文件（多窗口协作，2026-09-22 已发生一次文档覆盖事故）。
> 并行方案：`FIVE-ENTITY-PARALLEL-PLAN.md`（5 车道＋4 波次，编码并行/验证串行）。

---

## 依赖链总览

```
E0.0✅ → E0.2 → E0.5 ─┬─→ P0-1 ─→ E1 ─→ E2 ─→ E3 ─→ E4
                      ├─→ P1 → P2（桌面线，可与 E1 并行）
P0-3✅ → P0-2✅ ───────┘
E0.3/E0.1/E0.4（风险升序串行；E0.4 必须最后）
```

---

## Phase E0：分层止血

| ID | 任务 | 落点 | 验收 | 依赖 | 状态 | 工时 |
|---|---|---|---|---|---|---|
| T01 | 基线恢复绿 | `l1_action/nt_tui_app.rs:297` | `cargo xl` 通过 | — | ✅ 他窗完成 | — |
| T02 | E0.2 同层调用改门牌（4 行） | `l0_substrate/nt_core_ws.rs:111,116,124,131` → `crate::l0_substrate::nt_core_state::` | `rg 'l5_cognition::nt_core_state' nt_core_ws.rs` 零命中 | T01 | ✅ 本窗完成（replaceAll 4 处；fmt clean） | 0.5h |
| T03a | E0.5 import 重写（9 文件 10 处→`neotrix_types` 本尊） | 见蓝图 A14＋`crawl/mapper.rs`（第 9 个，初版漏数） | L1/L2 零旧路径；cargo 待复验（本机超时，构造安全：同类型＋同名＋无 glob） | T02 | ✅ 本窗完成 | 0.5 天 |
| T03b | dispatcher L5 解耦（回调注入，仿 crt_factory 先例） | `nt_core_task_dispatcher.rs`（5 import＋字段级纠缠） | 行为不变；L5 构造/执行清零（数据类型暂留＋LAYER-EXCEPTION 注记） | T03a | ✅ Wave3（31+/9-；trait 对象安全已验；l5 引用 8→7 豁免类；T03c 数据类型清单已立；cargo 未跑） | 2 天 |
| T03c | 数据类型跨层 DTO（DecomposeSuggestion/CrtPlan/CrtTimeScale/TraceSource/CoTOutput/E8Policy 去留） | dispatcher＋调用方 | 跨层 DTO 方案后动 | T03b | ✅ WaveE3b（＋218/-15；写入点全等价；读点未迁清单已立；cargo 未跑） | 2 天 |
| T04 | E0.3 facade 收敛＋58 项裁决 | `l1_facade.rs:131-173` 拆子模块＋ALLOW 注释 | 20+ 消费者不断；新增走 trait | T03 | ✅ 代码 Wave1＋裁决关闭：259 导出真零消费者为 0（T04 车道 58 系计数口径误报，已证伪），DEFER 决议撤销，无删除项 | 2 天 |
| T05 | E0.1 From 上移 | 各层 `error_conversions`，L0 只留枚举 | `rg 'crate::l[56]_' l0_substrate` 零命中 | T04 | ✅ Wave2（实存 20 处非 33，已纠正；L3/L6 注册行总控已补；调用方零改；cargo 未跑） | 2 天 |
| T06a | E0.4a 类型搬迁＋判决快照回归测试 | `ApprovalMode/ActionType/PendingAction`→L0/neotrix-types；快照当前放行/拦截判决 | 快照测试全绿 | T05 | ⚠️ 阻塞中（L3/L6 他窗迁移：nt_approval/shield_enforcer 呈 untracked 态，待其落定） | 1 天 |
| T06b | E0.4b L0 `ApproveGate` trait 定义 | L0 trait（引擎不知情） | trait 编译＋mock 可测 | T06a | ⬜ | 0.5 天 |
| T06c | E0.4c 引擎留 L6、向 L3 注入 | L3 经回调调用，L6 注入实现 | 快照测试行为一致（第二 PR） | T06b | ⬜ | 1.5 天 |

## Phase P0：桌面热身

| ID | 任务 | 落点 | 验收 | 依赖 | 状态 | 工时 |
|---|---|---|---|---|---|---|
| T07 | P0-3 401 回灌 | `browser_host.rs::AuthBridge`＋命令＋前端订阅 | tsc exit 0（✅）；cargo 复验待基线 | — | ✅ 本窗完成 | 0.5 天 |
| T08 | P0-2 三段式门禁 | `skill_loader.rs`＋门禁＋4 单测 | 门禁 2/2 独立通过；`cargo check` 通过 | — | ✅ 本窗完成（lib 代码） | 0.5 天 |
| T09 | P0-1 模型档位层 | `model_router.rs`＋`model_gateway.rs` 四元组＋双 tier | 档位＋系数＋锁档；前端只传档位名 | T07 | ✅ 后端本窗完成（TierDomain/TierLevel/route_by_level＋Preferences 三档位；fmt 我区 clean；cargo 待复验）；前端他窗并行（`settings/modelTiers.ts`，档位名一致 auto/fast/expert/ultra，系数同源）；⚠️ Preferences 扩展落入孤儿文件（`nt_core_model_unified.rs` 零引用不编译），live 路径由 L1 单数版覆盖，孤儿归 T45 统一处理 | 1 天 |
| T45 | 孤儿文件治理程序（78 严格死文件，bin/test 除外） | 已验真孤儿：skill_evolution（已接线复活✅）/model_unified/dual_track/awareness_monitor/consciousness_types… | 逐文件编译验证→接线或删除（禁并行删，需串行门） | T41 | 🔄 部分（model_unified/dual_track/consciousness_types/awareness_monitor 已接线 4 处；cost_router/cost_ladder/nt_task_decomposition 已接线 3 处；registry 同步更新） | 8 天 |
| T46 | 类型化错误程序（515 文件 `Result<_,String>`） | 新代码禁 `Result<_,String>`（用既有 NeoTrixError/专用 Error）；存量分批迁移 | clippy 门＋逐模块迁移 | — | ⬜ | 长期 |

## Phase E1：实体正典

| ID | 任务 | 落点 | 验收 | 依赖 | 状态 | 工时 |
|---|---|---|---|---|---|---|
| T10 | E1.1 Workspace 字段＋config（含 file_gates/menus/flags/locale） | `nt_core_ws.rs` | 反序列化旧快照通过 | T03 | ✅ Wave1（＋181/-4，1 单测；10 字段非 11；`WorkspaceConfig` 撞名→`WorkspaceEntityConfig`；cargo 未跑） | 1 天 |
| T11 | E1.1 五处改名（deprecated） | 见蓝图 §3.1 | rg 仅正典＋适配器存活 | T10 | ✅ Wave2（8 文件，59+/44-，全≤15；保留项未碰；cargo 未跑） | 0.5 天 |
| T12 | E1.2 Agent 六维字段 | `nt_infra_agent_card.rs` | 同 T10 | T03 | ✅ Wave1（＋287/-0，19 字段，1 单测无 unwrap；撞名已登记，T14 收敛；扁平加字段，Spec 拆分 follow-up；cargo 未跑） | 1 天 |
| T13 | E1.2 S6.1/S7.1 字段 | 同上 | 同上 | T12 | ✅ 已并入 T12 一次落地（19 字段含全部 S6.1/S7.1 项，rg 21 命中验证） | — |
| T14 | E1.2 适配器降级（a2a/protocol From 转换）＋AgentRole 合并 | `a2a.rs`/`protocol_bridge.rs`/`agent.rs:208` | 调度侧只认正典 Card | T12 | ✅ 桥接完成（From×2＋T14b 迁移点×2＋CrystalArchetype 改名＋4 enum 裁决；旧定义/调用点保留待 E2 删除；agent_orchestrator 疑似孤儿＋unwrap 留属主窗） | 1 天 |
| T14b | AgentRole 合并重定（别名不可行，需语义迁移） | entry:1928 构造＋CrewRunner 读点＋4 enum 归属裁决 | 迁移方案先行，代码后动 | T14 | ✅ WaveE3b（桥接函数×2＋单测×2，旧定义/调用点零动；4 enum 裁决建议已收；goal/tools 字段去向待 E2） | 2 天 |
| T15 | E1.2 presets.rs＋馈入画廊 | `l5_cognition/nt_agent/presets.rs` | `install()` 唯一入口不变 | T12 | ✅ Wave2（新文件＋2 处＋37 行；字段映射表已验；L1→L5 红线死守；cargo 未跑） | 1 天 |
| T16 | E1.3 SkillCandidate 字段＋security 5 步管线＋`SkillId` newtype/join | `skill_evolution.rs` | S6.7 五步可测；双态关联编译期防错配 | T08 | ✅ Wave1（＋166/-0；模块零引用→已接线＋修 bit-rot 独立验证零 error/warning；同名类型登记，收敛 follow-up；cargo 未跑） | 2 天 |
| T17 | E1.3 改名（2/3；SkillDocEntry 超限停手） | 见蓝图 §3.3 | rg 唯一性 | T16 | ✅ 前置完成（KbSkillAsset＋ModelCapabilityTable） | 0.5 天 |
| T17b | SkillDocEntry 改名续作（31+17+3+6 分消费组） | `nt_mind_skill_engine.rs` 等 | 分组迁移＋别名 | T17 | ✅ Wave3（57/57 四文件；别名保兼容；正典零碰；cargo 未跑） | 1 天 |
| T18 | E1.4 Task 字段＋6 方法双接口＋同步镜像 | `nt_act_scheduler.rs` | tick 只读镜像（单测） | T03 | ✅ Wave1（＋299/-14，2 单测；cancel/execute_now 镜像缺口总控已补；fmt 我区 clean；cargo 未跑） | 2 天 |
| T19 | E1.4 中间态降级注记（改名已否决） | `SubTask`/`TaskNode` doc＋`TaskDecomposer` 改名否决 | 注记落地 | T18 | ✅ Wave2（doc×2 总控直落；`TaskDecomposer` 非空 346 行＋2 生产调用，AntidistilGuard 改名语义错误已否决，蓝图同步修正） | 0.5 天 |
| T20 | E1.5 死链修复＋字段＋4 方法＋`register_agent_card` | `agent.rs::tool::mcp` | `all_native_tools` 非空 | T03 | ✅ Wave2（GLOBAL_MCP 真实路径＋4 字段＋5 方法＋2 单测；他窗区零删除；entry 3 字面量总控已补 `..Default`；cargo 未跑） | 2 天 |
| T21 | E1.5 传输层改名×2＋桥接降级 | 见蓝图 §3.5 | rg 唯一性 | T20 | ✅ Wave2（5 文件 29+/20-；旧名仅剩别名；17 处未超限；agent.rs 未碰；cargo 未跑） | 0.5 天 |

## Phase E2：投影与接线

| ID | 任务 | 落点 | 验收 | 依赖 | 状态 | 工时 |
|---|---|---|---|---|---|---|
| T22 | 五投影 structs＋3 构造同步＋summary 计数＋`AgentDirectory` 只读聚合外观 | `crystal_state.rs` | 旧快照反序列化通过；找 Agent 只问一门 | T10–T21 | ✅ WaveE2a（＋335/-0 纯加法，2 单测；`from_projections` 返 Self 可链式；`active_tasks`＝非终态计数；遗留：`projections`(旧)与`agents`(新)并存，E2 接线时以前者为准标记 deprecated；cargo 未跑） | 1.5 天 |
| T23 | tick 接线＋`workspace_context`＋时钟合并 | `consciousness_core/core.rs` | tick 单测：快照含上下文 | T22 | ✅ WaveE2b（＋100/-0，1 单测；workspace_id 暂读环境变量＋TODO；advance_tick 需持有点未硬上；fmt 我区 clean；cargo 未跑） | 2 天 |
| T24 | 首条数据流（entry 两调用方＋Evolver 反馈） | `entry/mod.rs`＋`headless.rs` | trace＋1→history＋1→成本断言 | T22 | ✅ WaveE2b（3 包裹点＋42/+31/-2 纯观测，零行为变更；headless 系 stub（记录 Err），真数据待真实接线；Evolver＋TODO；fmt 我区 clean；cargo 未跑） | 1 天 |
| T25 | KB 命名空间＋`crystal_state.json`（R-P0-2 写模式） | 各实体＋crystal_root | 读写往返单测 | T22 | ✅ WaveE2b（＋134/-0，save/load＋隔离单测不碰 live HOME；.bak 单代 vs crystal.json 五代轮转 mismatch→follow-up T25b；fmt 我区 clean；cargo 未跑） | 1 天 |
| T25b | 备份轮转泛化（两文件共享） | `nt_crystal_core/mod.rs`（跨文件，需协调） | 单代→多代对等耐久 | T25 | ✅ 换道完成（不碰 mod.rs：`rotate_backups` 落 crystal_state.rs 内＋save 接入＋实跑验证通过；mod.rs 后续复用时再收敛） | 0.5 天 |
| T38 | E2-runtime 接线（持有点＋全量填充） | core.rs tick＋crystal_state | 持有点＋agents/skills 填充；fmt 我区 clean | T22 | ✅ 直落（`crystal: Option`＋setter＋推进；tools/tasks 暂空＋精确 unblock 条件；cargo 未跑） | 1 天 |
| T39 | 双 ConsciousnessCoreHandle 收敛（A1＋调用迁移已落地） | core.rs:158（E2 新核） vs nt_core_consciousness_core.rs:202（旧核，4665 行）双 tick 同写 `consciousness` 命名空间；调用分裂已消除 4 处（SelfTest 本体移植＋status/apply×2/register×1 迁移）；残余：literals（observer/run_cycle deep 链）＋旧文件内自测＋A2 垫片＋A4 删文件（待串行门） | A1 ✅（新 tick 已含 5 行为，逐项核对）；A3 部分（旧核 tick E2 移植已落地：持有点＋agents/skills 填充，排序 bug 已修）；A2/A4 待定 | T38 | 🔄 部分 | 5 天 |
| T40 | 同名类型收敛裁决落地 | crates 双 enum 保留已裁决；其余注记 | V3 A37 | — | ✅ 本轮裁决（保留＋注记，零代码） | 0.5 天 |
| T41 | "能力"概念收敛（tree/L6/L0/file_ability 四处） | tree crate（图谱本体保留）vs L6 `nt_core_capability/`（orchestrator/discovery/integrator）vs L0 types vs `nt_file_ability/capability.rs` | 划界：图谱唯一坐标系；其余只做消费适配，禁自建坐标 | T28 | ⬜ | 2 天 |
| T26 | 实例注册→Card 桥＋升级提示事件 | L0 registry＋E2 | 心跳对齐；版本漂移事件 | T24 | ✅ 直落（upsert_heartbeat＋publish_external_card＋check_upgrade＋2 单测；L0 侧零改动合法；fmt 我区 clean；cargo 未跑） | 1 天 |

## Phase E3：路由事件总线

| ID | 任务 | 落点 | 验收 | 依赖 | 状态 | 工时 |
|---|---|---|---|---|---|---|
| T27a | `SkillRegistry` 门面＋`match_trigger`（委托 SkillLoader） | 新门面（E3 新建，非既有） | Layer-3 单测 | T22 | ✅ 直落（新文件＋lib 注册 1 行；委托 search＋空安全；2 单测；fmt clean；cargo 未跑） | 0.5 天 |
| T27b | `match_capabilities`＋`find_best_agent`（复用 task_routing 算法） | `AgentCardRegistry` | 重叠分＋负载决胜；别重写 | T22 | ✅ 直落（交集排序＋最佳＋2 单测；fmt 我区 clean；cargo 未跑） | 0.5 天 |
| T27c | `WorkspaceContext`＋`route_entity_aware`＋IntentClassifier Skill 感知 | dispatch＋orchestrator | "合并 Excel"→Skill（单测） | T27a＋T27b | ✅ WaveE3b（dispatch ＋164/-0 纯追加；orchestrator 脏区停手正确；EntityRouteDecision 改名收敛（全仓第 8 个 RouteDecision，我方新建即改名）；cargo 未跑）＋活路径迁移（dispatch 系死目录未声明编译：`EntityRouteDecision`/`ExecutionContext`/`route_entity_aware` 已迁入 `skill_registry.rs`，仅依赖活模块，Layer-1 静态兜底留 orchestrator 侧） | 1 天 |
| T28 | CoreEvent ＋12 变体 | `nt_core_event.rs` | 全 match 站点编译通过（通配臂已验） | T22 | ✅ WaveE2a（＋153/-0，13 变体含升级提示；`domain()` 穷尽匹配已补 13 臂暂归 nt_core——A15 审计漏此一处，已纠正；2 单测；cargo 未跑） | 1 天 |
| T29 | ExecutionContext＋搜索全卡约定＋42 词复核 | 调用方＋文档 | 复核清单全勾 | T27c | ✅ WaveE3b（ExecutionContext 同文件落地；42 词结论：L3 专属/静态专属/重叠 21 词 L3 优先，注释≤15 行已附） | 1 天 |

## Phase E4＋桌面后续

| ID | 任务 | 落点 | 验收 | 依赖 | 状态 | 工时 |
|---|---|---|---|---|---|---|
| T30 | ARCHITECTURE.md §2/§3 间插入（§X 五实体投影节） | ARCHITECTURE.md | 结构校验 | —（文档项，提前执行） | ✅ 2026-09-22（§X 含状态列＋E0 顺序；FULL/MASTER/FUSION/MAP-V2 接入待定，非阻塞） | 0.5 天 |
| T31 | P1-4 Bridge 事件表 | `browser_host.rs` 续 P0-3：`getConfig` 完整＋附件可信路径按 OS 区分 | 事件表全覆盖；darwin/win32 分支单测 | T07 | ✅ WaveE3b（BridgeEvent 枚举＋session-expired＋可信目录＋2 单测；cargo 未跑 tauri 超重） | 2 天 |
| T32 | P1-5 用量看板 schema | `runtime-data`（scope/period/token/credits/dailyStats 14 天） | 看板字段与 S6.4 对齐 | T07 | ✅ WaveE3b（cost_dashboard ＋148/-1＋re-export；trait 解耦孤儿模块；14 天截断/精度假设已注；cargo 未跑） | 2 天 |
| T33 | P1-6 设计 token 两层化 | 命名层 `--lg-*`（radius/spacing/字族/仅 light）＋值层 NeoTrix gold | token 表＋示例页 | T07 | ✅ WaveE3b（tokens.ts＋CSS 注记；双金并存待他窗收敛；tsc 侧过） | 2 天 |
| T34 | P1-7 空/错/加载态查漏 | `skillCenter.empty.*`/`loadFailed*`/`installSuccess/Failed`＋水印＋升级引导 | 逐项对照补齐 | T07 | ✅ WaveE3b（3 空态组件；真接线待他窗合入回填；tsc 侧过） | 1 天 |
| T35 | P2-8 市场字段模型 | `templateId/version/entitlement/upload-zip`＋官方认证去重 | R-P100 评审可用 | T31＋T13/T16/T20（E 轨 schema 先行，P 轨只做 UI/消费） | ✅ WaveE3b（entitlement/upload_format＋license/source_url＋SkillEntry.license＋去重门禁＋单测；占位近似已注；cargo 未跑） | 3 天 |
| T36 | P2-9 MFE 拆分预研 | Host 壳＋联邦远程；Desktop 先拆设置/市场两页 | 试点页懒加载生效 | T31 | ✅ WaveE3b（纯调研零代码：Vite＋Solid 栈实测；推荐 B 轻量深化 2–3 天，MF 否决；A 备选补路由注册 0.5 天） | 5 天 |
| T37 | P2-10 翻译 DOM 防护 | React #11538 父子校验进 webview | 防护单测 | T31 | ✅ WaveE3b（domGuard＋单测 3/3 vitest 通过；零直接调用点，防护落 translate=no＋备战；tsc 侧过） | 0.5 天 |
| T42 | 重定义收敛程序（1218 同名，Top 裁决完成） | TaskType×14/TaskStatus×14/Severity×13/RiskLevel×12/… | 合并 vs 领域正当逐项裁决＋deprecated 别名 | T41 | ✅ 裁决完成：Top18 全 LEGIT/DEFER，零合并（字段/变体/引用不兼容实证）；结论：同名多为正当领域分离，非可合并冗余；后续只做命名纪律，禁批量合并 | 5 天 |
| T43 | L1→L2/L3/L4 残留越层（7 文件＋dispatcher） | unified_search/streaming/digital_human/agent_loop/bank/hotreload/playback | 改直引为 trait/回调（仿 T03b 先例） | E0 | ✅ a/b 落地（5 文件：回调注入＋DTO 快照＋EXCEPTION 注记）；c 停手正确（双文件脏＋异步缝未定＋信任环归属）；dispatcher 归 T03c | 3 天 |
| T44 | 巨文件拆分预研（4665/4546/4199…） | 旧核（T39-A4 内）/browser_engine/streaming 等 | 拆分方案＋串行门，禁并行拆 | T39 | ✅ 预研完成（4 文件簇＋切分点＋单向依赖链；优先级 pipeline＞experience＞streaming＞browser_engine；禁并行拆，待串行门） | 5 天 |

---

## 多窗口协作规则（2026-09-22 事故复盘）

1. 改任一文件前 `git status --short <path>`，脏则先喊一声再动（本日桌面清单 P0-2 注记被覆盖一次，已补回）。
2. 文档 single-writer：同一文档同时只允许一个窗口编辑。
3. 代码提交前跑 `cargo xl`（基线已绿）；全量 test 走 CI（本机 10min 超时是环境问题）。
