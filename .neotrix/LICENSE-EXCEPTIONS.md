# 许可例外记录（人工签署，门不自动放行）

> 本文件是 `scripts/check-license.sh` 的**唯一**放行通道。
> 一条 deny 命中若无对应签署 ⇒ 门 FAIL。签署不是"消除问题"，
> 是**把决定权与责任显式落到人**（工具无法代替法务判断）。
>
> ⛔ 签署前必须读该 vendored 树自己的 `VENDOR.md` 许可节，那里写清了条款原文。

## 格式

每条一个 `## ACKNOWLEDGE-<n>` 段，字段必须齐全，缺一即视为未签署。
⛔ 下面代码块里是**格式说明**，门会跳过围栏内的内容 ——
**绝不要把真实路径填进模板当签署**，模板本身长得像签署是本文件最大的坑。

```
## ACKNOWLEDGE-<n>
tree:        <vendored 树路径，例如 apps/foo/bar>
clause:      <命中的条款关键字>
decision:    <accepted-with-condition | removed | upstream-permission-obtained>
condition:   <可验证的约束；decision=accepted-with-condition 时必填>
owner:       <签署人姓名>
date:        <YYYY-MM-DD>
review_by:   <YYYY-MM-DD>
evidence:    <授权书/法务意见/商业属性判定 的链接或路径>
```

`decision` 三选一：
- `accepted-with-condition` —— 接受风险，但必须写明 `condition` 与 `review_by`
- `removed` —— 该树已从仓库移除（须与实际一致，否则门仍红）
- `upstream-permission-obtained` —— 已取得书面授权（`evidence` 必填）

---

## 当前状态：**无有效签署** —— ACKNOWLEDGE-2 已于 2026-10-01 失效

### 认定变更时间线（不可删除）

| # | 日期 | 认定 | 后果 |
|---|---|---|---|
| 1 | 2026-10-01 | **商用** | ACKNOWLEDGE-1 的 `condition #3` 触发 → 失效 |
| 2 | 2026-10-01 | **开源非商用** | 触发条件消失 → 签 ACKNOWLEDGE-2 |
| 3 | 2026-10-01 | **商用**（再次） | ACKNOWLEDGE-2 的 `condition #1` 触发 → **再次失效** |

⚠️ **同日三次认定、两次反转。** 这不是「记录没维护好」，而是：
**当前状态缺少一个能自动判定「是否商用」的机器信号**，
所以每次认定变化都只能靠人重新说一遍，而人会变。
⇒ 这正是 ACKNOWLEDGE-2 `condition #1` 存在的原因：它让失效**可被复现**，
不需要临场判断。三次认定全部留痕，便于事后复盘。

### 商用认定下的处置（与前次不同：这次有重构方案）

`ACKNOWLEDG-1`/`-2` 均不适用于商用。商用要合规只有两条路：
**(A) 取得上游书面授权**（保留 vendored 树）
**(B) 自研重构，不带 vendored 代码**（`docs/architecture/FRONTEND-REBUILD-2026-10-01.md`）

**事实基线（实测，支持 B 可行）**：我方自有前端仅 **3 个 ts + 1 个 css / 684 行**，
依赖只有 `react` + `@tauri-apps/api/core` + 自有 `shim.ts`，
调用仅 **2 个** Tauri 命令（`log_frontend` / `neobot_api_call`），
对上游 `store`/`hooks` **零依赖**。
⇒ B 不是「重写 86K 行」，而是**逐能力裁决**：绝大多数 vendored 能力
（ssh / worktree / scheduler / model 切换 / 插件市场）是 **DSH 的功能，不是 NeoBot 的功能**。

---

<details>
<summary>历史条目（ACKNOWLEDG-1 / ACKNOWLEDGE-2 原文，保留以便追溯）</summary>

## 当前状态（历史）：曾签署 ACKNOWLEDGE-2（现已失效，见上）

### 事实变更记录（不可删除）

| 日期 | 事实 | 影响 |
|---|---|---|
| 2026-10-01 | 所有者确认**商用** | ACKNOWLEDGE-1 的 `condition #3` 触发 → **自动失效** |
| 2026-10-01 | 所有者改为**开源非商用**（同一日推翻上一条） | 失效原因消失 → 以 **ACKNOWLEDG-2** 重新签署 |

⚠️ 同日两次相反认定，说明**认定会变**。因此 ACKNOWLEDGE-2 的首要条件就是
**让失效可被机器复现**（见 condition #1）。

### 机器核验：认定与仓库内容一致【实测】

对全仓（排除 target/.worktrees/frontend）搜 8 个商用信号
`stripe / paddle / lemonsqueezy / subscription plan / enterprise license /
paid tier / pricing page / per-seat` ⇒ **8 条命中，逐条读原文全部是假阳性**：

- 7 条是 `PaddleOCR` / `PaddleOcrEngine` 命中 `paddle` 子串
- 1 条是 `nt_mcp_scan_secrets.rs:70` 的**密钥扫描黑名单** `"stripe-api-key"`
  （即本仓在*扫描*这类 key，不是用它收款）

⇒ 仓库内**零**定价/支付信号，「开源非商用」不是口头声明，与内容一致。

---

## ACKNOWLEDGE-2

tree:        apps/neobot-desktop/frontend
clause:      No Commercial Secondary Development (frontend/LICENSE.details)
status:      void            ← 2026-10-01 第三次认定（商用）触发 condition #1，作废
decision:    accepted-with-condition
owner:       NeoTrix 项目所有者 —— 决定由所有者作出，agent 于 2026-10-01 代为记录
             （不填具体人名：agent 不得冒充个人签署）
date:        2026-10-01
review_by:   2026-12-01
evidence:    VENDOR.md「2026-10-01 评估结论」节 + 本文件「机器核验」节
does_not_resolve: 见下方「残留风险」—— 签署**不**等于上游已授权

condition:

1. **一旦出现任何商用迹象，本条立即自动失效**（无需人工判断）。
   判据：新增支付/定价集成、改为非 MIT 许可、发布付费层、企业版/订阅版、
   或以「支持/咨询/托管」形式对该功能收费。
   ⚠️ 鉴于同日已发生一次认定反转，本条不得依赖「记得回来改」。

2. 上游 `LICENSE` 与 `LICENSE.details` **逐字保留**，不得删除或改写
   —— 由 `scripts/check-license.sh` 机器强制

3. MIT 署名与许可声明必须传播到**分发产物**中（MIT §「保留声明」义务）；
   尤其 vendored 树随二进制/安装包分发时，其 `LICENSE` + `LICENSE.details`
   必须随之到达用户

4. 2026-12-01 前必须重审

### 残留风险（签署**不**解决，需上游确认或法务判断）

条款原文禁止的是「以获取**商业利益**为目的的二次开发」，并写明
「**直接使用**本软件本身用于商业目的仍被允许」。

- ✅ **本项目自身非商用** ⇒ 不落入该禁止范围。回到基础 MIT，
  MIT 允许修改与再分发。**这是本条签署的依据。**
- ⚠️ **但根 `LICENSE` 是 MIT** ⇒ 一旦 NeoBot 以 MIT 发布，**下游获得整棵树
  （含 vendored 部分）的权利**，而**商业下游用户**所做的正是条款禁止的
  「商业二次开发」。
  ⇒ 该限制是随文件走的，**我们的非商用认定管不住下游**。
  这是本条**未解决**的核心问题，**只有两条出路**：
  (a) 上游书面确认「非商用二次开发 + MIT 再分发」在其允许范围内；
  (b) 把 vendored 部分从 MIT 可再分发范围中排除（技术上是可行的隔离，
      但会改变 NeoBot 的分发形态，属架构级决定）。
- ⚠️ 「非商用」在实践中难界定：免费但用于盈利场景、免费增值引流等。
  本条以**所有者声明**为准，不做技术判定。

⛔ 禁止的处置：为了让门变绿而删 deny 名单、改门脚本、或删除失效/变更记录。
门转绿的唯一正当原因是「取得了上游书面授权」并据此新签。

</details>

---

## 已确认不吸收的外部 URL（2026-10-07 起）
- **FRONTEND-LICENSE-COMMIT.md** 已由我在本轮的判断期间产出，但因缺少人力签署，本轮仅留档，后续由所有者填写。


> 这些 URL 在全域吸收/熔炼轮（TradingAgents / memvid / huashu / tester-army/e2e / Ix / leviathan / CarterPerez）中被明确裁决 NOT absorbed。
> 此节**永久留档**，避免下个窗口重复评估「强制吸收」。

### CarterPerez-dev/Cybersecurity-Projects

- URL:         <https://github.com/CarterPerez-dev/Cybersecurity-Projects>
- License:     **AGPL-3.0**（强 copyleft）
- 作者声明:    "copy directly"（鼓励直接复制）
- **裁决**:    ⛔⛔ **永不吸收** —— AGPL 传染性条款会强制整个闭源 NeoTrix 开源
- 评估日期:    2026-10-07
- 评估证据:    `LICENSE` 含 `GNU AFFERO GENERAL PUBLIC LICENSE version 3` 字样；
  README 文本含 `copy directly`

### huashu-art-motion

- URL:         <https://github.com/Unknown/huashu-art-motion> (待用户提供时取证)
- License:     MIT (部分字体/资产另计)
- **裁决**:    ⛔ 不吸收 —— 与本仓宪法/架构无直接能力对应（design/widget 库，与 NeoTrix 的熵注册表/治理结构关系不直）
- 评估日期:    2026-10-07

### ix-infrastructure/Ix

- URL:         <https://github.com/ix-infrastructure/Ix>
- License:     Apache-2.0
- **裁决**:    ⛔ 不吸收 —— 与 `.project-map` + `check-layer-deps.sh` **重复实现**，会破坏单一真源（CODE-TOPOLOGY/D-16 mandates）
- 评估日期:    2026-10-07

### elstongun/leviathan

- URL:         <https://github.com/elstongun/leviathan>
- License:     Apache-2.0
- **裁决**:    ⛔ 不吸收 —— 仓库仅 2 天历史，证据不足以承重（R-P79 §B10.4 记录真伪是唯一硬停）
- 评估日期:    2026-10-07

### TradingAgents / tester-army/e2e

- URL:         <https://github.com/TauricResearch/TradingAgents> / <https://github.com/tester-army/e2e>
- License:     Apache-2.0
- **裁决**:    ⛔ 暂不吸收 —— 设计层面吻合的功能（决策持久化/结算、录制—重放）**未进 Rust 主实现**；记录它们的存在但本窗口不实现
- 评估日期:    2026-10-07

### memvid/memvid

- URL:         <https://github.com/memvid/memvid>
- License:     Apache-2.0
- **裁决**:    ✅ **已吸收设计** —— checkpoint 帧校验和已落地（`7b3dfbbf`，FNV-1a 64，零外部依赖）
- 评估日期:    2026-10-07
