# Absorption R10 Triage：P1 批次 ~55 项

> **日期**: 2026-09-22 | **上游**: ROUND9 §1 P1 池＋补遗 | **性质**: 一遍排序（未深读项均标 unverified，读时复核）
> **收敛**: SIM-41 tripwire「R10 P1 批次 triage」提前关闭。R10 执行另立 SIM（SIM-44 起）。

## 1. R10 P0 短名单（8，先读序）

| 序 | 项 | 对隙 | 一句话 |
|----|----|------|--------|
| 1 | fast-jev-compaction / jev-ultrafast | F03 上下文＋VP 日志体积 | compaction 策略决定 dispatch 日志能否常开（unverified，读时确认算法） |
| 2 | jev-codex-router | EQ-08 分发器 | 跨 harness 路由实测形态，对照 D-1~D-6（unverified） |
| 3 | jev-security-scan | E12 证据交接 | 扫描→triage→patch 是否 PoC-only（unverified） |
| 4 | agent-desktop | EQ-18 Tauri 审计 | Agent 桌面 IPC/权限对照组（unverified） |
| 5 | awesome-hermes-agent | 技能格式＋多智能体 | 生态全景，定 skill 注册表字段（unverified） |
| 6 | KaLM-Jev | 技能＋Embedding | 检索式 skill 路由，对照路由记忆 D-6（unverified） |
| 7 | Agent-Reach | 编排 | 到达/委派语义，对照 SDB C 列（unverified） |
| 8 | jev-review | E12＋G04 | review 流水线 verdict 格式，对照 ADR-0004（unverified） |

## 2. 全单分类（A–H）

### A. 技能生态 → NTS-F/模板/G10
jev-skill · jev-cu · jev-mc · jev-mcp · jev-hu · hermes-jev-skills · super-hermes
（skill 包格式互操作；读 1–2 个代表＋awesome-hermes 全景即可，其余折叠）

### B. 路由/编排 → EQ-08/F01
ARES · astra-flash-orchestrator · Raven · multica · first-tree · recurse · winnow
（orchestrator 语义对照 dispatcher；recurse/multica 疑似框架，读时确认）

### C. 上下文/记忆 → F03
classifier-dev · maka-cu · brainapi2 · osiris
（分类/记忆 candidates；低优先级，F03 现状够用则 parked）

### D. 安全 → SEAL/E12
awesome-ai-security-tools · Exegol · ai-ctf
（工具集＋靶场：只取 taxonomy/靶题设计，不引工具链）

### E. 桌面 → EQ-18
PI-Desktop · deskport · OpenJev-Vision · beautifului · cumora
（agent-desktop 为主对照，其余折叠；beautifului 系 UI 状态设计， lukewarm）

### F. 具身/多模态 → L2/L3
Open-LLM-VTuber · openhuman · termgram · map3d · Crucix
（数字人/终端具身 levy；与 L3 门面设计联动，R10 不深读，留 P-task）

### G. 数据/基建 → 吸收/eval
arxiv-complete（论文语料候选， лицензия 待查） · Scrapling · patchright-enhanced
（抓取链：只记 pattern，不新增依赖） · meetily（待判，读时确认）

### H. 待判（名止）
tirith · Cairn · orca · openhermit · takt · dopbase · pond · mira · user-scanner ·
killmyidea · typesafe-mario · typesafe-mcp · mr-boxington · jev-trader
（名信号弱；R10 只在 A–G 读完有余量时抽 2–3 验名，否则整体 parked，
不算跳过——triage 即允许折叠）

## 3. H类抽验结论（M6，SIM-52）

| 项 | 蓝图分类 | 实际 | 名实 | 判定 |
|---|---|---|---|---|
| H-01 | L2+L5→L0 | dispatcher 在 L1（L5 使用，非 L2） | 层号过时 | **归档**——Phase 2 已将 dispatcher 移至 L1，原分类失效 |
| H-02 | L2（nt_core_bank 6处 TaskType） | TaskType 定义在 L0，nt_core_bank 在 L1，引用链 L1→L0 合规 | 名实不符 | **归档**——引用链合规，非越层 |
| H-05 | L2（playback source 类型→L0） | playback 在 L1，L2 facade 重导出 | 名实相符 | **保留**——L1→L2 重导出模式仍存在 |

> H-01/H-02 归档后，蓝图 §15 D-02 索引行标注"已归档"，不再参与下沉计数。
> H-03/04/06/07（Facade 类）仍待 Phase 3 SIM；H-08 仍待 P1-02 首批。

## 4. R10 执行约束（先立后读）

- 另立 SIM（SIM-44 起），沿用九格＋GO/GO-WITH-MITIGATION/NO-GO。
- 每源上限：README＋关键 1 文件；超限转 P-task，不在本轮深读。
- H 类默认不读；读 A–G 时顺带验名，证伪即折叠。
- 新条款预算 ≤2（R9 立 4，收敛期从紧；其余一律 bake note＋tripwire）。

---

*End of R10 Triage*
