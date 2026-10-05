# 八源许可证判定 — 2026-10-05

> **判据**：`ABSORPTION-PRECONDITION-GATE-2026-10-03.md`。
> **核实时间**：2026-10-05（取数用 GitHub API + LICENSE 原文读取）。
> ⚠️ **许可会变**：GitNexus 在取证前几分钟仍有 push。改判据须重跑。

## ⭐ 本批的核心教训：`NOASSERTION` ≠ 无许可证

三个源 API 返回 `NOASSERTION`（GitHub 认作 "Other"）。
**全部读原文后都是真限制，没有一个是「没许可证」**：

| 源 | SPDX | 原文实际是 | 后果 |
|---|---|---|---|
| GitNexus | `NOASSERTION` | **PolyForm Noncommercial 1.0.0** | ⛔ 禁商用 |
| htd-ai-augmented-education | `NOASSERTION` | **CC BY-NC 4.0** | ⛔ 禁商用 |

⭐⭐ **这两种比「无 LICENSE」更危险**：
无 LICENSE = 默认全权保留（你什么都拿不到，安全地失败）；
PolyForm-NC / CC-BY-NC = **「看起来能拿，拿到就违法」**。

（依据：AGENTS.md 的 `lightpanda` 曾显示 AGPL、`open-slide` 曾误判 ——
**星数与描述都不能替代读原文**。）

---

## 判定汇总

| 源 | ★ | 语言 | 许可（原文核实） | 可否取码 | 关键限制 |
|---|---|---|---|---|---|
| `HKUDS/Vibe-Trading` | 34,727 | Python | **MIT** | ✅ 可 | 仅标准署名 |
| `michael-denyer/pstack-claude` | 1,202 | JavaScript | **MIT** | ✅ 可 | **双版权**（Tan + Denyer），须保留两位 |
| `gian-gg/icon-marquee` | 16 | TypeScript | **MIT** | ✅ 可 | 仓龄 2 天，成熟度低 |
| `BootLoops-ai/bootloops` | 241 | Python | **代码 MIT + 文档 CC BY 4.0** | ⚠️ **分层** | 代码/文档许可不同，须分开对待 |
| `rizinorg/cutter` | 19,878 | C++ | **GPL-3.0** | ⚠️ **需裁决** | copyleft：整作品须 GPL、禁并入专有程序 |
| `abhigyanpatwari/GitNexus` | 47,728 | TypeScript | **PolyForm NC 1.0.0** | ⛔ 不可 | NC + 强制 `Required Notice` + 禁再许可 |
| `AFK-surf/Comma` | 172 | Elixir | **AGPL-3.0** | ⛔ 不可 | **网络 copyleft**：改后须对使用者开放源码 |
| `HytidelLegend/htd-ai-augmented-education` | 268 | Python | **CC BY-NC 4.0** | ⛔ 不可 | NC + 署名 + 须标注改动 |
| `anthropic.com/research/claude-shaped-science` | — | — | **仅文档，无许可声明** | 仅取思想 | 文章本身不可取码 |

---

## 逐源证据（逐字引用）

### ⛔ GitNexus — PolyForm Noncommercial 1.0.0
- LICENSE 首行：`PolyForm Noncommercial 1.0.0`
- 授权范围：`The licensor grants you a copyright license for the software to do
  everything you might do with the software that would otherwise infringe the
  licensor's copyright in it for any permitted purpose.`
- NC 段全文仅一句：`Any noncommercial purpose is a permitted purpose.`
- 强制署名：`You must ensure that anyone who gets a copy of any part of the
  software from you also gets a copy of these terms ... as well as copies of any
  plain-text lines beginning with 'Required Notice:'`
- 禁再许可：`These terms do not allow you to sublicense or transfer any of your
  licenses to anyone else`
- README 自述商用需另购：`GitNexus is available as an enterprise offering —
  fully managed SaaS or self-hosted deployment. Commercial use of the OSS version
  is also available with proper licensing.`

⇒ NeoTrix 商用 ⇒ **不满足 permitted purpose** ⇒ 只记设计。

### ⛔ Comma — AGPL-3.0
- 首两行：`GNU AFFERO GENERAL PUBLIC LICENSE` / `Version 3, 19 November 2007`
- §13 网络条款：`if you modify the Program, your modified version must prominently
  offer all users interacting with it remotely through a computer network ...
  an opportunity to receive the Corresponding Source of your version by providing
  access to the Corresponding Source from a network server at no charge`
- §10 `Sublicensing is not allowed`

⚠️ 关键区分：AGPL §2 `You may charge any price or no price for each copy that
you convey` ⇒ **「开源」≠「非商用」**（与 NC 类根本不同）。
但**网络 copyleft 对长期运营的服务冲突最重** ⇒ 仍判不可。

### ⚠️ cutter — GPL-3.0（取证路径曲折，记录之）
- `raw.../rizinorg/cutter/dev/LICENSE` ⇒ **404**（默认分支是 `dev` 不是 `main`，
  且文件不叫 `LICENSE`）。
- 改用 `api.github.com/repos/rizinorg/cutter/contents/?ref=dev` 列根目录
  ⇒ 实际文件名是 **`COPYING`**（35,148 字节）⇒ 取 `raw.../dev/COPYING` 得 200。
- 首两行：`GNU GENERAL PUBLIC LICENSE` / `Version 3, 29 June 2007`
- 文末关键句：`The GNU General Public License does not permit incorporating your
  program into proprietary programs.`

⭐ **这是本批最易漏的一个**：只看 `LICENSE` 会误判为「无许可证」，
而它是 copyleft。⇒ **文件名不是判据，内容才是。**

**⚠️ 门控空白区**：本仓白名单只列 MIT/Apache/BSD ⇒ 可取码；AGPL ⇒ 不可。
**GPL-3.0 两边都不在** ⇒ 不能默认当宽松处理。
裁决建议：闭源商用 ⇒ 不可；仅取设计 ⇒ 可。**列为待裁决项，不擅自放宽。**

### ⚠️ BootLoops（文章指向的代码仓）— 分层许可
- description 原文：`BootLoops 1.0: certified computational tools and house
  engines ... MIT; docs CC BY 4.0.`
⇒ **代码 MIT（可取，需署名）／文档 CC BY 4.0（可取，需署名+标注改动）**
⇒ ⛔ **不要整仓当 MIT 处理**。

### ✅ pstack-claude — MIT，但有血缘缺口
- 首三行：`MIT License` / `Copyright (c) 2026 Lauren Tan` /
  `Copyright (c) 2026 Michael Denyer`
- ⛔ description 自述是 `Poteto's pstack` 的移植（Cursor 原语翻译）。
  MIT 只覆盖**本仓**代码；上溯到原始 `pstack` / Cursor 的许可**本轮未取证**。
⇒ 取码前须补这一步。

---

## 📌 claude-shaped-science（文章，仅取思想）

作者 Matthew Schwartz（Stanford 高能理论物理，非 Anthropic 员工；
文末 Disclosure：`BootLoops is not an Anthropic project`）。

⭐ 核心主张（与本仓直接相关的一条）：
> `in almost all cases, Claude was technically correct, but the result was not
> all that interesting until the expert helped steer us`

⇒ 「技术上正确 ≠ 科学上平庸」——这正是本仓
`EMERGENCE-PLAN` §6.4「自报独立性」要防的东西的**人类版本**：
**独立判官也需要领域专家校准方向**，否则它会稳定地给出「正确但平庸」的高分。

另两条可记：① 「别硬拗，找 Claude-shaped 问题」
（只投给当前模型擅长的可编程/可验证问题）；② 同构方程是杠杆
（同一批方程跨物理/生态/群体遗传反复出现 ⇒ 一个方法四处用）。

---

## 下一批动作

| 优先 | 动作 | 依据 |
|---|---|---|
| **P0** | 补 `GPL-3.0` 到门控白名单的**显式裁决**（当前是空白区） | cutter 实测为 GPL-3.0，而白名单两边都不含它 |
| **P1** | 从 Vibe-Trading（MIT，34.7k★）取回测/多 agent 编排的**设计** | 唯一「大而可取码」的源 |
| **P1** | 把「技术正确但平庸 ⇒ 需专家校准」写进 `EMERGENCE-PLAN` 失效条件 | 文章核心主张，直击 §6.4 |
| P2 | pstack-claude 取码前补「原始 pstack 许可」取证 | 血缘缺口 |
| P2 | BootLoops 分层许可落地（代码 MIT ≠ 文档 MIT） | 分层许可 |

## ⛔ 否定结论（留档防重犯）

- ⛔ **不取码**：GitNexus（PolyForm NC）、Comma（AGPL）、htd-ai（CC BY-NC）。
  三者星数分别 47.7k / 172 / 268 —— **星数与许可无关**。
- ⛔ **不擅自放宽** cutter：GPL-3.0 在本仓门控里是空白区，
  「白名单没列」不等于「允许」。
- ⛔ **不把 BootLoops 整仓当 MIT**：代码与文档许可不同。