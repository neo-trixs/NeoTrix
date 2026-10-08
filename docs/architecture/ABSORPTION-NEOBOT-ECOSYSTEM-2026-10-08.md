# 吸收批次：Neobot 生态 11 仓（2026-10-08 · URL-only 熔炼）

> 规则：`NEOTRIX-STD-1.0.md` NTS-B10（B10.1 URL-only 入参 / B10.2 熔炼化为已有 /
> B10.3 前置门降级为分流 / **B10.4 记录真伪是唯一硬停**）。
> 取源方式：**零克隆**，README/LICENSE 均经 `raw.githubusercontent.com` 实拉
> （许可核实主路径 = raw 非 API）。README 全文已读；未逐文件读源码的部分标注「思想级」。

## 0. 真伪与许可裁决（B10.4）

| 仓 | 真伪 | 许可（raw 实读） | 熔炼级别 |
|---|---|---|---|
| FeiZhuLulu/DeepSeek-Bot | ✅ 存在 | Apache-2.0 | 代码可熔炼（思想级，源码未逐文件读） |
| dsh-tauri/deepseek-harness-desktop | ✅ | MIT | 代码可熔炼 |
| omdsh-dev/DSH-better-sidebar | ✅ | MIT | 代码可熔炼 |
| thinkany-ai/douchat | ✅ | **GPL 族**（"free software…redistribute" 措辞，细目未读全文） | ⛔ 仅思想；代码不得搬入 |
| dataelement/dsh-desktop | ✅ | MIT | 代码可熔炼 |
| ANDRETRIPOL/OpenGhost | ✅ | **双许可**：代码 MIT；名称/logo/动画/视觉设计**非 MIT**且禁商用 | 代码可熔炼；⛔ 视觉设计不得搬 |
| hikariming/dshfind | ✅ | **无 LICENSE** | ⛔ 仅思想 |
| dsh-market/dsh-market | ✅ | MIT | 代码可熔炼 |
| awesome-dsh-plugin/awesome-dsh-plugin | ✅ | **无 LICENSE** | ⛔ 仅思想 |
| nightly-labs/openbot | ✅ | **PolyForm Noncommercial 1.0.0** | ⛔ 非商用：仅思想，代码不得进生产 |
| CopilotKit/openmuse | ✅ | MIT | 代码可熔炼 |

**真伪全部为真**（11/11 raw=200）——与「无名小仓误映射率 68%」的教训相反，本批无幻觉仓；
但「存在」≠「内容如 README 所述」，未逐文件验证的功能在表中标「思想级」。

## 1. 熔炼映射（特性 → neobot 现有落点）

### A. 能力市场（直击 OPEN-DEFECTS P2-11「市场无 API/UI」）

| 外部特性 | 来源 | neobot 落点 |
|---|---|---|
| **宿主感知发现**：卡片声明 `engines.dsh` 版本下限，不匹配项**可见但标注**，过滤器只藏「已确认不匹配」 | dsh-market | `nt_capability_registry.rs`：条目补 `min_host` 声明字段；市场列表按当前 neobot 版本标注兼容性，**未声明的不得猜为不兼容** |
| **优雅自禁**：宿主过旧时市场自我禁用并在 console 明说，而非对着不存在的原语渲染 | dsh-market | `capability` 子命令加版本预检：低于所需宿主版本时明说「不可用」而非静默失败 —— 与本仓「静默失败 P0」纪律同构 |
| **收藏/备注/分组** = 本地 `state.json`，纯组织性、不碰启用状态 | dsh-market | `nt_store` 增 `market_state` 表（favorites/notes/groups 三列），明确「分组不改 enable」 |
| **市场聚合免登记**：GitHub topic 定时同步 → 插件库一天内自动收录，无需 issue/PR | dshfind（无许可，思想级） | `nt_capability_market.rs` 可加 topic 同步器；登记成本降到零 |
| **验收纪律**："if a description claims 46 tools, someone counts them" —— 每条提交对照其自身源码核数 | awesome-dsh-plugin（无许可，思想级） | 这就是本仓 M3「描述逐字判据」的同款；登记为外部佐证，不需新门 |

### B. 通道与多智能体（对接 `nt_channel*`）

| 外部特性 | 来源 | neobot 落点 |
|---|---|---|
| **FIFO 队列 + 暂停/恢复/取消 + 崩溃安全持久化** | openbot（PolyForm NC，思想级） | `nt_channel_dispatch.rs` 已有 outbox 降级律；对照补「暂停/恢复/取消」三态与崩溃恢复测试（本仓可自研实现，无许可问题） |
| **按 agent 的上下文监控 + 长线程自动压缩** | openbot | 即 OPEN-DEFECTS P1-6（`maybe_compact_context` 已接但摘要不进账本）—— 接线时把「压缩预算」记入 `nt_cost`，防止悄悄加钱 |
| **群组语义：`@` 点名回答 + lead-agent 派发 + 群专属工作区目录** | douchat（GPL 族，思想级） | `nt_channel_serve.rs` 群组路径：`@` 解析 → 目标 member 路由；语义自研 |
| **多 IM 渠道**（Telegram 已有；WeChat/Feishu 为可选扩位） | douchat | `nt_channel.rs` 的 channel trait 天然多渠；本期**不新增渠道**，仅在 roadmap 登记 |
| **可编辑 Markdown 人格档**（soul/bootstrap/用户画像/记忆 四文件） | douchat | neobot 已有 `skill`/`memory` 子命令；对照补「人格档改动下一封信生效」语义即可（已接近） |

### C. 桌面发布就绪（对接 neobot-desktop，本仓外）

| 外部特性 | 来源 | 落点 |
|---|---|---|
| **profiles/plugins/workspaces 存应用目录之外**（升级不清用户数据） | dsh-desktop / harness-desktop（MIT） | neobot `init` 的数据目录约定已 local-first；对照确认 config/db/附件**不在二进制目录**即可 |
| **更新检查**：启动后 + 每 6 小时；**下载前先问**；可跳过一版 | dsh-desktop | roadmap：桌面端 updater 采用「问后再装」，不做静默自更 |
| **stable / preview 双通道**；preview 明示「可能不兼容社区插件」 | dsh-desktop | 发布 tag 策略照抄：`vX.Y.Z` stable + `vX.Y.Z-pre.N` preview |
| **签名与公证**（macOS codesign+notarize / Windows 签名） | dsh-desktop, harness-desktop | CI release.yml 待补签名步骤（本仓 CI 已在 2026-09-28 修过三处旧债） |

### D. UI/交互（对接 neobot-ui，本仓外；OpenGhost 视觉⛔不搬）

| 外部特性 | 来源 | 落点 |
|---|---|---|
| **服务化侧边栏**：宿主提供原生 tab 注册（`registerTab`/`registerFileViewer`），插件不自绘面板，另开放 `ctx.betterSidebar` 服务给所有插件 | better-sidebar（MIT） | neobot-ui 的侧边栏演进方向：tab 类型注册制 + 共享 ctx 服务；避免每插件自绘 |
| **Take control**：agent 浏览器会话与用户共享，一键接管同一会话 | openmuse（MIT） | neobot 已有 `control`（take-the-wheel）子命令 —— 语义对齐点：接管的是**同一会话**而非新开 |
| **澄清卡（generative UI）**：agent 请求决策时给结构化选项卡，用户点选后继续 | openmuse | roadmap：`convo` 消息类型增 `clarify` 卡片；TUI 侧先做（渲染简单） |
| **停止保留草稿**；工作中来信标蓝点；发出后长消息折叠 12 行 | OpenGhost（代码 MIT） | neobot-ui 细节交互清单 |
| **compaction 后保留最后几步逐字原文** | OpenGhost | 与 P1-6 压缩接线同一张票：摘要 ≠ 全忘，最后 N 步保原文 |

## 2. 本批「化为已有」已落地的部分（本会话）

吸收对照中**反向**发现并修复了 neobot 本体的 3 个真缺陷（详见 `sessions/handoff-2026-10-08-ecosystem-audit.md`）：

1. **自治梯度 L3 结构上不可达**（openbot「per-agent 健康→自治」对照 `run.rs` 时发现）：
   `_LoopReadyScore` 降权后满分 65，而 `_autonomy_tier` 阈值仍是降权前的 80/50
   ⇒ L3（自主进化档）**永远到不了**。已重标定 65/40，4+1 个陈旧测试同步修正，全绿。
2. **层门注释假阳性**（dsh-market「宿主感知」对照词汇表文档时复现）：`nt_core_event_bus.rs`
   文档注释里的 `l6_meta` 字面量被 `check-layer-deps.sh` 当跨层引用。已改写措辞避开，
   剩余 1 条 NEW 是他窗在途 COST_TRACKER 接线（真实依赖，勿动）。
3. **neobot 编译阻塞已由他窗解除**：handoff §6 的三处错误实测已修
   （`cargo check -p neotrix --lib` = 0 error；`cargo test -p neotrix-neobot` = **597/597 绿**）。

## 3. 未做（明示）

- 11 仓均为**思想级熔炼**为主 —— 除本仓反向修复的 3 项外，未搬任何外部代码行。
- douchat/GPL、openbot/PolyForm-NC、两无许可仓：**零代码搬入**，仅设计思想入表。
- C/D 组落点在 `~/Downloads/Neo/neobot`（独立 workspace），本仓窗口不动；已在本表给出可执行规格。
