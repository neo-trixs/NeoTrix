# 吸收批次：34 源（2026-10-08，neobot 导向）

> **入参**：34 条裸 URL（31 repo + 3 arxiv paper），其中 2 条（open-design、Captain_Who）
> 为第二轮追加、目标明确为「吸收完善 neobot」。无其他说明文字。
> **依据**：`NEOTRIX-STD-1.0.md` **NTS-B10**（B10.1–B10.4）· 操作面
> `skills/external-absorption/SKILL.md` §URL-only 熔炼模式。
> **⛔ 本批未取任何逐字代码。** 取设计形状，取信号字段，取机制判据。
> **原始证据矩阵**落 `notes/absorption-2026-10-08-evidence-ALL.md`（过程件，gitignored）。

---

## 0. 信号初筛 —— 许可分档

【实测】`raw.githubusercontent.com` 逐仓 × {main,master} × LICENSE 系文件取首行：

| 许可档 | 仓 | 数 |
|---|---|---|
| ✅ **Apache-2.0** | openhanako / harness-evals / mubeng / open-science / laya / omnigent / row-bot / Atomic-Chat / Edge0 / open-design / Captain_Who | **11** |
| ✅ **MIT** | freellmapi / dsh-im / shijianzhong-rea / EnterpriseAgentFramework / wails / magpie / karotte / paseo-bots / airgorah / Herald-OS / open-steps / cues / morluto-rea / mdream / LongHorizon-Harness / v | **16** |
| ⛔ **AGPL-3.0** | massCodeIO/massCode | **1** |
| ⛔ **GPL-3.0** | The-Last-Math-Competition/The-Last-Math-Competition | **1** |
| ⛔ **无 LICENSE** | Quincunx33/Ai-jailbreak | **1** |
| ✅ **CC0-1.0** | OpenCommunityEdition/OpenCE | **1** |

⇒ **28 宽松（含 CC0）/ 3 受限或无**（AGPL-3.0 ×1、GPL-3.0 ×1、无 LICENSE ×1）。
受限三条全部「只取设计，不取代码」。

⭐ 信号里最强的两个：`nexu-io/open-design`（**99,918★**，「DeepSeek Harness 设计插件」）
与 `NandhaKishorM/laya`（**31,520★**，非自回归 System-1 决策引擎）。

---

## 1. ⭐ 关键实测发现

1. **shijianzhong/rea 与 morluto/rea 是同族/同项目**：两者 README 标题均为
   「REA: Reverse Engineer Anything —— One MCP for reverse engineering」，
   发布均为 `rea-agents`。shijianzhong 只有 48★/主语言 None，morluto 17,126★ ⇒
   **morluto/rea 为 canonical，shijianzhong/rea 为分支/镜像**，合并按 1 源处理。
2. **Ai-jailbreak 无任何 LICENSE 文件**（目录列表实测无 license 系文件）⇒
   NTS-B10.4 唯一硬停项：只取设计，禁逐字代码，记录必须写「无 LICENSE」。
3. **massCode 是 AGPL-3.0** ⇒ 与 `context_fs.rs` 自述「基于 OpenViking 模式」的
   AGPL 教训同型：**只取设计**，记录写清来源。
4. **2 个 README 头部即说「reverse engineer」的 rea（×2）** ⇒ 本批无「判定推翻」
   级别的新事实；其余均为常规强化/intake 判定。

---

## 2. 熔炼五段

| 段 | 本批实测 |
|---|---|
| **0 信号初筛** | ✅ 本篇 §0 即成品（raw 主路径，API 交叉） |
| **1 零克隆取源** | ✅ raw README 32/32 可达 + 3 arxiv abs 页，**0 BLOCKED**，0 clone。子代理 3 次 cancel ⇒ 主 agent 兜底手工取证（skill §子代理失败兜底协议） |
| **2 熔炼成束** | ✅ 同原语 `context_bundle()`；依赖清单维仍缺 |
| **3 化为已有** | ✅ 全部落既有模块（NT-IO/SHIELD/ACT/MIND/CORE/WORLD/MEMORY），**零新建平行模块** |
| **4 落账** | ✅ 本篇入库 + KB absorb-node（§4） |

---

## 3. 判定总账

| 类别 | 数 | 说明 |
|---|---|---|
| 强化 | 22 | openhanako / freellmapi / dsh-im / mubeng / magpie / MIRA(arxiv) / open-science / massCode / laya / omnigent / SquidAgent(arxiv) / row-bot / Atomic-Chat / open-steps / Edge0 / morluto-rea(同族) / mdream / LongHorizon-Harness / open-design / Captain_Who / harness-evals / paseo-bots |
| 新增→📋 intake | 2 | HarnessEval-W(arxiv) 评测台 / karotte RL 环境（两者都需要工程量或评测台依赖面） |
| intake（设计/对照） | 10 | Ai-jailbreak(无L)/EnterpriseAgentFramework/wails/Herald-OS/airgorah/cues/OpenCE/v/Last-Math(GPL)/shijianzhong-rea(同族) |
| **合计** | **34** | |

⭐ **22 : 2 : 10** —— 本批**几乎零真正新增**。
与 2026-10-07 批次的 72:14 同型：我方的 agent/memory/invoke/orchestrate
分层比多数源更深 ⇒ **本批真实产出是 §0 许可分档 + neobot 缺口清单（§5），
不是新能力。**

---

## 4. 落账（KB）

【实测】KB 真实库在 `~/.neotrix/knowledge.db`（仓内 `.neotrix/knowledge.db`
只有 kv_store）；无常驻 neotrix-experience 进程占用 WAL ⇒
`./target/debug/neotrix-experience absorb-node … --apply-capability` 安全。
实跑结果（2026-10-08）：

| 指标 | 值 |
|---|---|
| dry-run would_insert | 34 |
| 实插 | **25**（`domain='absorption-2026-10-08'`） |
| 已存在而跳过（URL 去重） | 9 |
| 自动生成 hub 节点 | 1（`hub_1791440632`） |
| FTS 行数（该 domain） | 26（25 batch + 1 hub） |
| 全库 nodes 总行数 | 854,098 |

节点 JSON 由本篇 §0+§3 生成（`notes/` 过程件，gitignored），
含 `meta.absorbed_capability` 四元组 `{branch,capability,evidence,mapped_at}`。

**全域 code map 网络刷新（同 session）**：

| 产物 | 命令 | 结果 |
|---|---|---|
| `.project-map/codemap.json` | `python3 scripts/ops/nt_mapgen.py` | layered 2483 files/789,425 loc；second-tree 仅 1 file/40 loc |
| `docs/architecture/CODE-TOPOLOGY.md` | `python3 scripts/ops/nt_topology.py` | regenerated，RC=0 |
| 断言核对 | `python3 scripts/ops/nt_map_reconcile.py` | 35 holds / 0 violated / 0 unknown |
| 能力注册确定性 | `bash scripts/ops/nt-registry-determinism.sh` | PASS（5 进程逐字节一致，318 节点/43 边） |

⛔ 边表 `edges-*.jsonl`（`nt_calledges.py` ~30min 低频生成物）本会话**未重抽取** ——
本会话零源码改动（仅 KB 节点 + 文档 + codemap 重建），边表 mtime 仍新于全部源码，
重抽取只会得到同构输出。

---

## 5. neobot 吸收缺口清单（「吸收完善 neobot」的可执行产出）

| # | neobot 缺口 | 源（许可） | 裁决 |
|---|---|---|---|
| 1 | 多渠道 IM 接入（飞书/微信/钉钉/企微/QQ/Slack/TG/Discord/WhatsApp） | dsh-im（MIT） | ✅ 强化已有通道树 |
| 2 | quota 耗尽切账号 + 本地 OpenAI 兼容网关 | magpie（MIT）、freellmapi（MIT） | ✅ 强化 `nt_store_quota` 路由 |
| 3 | 桌面发布双通道 + 问后再装 | Captain_Who（Apache-2.0）、Herald-OS（MIT） | 📋 路线图（签名/公证未决） |
| 4 | 设计工作区（协同设计 agent） | open-design（Apache-2.0，99,918★） | 📋 路线图 |
| 5 | 长程 plan→act→verify→checkpoint→recover | LongHorizon-Harness（MIT）、MIRA（论文） | 📋 intake→l6_meta loop checkpoint |
| 6 | 并行调度重探索成本度量 | SquidAgent（论文） | 📋 intake |
| 7 | eval 评测台（Score 0-1 + threshold） | harness-evals（Apache）、HarnessEval-W（论文） | 📋 intake |
| 8 | System-1 快速判定路径 | laya（Apache，31,520★） | ✅ 强化 nt_judge |
| 9 | 本地推理 35B@2.5GB（SSD 流式） | Edge0（Apache）、Atomic-Chat（Apache） | 📋 intake→LOCAL-LLAMA 档 |
| 10 | 完全离线 persona 工作区 | openhanako（Apache）、row-bot（Apache）、paseo-bots（MIT） | ✅ 强化 persona/memory |

---

## 6. 方法论沉淀

1. ⭐ **子代理派发 cancel ≠ 停工**：兜底协议直接切主 agent 手工 raw/webfetch
   取证，4 字段模式不变、 artifact 不变。
2. ⭐ **同族分支合并判定**：两条 URL 指向同一项目的分支/镜像（shijianzhong/rea ↔
   morluto/rea）时，按 canonical（高★、主语言非 None）合并计数，避免「新增数」虚高。
3. ⭐ **许可分档永远四桶**：宽松 / AGPL / GPL / 无 LICENSE（+CC0）。CC0 与宽松同档；
   无 LICENSE 与 AGPL 同档处理（只取设计）。
4. ⭐ **「吸收完善 neobot」类批次的正确产出是「缺口清单 + 裁决」，不是新模块**
   —— R-P79 判据是「有活调用点」；本批 34 源产出 22 强化，0 新建模块。
