# 吸收：olmo-core / universal-modder（2026-10-04）

> ⚠️ `Niko1221/Strata` 本会话**已吸收**（`fc5a1107`，MIT）⇒ 本篇不重复。

| 源 | ★ | commits | 许可档 |
|---|---|---|---|
| `rehan-remade/universal-modder` | 2.8k | 23 | ✅ MIT |
| `allenai/olmo-core`（AI2） | 1.7k | 909 | ⛔ **Apache-2.0**（有专利授权） |

---

## 一、`universal-modder` —— ⭐⭐ 三条与本仓纪律**同构**的设计

### 1.1 ⭐⭐ **单一权威 + 多个 agent 入口**
`.agents/skills`（Codex）、`.claude/skills`、`.gemini/skills`、`.github/skills`
**全部软链到同一个 `skills/`**；指令以 `AGENTS.md` 为**唯一正文**，
`CLAUDE.md` / `GEMINI.md` **只指向它**；MCP 配置按 agent 各存一份文件名。

⇒ **一份内容，N 个入口。**
⭐ 与本仓同构（`skills/` + `AGENTS.md` 为权威），但**它把「入口」显式列成了软链**，
而我们靠约定。⇒ 可对照项：**我们是否有 agent 间内容漂移**（未查证）。

### 1.2 ⭐⭐⭐ field note 模板**必含「how it was verified」**
`knowledge/` 下的每条笔记固定五项：
1. **确切的、奏效的版本**
2. 路线，**以及为什么选它**
3. 该引擎**真正**做什么
4. ⭐ **`how it was verified`**
5. 坑：**症状 → 原因 → 修法**

⇒ ⭐⭐ **第 4 项是硬要求** —— 「怎么验证的」与结论**同级**，不是可选项。
⇒ 目标（原文）：**「下一个 agent 从上一个停下的地方开始，
而不是重新发现同样的坑。」**
⇒ ⭐ **本仓的文档缺这一栏**。本会话我反复做的是「先量后动 / 读现场」，
但**没有把它写进文档模板的必填项** ⇒ 靠个人纪律 ⇒ 不可继承。
⇒ 这是本篇**最值得吸收的一条**。

### 1.3 ⭐⭐ 外部产出**经人类批准**才进库
`um kb pr … --yes` 的描述是「**after your human says OK**: branch, push, PR」。
⇒ agent 可以**写**笔记，但**合并**要人批。
⇒ 与本仓 `.neotrix/task-index.json` 的 pre-commit 校验同一思路，
但它把「人」放在**内容入口**而非只放在**代码入口**。

### 1.4 ⭐ 安全规则**独立成文件**且**有阻断门**
- 安全推理集中在 `skills/mod-any-game/references/safety.md`，**不内联在主流程**
- ⭐ `um publish check`是**阻断门**：拦住「夹带游戏文件 / 反编译代码 / 泄露密钥」
- 「杀进程**只按 PID**」「动键鼠前先问」—— 均为**具体到可执行**的约束

⇒ 与我本会话修的「cookie 值 redact」同源：**把「不要泄露」变成机制而非提醒**。

---

## 二、`olmo-core` —— 三条可对照项（⛔ 均未查证我方前提）

### 2.1 ⭐ **点号路径配置覆盖**
```
--train_module.optim.lr=6e-3
```
⇒ 一个**统一的配置寻址方案**（`模块.子模块.字段=值`）。
⭐ 我方有 `nt_types.rs` /能力注册表 / 层参数，但**是否已有统一寻址**未查证。

### 2.2 ⭐⭐ Docker **只装依赖、不装本包**
原文：「They do not come with the Olmo-core package installed, **only its
dependencies, to accommodate for regular code changes**」+「在自己的集群上
**可能因硬件/驱动/CUDA 版本不同而不可用**」。

⇒ **镜像与代码解耦**的部署纪律，且**把不可用的原因写进 README**。
⭐ 与我本会话的 `nt_build_lock` / worktree纪律同族：
**把「为什么会坏」提前写下来**。

### 2.3 ⭐ 三层推理兼容性**显式声明**
HF transformers · vLLM · 自家 beta（`generate.chat`）⇒ **三档并列写清**，
不说「都能用」。

### 2.4 ⭐ 三道**分开**的门
`make style-check`（isort+black）· `make lint-check`（ruff）· `make type-check`（mypy）
⇒ 风格 / lint / 类型**三门独立**，不合并成一个「检查」。

### 2.5 ⭐ 仓内带 agent 面向资产
`AGENTS.md` + `CLAUDE.md` + **`.claude/skills/training-smoke-test`**
⇒ ⭐⭐ **把「冒烟测试」做成 skill**，而不只是 CI 里的一条命令
⇒ 与本会话反复做的「把纪律变成机器可判」同向。

---

## 三、前置门结论（**本篇全部 ⛔ 不落代码**）
| 源 | ① 危险面 | ② 可达 | ③ 我方 | 裁定 |
|---|---|---|---|---|
| universal-modder §1.2 | ✅ **本会话实证 4+ 次**：我更正了多处陈旧断言，而文档**没有「怎么验证的」栏** | ✅ `docs/architecture/` | ✅ | **通过** ⇒ 但需先定**模板**，非改单篇 |
| universal-modder §1.1 | ⛔ 未查证我方是否有 agent 间漂移 | — | — | ⛔ 待查 |
| olmo-core §2.1 | ⛔ 未查证我方是否已有统一配置寻址 | — | — | ⛔ 待查 |
| olmo-core §2.4/§2.5 | ✅ 我方已有 4 个门 + 1 个 task-index | ✅ | ✅ | ✅ 已有等价物 |

⇒ ⭐ **唯一下一步该做**：给 `docs/architecture/` 的结论类文档
**加一个必填的「验证方式」栏**，并**先在一篇上试点**（不批量）。
