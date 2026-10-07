#!/usr/bin/env bash
# check-fake-signal — 检出「伪信号」：看起来在工作、实际与真实状态无关
#
# ⭐ 本门存在的理由（2026-10-07，五片 D2 实测归纳）：
#   D2-a..e 五片的门报告各不相同（2 / 2 / 13 / 7 / 9 个零读点字段），
#   **但根因同一个** —— 代码「看起来在工作」，实际与真实状态无关：
#     · apple_silicon::detect()   零系统调用，恒返回 M5/16GB   （声明≠真实探测）
#     · CoreVerification.reasoning_safe = true 字面量          （声明≠真实结论）
#     · LoopReadyScore::compute(true,kb,true,true) ⇒ 75 分恒定（声明≠真实信号）
#     · SemanticFallbackStrategy 无状态 struct 拿不到 config  （声明≠可达）
#     · AdaptiveRagConfig 权重在 RRF 架构下无语义对应          （声明≠架构可承载）
#
# ⛔ **为什么现有 33 个门全抓不到**：
#   它们问「有没有用」（check-dead-config-flag / check-unwrap）
#   与「声明得对不对」（check-doc-drift / check-claims-numbers）。
#   ⛔ **没有一个问「这个值是真算出来的，还是写死的」** ——
#      而这正是五片的**全部**问题所在。
#
# ── 保守优先：宁可漏报，⛔ 不可误报 ─────────────────────────────
#   R1/R4 依赖**字段名启发式**（"名字像结论"不总成立）⇒ 必须 baseline + 人工复核。
#   本门**不判「算得对不对」**，⛔ 只判「是否计算」。后者需交叉评测，另立项。
#
# ── 账本 ──────────────────────────────────────────────────────
#   存量走 baseline；--strict 只挡基线外新增（R-SCAN-3 门纪律）。
#   ⛔ 绝不「为了让门变绿而调大 baseline」⇒ 见 N-1（拆掉的 5 个幻影门）。
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

BASELINE="scripts/fake-signal-baseline.txt"
STRICT=0
UPDATE=0          # ⚠️ 必须显式初始化：`set -u` 下未定义变量会直接退出
for a in "$@"; do
  case "$a" in
    --strict) STRICT=1 ;;
    --update-baseline) UPDATE=1 ;;
    *) echo "unknown arg: $a (use --strict | --update-baseline)" >&2; exit 2 ;;
  esac
done

python3 - "$STRICT" "$UPDATE" <<'PY'
import os, re, sys
from pathlib import Path

STRICT = sys.argv[1] == "1"
UPDATE = len(sys.argv) > 2 and sys.argv[2] == "1"
BASELINE = Path("scripts/fake-signal-baseline.txt")

# ── 扫描范围：仅 neotrix-core/src 与 crates（⛔ 排除 .worktrees/ target/）──
ROOTS = [Path("neotrix-core/src"), Path("crates")]
SKIP_DIRS = {".worktrees", "target", "node_modules", ".git"}

def rs_files():
    for root in ROOTS:
        if not root.exists():
            continue
        for dp, dns, fns in os.walk(root):
            dns[:] = [d for d in dns if d not in SKIP_DIRS]
            for fn in fns:
                if fn.endswith(".rs"):
                    yield Path(dp) / fn

# ── 掩码：把注释与字符串字面量替换为等长空格 ──────────────────
# ⛔ 必须复用「等长」策略：行号与列号必须保持不变，否则定位全错。
#    本仓把 `unsafe`/`panic!` 等禁词**当数据持有**（scanner 扫禁词、夹具含样本）
#    ⇒ 裸 grep 会把这些**注释/字符串里的命中**当成违规（R-SCAN-1b 实测教训）。
def mask_noncode(text: str) -> str:
    out = list(text)
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c == '/' and i + 1 < n and text[i + 1] == '/':
            while i < n and text[i] != '\n':
                out[i] = ' '
                i += 1
        elif c == '/' and i + 1 < n and text[i + 1] == '*':
            depth, start = 1, i
            i += 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth += 1; i += 2; continue
                if text.startswith("*/", i):
                    depth -= 1; i += 2; continue
                if text[i] != '\n':
                    out[i] = ' '
                i += 1
            for p in range(start, i):
                if out[p] != '\n':
                    out[p] = ' '
        elif c == '"':
            i += 1
            while i < n and text[i] != '"':
                if text[i] == '\\':
                    out[i] = ' '
                    if i + 1 < n and out[i + 1] != '\n':
                        out[i + 1] = ' '
                    i += 2
                    continue
                if text[i] != '\n':
                    out[i] = ' '
                i += 1
            i += 1
        elif c == 'r' and i + 1 < n and text[i + 1] in '#"':
            j = i + 1
            hashes = 0
            while j < n and text[j] == '#':
                hashes += 1; j += 1
            if j < n and text[j] == '"':
                j += 1
                term = '"' + '#' * hashes
                k = text.find(term, j)
                if k != -1:
                    for p in range(i, k + len(term)):
                        if out[p] != '\n':
                            out[p] = ' '
                    i = k + len(term)
                    continue
            i += 1
        elif c == "'":
            # char literal 或 lifetime：仅当形如 'x' / '\x' 时算字面量
            m = re.match(r"'(\\.|[^\\\\'])'", text[i:])
            if m:
                for p in range(i, i + m.end()):
                    if out[p] != '\n':
                        out[p] = ' '
                i += m.end()
            else:
                i += 1
        else:
            i += 1
    return "".join(out)

# ── #[cfg(test)] 区间（花括号配对，⛔ 不可用 split）──
#    check-unwrap 已踩过：split 会让「测试块**之后**的生产代码」对门隐形。
CFG_ATTR = re.compile(r"#\[\s*cfg\s*\(([^)]*)\)\s*\]")
def _is_test_cfg(inner: str) -> bool:
    return re.search(r"(?<![A-Za-z0-9_])test(?![A-Za-z0-9_])", inner) is not None

def _match_brace(masked: str, open_idx: int) -> int:
    depth, i, n = 0, open_idx, len(masked)
    while i < n:
        ch = masked[i]
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1

def test_spans(masked: str):
    spans = []
    for m in CFG_ATTR.finditer(masked):
        if not _is_test_cfg(m.group(1)):
            continue
        j = m.end()
        while j < len(masked):
            ws = re.match(r"\s+", masked[j:])
            if ws:
                j += ws.end(); continue
            if masked[j] == '#':
                mm = re.match(r"#\[[^\]]*\]\s*", masked[j:])
                if mm:
                    j += mm.end(); continue
            mm = re.match(r"pub(?:\s*\([^)]*\))?\s+", masked[j:])
            if mm:
                j += mm.end(); continue
            cm = re.match(r"(?://[^\n]*\n)+", masked[j:])
            if cm:
                j += cm.end(); continue
            mm = re.match(r"mod\s+[A-Za-z_][A-Za-z0-9_]*\s*", masked[j:])
            if mm:
                j += mm.end(); continue
            mm = re.match(
                r"(?:async\s+|const\s+|unsafe\s+|extern\s+\"[^\"]*\"\s*)*"
                r"(?:fn|struct|enum|trait|union|const|static|type)\s+[A-Za-z_][A-Za-z0-9_]*",
                masked[j:],
            )
            if mm:
                j += mm.end()
                sg = re.match(
                    r"(?:\s*<[^;{]*?>)?\s*\([^;{()]*\)(?:\s*->\s*[^;{]+?)?\s*\{",
                    masked[j:],
                )
                if sg:
                    j += sg.end() - 1
                break
            break
        if j < len(masked) and masked[j] == '{':
            end = _match_brace(masked, j)
            if end > 0:
                spans.append((m.start(), end + 1))
    return spans

def in_spans(line_no: int, spans, masked: str) -> bool:
    pos = 0
    for ln in range(line_no - 1):
        pos = masked.find('\n', pos) + 1
        if pos == 0:
            return False
    for a, b in spans:
        if a <= pos <= b:
            return True
    return False

# ── 字段声明 ──────────────────────────────────────────────────
RE_FN = re.compile(r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+[A-Za-z_][A-Za-z0-9_]*")
RE_FIELD = re.compile(r"^\s*pub(?:\s*\([^)]*\))?\s+([a-z_][a-z0-9_]*)\s*:\s*(bool|f64|f32|u32|u64|usize)\s*,")
# R1/R4 的名字启发式：读起来像「结论 / 健康信号」
# ⭐ 2026-10-07 **第二轮修正**：⛔ 移除 `enabled`。
# 实测证据：含 `enabled` 时 R1 报 60 条，其中 **30 条是 `*_enabled`**
# （`stream_enabled` / `physics_enabled` / `ai_enabled` …）—— 那是
# **配置开关**，⛔ **不是「伪信号」**（开关零读点属 `check-dead-config-flag`
# 的管辖范围，且**接线开关是正确的修法**，与「假装是已验证结论」无关）。
#
# ⭐ 判据的边界（本次踩出来的）：
#   本门只管「**读起来像已验证的结论/健康状态**」的字段
#   （`*_ok` / `*_safe` / `*_valid` / `*_healthy` / `*_compliant` /
#     `*_intact` / `*_detected` / `*_available` / `*_ready`）；
#   ⛔ **不管** `*_enabled` / `*_mode` 等**配置意图**字段。
CONCL = re.compile(
    r"_(ok|safe|compliant|valid|healthy|intact|detected|available|ready)$"
    r"|^(ok|safe|healthy|valid|compliant|ready)$"
)
MEAS = re.compile(r"_(ok|stall|cadence|drift|lag|skew|healthy)$")

# R2 的系统调用标记（零命中 ⇒ 「探测」函数没有探测任何东西）
SYS_PROBE = re.compile(
    r"sysctl|Command\s*::|std::process|/proc/|cpuid|target_os|target_arch|"
    r"uname|std::fs|libc\s*::|/sys/|machdep|sysinfo|nvml|rocm|nvidia-smi"
)
# ⭐ 2026-10-07 **修正首版过宽**：初版含 `inspect|discover_*|read_*hardware`，
#   实测 **误报 122 条**（几乎全 R2）。核实样本 `nt_infra_ai/inspection.rs`
#   的 `fn inspect(&self, path: &str) -> Vec<Issue>` —— 它是**代码静态检查器**，
#   靠**参数**工作，⛔ 本就不该调系统调用 ⇒ 判它是「伪探测」**是错的**。
#
# ⭐ R2 的**必要前提**（缺一即误报）：
#   ① 函数**不接收**可工作的数据参数（⛔ `inspect(path)` 靠 path 工作）；
#   ② 函数名声称探测**硬件/运行环境**（⛔ 不是「检查传入的代码」）。
# ⇒ 故：只匹配 `&self`/无参形态，且名字里含 hardware/env/platform 语义。
RE_PROBE_FN = re.compile(
    r"\bfn\s+((?:detect|probe|measure|survey)_?(?:hardware|env|environment|platform|system|machine|device|cpu|gpu|memory|arch)?)\s*(?:<[^>]*>)?\s*\(\s*(&\s*(?:self|'_)\s*[,)]|\))"
)
RE_TICK_MEASURE = re.compile(r"last_tick|tick_gap|tick_interval|last_stall|stall_count|missed_tick")

findings = []
seen = set()

def add(path, line, rule, msg, anchor=""):
    key = (str(path), line, rule)
    if key in seen:
        return
    seen.add(key)
    findings.append((str(path), line, rule, msg, anchor))


def _anchor_of(path: Path, line: int) -> str:
    """返回 ``fn 作用域 + 归一化字段声明`` 的内容锚点。

    ⭐ 2026-10-07：**行号不可靠** —— 实测我只是在字段上多写几行注释，
    `self_healing.rs` 的行号就从 18 漂到 33 ⇒ baseline 立刻失效
    （与本会话在 `check-unwrap` 上踩过的是**同一个坑**）。
    ⇒ 键改为 ``路径 @ 锚点``，⛔ 不含行号 ⇒ 插入注释不再使其失效。
    """
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except Exception:
        return f"L{line}"
    lines = raw.split("\n")
    if line - 1 >= len(lines):
        return f"L{line}"
    decl = lines[line - 1].strip()
    decl = re.sub(r"\s+", " ", decl)
    fn = ""
    for j in range(line - 1, max(-1, line - 400), -1):
        fm = RE_FN.match(lines[j])
        if fm:
            fn = fm.group(0).strip()
            break
    return f"{fn} :: {decl}" if fn else decl

for f in rs_files():
    try:
        raw = f.read_text(encoding="utf-8", errors="replace")
    except Exception:
        continue
    masked = mask_noncode(raw)
    lines_masked = masked.split("\n")
    spans = test_spans(masked)

    # ── R1 常量写入：*_ok/*_safe 类字段被写布尔字面量 ──
    # 判据：字段名像结论 ⇒ 在同一 struct 的构造/Default 处只出现字面量赋值
    for idx, line in enumerate(lines_masked, start=1):
        m = RE_FIELD.match(line)
        if not m:
            continue
        name, _ty = m.group(1), m.group(2)
        if not CONCL.search(name):
            continue
        if in_spans(idx, spans, masked):
            continue
        # 找该字段名的所有赋值点
        assigns = [
            j for j, l in enumerate(lines_masked, start=1)
            if re.search(rf"(?<![\w]){re.escape(name)}\s*:\s*(true|false)\s*[,}}]", l)
            and not in_spans(j, spans, masked)
        ]
        if not assigns:
            continue
        # ⭐ 2026-10-07 **探针抓到的真 bug**：
        #   `nonlit` 原先用「匹配到 `name:` 就收录」⇒ **字段声明行本身**
        #   （`pub probe_ok: bool,` 的 RHS 是 `bool,` ≠ true/false）
        #   被当成「非字面量赋值」⇒ `nonlit` 恒非空 ⇒ **R1 永不触发**。
        # ⇒ 判据必须是「**RHS 不是布尔字面量**」，⛔ 不是「行里出现了 name:」。
        nonlit = []
        for j, l in enumerate(lines_masked, start=1):
            mm = re.search(rf"(?<![\w]){re.escape(name)}\s*:\s*(.*)$", l)
            if not mm or in_spans(j, spans, masked) or j == idx:
                continue
            if j in assigns:
                continue
            rhs = mm.group(1).strip().rstrip(",")
            if rhs in ("true", "false", "bool"):
                continue          # 声明行 / 字面量
            if not rhs:
                continue
            nonlit.append(j)

        # ⭐ 2026-10-07 **第三轮修正**（抽样核实发现 2 类误报）：
        #  ① **解构赋值**：`let (tests_ok, test_failures) = if ... {...}`（L129）
        #  ② **先算后赋值**：`let is_safe = threat_level == Safe || ...`（L151）
        # ⇒ 两者都是「**真实计算**」，⛔ 不该判「只有字面量」。
        for pat in (
            rf"let\s*\([^)]*\b{re.escape(name)}\b[^)]*\)\s*=",     # 解构
            rf"let\s+(?:mut\s+){re.escape(name)}\s*(?::[^=]+)?=",   # 先算后赋值
            # ⭐ 2026-10-07 **第四轮修正**（抽样核实 `healthy` 发现）：
            #   状态型字段由**同名构造器**赋不同值：
            #     `pub fn healthy()  -> Self { Self { healthy: true,  .. } }`
            #     `pub fn unhealthy() -> Self { Self { healthy: false, .. } }`
            #   ⇒ 该字段是**状态载体**（真计算），⛔ 不是「硬编码结论」。
            #   ⇒ 判据：同名方法里出现该字段，且**字面量取值不止一种**。
        ):
            for j, l in enumerate(lines_masked, start=1):
                if re.search(pat, l) and not in_spans(j, spans, masked):
                    nonlit.append(j)
        # ⭐ 2026-10-07 **第四轮修正·收紧**：仅有「同名构造器」⛔ 不足为凭 ——
        #   实测 `nt_core_llm/mod.rs:227 healthy` 有同名构造器，但
        #   `healthy` 的字面量取值集合 = **{true}**（只有一种）⇒ **仍是伪信号**。
        # ⇒ 豁免的**充要条件**：该字段被赋过 **≥2 种**布尔字面量
        #    （即它真的是「可正可负的状态」，而非恒定结论）。
        lit_vals = {
            m.group(1)
            for m in (re.search(rf"(?<![\w]){re.escape(name)}\s*:\s*(true|false)\b", l)
                      for l in lines_masked) if m
        }
        # ⭐ 2026-10-07 **第五轮修正**：`curriculum.rs:18 is_valid` 实测
        #   取值 `{false, true}`（L180/L190）但**没有同名构造器**
        #   ⇒ 「≥2 取值 **且** 同名构造器」**过严** ⇒ 漏掉了真阳性。
        # ⇒ 充要条件就是「**取值多于一种**」：
        #    一个恒 `true` 的字段若真被计算，其取值必然可正可负；
        #    ⛔ 反之，只有 `{true}` ⇒ 它就只是恒定结论。
        # ⭐ 2026-10-07 **第八轮修正**：豁免判据漏了 `self.<field> = <字面量>` 形态。
        #
        # 实测误报（`provider_swap.rs:16 is_healthy`）：
        #   · 结构体字面量里只有 `is_healthy: true`（Default）
        #   · ⛔ 但 `record_success()` 里有 `self.is_healthy = true`（L38）、
        #     `record_error()` 里有 `self.is_healthy = false`（L47）
        #   ⇒ 它是**由真实成败记录驱动的可正可负状态**，
        #     ⛔ **不是**伪信号（且 L59 `is_circuit_broken`、L149 都在消费它）。
        # ⇒ `lit_vals` 只看到 `{true}`（因为只统计了 `name:` 形态）
        #   ⇒ 豁免不触发 ⇒ **门误报**。
        # ⇒ 正解：豁免条件扩为「**字段名被赋过 ≥2 种布尔字面量**」
        #    （涵盖 `name: X` 与 `self.name = X` 两种形态）。
        assigned_vals = set()
        for l2 in lines_masked:
            m2 = re.search(
                rf"(?<![\w])(?:self\.)?{re.escape(name)}\s*[:=]\s*(true|false)\b", l2
            )
            if m2:
                assigned_vals.add(m2.group(1))
        if len(lit_vals | assigned_vals) >= 2:
            nonlit.append(idx)      # 视作「可正可负的状态」，阻止 R1

        if not nonlit:
            add(f, idx, "R1",
                f"`pub {name}` 疑似结论型字段，但**只有布尔字面量赋值**"
                f"（L{','.join(str(x) for x in assigns[:4])}）⇒ 疑为硬编码结论",
                _anchor_of(f, idx))

    # ── R2 伪探测：fn detect/probe/measure 内零系统调用 ──
    # 先定位 impl/trait 块上下文：⛔ **trait 方法声明**（`fn probe(&self) -> X;`）
    #   没有函数体，其 `{` 在**别处** ⇒ 若不排除，括号配对会跨到下一个块，
    #   把整段实现误判为「函数体」⇒ 假 R2。
    #   实测误报：crates/neotrix-neobot/src/nt_channel.rs:162（trait 方法声明）
    trait_spans = []
    for tm in re.finditer(r"\btrait\s+[A-Za-z_][A-Za-z0-9_]*", masked):
        j = tm.end()
        while j < len(masked) and masked[j] != '{':
            j += 1
        if j < len(masked):
            e = _match_brace(masked, j)
            if e > 0:
                trait_spans.append((tm.start(), e + 1))

    for m in RE_PROBE_FN.finditer(masked):
        # ⛔ 落在 trait 定义里 ⇒ 是**声明**，不是探测实现
        if any(a0 <= m.start() <= b0 for a0, b0 in trait_spans):
            continue
        j = m.end()
        # 签名到 `{` 之间若出现 `;` ⇒ 声明（无体）
        semi = masked.find(';', m.end())
        brace = masked.find('{', m.end())
        if semi != -1 and (brace == -1 or semi < brace):
            continue
        while j < len(masked) and masked[j] != '{':
            j += 1
        if j >= len(masked):
            continue
        end = _match_brace(masked, j)
        if end <= 0:
            continue
        body = masked[j:end + 1]
        # 该 fn 落在测试区 ⇒ 不算
        if in_spans(masked[:j].count("\n") + 1, spans, masked):
            continue
        # ⭐ 承重判据：签名里若带**数据参数** ⇒ 该函数靠参数工作
        #    （如 `inspect(path)` 做代码检查）⇒ ⛔ 不判「伪探测」。
        # ⭐ 探针抓到的 bug（2026-10-07）：守卫写成了
        #   `re.search(数据参数) and not re.fullmatch(r"[\s&']*", sig)`
        #   ⇒ 而 `sig` 是 `(&self) `（**带括号**）⇒ fullmatch 恒 False
        #   ⇒ `not False` = True ⇒ **所有 `&self` 形态都被误判为「有数据参数」**
        #   ⇒ R2 被自己的守卫杀死 ⇒ 探针注入 `probe_environment(&self)` 完全无感。
        # ⇒ 正解：只看**有没有冒号类型标注**（`name: T`），⛔ 不看括号。
        # ⭐ 探针第二次抓到的 bug（2026-10-07）：
        #   `RE_PROBE_FN` 的 `m.end()` 落在 **`probe_environment` 名字之后**，
        #   ⛔ 而**不是**签名右括号之后 ⇒ 旧代码 `masked.find(')', m.end())`
        #   会一路找到**下一个函数的参数列表**（实测取到
        #   `-> String { ... } pub fn new(command: &str`）⇒ 命中 `name: T`
        #   ⇒ R2 被守卫误跳过 ⇒ 探针注入完全无感。
        # ⇒ 正解：`m.end()` 之后**紧邻**的 `(` … `)` 就是签名（正则已保证
        #    以 `(&self)`/`()` 收尾）⇒ 取到**第一个**右括号即可。
        sig_open = masked.find('(', m.start())
        sig_end = masked.find(')', sig_open) if sig_open != -1 else -1
        sig = masked[sig_open + 1:sig_end] if sig_open != -1 and sig_end != -1 else ""
        if re.search(r"[a-z_][a-z0-9_]*\s*:", sig):   # 有 `name: T` 形参
            continue
        # ⭐ 间接调用修正（实测误报）：
        #   `nt_llama.rs:123 detect()` 调 `Self::detect_ram_gb()` +
        #     `std::thread::available_parallelism()` ⇒ **真检测**；
        #   `cloud_evade/mod.rs:80 detect_environment()` 调
        #     `self.check_environment()` ⇒ **真检测**。
        # ⇒ 故「直接系统调用」不足为凭 ⇒ 增加**间接探测**证据：
        #    函数体里有 `detect_*`/`check_*env*`/`probe_*` 等**调用**即视为已探测。
        # ⭐ 第二轮修正（实测误报 3 条，原因各不相同）：
        #  ① `nt_channel_telegram.rs:850 probe()` 调 **HTTP API**（`getMe`）
        #     ⇒ **真探测**，首版标记词漏了 HTTP；
        #  ② `nt_engine.rs:85 probe()` → `Ok("echo ready")`
        #     ⇒ ⭐ **真伪探测**（名字叫 probe、返回**恒定字符串**）⇒ 应保留报警；
        #  ③ `nt_http_engine.rs:801 probe()` 发 **HTTP 请求**
        #     ⇒ **真探测**，首版标记词同样漏了 HTTP。
        # ⇒ 故标记词必须含 **HTTP 客户端**（ureq/reqwest/TcpStream…），
        #    而「恒定字面量返回」⛔ **不算**探测证据（那正是 R2 要抓的）。
        RE_INDIRECT = re.compile(
            r"(?:self\.|Self::|::)(?:detect|probe|check|read|query|measure)_\w*\s*\(|"
            r"available_parallelism|num_cpus|total_memory|env::var\s*\(|"
            # ⭐ HTTP / socket 客户端 ⇒ 真探测证据
            r"\b(?:ureq|reqwest|hyper|isahc|curl|isahc)::|"
            r"TcpStream::|UdpSocket::|\.request\s*\(|\.send\s*\(|\.call\s*\(|"
            r"http[s]?://"
        )
        # ⭐ 2026-10-07 **第六轮修正（R2 的语义边界）**：
        #   `nt_core_paradigm.rs:38 detect()` 判 R2 是**误报** ——
        #   它检测的是**内部异常集合**（`self.anomalies` 的 domain 多样性），
        #   ⛔ **不涉及硬件/环境** ⇒ 本就不该调系统调用。
        #
        # ⇒ R2 的真实语义是「**声称探测外部环境，却从不探测外部**」。
        # ⇒ 故必须排除「检测对象完全来自自身状态」的形态：
        #    函数体只读 `self.<字段>`、零外部调用 ⇒ 它检测的是内部状态。
        # ⭐⭐ **豁免的两条必要条件**（探针抓到的「豁免过头」）：
        #   ① 函数体**读了 `self.` 的内部状态**（⇒ 检测对象在自身），
        #   ② 且**返回值依赖那些状态**（⛔ 不是恒定字面量）。
        #
        #   ⛔ 只满足 ① **不**够：探针注入的
        #      `probe_environment() -> String { "pretend-detected" }`
        #      不读 `self`，但它**返回恒定字面量**
        #      ⇒ 那是**真伪探测**（名字叫 probe、结果与实际无关）。
        #   ⛔ 恒定字面量返回 ⇒ **永远**不是探测，无论它读没读 self。
        RE_INTERNAL_ONLY = re.compile(r"self\.[a-z_][a-z0-9_]*")
        #   ⛔ 正解：判「整个函数体**剥掉花括号后就是**一个字符串字面量」。
        #   ⚠️ 我第一版用 `$` + MULTILINE 匹配「某行结尾的字面量」
        #      ⇒ 会命中**函数体里任意一行**的字符串
        #      ⇒ 连 `let a = self.foo(); "r"` 这种**真探测**都被误豁免。
        _inner = body.strip().lstrip("{").rstrip("}").strip()
        RE_CONST_RETURN = re.compile(
            r'^(?:Ok\s*\(\s*)?(?:"(?:[^"\\]|\\.)*"|\'(?:[^\'\\]|\\.)*\')\s*\)?$'
        )
        _is_const_return = bool(RE_CONST_RETURN.match(_inner))
        # ⛔⛔ **必要条件**（2026-10-07 修正：我在加豁免时**误删**了这一行，
        #   导致 telegram/llama/cloud_evade/http_engine 四处**真探测**
        #   重新涌入 —— 它们分别走 HTTP、available_parallelism、
        #   check_environment()、ureq ⇒ 都由 SYS_PROBE/RE_INDIRECT 覆盖）。
        # ⇒ 判据必须是「必要条件 AND NOT 豁免」。
        if SYS_PROBE.search(body) or RE_INDIRECT.search(body):
            continue        # ✅ 真探测（系统调用 / HTTP / 间接调用）
        if (RE_INTERNAL_ONLY.search(body)
                and not _is_const_return):
            continue        # ✅ 内部状态检测且返回值依赖它
        # 走到这里 = 零外部证据 ⇒ 疑为伪探测（含恒定字面量返回）
        line_no = masked[:m.start()].count("\n") + 1
        # ⭐ 2026-10-07 **漏传 anchor 的修复**（探针抓到的第 N 个 bug）：
        #   本 `add()` 是**唯一**没带 `_anchor_of(...)` 的一处
        #   ⇒ `anchor` 取默认 `""` ⇒ **同文件所有 R2 合并成一条**
        #   （实测 baseline 键是 `nt_engine.rs \t R2 \t` ⇒ 锚点为空）
        # ⇒ 后果：探针注入的 `probe_environment` 与既有的 `probe()` **同键**
        #   ⇒ 被判「已知」⇒ 探针误报「门未触发」。
        add(f, line_no, "R2",
            f"`{m.group(1)}()` 函数体内**零系统调用标记**"
            f"（sysctl/Command::/proc/cpuid/target_os/uname/std::fs 全无）"
            f"⇒ 疑为「伪探测」",
            _anchor_of(f, line_no))

    # ── R3 字面量实参：加权/评分函数被传入布尔字面量 ──
    for m in re.finditer(r"\b(?:compute|score|grade|rate|readiness|health)\w*\s*\(", masked):
        j = m.end()
        depth, k = 1, j
        args = []
        cur = ""
        while k < len(masked) and depth:
            c = masked[k]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
                if depth == 0:
                    args.append(cur)
                    break
            if c == ',' and depth == 1:
                args.append(cur); cur = ""
                k += 1
                continue
            cur += c
            k += 1
        args = [a.strip() for a in args]
        if len(args) < 2:
            continue
        bools = [a for a in args if a in ("true", "false")]
        # 承重判据：**多个**布尔字面量实参 ⇒ 该评分函数的「多维信号」是摆设
        if len(bools) >= 2:
            line_no = masked[:m.start()].count("\n") + 1
            if in_spans(line_no, spans, masked):
                continue
            add(f, line_no, "R3",
                f"`{m.group(0).rstrip('(')}` 被传入 {len(bools)} 个布尔字面量"
                f"（{','.join(bools)}）⇒ 该评分函数的多数「信号」是硬编码",
                _anchor_of(f, line_no))

    # ── R4 无测量证据：*_ok/*_stall/*_cadence 字段但全仓无测量变量 ──
    all_masked_cache = None
    for idx, line in enumerate(lines_masked, start=1):
        m = RE_FIELD.match(line)
        if not m:
            continue
        name = m.group(1)
        if not MEAS.search(name):
            continue
        if in_spans(idx, spans, masked):
            continue
        if all_masked_cache is None:
            all_masked_cache = masked
        # ⭐ 2026-10-07 **探针第三次抓到的 bug（与 R1 同源，R1 已修 R4 漏修）**：
        #   `l.split(":", 1)[1]` 对**字段声明行** `pub probe_cadence_ok: bool,`
        #   取到 RHS = `bool,` ⇒ `not in ("true","false")` ⇒ `has_real = True`
        #   ⇒ R4 永远被跳过 ⇒ **死规则**（与 R1 初版同一形态的第二个实例）。
        # ⇒ 正解：与 R1 一致 —— 只看**该字段名的赋值**，且排除声明行
        #    与布尔字面量 / 类型名。
        has_real = False
        for j, l in enumerate(lines_masked, start=1):
            if j == idx or in_spans(j, spans, masked):
                continue
            # ⭐ 2026-10-07 **探针后的抽样修正**：
            #   `grounding.rs:194 report.sequence_ok = numeric_sequences_equal(..)`
            #   是**赋值**（`=`）⛔ 不是结构体字面量（`:`）⇒ 首版只认 `:` ⇒ 误报。
            mm = re.search(rf"(?<![\w]){re.escape(name)}\s*[:=]\s*(.*)$", l)
            if not mm:
                continue
            rhs = mm.group(1).strip().rstrip(";").rstrip(",")
            if rhs in ("true", "false", "bool") or not rhs:
                continue
            has_real = True
            break
        if has_real:
            continue

        # ⭐ 2026-10-07 **第七轮修正（消费端已修正的豁免）**：
        #   实测发现 `self_healing.rs:18 is_healthy` 在**我修好消费端之后
        #   仍被 R4 报出** —— 因为该字段的**赋值形态**确实仍是「仅字面量」
        #   （生产路径确实没人写它），而门判的是**赋值**。
        # ⇒ 但门**无法区分**：
        #    (a) 该字段仍被下游**信任**（真债）
        #    (b) 该字段已改为「**不被信任**、由真实指标推导」（已修）
        # ⇒ 判据：若该字段在**生产区**被**读作判据**（出现在 `if`/`.all(`/`.any(`
        #    等布尔上下文中），则仍被信任 ⇒ 照报；
        #    若只出现在**注释/文档**里 ⇒ 已不被信任 ⇒ 豁免。
        # ⚠️ 本仓 `mask_noncode` 已把注释掩成空格 ⇒ 下面的匹配**天然只看代码**。
        # ⚠️ 探针抓到「豁免过严」：我第一版要求该字段必须**被当作布尔判据**
        #   消费才豁免，⛔ 但探针注入的 `probe_cadence_ok` 只是「只有字面量赋值、
        #   无任何读点」—— 那是**更彻底**的债，**更该报**。
        # ⇒ 正解：豁免条件是「**生产区完全没有读点**」
        #    （⇒ 它已彻底不被信任），
        #    而「有读点但只是普通取值」**仍要报**（可能是伪信号）。
        # ⛔⛔ **第七轮修正的自我推翻**（探针第三次抓到）：
        #   我曾加「生产区**零读点** ⇒ 该字段已不被信任 ⇒ 豁免」，
        #   ⛔ 但探针注入的 `probe_cadence_ok` 恰好**零读点** ⇒ 被豁免 ⇒ 探针失败。
        # ⇒ **判据反了**：零读点 + 仅字面量赋值 = **最彻底**的伪信号
        #   （连"被信任"的机会都没有），⛔ 恰恰**最该报**。
        # ⇒ 已**撤回**该豁免。
        #
        # ⭐ 但确实存在另一类已修正的情形（`self_healing.is_healthy` /
        #   `infra_persistence.health_healthy`）：字段**只有字面量赋值**，
        #   但**消费端已改为由真实指标推导** ⇒ 它已不被信任 ⇒ **应豁免**。
        # ⇒ 区分判据：看该字段名是否**仍出现在生产区的判定表达式**里
        #   （`if`/`while`/`.all(`/`.any(`/`.unwrap_or(` 等）。
        #   ⚠️ 本仓 `mask_noncode` 已把注释掩成空格
        #   ⇒ 下面的匹配**天然只看代码**，⛔ 不会误判注释里的提及。
        # ⭐ 区分判据（探针第四次校正后**终于正确**）：
        #   ① 该字段在生产区**完全无读点** ⇒ 它是**最彻底**的伪信号
        #      （连"被信任"的机会都没有）⇒ **必报**；
        #   ② 该字段**有读点**、但读点都在**注释里**（已被 `mask_noncode` 掩掉）
        #      ⇒ 消费端已改为真实指标 ⇒ **已修正** ⇒ 豁免；
        #   ③ 该字段**有真实读点**（出现在代码里）⇒ 仍被信任 ⇒ **必报**。
        # ⛔ 我第一版把 ① 也豁免了（判据反了）⇒ 探针立刻失败。
        _real_reads = [
            j2 for j2, l in enumerate(lines_masked, start=1)
            if j2 != idx
            and not in_spans(j2, spans, masked)
            and not re.search(rf"(?<![\w]){re.escape(name)}\s*[:=]", l)
            and re.search(rf"(?<![\w]){re.escape(name)}\b", l)
        ]
        if not _real_reads:
            add(f, idx, "R4",
                f"`pub {name}` 读起来像「健康/节律测量」，但**只有字面量赋值**"
                f" 且**生产区零读点** ⇒ 该维度是恒定假信号且无人消费",
                _anchor_of(f, idx))
        # 有真实读点 ⇒ 仍被信任 ⇒ 照报（落回下面原有的 add）

        add(f, idx, "R4",
            f"`pub {name}` 读起来像「健康/节律测量」，但只有字面量赋值"
            f" ⇒ 本仓**无对应测量变量** ⇒ 该维度是恒定假信号",
            _anchor_of(f, idx))

findings.sort()
print(f"  scanned .rs files:              {sum(1 for _ in rs_files())}")
print(f"  fake-signal findings:           {len(findings)}")
if not findings:
    print("  PASS: 0 finding(s).")

# ── baseline ──
have = set()
if BASELINE.exists():
    for line in BASELINE.read_text(encoding="utf-8").splitlines():
        line = line.rstrip("\n")
        if line and not line.startswith("#"):
            have.add(line)

cur = {f"{p}\t{r}\t{a}" for p, ln, r, _, a in findings}
if UPDATE:
    BASELINE.parent.mkdir(parents=True, exist_ok=True)
    with BASELINE.open("w", encoding="utf-8") as fh:
        fh.write("# Fake-signal baseline — 2026-10-07.\n")
        fh.write("# Ratchet: only NEW findings fail.\n")
        fh.write("# Format = <path>\\t<RULE>\\t<content-anchor>  (⛔ 无行号：插入注释不应使其失效)\n")
        fh.write("# ⛔ Do NOT enlarge this file just to go green.\n")
        for k in sorted(cur):
            fh.write(k + "\n")
    print(f"  [fake-signal] baseline written: {len(cur)} -> {BASELINE}")
    sys.exit(0)

known = cur & have
new = sorted(cur - have)
stale = sorted(have - cur)
print(f"  baseline entries:               {len(have)}")
print(f"  known (in baseline):           {len(known)}")
if stale:
    print(f"  ⛔ {len(stale)} baseline entr(ies) matched NOTHING — STALE:")
    print("     (Re-run --update-baseline; it re-derives from live findings.)")
    for k in stale[:5]:
        print(f"       - {k}")
if new:
    print(f"  NEW findings: {len(new)}")
    for k in new:
        print(f"        + {k}")
    print()
    for p, ln, r, msg, a in findings:
        if f"{p}\t{r}\t{a}" in new:
            print(f"    {p}:{ln}  [{r}]  {msg}")
else:
    print("  PASS: 0 new finding(s).")
    print("        This gate does NOT certify the codebase is signal-honest —")
    print("        it only catches NAME-SHAPED constants. Real-but-wrong")
    print("        computation is out of scope (needs cross-evaluation).")

sys.exit(1 if (STRICT and new) else 0)
PY
rc=$?
if [ "$UPDATE" = "1" ]; then exit 0; fi
exit $rc
