#!/usr/bin/env python3
r"""安全接线门 —— 抓「造了资产但没人调用」。

## ⚠️ 写操作声明（R-SCAN-4 / R-SCAN-4 事故后置入）

**本脚本只读。** 除 `--baseline <路径>` 这一模式外，**没有任何写操作**：
只读文件（open 只用 "r"）+ 只写 stdout/stderr。本脚本不执行任何子进程，
不调用 cargo，不碰 target/。

⛔ 本脚本的输出文案里**绝不出现反引号包裹的命令**。仓库历史上
`scripts/check-disk.sh` 在「建议」文案里写了反引号示例命令，bash 把它当命令
替换真的执行了 `cargo clean`，删掉了 115.2 GiB。反引号在 bash 里不是排版。

## 本门度量的是第四类「看不见」

| 类别 | 谁能看到 | 状态 |
|---|---|---|
| EMPTY / UNDECLARED / TRACKED | `scripts/check-truth-surface.sh` | ✅ 已覆盖 |
| 「编译了但零生产调用方」 | —— | ⛔ **本门新增** |
| 「注册式命名扩展 API 零注册」 | `scripts/check-ext-wiring.py` | ✅ 已覆盖 |

⭐ **为什么不能只加一个 ext-wiring 的关键词**：`check-ext-wiring.py` 的 API 正则
是 `pub fn (on_|register_|add_hook|subscribe)\w*\s*\(`，即**只认注册式命名**。
`DnsEgressPolicy` / `ToolSandbox` / `SecurityManager` 这类**非注册式的策略资产**
它一条都看不见 —— 而它们恰恰是安全资产的主力形态。

⭐ **五类发现**
- `ORPHAN_MODULE` —— 目录里有 `.rs`、看起来像个模块（有自己的 `mod.rs`
  或目录内 ≥2 个 `.rs`），但**父目录下没有任何 `.rs` 声明 `mod <dirname>;`**
  ⇒ 这些文件**从未被编译**。抓 `nt_shield/guard/agent_guardrails/`（1,492 行）。
- `ZERO_CONSUMER` —— 是 `pub`、定义文件 ≥ 60 行、**已被 `mod` 声明（确实编译）**、
  但在非测试非自身文件里找不到任何消费者，**且治理判据至少有一路是
  name / file / 方法名**。
- `PATH_ONLY` —— 同上，但**唯一**的治理判据是「所在目录名含治理词」（见下节）。
- `TEST_ONLY` —— 被测试引用但无生产消费者（与 ZERO_CONSUMER 分开列）。
- `WIRED_OK` —— 有 ≥1 个非测试非自身消费者的资产（**只计数，不逐个列出**）。

## ⭐ 本门修掉的两处「现有门的缺陷」

### 缺陷 1：`#[cfg(test)]` 的切分必须是花括号配对，不能是 split

`scripts/check-ext-wiring.py` 的 `prod_body()` 是
`text.split("#[cfg(test)]")[0]`。一旦某文件把测试 mod 放在**前面**，
其后整份文件都被当成测试代码 ⇒ 消费者隐形 ⇒ 资产被判死。

实测（2026-10-05）：`neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/stagnation.rs`
的 `#[cfg(test)]` 在第 32 行 / 共 819 行 ⇒ 后面 787 行对该门隐形。
根因是 split 无视 `mod tests { ... }` 的真实边界。**本门用花括号配对**：
从 `#[cfg(test)]` 之后的 `{` 开始数括号（掩码后字符串/注释已清空，
括号不会被字面量里的 `{` 干扰），匹配到配对 `}` 为止，并**同时**处理文件中
多处 `#[cfg(test)]`。

### 缺陷 2：判「消费者」前必须剥注释与字符串字面量

本仓把禁词/示例签名**当数据持有**。裸 grep 会把字面量当代码。

⭐ **已有实证（R-SCAN-1b 的实例）**：
`neotrix-core/src/l5_cognition/nt_mind/nt_mind/infrastructure/code_review.rs:500`
有 `let code = r#"Command::new("sh").args(["-c", &cmd])"#;` —— 那是**字符串字面量**
（一个 OWASP 命令注入测试的输入样本），不是代码调用。

本门实现一个词法扫描器 `mask_noncode()`，把注释（`//`、`///`、`/* */`，
块注释按 Rust 语义**可嵌套**）与字符串/字符/字节/裸字符串字面量
（`"..."`、`r"..."`、`r#"..."#`、`br##"..."##`、`b"..."`、`'a'`、`'\n'`、`'\u{1F}'`）
全部替换成**等长空格**，从而**保持字节偏移不变**（行列号与切片照常可用），
再在剩下的真实代码上做标识符匹配。

`'a'` vs 生命周期 `'a`：前瞻判定 —— `'` 后若是反斜杠转义、或标识符紧跟
第二个 `'` ⇒ 字符字面量；否则是生命周期（不动它，否则会吃掉后面整段代码）。

## ⭐ 语义判定口径（requirement 4：不做关键词预筛）

**先全量枚举所有 `pub` 项（定义文件 ≥ 60 行），再按语义分类。**
⛔ **不用路径/名字先筛 `secur|policy|gate|...` 再枚举** —— 试过，
`demote_mislabeled`、`SkillAudience::admits`、`ActionFacade` 全部漏掉
（名字里根本没有安全词，但它们确实是策略/授权/治理动作资产）。

分类信号（三路取或，**都只在枚举之后**参与分类）：

1. **名字信号** —— 资产名、文件名（stem）、所在路径命中治理词表。
2. **方法信号** —— 在该资产**自己的 `impl` 块**内（花括号配对切出）出现的
   `fn` 名命中治理词表。这一路专治「名字中性、方法语义明确」的资产：
   `SkillAudience::admits`、`ActionFacade::authorize`。
3. **治理动作信号** —— 方法名匹配 `demote|promote|revoke|quarantine|escalat|
   rollback|heal|repair|reconcile` 等**改变授权/等级/存续状态的动作**。
   这一路专治 `demote_mislabeled`（能力诚实度自愈动作）。

### ⭐ `PATH_ONLY`：把「仅目录名命中」从 ZERO_CONSUMER 里拆出来（2026-10-06）

⭐ **为什么拆**：`classify()` 里 name / file / path / 方法名四路是**取或**的。
当三路都落空、只剩 path 一路命中时，「它是个安全资产」这件事的**全部**证据
就是**它待在哪个目录里** —— 一个目录名会把该目录下**每一个** `pub` 项都算成
安全资产。这类条目逐条**没有行动价值**，混在 ZERO_CONSUMER 里会让使用者
对整个门失去信任（AGENTS.md §5：噪音门记录比没有门更危险）。

⭐ **实测（2026-10-06，重算前的基线 1,969 条 ZERO_CONSUMER）**：

| 判据 | 条数 |
|---|---|
| `why == "path"`（仅目录名） | **693**（35.2%） |
| 含 name / file 判据 | 1,199 |
| 含 `method:` 判据 | 77 |

693 条触发的目录名高度集中：`nt_shield`(495) / `gateway`(86) /
`social_access`(50) / `nt_core_gate`(42) / `healing`(19) / `health`(11) …
⇒ 只覆盖 **181 个文件 / 59 个目录**，其中 483/693 是 snake_case（`pub fn` /
`pub mod` / `pub type`）—— 对这三类 item **方法名那一路根本不会执行**
（见 `classify()` 的 `if kind in (...)`）⇒ 连潜在的语义信号都没被看过。

⭐ **为什么不把 `file` 一路也拆出去**（实测后确认边界是对的）：305 条 file-only
的文件名是 `nt_approval.rs` / `nt_judge.rs` / `nt_law_gate.rs` /
`nt_permission_profiles.rs` 这种**具体治理模块名**，不是整 crate 的统称
⇒ 是可用判据，不是噪音。path 一路之所以不同，是因为它匹配的是**路径里任意
一段**，包括 crate 名与中间层目录名。

⚠️ **`TEST_ONLY` 未拆**：实测 168 条里有 53 条同样是仅目录名判据。本门只在
`--audit` 里把这个残余数**显式报出来**，不改变其归类（拆它会动到另一类发现
的语义，超出本次范围）。⇒ 弱信号总量 = `PATH_ONLY` + 该残余数。

治理词表（SKIP 逻辑见 `POLICY_WORDS`）分四簇：
- **安全/权限**：`security secure shield guard sandbox jail isolat confine
  egress firewall blocklist denylist allowlist permission privilege auth
  authorize authorize* admit allow grant revoke quota scope rbac acl`
- **策略/门禁**：`policy polic* gate gatekeeper guardrail checkpoint admission
  regulate constraint invariant veto breaker circuit throttle limit ratelimit
  budget cap filter sanity screen`
- **校验/审计**：`validate valid verify audit approv attest seal sign
  compliance conform proof credential token secret`
- **治理动作**：`demote promote revoke quarantine escalate rollback heal
  repair reconcile demotion promotion verdict judge`

⭐ 口径的**已知泄漏**（必须明说）：词表终究是词表。
`--all-items` 模式会把分类为 `OTHER`（非治理类）的资产也计数报出，
让「漏了几条」可测量，而不是宣称零漏。

⚠️ **弱消费者信号的口径**（呼应 DIR-REMEDY §2.5「导出 ≠ 调用」）：
消费者 = 其它文件非测试区的**标识符出现**（含 `use` 导入与 `pub use` 再导出）。
这是**弱信号** —— 只被 `use` 提到的资产算 `WIRED_OK`，但会被单独计数为
`WEAK_ONLY`（info，不判红），让「只导出没调用」仍可见。

## ⭐ ORPHAN_MODULE 与 check-truth-surface.sh 的重叠（必须说清）

`check-truth-surface.sh` 的 `UNREACHABLE`/`UNDECLARED` 用**传递 mod 可达性**
覆盖了同一批文件（含 `agent_guardrails/**` 4 个文件，见
`scripts/truth-surface-baseline.txt` 的 UNREACHABLE 段）。
⇒ **本门的 `ORPHAN_MODULE` 在范围上与它高度重叠**，不是新信息。

本门的**增量**在两点：
(a) 本门**不重复**列 UNREACHABLE 已有的存量 —— 输出里会标
    `=truth-surface` 表示该目录/文件已在 `truth-surface-baseline.txt` 里；
    真正的新增（不在 truth-surface 基线里的孤儿）是**独立的一条红**，
    因为 truth-surface 的基线一旦被 `--update-baseline` 重算就会把它合法化。
(b) 本门把「零生产消费者」这件 truth-surface **完全没有的类别**独立建模。

## 用法

    python3 scripts/ops/nt_security_wiring.py --audit         # 打印清单, 不判失败
    python3 scripts/ops/nt_security_wiring.py --strict        # 有新增发现 => exit 1
    python3 scripts/ops/nt_security_wiring.py --baseline      # 写默认基线路径
    python3 scripts/ops/nt_security_wiring.py --baseline PATH # 写到指定路径
    python3 scripts/ops/nt_security_wiring.py --self-test     # 注入式负向测试
    python3 scripts/ops/nt_security_wiring.py --all-items     # 额外报出 OTHER 类计数

基线默认路径：`scripts/security-wiring-baseline.txt`（格式对齐
`scripts/truth-surface-baseline.txt`：`#` 理由注释行 + `KIND 定位符` 条目行）。

⭐ **基线格式区分强弱信号靠的是 KIND 前缀本身**：`ZERO_CONSUMER <loc>` 与
`PATH_ONLY <loc>` 各占一种前缀，行尾的 `# L<n> (<判据>)` 注释同时把判据
写进文件，所以「哪些条目只有目录名判据」在基线文件里也一眼可查。
`read_baseline()` **不需要**为新 KIND 改动（它按 `KIND <loc>` 逐字读），
拆分前的旧基线由 `fold_split_baseline()` 在比较阶段补齐。

⛔ **建基线是需要判断的动作** —— 一旦建立，当前这些存量发现就被「合法化」。
基线里必须逐条写理由（沿用 truth-surface 的做法：重算时保留既有 `#` 行）。
"""

import os
import re
import shutil
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

ROOTS = ["neotrix-core/src", "neotrix-core/tests", "crates", "apps", "neobot"]
BASELINE = os.path.join(REPO, "scripts", "security-wiring-baseline.txt")
TRUTH_BASELINE = os.path.join(REPO, "scripts", "truth-surface-baseline.txt")

SKIP_DIRS = ("/target", "/node_modules", "/.git", "/.worktrees", "/target-metadata")

#: 不参与 ORPHAN_MODULE 判定的目录名。
#: `src` / `bin` 是 **Cargo 自动发现**的约定目录 —— crate 根下的 `src/lib.rs`
#: 与 `src/bin/*.rs` 不需要（也不能）有任何 `mod` 声明就编译。
#: 实测（未排这些之前）：`crates/neotrix-audit/src` 等 11 条全是这类假阳性，
#: 因为 crate 根目录（`crates/neotrix-audit/`）里**一个 .rs 都没有**，
#: `lib.rs` 在 `src/` 里面 ⇒ 「父目录下没有 .rs 声明它」字面成立。
SKIP_BASENAMES = {
    "src", "bin", "bins",                       # Cargo 自动发现
    "test", "tests", "benches", "examples",      # Cargo 约定
    "build", "fixtures", "snapshots",            # 非模块目录
}

#: 定义文件至少这么多行才算「资产」（低于此规模零消费者多半无意义）
MIN_DEF_LINES = 60


# ══════════════════════════════════════════════════════════════════
# 1. 词法扫描：注释 + 字符串/字符字面量 -> 等长空格
# ══════════════════════════════════════════════════════════════════

_IDENT_START = re.compile(r"[A-Za-z_]")


def mask_noncode(src, keep_strings=False):
    """把注释与字符串/字符字面量替换成等长空格，字节偏移保持不变。

    ``keep_strings=True`` 时**只**剥注释、保留字符串内容。
    ⭐ 唯一用途：提取 ``#[path = "…"]`` 的属性值 —— 路径**就在字符串里**，
    用全掩码文本去匹配它必然什么都找不到（实测：`bin/experience` 因此
    被误报成孤儿，尽管 `bin/experience.rs` 用 7 个 `#[path]` 挂了它）。
    对属性值而言，剥注释已足够防误报，字符串内容是数据不是代码。

    返回值与 ``src`` 等长 ⇒ 下游的 ``count("\\n", 0, off)``、``slice``、
    ``find`` 全部仍然有效。

    覆盖：`//`（含 `///` `//!`）、`/* */`（**Rust 语义可嵌套**）、
    `"..."`、`r"..."`、`r#"..."#`、`br#"..."#`、`b"..."`、
    `'c'`、`'\\n'`、`'\\u{1F600}'`。生命周期 `'a` **不当作字面量**。
    """
    out = list(src)
    n = len(src)
    i = 0
    while i < n:
        c = src[i]

        # ── 行注释
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            while i < n and src[i] != "\n":
                out[i] = " "
                i += 1
            continue

        # ── 块注释（可嵌套）
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            depth = 0
            while i < n:
                if src[i] == "/" and i + 1 < n and src[i + 1] == "*":
                    depth += 1
                    out[i] = " "
                    out[i + 1] = " "
                    i += 2
                    continue
                if src[i] == "*" and i + 1 < n and src[i + 1] == "/":
                    depth -= 1
                    out[i] = " "
                    out[i + 1] = " "
                    i += 2
                    if depth == 0:
                        break
                    continue
                if src[i] != "\n":
                    out[i] = " "
                i += 1
            continue

        # ── 原始字符串 / 字节串：r".." / r#".."# / br#".."# / b".."
        if c in "rbR" or (c == "b" and i + 1 < n and src[i + 1] in "rR"):
            j = i
            if src[j] in "bB":
                j += 1
            raw = False
            if j < n and src[j] in "rR":
                raw = True
                j += 1
            if j < n and src[j] == '"':
                hashes = 0
                while j + hashes < n and src[j + hashes] == "#":
                    hashes += 1
                close = '"' + "#" * hashes
                start = j + hashes + 1
                k = src.find(close, start)
                end = n if k < 0 else k + len(close)
                if not keep_strings:
                    for p in range(i, end):
                        if out[p] != "\n":
                            out[p] = " "
                i = end
                continue
            if not raw and j < n and src[j] == '"':
                # 普通字节串 b"..."
                k = j + 1
                while k < n:
                    if src[k] == "\\":
                        k += 2
                        continue
                    if src[k] == '"':
                        k += 1
                        break
                    k += 1
                end = min(k, n)
                if not keep_strings:
                    for p in range(i, end):
                        if out[p] != "\n":
                            out[p] = " "
                i = end
                continue
            i = max(j, i + 1)
            continue

        # ── 普通字符串
        if c == '"':
            k = i + 1
            while k < n:
                if src[k] == "\\":
                    k += 2
                    continue
                if src[k] == '"':
                    k += 1
                    break
                k += 1
            end = min(k, n)
            if not keep_strings:
                for p in range(i, end):
                    if out[p] != "\n":
                        out[p] = " "
            i = end
            continue

        # ── 字符字面量 vs 生命周期
        if c == "'":
            nxt = src[i + 1] if i + 1 < n else ""
            if nxt == "\\":
                k = i + 2
                while k < n and src[k] != "'":
                    k += 1
                end = min(k + 1, n)
                if not keep_strings:
                    for p in range(i, end):
                        out[p] = " "
                i = end
                continue
            if _IDENT_START.match(nxt):
                j = i + 1
                while j < n and (src[j].isalnum() or src[j] == "_"):
                    j += 1
                if j < n and src[j] == "'":
                    if not keep_strings:
                        for p in range(i, j + 1):
                            out[p] = " "
                    i = j + 1
                    continue
                i = j if j > i + 1 else i + 1  # 生命周期：不动
                continue
            if nxt == "'":
                if not keep_strings:
                    for p in range(i, i + 2):
                        out[p] = " "
                i += 2
                continue
            i += 1
            continue

        i += 1
    return "".join(out)


# ══════════════════════════════════════════════════════════════════
# 2. #[cfg(test..)] 块的真实范围（花括号配对）
# ══════════════════════════════════════════════════════════════════

_CFG_ATTR = re.compile(r"#\[\s*cfg\s*\(([^)]*)\)\s*\]")


def _is_test_cfg(inner):
    return re.search(r"(?<![A-Za-z0-9_])test(?![A-Za-z0-9_])", inner) is not None


def _match_brace(masked, open_idx):
    """``open_idx`` 指向 ``{``；返回配对 ``}`` 的下标（含）。找不到返回 -1。"""
    depth = 0
    i = open_idx
    n = len(masked)
    while i < n:
        ch = masked[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def test_spans(masked):
    """返回 ``#[cfg(test)]`` 内联 mod 块的 ``(start, end)`` 字节区间列表（含）。

    同时覆盖两种形态：
    · ``#[cfg(test)] mod tests { ... }``      → 从 ``{`` 配对到 ``}``
    · ``#[cfg(test)] mod tests;``             → 无块，空区间（不切）
    """
    spans = []
    for m in _CFG_ATTR.finditer(masked):
        if not _is_test_cfg(m.group(1)):
            continue
        # ⛔ 两条容易漏的跳：
        #   ① attribute 与 `mod` 之间有换行（`#[cfg(test)]\nmod tests {`），
        #      所以每轮都要先吃空白；
        #   ② `mod <name>` 本体本身。漏掉 ② 时 `masked[j]` 指向 `m` 而不是 `{`
        #      ⇒ 括号配对整段跳过 ⇒ 测试区被并入生产区 ⇒ TEST_ONLY 永不出现。
        j = m.end()
        while j < len(masked):
            ws = re.match(r"\s+", masked[j:])
            if ws:
                j += ws.end()
                continue
            mm = re.match(r"pub(?:\s*\([^)]*\))?\s+", masked[j:])
            if mm:
                j += mm.end()
                continue
            if masked[j] == "#":
                mm = re.match(r"#\[[^\]]*\]\s*", masked[j:])
                if mm:
                    j += mm.end()
                    continue
            mm = re.match(r"mod\s+[A-Za-z_][A-Za-z0-9_]*\s*", masked[j:])
            if mm:
                j += mm.end()
                continue
            # ⭐ 2026-10-07 补：`#[cfg(test)]` 也可以**直接挂在 item 上**，
            #    不带 `mod` 包裹。此前此处直接 `break` ⇒ `masked[j]` 指向 `fn`
            #    而非 `{` ⇒ 该属性**不产生任何区间** ⇒ 整个 item 被划进
            #    **生产区** ⇒ 报出假的 ZERO_CONSUMER。
            #
            # 实测受害（instrument 实证，非推断）：
            #   crates/nt-core-capability-tree/src/dispatch.rs
            #     L130 `#[cfg(test)] pub fn clear_for_tests()`
            #     L138 `#[cfg(test)] pub fn test_guard()`
            #   两者**只**被同文件 L145 的 `#[cfg(test)] mod tests` 使用
            #   ⇒ 生产零消费者 ⇒ 门本不该报。
            #   但 instrument 显示 test_spans 只产出 **1** 个区间（L145..L212）
            #   ⇒ L130/L138 两个属性被完全跳过。
            #
            # ⚠️ 我曾两次提交「这里加个 item 正则」的修法，**两次都零效果**
            #    （对照实验：修前修后 `当前/新增` 完全相同）。
            #    根因：单段 `item` 正则**连 `clear_for_tests` 都匹配不到**
            #    —— 因为 `fn` 之后还有**返回类型**才到 `{`。
            # ⇒ 故必须**两段式**：先吃掉 item 关键字与名字，
            #    再单独跳过函数签名（泛型 / 参数 / 返回类型）抵达 `{`。
            mm = re.match(
                r"(?:async\s+|const\s+|unsafe\s+|extern\s+\"[^\"]*\"\s*)*"
                r"(?:fn|struct|enum|trait|union|const|static|type)\s+"
                r"[A-Za-z_][A-Za-z0-9_]*",
                masked[j:],
            )
            if mm:
                j += mm.end()
                # 第二段：跳到函数体的 `{`（含泛型 / 参数 / 返回类型）
                sg = re.match(
                    r"(?:\s*<[^;{]*?>)?\s*\([^;{()]*\)(?:\s*->\s*[^;{]+?)?\s*\{",
                    masked[j:],
                )
                if sg:
                    j += sg.end() - 1      # 停在 `{` 上
                    break                  # 交由下面的 `{` 判定处理
                # 非函数体（如 `const X: T = ...;` / `type X = Y;`）⇒ 交回
                break
            break
        if j < len(masked) and masked[j] == "{":
            end = _match_brace(masked, j)
            if end > 0:
                spans.append((m.start(), end + 1))
    return spans


def _blank(text, spans):
    buf = list(text)
    for a, b in spans:
        for p in range(a, min(b, len(buf))):
            if buf[p] != "\n":
                buf[p] = " "
    return "".join(buf)


def prod_and_test(masked):
    """返回 (生产区, 测试区)，两者互斥且都仍是掩码后的等长文本。"""
    spans = test_spans(masked)
    return _blank(masked, spans), _blank(masked, [(0, len(masked))] if not spans
                                          else [(0, spans[0][0])] +
                                          [(spans[k][1], spans[k + 1][0])
                                           for k in range(len(spans) - 1)] +
                                          [(spans[-1][1], len(masked))])


def in_any_test_dir(path):
    return os.sep + "tests" + os.sep in path or path.endswith(os.sep + "tests")


# ══════════════════════════════════════════════════════════════════
# 3. 全量枚举 pub 资产
# ══════════════════════════════════════════════════════════════════

PUB_ITEM = re.compile(
    r"\bpub\s*(?:\(\s*[^)]*\s*\)\s*)?"
    r"(struct|enum|trait|union|fn|mod|type)\s+"
    r"([A-Za-z_][A-Za-z0-9_]*)"
)
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
FN_DEF = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")
MOD_DECL = re.compile(
    r"(?:^|[;{}\s])pub(?:\s*\([^)]*\))?\s+mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*[;{]"
)
MOD_DECL_PLAIN = re.compile(
    r"(?:^|[;{}\s])mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*[;{]"
)
#: `#[path = "experience/exp_util.rs"] mod exp_util;` —— 模块名与**目录名无关**。
#: 实测 `neotrix-core/src/bin/experience.rs` 用它挂 `bin/experience/*.rs` 7 个文件；
#: 只按「mod 名 == 目录名」判会把这个活目录误报成孤儿 ⇒ 另开一路：
#: 只要宿主里有指向该目录的 `#[path = "<dir>/…"]` 就认为已接线。
PATH_ATTR = re.compile(r"#\s*\[\s*path\s*=\s*\"([^\"]+)\"")


class FileInfo:
    __slots__ = ("path", "nlines", "prod", "test", "masked", "pathattrs")

    def __init__(self, path, text):
        self.path = path
        self.nlines = text.count("\n") + 1
        self.masked = mask_noncode(text)
        self.prod, self.test = prod_and_test(self.masked)
        # 只剥注释（保留字符串）—— 专供 #[path="…"] 属性值提取
        self.pathattrs = mask_noncode(text, keep_strings=True)


def collect_files(roots, base):
    """读入 roots 下所有 .rs；返回 ``{relpath: FileInfo}``（按给定 roots 顺序）。"""
    out = {}
    for r in roots:
        rabs = r if os.path.isabs(r) else os.path.join(base, r)
        if not os.path.isdir(rabs):
            continue
        for dp, dn, fn in os.walk(rabs):
            dn[:] = [d for d in dn if d not in SKIP_DIRS]
            if any(s in dp for s in SKIP_DIRS):
                continue
            for f in sorted(fn):
                if not f.endswith(".rs"):
                    continue
                p = os.path.join(dp, f)
                rel = os.path.relpath(p, base)
                if rel in out:
                    continue
                try:
                    with open(p, encoding="utf-8", errors="ignore") as fh:
                        text = fh.read()
                except OSError:
                    continue
                out[rel] = FileInfo(rel, text)
    return out


def _scan(body, seen, called, idx):
    """把 body 里的标识符并入 seen（出现）/ called（紧跟左括号）两张表。"""
    for m in IDENT.finditer(body):
        name = m.group(0)
        s = seen.get(name)
        if s is None:
            seen[name] = s = set()
        s.add(idx)
        j = m.end()
        while j < len(body) and body[j] in " \t\r\n":
            j += 1
        if j < len(body) and body[j] == "(":
            cs = called.get(name)
            if cs is None:
                called[name] = cs = set()
            cs.add(idx)


def index_consumers(files):
    """建立 4 张反向索引（标识符 -> 文件下标集合）。

    · ``seen`` / ``called``：**生产区**。排除 tests/ 目录下的整个文件、
      以及每个文件里 ``#[cfg(test)]`` 块以外的一切 ⇒ 「生产消费者」判据。
    · ``tseen`` / ``tcalled``：**测试区** = ``#[cfg(test)]`` 块内的内容
      ∪ tests/ 目录下的文件内容 ⇒ 只用来区分 TEST_ONLY，不参与「已接线」判定。

    ⭐ 为什么必须分两张表：TEST_ONLY 这一类**要求**能看到测试里的引用；
    若只索引生产区，`test_hits` 恒为空 ⇒ TEST_ONLY 永远不会出现，
    「被测试引用但无生产消费者」会被错报成 ZERO_CONSUMER。
    """
    seen, called, tseen, tcalled = {}, {}, {}, {}
    keys = list(files)
    for idx, key in enumerate(keys):
        fi = files[key]
        if in_any_test_dir(fi.path):
            _scan(fi.masked, tseen, tcalled, idx)
            continue
        _scan(fi.prod, seen, called, idx)
        _scan(fi.test, tseen, tcalled, idx)
    return keys, seen, called, tseen, tcalled


def has_use_only(files, keys, seen, name, self_idx):
    """该资产的全部**生产**消费者出现是否都落在 ``use`` / ``pub use`` 语句里。

    ⇒ 「只导出没调用」（呼应 DIR-REMEDY §2.5「导出 ≠ 调用」）。
    """
    hits = {i for i in (seen.get(name) or set()) if i != self_idx}
    if not hits:
        return False
    rx = re.compile(r"\b" + re.escape(name) + r"\b")
    for idx in hits:
        prod = files[keys[idx]].prod
        for m in rx.finditer(prod):
            line_start = prod.rfind("\n", 0, m.start()) + 1
            prefix = prod[line_start:m.start()]
            if not re.search(r"\buse\b[^;]*$", prefix):
                return False        # 有一处不是 use ⇒ 是真调用
    return True


# ══════════════════════════════════════════════════════════════════
# 4. 语义分类（requirement 4：枚举之后才分类）
# ══════════════════════════════════════════════════════════════════

POLICY_WORDS = re.compile(
    r"secur|shield|guard|sandbox|jail|isolat|confine|egress|firewall|"
    r"blocklist|denylist|allowlist|whitelist|permission|privilege|"
    r"authoriz|authentic|admit|grant|revoke|rbac|\bacl\b|access|"
    r"polic|gate|gatekeeper|guardrail|checkpoint|admission|regulat|"
    r"constraint|invariant|veto|breaker|throttl|ratelimit|budget|cap\b|"
    r"filter|sanit|screen|scrub|redact|mask_secret|"
    r"valid|verif|audit|approv|attest|compliance|conform|proof|"
    r"credential|token|secret|keyring|"
    r"demote|promote|quarantine|escalat|rollback|heal|repair|reconcile|"
    r"demotion|promotion|verdict|judge|enforc",
    re.IGNORECASE,
)

def _name_hit(text):
    """治理词表命中。

    ⛔ 词表里**禁止出现空的分隔分支**（末尾多写一个 `|`）——
       `...|enforc|` 会编译成含空分支的正则，`search` 在任何位置都返回
       零宽匹配 ⇒ 谓词恒真 ⇒ 全仓每个资产都被判成治理类。
       本机实测症状：`_name_hit("tile")` 返回 True。
       ⇒ 用 `search` 之后**必须**断言 `m.group(0)` 非空
       （见 `--self-test` 的谓词语义断言）。
    """
    if not text:
        return False
    return POLICY_WORDS.search(text) is not None


IMPL_HEAD = re.compile(r"\bimpl\b[^;{]*?\b([A-Za-z_][A-Za-z0-9_]*)\b")


def own_method_names(masked, type_name):
    """切出该类型**自己**的 ``impl`` 块，收集其中定义的 ``fn`` 名。"""
    names = set()
    for m in IMPL_HEAD.finditer(masked):
        if m.group(1) != type_name:
            continue
        j = masked.find("{", m.end())
        if j < 0:
            continue
        end = _match_brace(masked, j)
        if end < 0:
            continue
        for fm in FN_DEF.finditer(masked, j, end):
            names.add(fm.group(1))
    return names


#: 拆分出来的弱信号类别名（基线里作为 `KIND` 前缀，与 ZERO_CONSUMER 平级）。
PATH_ONLY = "PATH_ONLY"

#: 「唯一判据是所在目录名」时 `classify()` 写进 `why` 的那个标记。
WEAK_PATH_WHY = "path"


def classify(fi, kind, name, masked):
    """四路信号取或：名字 / 文件名 / 目录名 / 自身 impl 方法名。

    返回 ``(是否治理类, 命中理由, 是否仅目录名)``。

    ⛔ **不要用 `why == "path"` 反推「是否仅目录名」**：`why` 是
    `"+".join(reasons[:3])`，被截断到 3 段。虽然 path 在 reasons 里排第 3
    （下标 2）恰好不会被截掉、当前字符串比较是对的，但那是**巧合**：任何人
    往 reasons 里插一路信号就会静默改变语义。⇒ 显式返回第三项。
    """
    reasons = []
    if _name_hit(name):
        reasons.append("name")
    stem = os.path.splitext(os.path.basename(fi.path))[0]
    if _name_hit(stem):
        reasons.append("file")
    d = os.path.dirname(fi.path)
    if _name_hit(d.replace(os.sep, "_")):
        reasons.append("path")
    if kind in ("struct", "enum", "trait", "union"):
        methods = own_method_names(masked, name)
        for mn in sorted(methods):
            if _name_hit(mn):
                reasons.append("method:" + mn)
                break
    if not reasons:
        return False, "", False
    return True, "+".join(reasons[:3]), reasons == [WEAK_PATH_WHY]


# ══════════════════════════════════════════════════════════════════
# 5. ORPHAN_MODULE
# ══════════════════════════════════════════════════════════════════

def find_orphan_modules(files, base):
    """目录里有 .rs、看起来像模块，但父目录下没有任何 .rs 声明它。"""
    dirs = {}
    for rel in files:
        d = os.path.dirname(rel)
        dirs.setdefault(d, []).append(rel)

    orphans = []
    for d in sorted(dirs):
        base_name = os.path.basename(d)
        if not base_name or base_name in SKIP_BASENAMES:
            continue
        members = dirs[d]
        rs_files = [f for f in members if f.endswith(".rs")]
        if not rs_files:
            continue
        looks_like_mod = ("mod.rs" in rs_files) or (len(rs_files) >= 2)
        if not looks_like_mod:
            continue
        parent = os.path.dirname(d)
        hosts = []
        if parent:
            try:
                for f in sorted(os.listdir(os.path.join(base, parent))):
                    if f.endswith(".rs"):
                        hosts.append(os.path.join(parent, f))
            except OSError:
                pass
            # ⭐ Rust 的 `foo.rs` + `foo/` 约定：模块文件 `foo.rs` 与同名目录
            #    `foo/` **平级**，且子模块声明写在 `foo.rs` 里，不在 `foo/` 里。
            #    只扫 `dirname(D)` 会漏掉这个宿主 ⇒ 把活目录误判成孤儿。
            #    实测：`neotrix-core/src/bin/experience.rs` 声明 `mod exp_absorb;`
            #    等，而 `bin/experience/*.rs` 就在同级的 `bin/` 下；
            #    `l1_action/nt_file_ability.rs` 声明 `pub mod pdf; pub mod excel;`
            #    等，而 `nt_file_ability/{pdf,excel,merge,visual}/` 也在 `l1_action/` 下。
            grand = os.path.dirname(parent)
            if grand:
                # 宿主候选：`foo.rs` + `foo/` 约定下，目录 P 的宿主是
                # `<P 的父>/<P 的名字>.rs`
                owner = os.path.join(grand, os.path.basename(parent) + ".rs")
                if os.path.exists(os.path.join(base, owner)):
                    hosts.append(owner)
        decl_prod, decl_test, by_path = False, False, False
        # `#[path]` 里的目录部分（相对宿主所在目录）
        dir_prefixes = {d, os.path.basename(d)}
        for h in hosts:
            key = h.replace(os.sep, "/")
            if key not in files:
                continue
            fi = files[key]
            for rx in (MOD_DECL, MOD_DECL_PLAIN):
                if any(mm.group(1) == base_name for mm in rx.finditer(fi.prod)):
                    decl_prod = True
            for rx in (MOD_DECL, MOD_DECL_PLAIN):
                if any(mm.group(1) == base_name for mm in rx.finditer(fi.test)):
                    decl_test = True
            # `#[path = "experience/exp_util.rs"]` ⇒ 目录 experience 已接线
            # （模块名 exp_util 与目录名无关，故必须单看 path 属性）
            # ⚠️ 属性值在**字符串**里，所以要在 pathattrs（保留字符串的掩码）
            #    上匹配，不能用 prod/test（全掩码）。
            # ⛔ pathattrs 不区分 cfg(test)：属性值在字符串里，剥注释后的
            #    文本已无法可靠区分生产/测试区。取**保守**方向：见到
            #    #[path] 指向该目录就算已接线（宁可漏报 ORPHAN_MODULE，
            #    不可误报一个活目录）。
            for pm in PATH_ATTR.finditer(fi.pathattrs):
                tgt = pm.group(1).replace("\\", "/")
                if any(tgt == p or tgt.startswith(p.rstrip("/") + "/")
                       for p in dir_prefixes):
                    by_path = True
                    break
        if decl_prod or by_path:
            continue
        loc = sum(files[m].nlines for m in rs_files)
        orphans.append({
            "dir": d,
            "files": len(rs_files),
            "loc": loc,
            "test_only_decl": decl_test,
        })
    return orphans


# ══════════════════════════════════════════════════════════════════
# 6. 主分析
# ══════════════════════════════════════════════════════════════════

def analyze(roots, base):
    files = collect_files(roots, base)
    keys, seen, called, tseen, tcalled = index_consumers(files)
    idx_of = {k: i for i, k in enumerate(keys)}

    orphans = find_orphan_modules(files, base)
    orphan_dirs = {o["dir"] for o in orphans}

    findings = []          # dict: kind / loc / why
    wired = 0
    weak_only = 0
    policy_total = 0
    other_total = 0
    too_small = 0

    for key in sorted(files):
        fi = files[key]
        d = os.path.dirname(key)
        if d in orphan_dirs:
            continue                      # 整个目录都从未编译，资产层面不重复报
        if fi.nlines < MIN_DEF_LINES:
            too_small += 1
            continue
        self_idx = idx_of[key]
        seen_here = set()
        for m in PUB_ITEM.finditer(fi.prod):
            kind, name = m.group(1), m.group(2)
            if (kind, name) in seen_here:
                continue
            seen_here.add((kind, name))
            is_policy, why, weak_path = classify(fi, kind, name, fi.masked)
            if not is_policy:
                other_total += 1
                continue
            policy_total += 1
            line = fi.prod.count("\n", 0, m.start()) + 1

            # 「有生产消费者」：生产区出现（fn 还要求紧跟左括号）
            pool = called if kind == "fn" else seen
            prod_hits = {i for i in (pool.get(name) or set()) if i != self_idx}
            if prod_hits:
                wired += 1
                if has_use_only(files, keys, seen, name, self_idx):
                    weak_only += 1
                continue

            # 无生产消费者 ⇒ 看测试区有没有引用（区分 TEST_ONLY / ZERO_CONSUMER）
            tpool = tcalled if kind == "fn" else tseen
            test_hits = {i for i in (tpool.get(name) or set()) if i != self_idx}
            if test_hits:
                findings.append({
                    "kind": "TEST_ONLY",
                    "loc": f"{key}::{name}",
                    "line": line,
                    "why": why,
                    "weak": weak_path,
                })
            else:
                # ⭐ 仅目录名判据 ⇒ 归入 PATH_ONLY，不与真需要人工定性的
                #    ZERO_CONSUMER 混在一起（见模块 docstring 的实测表）。
                findings.append({
                    "kind": PATH_ONLY if weak_path else "ZERO_CONSUMER",
                    "loc": f"{key}::{name}",
                    "line": line,
                    "why": why,
                    # `weak` 供报告层披露「其它类别里同样的弱信号残余」
                    # （实测 TEST_ONLY 也有 53 条仅目录名判据）。
                    "weak": weak_path,
                })

    for o in orphans:
        findings.append({
            "kind": "ORPHAN_MODULE",
            "loc": o["dir"],
            "line": o["files"],
            "why": ("declared-only-under-cfg(test) " if o["test_only_decl"] else "")
                   + f"{o['files']} files / {o['loc']} LOC never compiled",
            "weak": False,
        })

    order = {"ORPHAN_MODULE": 0, "ZERO_CONSUMER": 1, PATH_ONLY: 2, "TEST_ONLY": 3}
    findings.sort(key=lambda f: (order[f["kind"]], f["loc"]))
    return {
        "findings": findings,
        "wired": wired,
        "weak_only": weak_only,
        "policy_total": policy_total,
        "other_total": other_total,
        "too_small": too_small,
        "nfiles": len(files),
        "orphan_dirs": {o["dir"] for o in orphans},
    }


# ══════════════════════════════════════════════════════════════════
# 7. 基线 I/O
# ══════════════════════════════════════════════════════════════════

def read_baseline(path):
    """读基线：忽略空行与 `#` 理由注释；返回 ``{kind: set(loc)}`` 与理由行列表。

    ⚠️ **本函数刻意不认识 `PATH_ONLY`** —— 它按 `KIND <loc>` 逐字读，
    新的 KIND 前缀天然被接受（无需改动即可读回拆分后的基线），
    而拆分前的旧基线里那 693 条仍以 `ZERO_CONSUMER` 存在。
    那部分由 `fold_split_baseline()` 在**比较阶段**补齐，避免动到这里的
    「读到什么就是什么」语义。
    """
    known, reasons = set(), []
    if not os.path.exists(path):
        return known, reasons
    with open(path, encoding="utf-8") as fh:
        for ln in fh:
            s = ln.rstrip("\n")
            if s.lstrip().startswith("#"):
                reasons.append(s.rstrip())
                continue
            s = s.strip()
            if not s:
                continue
            # ⛔ 键必须与 `baseline_keys()` 逐字一致：`KIND <loc>`，
            #    且要**剥掉条目行尾的 `# L123 (reason)` 注释**。
            #    漏了这一步 ⇒ 读回来的键与算出来的键对不上 ⇒ 每条都判成
            #    "新增" ⇒ 基线永远绿不了。（沿用 truth-surface 的读法。）
            head = s.split("  #", 1)[0].strip()
            head = re.sub(r"\s+#.*$", "", head).strip()
            if head:
                known.add(head)
    return known, reasons


def baseline_keys(findings):
    return {f"{f['kind']} {f['loc']}" for f in findings}


#: 拆分前后的 KIND 对：(旧 KIND, 新 KIND)。
SPLIT_PAIRS = (("ZERO_CONSUMER", PATH_ONLY),)


def fold_split_baseline(known, cur):
    """让拆分前的旧基线继续有效；返回 ``(有效键集, 被折叠的条数)``。

    ⭐ 为什么要这一层：把 `PATH_ONLY` 从 `ZERO_CONSUMER` 里拆出来，会让
    旧基线里那 693 条 `ZERO_CONSUMER <loc>` 与新算出的 `PATH_ONLY <loc>`
    **键对不上** ⇒ `--strict` 会一次性报 693 条「新增」+ 693 条「已消失」。
    那不是新增技术债，只是**重新定性**，不该让门红。

    ⛔ 折叠是**有条件的**：只有当旧基线里**逐字存在** `ZERO_CONSUMER <loc>`
    这一行时，才认为它覆盖了 `PATH_ONLY <loc>`。⇒ 基线已经重算成新格式后，
    本函数是**空操作**；且永远不会掩盖真正的新增条目（新条目在旧基线里
    不存在对应行）。返回的计数用于在报告里显式披露这次重新定性。
    """
    eff, folded = set(known), 0
    for old_kind, new_kind in SPLIT_PAIRS:
        for k in cur:
            if not k.startswith(new_kind + " "):
                continue
            loc = k[len(new_kind) + 1:]
            if k in eff:
                continue
            if f"{old_kind} {loc}" in known:
                eff.add(k)
                folded += 1
    return eff, folded


def entry_lines(f):
    if f["kind"] == "ORPHAN_MODULE":
        return f"ORPHAN_MODULE {f['loc']}"
    return f"{f['kind']} {f['loc']}  # L{f['line']} ({f['why']})"


# ══════════════════════════════════════════════════════════════════
# 8. 报告
# ══════════════════════════════════════════════════════════════════

TRUTH_KNOWN = None


def load_truth_baseline():
    global TRUTH_KNOWN
    if TRUTH_KNOWN is not None:
        return TRUTH_KNOWN
    s = set()
    if os.path.exists(TRUTH_BASELINE):
        with open(TRUTH_BASELINE, encoding="utf-8") as fh:
            for ln in fh:
                s.add(ln.split(" ", 1)[1].strip()
                      if ln.startswith(("UNDECLARED ", "UNREACHABLE ")) else ln.strip())
    TRUTH_KNOWN = s
    return s


def report(res, all_items):
    counts = {}
    weak_by_kind = {}
    for f in res["findings"]:
        counts[f["kind"]] = counts.get(f["kind"], 0) + 1
        if f.get("weak"):
            weak_by_kind[f["kind"]] = weak_by_kind.get(f["kind"], 0) + 1
    n_zc = counts.get("ZERO_CONSUMER", 0)
    n_po = counts.get(PATH_ONLY, 0)
    n_to = counts.get("TEST_ONLY", 0)
    weak_to = weak_by_kind.get("TEST_ONLY", 0)
    print("security-wiring audit — 「造了资产但没人调用」")
    print(f"  扫描 .rs: {res['nfiles']} · 定义文件 ≥{MIN_DEF_LINES} 行的文件: "
          f"{res['nfiles'] - res['too_small']}（<{MIN_DEF_LINES} 行跳过 {res['too_small']}）")
    print(f"  治理类资产（枚举后按语义分类）: {res['policy_total']}"
          f" · WIRED_OK: {res['wired']}（其中弱消费者/仅 use 提到: {res['weak_only']}）")
    print(f"  ORPHAN_MODULE: {counts.get('ORPHAN_MODULE', 0)}"
          f" · ZERO_CONSUMER: {n_zc} · {PATH_ONLY}: {n_po} · TEST_ONLY: {n_to}")
    print(f"  ⇦ 需人工判定: ZERO_CONSUMER {n_zc} 条（判据含 name/file/方法名）"
          f" · {PATH_ONLY} {n_po} 条（判据**仅**目录名 = 弱信号，见判定口径）")
    if weak_to:
        print(f"  ⓘ 残余弱信号: TEST_ONLY 里有 {weak_to}/{n_to} 条同样仅目录名判据"
              f"（本门未拆该类，勿把它们当强信号）")
    if all_items:
        print(f"  ⓘ OTHER（非治理类，按词表漏过）: {res['other_total']}")

    truth = load_truth_baseline()
    for kind in ("ORPHAN_MODULE", "ZERO_CONSUMER", PATH_ONLY, "TEST_ONLY"):
        group = [f for f in res["findings"] if f["kind"] == kind]
        if not group:
            continue
        print(f"\n── {kind} ({len(group)}) ──")
        for f in group:
            tag = ""
            if kind == "ORPHAN_MODULE":
                rel = f["loc"].replace(os.sep, "/") + "/"
                if rel in truth or any(t.startswith(rel) for t in truth):
                    tag = "   [=truth-surface 基线内]"
            print(f"  {entry_lines(f)}{tag}")
    print("\n判定口径：")
    print("  · 枚举 = 全量 pub 项（struct/enum/trait/union/fn/mod/type），"
          "不做路径或名字预筛；治理类判定在枚举之后，四路信号（资产名 / "
          "文件名 stem / 所在目录名 / 自身 impl 方法名）取或。")
    print(f"  · {PATH_ONLY} 与 ZERO_CONSUMER 的**唯一差别是判据强度**："
          f"{PATH_ONLY} = name / 文件名 / 方法名三路全部落空，"
          "治理判据**只剩「所在目录名含治理词」**这一条。")
    print(f"    ⇒ {PATH_ONLY} 那 {n_po} 条**不等于**资产有问题：一个目录名会把"
          "该目录下所有 pub 项都算成安全资产，")
    print("      逐条无行动价值。它是**分类噪音**，需要看的是那 "
          f"{n_zc} 条含 name/file/方法名判据的。")
    print("  · 消费者 = 其它文件**生产区**（花括号配对切掉 #[cfg(test)]）里的标识符出现，"
          "含 use 导入；判前已用词法扫描把注释与字符串/字符字面量掩成等长空格。")
    print("  · tests/ 目录与 #[cfg(test)] 块都不算生产消费者。")
    print("  · WIRED_OK 只计数不逐个列出（避免噪音）。")
    print("  · ⚠️ 弱信号的另一面：对 pub fn / pub mod / pub type 三类 item，"
          "方法名那一路**根本不执行**")
    print("    ⇒ 连潜在的语义信号都没被看过，这也是 path-only 判据偏弱的第二个原因。")


# ══════════════════════════════════════════════════════════════════
# 9. main
# ══════════════════════════════════════════════════════════════════

def main(argv):
    base_path = None
    for i, a in enumerate(argv):
        if a == "--baseline":
            nxt = argv[i + 1] if i + 1 < len(argv) else None
            if nxt and not nxt.startswith("-"):
                base_path = nxt
            break
    if "--baseline" in argv or "--update-baseline" in argv:
        target = base_path or BASELINE
        res = analyze(ROOTS, REPO)
        _, reasons = read_baseline(target)
        os.makedirs(os.path.dirname(os.path.abspath(target)) or ".", exist_ok=True)
        marker = "# generated by scripts/ops/nt_security_wiring.py --baseline"
        with open(target, "w", encoding="utf-8") as fh:
            # ⛔ 既有 `#` 理由行会被原样保留（沿用 truth-surface 的做法）。
            #    但 marker 本身也要保留 —— 原来这里无条件再写一遍 marker，
            #    于是**每重算一次就多一行重复表头**（实测 2026-10-06：已积累 2 行）。
            if reasons:
                fh.write("\n".join(r for r in reasons if r.strip() != marker) + "\n")
            fh.write(marker + "\n")
            for f in res["findings"]:
                fh.write(entry_lines(f) + "\n")
        counts = {}
        weak = 0
        for f in res["findings"]:
            counts[f["kind"]] = counts.get(f["kind"], 0) + 1
            if f.get("weak"):
                weak += 1
        print(f"基线已写: {target}")
        print(f"  ORPHAN_MODULE {counts.get('ORPHAN_MODULE', 0)}"
              f" · ZERO_CONSUMER {counts.get('ZERO_CONSUMER', 0)}"
              f" · {PATH_ONLY} {counts.get(PATH_ONLY, 0)}"
              f" · TEST_ONLY {counts.get('TEST_ONLY', 0)}"
              f" · WIRED_OK {res['wired']}（只计数，未入基线）")
        print(f"  ⓘ 弱信号（判据仅目录名）{weak} 条，其中 {counts.get(PATH_ONLY, 0)} 条已单列为 "
              f"{PATH_ONLY}，其余混在 TEST_ONLY 里。")
        print("  ⛔ 基线把上面这些存量「合法化」。每一条都必须有理由；")
        print("     重算时既有 # 理由行会被保留（沿用 truth-surface 的做法）。")
        return 0

    res = analyze(ROOTS, REPO)
    report(res, "--all-items" in argv)

    if "--audit" in argv:
        print("\n--audit: 只打印，不判失败。")
        return 0

    known, _ = read_baseline(BASELINE)
    if not os.path.exists(BASELINE):
        print("\nⓘ 基线文件不存在（首次运行）。")
        print("   建基线是需要判断的动作 —— 它会把当前存量合法化。")
        print("   确认逐条定性后，用 --baseline 生成；理由写进文件的 # 行。")
        if "--strict" in argv:
            print("\nFAIL: --strict 需要先建立基线（scripts/security-wiring-baseline.txt）。")
            return 1
        return 0

    cur = baseline_keys(res["findings"])
    known_eff, folded = fold_split_baseline(known, cur)
    fresh = sorted(cur - known_eff)
    stale = sorted(known - cur)
    print(f"\n基线: {len(known)} 条 · 当前: {len(cur)} 条 · 新增: {len(fresh)}")
    if folded:
        print(f"ℹ️ {folded} 条由旧基线的 ZERO_CONSUMER 重新定性为 {PATH_ONLY}"
              "（仅目录名判据）—— 不计入新增")
    if stale:
        print(f"ℹ️ 基线中 {len(stale)} 条已消失（不判红，可清理）")

    if fresh:
        n_po_fresh = sum(1 for k in fresh if k.startswith(PATH_ONLY + " "))
        print(f"\nFAIL: 新增 {len(fresh)} 条发现"
              + (f"（其中 {PATH_ONLY} {n_po_fresh} 条为弱信号判据）" if n_po_fresh else "")
              + "：", file=sys.stderr)
        for k in fresh:
            print("  " + k, file=sys.stderr)
        print(
            "\n⛔ 这**不等于**死代码。逐条定性后再决定：\n"
            "  · ORPHAN_MODULE  ⇒ 文件从未编译。先问「补 pub mod 后能编译吗」\n"
            "    （truth-surface 基线记 nt_memory/mod.rs 的 hybrid_retrieval 是\n"
            "     「声明被注释掉 + 编译不过」，补声明会让干净检出红）。\n"
            f"  · ZERO_CONSUMER ⇒ 接上消费者，或写明正当理由进基线。\n"
            f"  · {PATH_ONLY} ⇒ **先别改代码**。判据只有目录名，多半是分类噪音；\n"
            "    真要接线请先确认这个 pub 项真的是安全资产（静态匹配判不出该不该接线）。\n"
            "  · 有正当理由 ⇒ 加进 scripts/security-wiring-baseline.txt 并注明理由\n"
            "  · 定性不了 ⇒ 留红，别加白名单\n"
            "  · 本门度量「有没有生产消费者」；**静态匹配无法判定该不该接线**。",
            file=sys.stderr,
        )
        return 1

    print("PASS: 无新增「造了资产但没人调用」的发现")
    return 0


# ══════════════════════════════════════════════════════════════════
# 10. --self-test：注入式负向测试（夹具全在系统临时目录，不落仓库）
# ══════════════════════════════════════════════════════════════════

#: 夹具填充（必须让夹具文件 ≥ MIN_DEF_LINES 行）
_PAD = "\n".join("// filler line %d" % k for k in range(50))


def _asset_body(name):
    return (
        "use std::collections::BTreeMap;\n"
        f"/// fixture asset {name}\n"
        f"pub struct {name} {{\n"
        "    pub registry: BTreeMap<String, u64>,\n"
        "    pub budget: u64,\n"
        "    pub revoked: Vec<String>,\n"
        "}\n"
        f"impl {name} {{\n"
        "    pub fn new() -> Self {\n"
        "        Self { registry: BTreeMap::new(), budget: 0, revoked: Vec::new() }\n"
        "    }\n"
        "    pub fn authorize(&self, key: &str) -> bool {\n"
        "        self.registry.contains_key(key) && !self.revoked.iter().any(|r| r == key)\n"
        "    }\n"
        "    pub fn revoke(&mut self, key: &str) {\n"
        "        self.revoked.push(key.to_string());\n"
        "    }\n"
        "}\n" + _PAD + "\n"
    )


def self_test():
    tmp = tempfile.mkdtemp(prefix="nt-secwire-selftest-")
    ok = True

    def say(good, msg):
        nonlocal ok
        print(("SELFTEST OK: " if good else "SELFTEST FAIL: ") + msg)
        if not good:
            ok = False

    try:
        def w(rel, body):
            path = os.path.join(tmp, rel)
            os.makedirs(os.path.dirname(path) or tmp, exist_ok=True)
            with open(path, "w", encoding="utf-8") as fh:
                fh.write(body)
            return path

        # ---- 夹具骨架：lib.rs 声明全部被声明的模块（orphan_dir 故意不声明）----
        w("lib.rs", (
            "pub mod zc;\n"
            "pub mod lit;\n"
            "pub mod lit_consumer;\n"
            "pub mod late;\n"
            "pub mod callafter;\n"
            "pub mod testonly;\n"
            "pub mod to_consumer;\n"
            "pub mod wired_ok;\n"
            "pub mod wired_ok_consumer;\n"
        ))
        # ⚠️ (g)/(h) 的夹具目录必须**放在 `src/` 之下**：夹具根是 tmp 本身，
        #    根级目录的 `parent` 为空 ⇒ `find_orphan_modules()` 的宿主扫描
        #    整段被跳过 ⇒ 该目录必被判成 ORPHAN_MODULE，而 `analyze()` 会
        #    `continue` 跳过孤儿目录里的**全部**文件 ⇒ 分类根本不会执行。
        #    （真实仓库不受影响：ROOTS 是 `neotrix-core/src` 等，relpath 恒有
        #    ≥2 段，`parent` 不会为空。）`src` 在 SKIP_BASENAMES 里，
        #    且 `src/mod.rs` 显式声明两个子模块 ⇒ 不产生孤儿。
        w("src/mod.rs", "pub mod nt_shield;\npub mod misc;\n" + _PAD + "\n")
        # (a) ORPHAN_MODULE：有 mod.rs + 2 个 .rs，但 lib.rs 不声明 orphan_dir
        w("orphan_dir/mod.rs", "pub mod guard_part;\n" + _PAD + "\n")
        w("orphan_dir/guard_part.rs",
          "/// destructive-command detector fixture\n"
          "pub fn detect_rm_rf(cmd: &str) -> bool { cmd.contains(\"rm -rf\") }\n" + _PAD + "\n")
        # (b) ZERO_CONSUMER：已 mod 声明、确实编译，但无任何消费者
        w("zc.rs", _asset_body("ZeroConsumerPolicy"))
        # (c) 只有注释/字符串字面量提到 LitPolicy => 不得被当成消费者
        w("lit.rs", _asset_body("LitPolicy"))
        w("lit_consumer.rs", (
            "/// doc mentions LitPolicy only\n"
            "// LitPolicy again\n"
            "/* block LitPolicy */\n"
            "pub fn describe() -> String {\n"
            '    let sample = r#"Command::new("sh").args(["-c", &cmd])"#;\n'
            '    let plain = "LitPolicy";\n'
            '    format!("{} {}", sample, plain)\n'
            "}\n" + _PAD + "\n"
        ))
        # (d) 测试块在文件最前面，生产代码在后面 => 生产代码不得隐形
        w("late.rs", _asset_body("LateAsset"))
        w("callafter.rs", (
            "#[cfg(test)]\n"
            "mod tests {\n"
            "    #[test]\n"
            "    fn t() { let _ = 1; }\n"
            "}\n"
            "use crate::late::LateAsset;\n"
            "/// production code that lives AFTER the cfg(test) block\n"
            "pub fn call_late(a: &LateAsset) -> u64 { a.budget }\n" + _PAD + "\n"
        ))
        # (e) TEST_ONLY：只被 #[cfg(test)] 引用
        w("testonly.rs", _asset_body("TestOnlyPolicy"))
        w("to_consumer.rs", (
            "#[cfg(test)]\n"
            "mod tests {\n"
            "    use crate::testonly::TestOnlyPolicy;\n"
            "    #[test]\n"
            '    fn t() { let p = TestOnlyPolicy::new(); assert!(!p.authorize("k")); }\n'
            "}\n" + _PAD + "\n"
        ))
        # (f) WIRED_OK 对照组
        w("wired_ok.rs", _asset_body("WiredOkGate"))
        w("wired_ok_consumer.rs",
          "use crate::wired_ok::WiredOkGate;\n"
          'pub fn check(g: &WiredOkGate) -> bool { g.authorize("k") }\n' + _PAD + "\n")
        # (g) PATH_ONLY：资产名 / 文件名 / 方法名全部治理词落空，
        #     **只有目录名** nt_shield 命中 ⇒ 必须归 PATH_ONLY，不得混进 ZERO_CONSUMER。
        w("src/nt_shield/mod.rs", "pub mod netlink;\n" + _PAD + "\n")
        w("src/nt_shield/netlink.rs", (
            "/// neutral name, neutral stem, neutral method names\n"
            "pub struct Relay {\n"
            "    pub hops: u32,\n"
            "    pub window: u32,\n"
            "}\n"
            "impl Relay {\n"
            "    pub fn new(hops: u32) -> Self { Self { hops, window: 4 } }\n"
            "    pub fn step(&mut self) { self.hops += 1; }\n"
            "    pub fn flush(&mut self) { self.window = 4; }\n"
            "}\n" + _PAD + "\n"
        ))
        # (h) 方法判据：资产名 / 文件名 / 目录名全落空，只有方法名命中
        #     ⇒ 必须**留在** ZERO_CONSUMER（不能被 path-only 规则误吞）。
        w("src/misc/mod.rs", "pub mod telemetry;\n" + _PAD + "\n")
        w("src/misc/telemetry.rs", (
            "/// neutral name, neutral stem, neutral dir => only the method matches\n"
            "pub struct Probe {\n"
            "    pub id: u64,\n"
            "    pub seen: u32,\n"
            "}\n"
            "impl Probe {\n"
            "    pub fn new(id: u64) -> Self { Self { id, seen: 0 } }\n"
            "    pub fn verify_payload(&mut self) -> bool { self.seen += 1; self.seen > 0 }\n"
            "}\n" + _PAD + "\n"
        ))

        res = analyze(["."], tmp)
        kinds = {}
        for f in res["findings"]:
            kinds.setdefault(f["kind"], set()).add(f["loc"])

        # (a) ORPHAN_MODULE
        orph = kinds.get("ORPHAN_MODULE", set())
        say("orphan_dir" in orph,
            f"(a) ORPHAN_MODULE 夹具被抓到 —— orphan_dir 在发现集内（发现集: {sorted(orph)}）")
        say(not any(o.startswith("zc") for o in orph),
            "(a2) 已声明的 zc 未被误报成孤儿模块")

        # (b) ZERO_CONSUMER
        zc = kinds.get("ZERO_CONSUMER", set())
        say("zc.rs::ZeroConsumerPolicy" in zc,
            "(b) ZERO_CONSUMER 夹具被抓到")

        # (c) 字面量/注释不算消费者
        lit = kinds.get("ZERO_CONSUMER", set()) | kinds.get("TEST_ONLY", set())
        say("lit.rs::LitPolicy" in lit,
            "(c) 只有字符串字面量/注释提到 LitPolicy => 未被当成消费者（判为零消费者）")

        # (d) cfg(test) 在文件最前 => 其后的生产代码不得隐形
        late = kinds.get("ZERO_CONSUMER", set()) | kinds.get("TEST_ONLY", set())
        say("late.rs::LateAsset" not in late,
            "(d) cfg(test) 在文件最前面的夹具：LateAsset 未被误判成死代码"
            "（花括号配对让 callafter.rs 的生产 fn 可见）")

        # (e) TEST_ONLY
        say("testonly.rs::TestOnlyPolicy" in kinds.get("TEST_ONLY", set()),
            "(e) TEST_ONLY 夹具被抓到（与 ZERO_CONSUMER 分列）")

        # WIRED_OK 对照组必须不被报
        allbad = kinds.get("ZERO_CONSUMER", set()) | kinds.get("TEST_ONLY", set())
        say("wired_ok.rs::WiredOkGate" not in allbad,
            "(f) WIRED_OK 对照组未被误报")

        # (g) PATH_ONLY：仅目录名判据 ⇒ 归 PATH_ONLY，且不得留在 ZERO_CONSUMER
        po = kinds.get(PATH_ONLY, set())
        zc = kinds.get("ZERO_CONSUMER", set())
        say("src/nt_shield/netlink.rs::Relay" in po,
            f"(g) 仅目录名判据的资产被归入 {PATH_ONLY}（发现集: {sorted(po)}）")
        say("src/nt_shield/netlink.rs::Relay" not in zc,
            f"(g2) 仅目录名判据的资产**没有**混进 ZERO_CONSUMER"
            f"（ZC 实测: {sorted(zc)}）")
        say(all(f["why"] == "path" for f in res["findings"] if f["kind"] == PATH_ONLY),
            "(g3) 不变式: kind == PATH_ONLY ⇔ 判据恰好是 (path)"
            "（防止有人手改 classify 而两边脱钩）")

        # (h) 方法判据 ⇒ 必须留在 ZERO_CONSUMER
        say("src/misc/telemetry.rs::Probe" in zc,
            "(h) 方法名判据(method:verify_payload)的资产仍归入 ZERO_CONSUMER")
        say("src/misc/telemetry.rs::Probe" not in po,
            "(h2) 方法名判据的资产未被误降级到 PATH_ONLY")
        say("zc.rs::ZeroConsumerPolicy" in zc and "zc.rs::ZeroConsumerPolicy" not in po,
            "(h3) 资产名判据(name)的资产仍归入 ZERO_CONSUMER")

        # 缺陷 1 的直接断言：朴素 split 会让 (d) 的消费者隐形
        naive = open(os.path.join(tmp, "callafter.rs"), encoding="utf-8").read()
        if "LateAsset" in naive.split("#[cfg(test)]")[0]:
            say(False, "(d2) 朴素 split 基线本身异常，夹具未复现原缺陷")
        else:
            say("LateAsset" not in naive.split("#[cfg(test)]")[0],
                "(d2) 朴素 split 对该夹具**会**漏掉消费者（复现原缺陷）")

        # 缺陷 2 的直接断言：朴素 grep 会把字面量当消费者
        raw = open(os.path.join(tmp, "lit_consumer.rs"), encoding="utf-8").read()
        say("LitPolicy" in raw,
            "(c2) 朴素 grep 对该夹具**会**把字面量当消费者（复现原缺陷）")
        say(not re.search(r"\bLitPolicy\b",
                          mask_noncode(raw).split("\"\"\"")[0]),
            "(c3) 掩码后字面量/注释里的 LitPolicy 已消失")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    # ⭐ 掩码器单元断言：(源, 必须消失的标识符, 必须保留的标识符)
    # ⛔ 不断言「标识符全集相等」—— 关键字（let/fn）也是标识符，那样写断言
    #    既脆弱又与本门用途无关。本门只关心「字面量/注释里的词消失、代码里的词留下」。
    cases = [
        ('let s = r#"rm -rf Cmd::new"#; let x = 1;',
         ["rm", "rf", "Cmd", "new"], ["let", "s", "x"]),
        ("/* outer /* inner */ still */ let x = 1;",
         ["outer", "inner", "still"], ["let", "x"]),
        ("let c = '\\u{1F600}'; let d = 'z';",
         ["1F600"], ["let", "c", "d"]),
        ("// commented_verified_token\nfn f<'a>(x: &'a u8) -> u8 { *x }",
         ["commented_verified_token"], ["fn", "f", "x", "u8"]),
        ('let e = br##"raw ## bytes"##; let g = 2;',
         ["raw", "bytes"], ["let", "e", "g"]),
        ('let msg = "line1\\nline2"; let h = 3;',
         ["line1", "line2"], ["let", "msg", "h"]),
    ]
    # ⭐ 谓词语义断言：治理词表必须「有命中 / 无命中」两头都对。
    # ⛔ 这条是被真 bug 逼出来的：词表末尾多一个 `|` ⇒ 产生空分支 ⇒
    #    `search` 在任意位置零宽匹配 ⇒ `_name_hit` 恒真 ⇒ 全仓每个 pub 项
    #    都被当成治理类（实测 `_name_hit("tile") == True`）。
    #    ⇒ 谓词型常量必须被断言，不能只靠「跑起来没崩」。
    for word, want in [("tile", False), ("benchmark", False), ("calls", False),
                       ("with_transparent", False), ("world_to_local", False),
                       ("DnsEgressPolicy", True), ("ToolSandbox", True),
                       ("SecurityManager", True), ("demote_mislabeled", True),
                       ("AdmissionControl", True), ("verify_query", True),
                       # ⭐ 下面 6 条是 (g)/(h) 两个夹具的**前提**：它们的资产名 /
                       #    文件名 / 目录名必须**真的**不命中治理词表，否则夹具会
                       #    因为一个自己都没料到的词命中而被误判成强信号，
                       #    于是 (g)/(h) 变成永真的假测试。
                       ("Relay", False), ("netlink", False),
                       ("Probe", False), ("telemetry", False),
                       ("misc", False),
                       ("nt_shield", True), ("verify_payload", True)]:
        got = _name_hit(word)
        say(got == want,
            f"谓词 _name_hit({word!r}) = {got}（期望 {want}）")

    for src, gone, kept in cases:
        masked = mask_noncode(src)
        got = set(IDENT.findall(masked))
        bad = [w for w in gone if w in got] + [w for w in kept if w not in got]
        if bad or len(masked) != len(src):
            say(False, f"掩码器断言失败 {src!r}: 意外 {bad!r} · 剩余 {sorted(got)!r} "
                       f"· 等长 {len(masked) == len(src)}")
        else:
            say(True, f"掩码器 {src[:34]!r} -> 剩余 {sorted(got)} · 偏移等长")

    # ⭐ `fold_split_baseline()` 单元断言：拆分前的旧基线必须继续有效。
    #    这段逻辑**只在比较阶段**生效，坏掉的表现是 `--strict` 一次性报
    #    693 条假「新增」—— 即门在基线重算之前一直红。
    L = "neotrix-core/src/x.rs::Thing"
    old_base = {f"ZERO_CONSUMER {L}", "ZERO_CONSUMER other.rs::A", "TEST_ONLY t.rs::B"}
    new_cur = {f"{PATH_ONLY} {L}", "ZERO_CONSUMER other.rs::A", "TEST_ONLY t.rs::B"}
    eff, folded = fold_split_baseline(old_base, new_cur)
    say(folded == 1 and not (new_cur - eff),
        f"fold: 旧基线的 ZERO_CONSUMER 覆盖了 {PATH_ONLY}（折叠 {folded} 条，"
        f"新增 {len(new_cur - eff)} 条）")
    fresh_base = {f"{PATH_ONLY} {L}", "ZERO_CONSUMER other.rs::A", "TEST_ONLY t.rs::B"}
    eff2, folded2 = fold_split_baseline(fresh_base, new_cur)
    say(folded2 == 0 and not (new_cur - eff2),
        f"fold: 基线已是新格式时为空操作（折叠 {folded2} 条）")
    eff3, folded3 = fold_split_baseline(set(), {f"{PATH_ONLY} brand_new.rs::C"})
    say(folded3 == 0 and f"{PATH_ONLY} brand_new.rs::C" not in eff3,
        "fold: 旧基线里没有的定位符**不会**被折叠（不掩盖真正的新增）")

    print("\nSELFTEST " + ("OK — 全部绿" if ok else "FAIL"))
    return 0 if ok else 1


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(self_test())
    sys.exit(main(sys.argv[1:]))