# 吸收：`YuJunZhiXue/github-skill-forge`（2026-10-07）

> **入参**：一条裸 URL（无其他信息）。
> **依据**：`NEOTRIX-STD-1.0.md` **NTS-B10**（含 B10.1–B10.4）· 操作面 `skills/external-absorption/SKILL.md` §URL-only 熔炼模式。
> **本篇 ⛔ 未取任何逐字代码** —— 见 §1 许可判定。

---

## 0. 信号初筛

| 项 | 值 | 取数方式 |
|---|---|---|
| 仓 | `YuJunZhiXue/github-skill-forge` | 用户 URL |
| ★ / fork | **745** / 89 | 仓库页 About 栏 |
| commits | **9** | 仓库页 History |
| 语言 / 形态 | Python 单脚本 `forge.py` + `SKILL.md`（**外部仓路径，非本仓**） | 仓库页文件树 |
| **许可** | ⛔ **无 LICENSE** | 见 §1 |

⇒ **信号充分但不惊艳**（高 star / 极少 commit ⇒ 单点工具，非系统）。
⭐ 按 NTS-B10.1，**这些字段只记录、不构成否决**；本篇仍照常完成吸收。

---

## 1. ⛔ 许可判定：**无 LICENSE ⇒ 只取设计**

【实测】`raw.githubusercontent.com/.../main/` 四个候选路径全 **404**：

```
LICENSE -> 404   LICENSE.md -> 404   LICENSE.txt -> 404   COPYING -> 404
```

GitHub API 因匿名限流返回 403（未取到 `license.spdx_id`），但 raw 404 已是决定性的。
仓库页 About 栏亦无 License 徽标。

⇒ **默认全部版权保留** ⇒ 本篇吸收的是**设计形状**（五段流水线 + 上下文成束 + 信号初筛），
**不是** `forge.py` 的实现。对照本仓既有判据
（`ABSORPTION-AGENT-ARCH2-2026-09-29.md` 记录的「3 仓无 LICENSE ⇒ 只取设计」）。

⭐ **这条不是否决，是取材方式。** NTS-B10.4：吸收永不因「开源/无许可」被拦，
但**记录必须与事实一致** —— 记录是「无 LICENSE / 仅设计」，那它就该这么写。

---

## 2. 四字段吸收矩阵

| Source | Pattern（机制） | NeoTrix 映射节点 | 判定 | 消费者（R-P79） |
|---|---|---|---|---|
| forge §Zero-Clone | 不 `git clone`，走 GitHub API 扫仓，产上下文包 | ⛔ 原引用 `scripts/kb_batch_absorb.py` **实测已不存在**（deadlink）⇒ 落点改为 **`webfetch` + raw 路径**（2026-10-07 实测：GitHub API 匿名 403，**raw 全程可用**，故主路径是 raw 非 API） | **强化** | 已在用；本篇把「clone 非必需」升为规范（B10.1） |
| forge §Lite-RAG | 剔杂物 → 文件树 + README/文档 + **依赖清单**（`requirements.txt`/`package.json`/`pyproject.toml`）→ 单个 `context_bundle.md` | `l2_perception/nt_core_code_search.rs` 的 `context_bundle()` | **强化**（非新增） | 同原语我方已有；**依赖清单这一维是我方缺口** → §3 |
| forge §镜像加速 | 多组 API 镜像轮换 + 多线程，绕 GitHub 频率限制 | 已有等价路径见 `ABSORPTION-INDEX-user-urls` Cycle 232 条（fake-ip DNS 污染绕行） | **强化** | 已有 |
| forge §质量初筛 | stars + 活跃度筛掉未写完/坑仓库，低 star 需 `--force` | `scripts/ops/nt_absorption_enrich.py`（已取 `stars/license/archived/pushed_at/language/topics`） | **强化** | 数据已有；⭐ 由「记录」升级为「**默认即 force，不作否决**」 |
| forge 全局 | 「制造技能的技能」= 元技能自我锻造 | `skills/external-absorption/SKILL.md` 本节 | **强化** | 熔炼模式已成为默认路径 |

⇒ **5 条全部「强化」，零「新增」。**
⭐ 这个结果本身就是结论：**我方的零克隆吸收流水线已经建成**，
本仓的价值不在补能力，而在**给这套流水线一个规范化的入口契约**（B10.1–B10.4）。

---

## 3. 熔炼五段（默认路径）

```
0 信号初筛 → 1 零克隆取源 → 2 熔炼成束 → 3 化为已有 → 4 落账
```

| 段 | 本仓既有实现 | 状态 |
|---|---|---|
| 0 信号初筛 | `nt_absorption_enrich.py` 6 字段 | ✅ 有数据 · ⬜ 本篇升级语义为「不否决」 |
| 1 零克隆取源 | `kb_batch_absorb.py` | ✅ |
| 2 熔炼成束 | `l2_perception/nt_core_code_search.rs` 的 `context_bundle()` | ⬜ **依赖清单维度缺口**（见 §4） |
| 3 化为已有 | `absorb_to_capability.py` + NT- 分支树 | ✅ |
| 4 落账 | Rust CLI `absorb-node` / `update-node-metadata` + ADR/SIM | ✅ |

---

## 4. 唯一真缺口：熔炼成束的「依赖清单」维度

forge 在上下文包里除文件树/README 外，还收集 **依赖清单**
（`requirements.txt` / `package.json` / `pyproject.toml`）——
⭐ 这是**「外部仓的依赖面」**，能直接回答「吸收它会引入什么」。
我方 `context_bundle()` 未覆盖此维度。

**接线裁决（Cycle 1201 三选一）**：
- ✅ **本 session 接线** —— 不做。
  理由：属 `nt_core_code_search`（L2 感知）职责扩张，且需定向裁定「依赖清单」对
  我方（Rust workspace）是否已有等价物（`Cargo.toml` 面）。**未经核实不得开刀** ——
  这正是本仓反复犯的错（`ABSORPTION-PRECONDITION-GATE` §二：「用一个不完整的检索直接跳到『所以该做 X』」）。
- 📋 **路线图降级** —— 依赖面**已有等价物**：`cargo metadata` / `nt_absorption_enrich.py` 的
  `topics`+`language`。⇒ 待 §二三问核实后再定。
- ❌ 拒绝 —— 不选（无实体依据）。

⇒ **本篇落「📋 待核实」，不落「✅ 接线」。**
按 NTS-B10.2，**它进演化账本作 intake，不被丢弃**。

---

## 5. 规则改动（本会话产出）

| 文件 | 改什么 |
|---|---|
| `docs/standards/NEOTRIX-STD-1.0.md` | **NTS-B10** 增 B10.1 URL-only / B10.2 熔炼化为已有 / B10.3 前置门降级为分流 / B10.4 记录真伪是唯一硬停 |
| `skills/external-absorption/SKILL.md` | 增「默认路径：URL-only 熔炼模式」+ 裸 URL / 「熔炼」触发词 |
| `docs/architecture/ABSORPTION-PRECONDITION-GATE-2026-10-03.md` | 增「已被 NTS-B10.3 改判」指针（原文保留，不改写历史） |

⭐ **保留不动的**：`scripts/check-license.sh`。
它是**记录一致性**门（明写「不做法律判断」）⇒ 与 B10.4 同向，**不是要被拆掉的闸**。

---

## 6. 方法论沉淀

⭐ **「9 commits + 745★ + 无 LICENSE」这一组合，恰好示范了 NTS-B10 的全部四小节**：
高 star 证明它值得看（B10.1 不因冷门否决）；无 LICENSE 证明它只配被读设计（B10.4）；
单文件结构证明它补不了我方流水线（5 条全「强化」）；而「制造技能的技能」这个元概念
反倒提醒我方：**吸收管线的入口契约，本身就是可被吸收的产物。**

⇒ 一句话：**外源不必比内源深，只要它让内源的入口更锋利。**
