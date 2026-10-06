# 十三源批量判定（2026-10-06）

用户批量提交 12 个源 + 早前的 `TencentCloud/Octop`。
**先批量过 LICENSE 门，再挑真正有机制增量的深读** ——
不为「吸收了」的观感而放松门槛（那等于作废刚立的规矩）。

台账 `repos.csv` 现 **539 条**。

---

## 一、许可证判定（**逐个读原文**，非依赖 API 标签）

| 源 | License | ★ | 判定 |
|---|---|---|---|
| `Imbad0202/academic-research-skills` | **CC BY-NC 4.0** | 50,557 | ⛔ **NOT-CODE** |
| `K-Dense-AI/scientific-agent-skills` | MIT | 47,701 | ✅ 设计 |
| `Yuan1z0825/nature-skills` | **Apache-2.0** | 46,039 | ✅ 设计 |
| `pingdotgg/t3code` | MIT | 25,650 | ✅ 可取码（未读，见下） |
| `wanshuiyin/Auto-claude-code-research-in-sleep` (ARIS) | MIT | 17,019 | ✅ 设计 ⭐ 见第三节 |
| `Orchestra-Research/AI-Research-SKILLs` | MIT | 13,283 | ✅ 设计 |
| `HKUSTDial/Supervisor-Skills` | **CC BY-NC-SA 4.0** | 8,280 | ⛔ **NOT-CODE** |
| `TencentCloud/Octop` | MIT | 7,029 | ✅ 设计 |
| `WUBING2023/PaperSpine` | MIT | 5,767 | ✅ 设计 |
| `brycewang-stanford/Auto-Empirical-Research-Skills` | **CC BY-SA 4.0** | 4,495 | ⛔ **NOT-CODE** |
| `jin-s13/ai-research-writing-skill` | MIT | 36 | ✅ 设计 |
| `dongzhuoyao/academic-figure-generator` | **无 LICENSE 文件** | 10 | ⛔ **NOT-CODE** |
| `ravmike/deep-research-skill` | **无 LICENSE 文件** | 6 | ⛔ **NOT-CODE** |

**5 个不可取码**（38%），其中 3 个是本批最高星（50.6k / 8.3k / 4.5k）。

### ⚠️ 为什么必须读原文，而不能用 API 标签
`Imbad0202` / `HKUSTDial` / `brycewang-stanford` 三者的 API 返回 **`NOASSERTION`**。
⛔ `NOASSERTION` 的含义是「**GitHub 无法分类**」，**不是**「宽松」。
读原文后拿到的是：

```
Imbad0202   : This work is licensed under the Creative Commons
               Attribution-NonCommercial 4.0 International License.
HKUSTDial   : Creative Commons Attribution-NonCommercial-ShareAlike 4.0
brycewang   : Creative Commons Attribution-ShareAlike 4.0 International
```

`NOASSERTION` 里藏着 **NC（禁商用）** 与 **SA（传染性 copyleft）** ——
若只看标签就放行，等于把禁商用内容放进要分发的仓。
另两个（`dongzhuoyao` / `ravmike`）**连 LICENSE 文件都没有**
⇒ 默认版权保留 ⇒ ⛔（且星数仅 10 / 6，价值也低）。

---

## 二、这批源到底是什么，以及一个我必须说清的事实

13 个里 **10 个是「agent skills 集合」**（Claude Skills 形态：`SKILL.md` + frontmatter），
主题高度集中：**学术科研工作流**（检索→写作→评审→润色→投稿、图表生成、实证研究）。

⚠️ **因此**：把它们接进 NeoTrix 是一个**产品方向决策**，不是缺陷修复。
我没有擅自把 8 个 skill 包转换接入 —— 那会是一次没人要求过的方向性投入。

### 但前置条件已核实（这是有价值的部分）
本仓**确实有** skills 机制，且有正式契约：
- `skills/SKILL-SPEC.md` —— NT-* skill 接口契约
  （源自 Easel 的 `SKILL.md` 模式，2026-09-08 八源批次吸收）
  要求目录形态：`SKILL-SPEC.md` + `references/` + **`scripts/`** + `tests/`
- 代码侧：`neotrix-types` 的 `skill.rs` / `skill_tree.rs`、
  `neotrix-gateway` 的 `skill_registry.rs`（`SkillFrontmatter` / `with_defaults()`）

⇒ **接得上**，但这 10 个是 Claude Skills 形态 ⇒ 属**转换**而非即插即用。

---

## 三、⭐ 唯一一个**挑战了我方规格**的发现（值得单独记）

`wanshuiyin/Auto-claude-code-research-in-sleep`（MIT，17k★）自述：
*"**Lightweight Markdown-only skills** for autonomous ML research"*。

**markdown-only = skill 里没有可执行脚本。**

而我方 `SKILL-SPEC.md` **要求 `scripts/`（运行期被调用的可执行代码）**。

⇒ 这不是「抄一个 skill」，而是**对我方规格的一个反问**：
> skill 路径上**到底该不该有可执行代码**？

值得考虑的收益（尚未实施，仅记录）：
· 可执行脚本是 skill 供应链的**执行面**；纯 markdown 把这一面去掉了
· 与本轮吸收的 uber/ADR 纪律同族：**不可信来源的代码不该获得执行权**
  （「审计不抄内容」「不可信段不进可信槽位」都是这条的延伸）
· 代价：失去脚本化的确定性能力（计算类 skill 会变弱）

⚠️ **不动规格**：改 `SKILL-SPEC.md` 会影响全部既有 NT-* skill，
属独立立项。记在此处供裁决。

---

## 四、本轮实际落地的缺陷修复（与这批源无关，来自上一轮分析）

见提交 `7a8d4322`：**审批绑定内容指纹，堵住「批准 A、执行 B」的 TOCTOU**。
外部依据是一份公开的系统提示纪律（确认只覆盖用户当时看到的那份内容）
与 `uber/ADR` 的 provenance 纪律。
⚠️ 该笔**尚未接线到执行点** —— 受 `CLAIMED-BUT-NOT-ENFORCED` 第 9 项
记录的同一依赖方向约束限制，已在提交信息里显式标注。

---

## 五、明确**未**做的事（避免下一个人重复评估）

| 事项 | 为何不做 |
|---|---|
| 转换接入 8 个 MIT skill 包 | **产品方向决策**，非缺陷修复；且需逐个转换（Claude Skills → NT 契约） |
| 取 NC / SA / 无 LICENSE 源的码 | 门未过 |
| 改 `SKILL-SPEC.md` 去掉 `scripts/` | 影响全部既有 skill，属独立立项 |
| 深读 `t3code` / `Octop` 的具体机制 | 已登记（MIT）；待明确「是否要接 TypeScript/Python 双栈的 agent 框架」后再读，避免读了却无处接线（**导出 ≠ 接入**） |
