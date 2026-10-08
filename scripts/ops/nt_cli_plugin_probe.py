#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
nt_cli_plugin_probe.py — 外部 CLI descriptor 探活门（TODO.md F3）

============================ 判据的**唯一真源** ==============================
本门**不复刻**判据，而是**从 Rust 源码里抽**判据。真源文件：
    neotrix-core/src/l1_action/nt_act/nt_act_dev_tools/external_cli_plugins.rs
从里面正则抽出三样东西，每样都对应 Rust 侧一段**已存在**的语义：

  1. `pub struct ExternalCliPlugin { ... }` 的字段集
     ⇒ C2「未知字段」的合法集、必填集（无 #[serde(default)] 的字段）
  2. 每个字段的 Rust 类型
     ⇒ C4「类型」判据（String / Vec<String> / Option<PathBuf>）
  3. `fn default_mode() -> String { "interactive" }`
     ⇒ mode 的缺省值（C3 判定「没写 mode」等价于 interactive）

**为什么必须抽而不是手抄**（L8 家族：绿色≠有效）：
  抄一份字段名单，于是「Rust 加了字段」这件事**永远不会**让门变化 ⇒
  门报告的是一份可能已经陈旧的常量。而 `--self-test` 里那条
  `TRUTH-rust-field-set-unchanged` 断言会在真源漂移时**主动变红**。

============================ 判据清单 ==============================
C1 必填字段：无 #[serde(default)] 的字段必须存在，且 String 型必须非空字符串。
C2 未知字段：doc 的键 ⊄ Rust 字段集 ⇒ 判红（serde 默认**静默忽略**未知字段
    ⇒ 写了等于没写，且作者以为它生效了）。
C3 mode 枚举：mode ∉ {interactive, headless} ⇒ 判红。
    ⚠️ 这一条是**策略**不是机器派生（Rust 侧 mode 是 String 不是 enum）：
    capability() 只判 `== "interactive"`，别的值一律落到 "external_cli"。
    ⇒ 枚举集合在本文件里写死，并配一条 headless 的 WARN（见下）。
C4 类型：Vec<String> 必须是非空元素全为字符串的数组；Option<PathBuf> 若出现
    必须是字符串或 null；String 必须是字符串。
C5 JSON 可解析：解析失败 ⇒ 判红。**这是全门最阴的一条**：
    load_external_cli_plugins() 是 `if let Ok(p) = serde_json::from_str(..)`
    ⇒ 解析失败的 descriptor 被**静默跳过** ⇒ 插件隐形，且零报错。
C6 防空转（fail-closed）：目录存在但 0 个 *.json ⇒ 判红
    （"a glob that silently matches nothing would make this contract vacuous"）。
C7 目录不存在（CI runner 上就是这样）⇒ advisory 绿 + **显式说出口**，
    ⛔ 不得静默绿（静默绿会让读者以为门验过了）。
C8 探活：逐个跑 probe_available() 的等价检查，失败 ⇒ 判红并**指名文件**。

============================ C8 为什么不是 `command -v` ==============================
`probe_available()` 的真语义（external_cli_plugins.rs:66-79 + nt_model_cli.rs:
capture_model_command）是四步串联，缺一不可：
  1. spawn 成功（Command::new 走 PATH 解析；无目录分量的名字由 PATH 找）
  2. **退出码为 0**
  3. 未超时（10s）
  4. stdout 经 strip_ansi + trim 后**非空**
⛔ `command -v` 只判第 1 条（且对**绝对路径**会失败，而 descriptor 允许写
绝对路径 ⇒ 两者不等价）⇒ 用 command -v 会同时**漏报**（退出非 0 的命令、
空 stdout 的命令）并**误报**（把合法的绝对路径判成不存在）。

============================ 只读性（硬约束） ================================
默认与 --strict 形态**零文件写操作**：
  · 读：Rust 真源 + descriptor 目录（只 os.listdir / open(只读)）
  · 跑探活：subprocess.run([command, *args])，**shell=False**（与 Rust 的
    Command::new 同语义，且不经过 shell ⇒ 不存在命令注入面）
⛔⛔ **必须说出口的副作用**：探活会**真的 spawn descriptor 里写的那个命令**
   （带 probe_args，缺省 --version）。这是 C8 的定义本身（也是 Rust 侧
   probe_available 的定义）⇒ **「只读」指的是不写仓库/不写 HOME，不是
   「不执行任何东西」**。后果：一份恶意 descriptor 能让本门执行任意
   可执行文件。因此 CI **只接 --self-test**（夹具是本文件内的 /bin/echo 与
   一个不存在的名字），⛔ 不接 live 探活（CI runner 上既没有插件目录，
   接了也只是结构上不可能失败的空门 —— L8）。
唯一写入面是 --self-test，且夹具只能落在 tempfile.mkdtemp() 建的系统临时目录：
见 _mk_fixture_dir() —— 它**结构性地**拒绝任何不在临时根下的路径
（这是把「绝不碰 HOME / 绝不落仓库」从散文变成代码，而不是一句承诺）。

用法：
  nt_cli_plugin_probe.py                  advisory（报告，exit 0）
  nt_cli_plugin_probe.py --strict         任一失败 exit 1
  nt_cli_plugin_probe.py --list           只列不探活，恒 0
  nt_cli_plugin_probe.py --self-test      注入式负向测试（唯一有写面的形态）

退出码：0 PASS / 1 FAIL / 2 用法错或环境缺失 / 3 真源解析失败（判据无从执行）
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TRUTH_REL = os.path.join("neotrix-core", "src", "l1_action", "nt_act",
                         "nt_act_dev_tools", "external_cli_plugins.rs")
TRUTH = os.path.join(REPO, TRUTH_REL)

# 与 Rust 侧 Duration::from_secs(10) 一致
PROBE_TIMEOUT_S = 10

# C3 的策略常量（不是机器派生的，见头注）
ALLOWED_MODES = ("interactive", "headless")
DEFAULT_MODE = "interactive"  # 与 Rust fn default_mode() 的返回值一致

# ──────────────────────────────────────────────────────────────────────
# 真源抽取
# ──────────────────────────────────────────────────────────────────────
_STRUCT_RE = re.compile(
    r"pub\s+struct\s+ExternalCliPlugin\s*\{(.*?)\n\}", re.S)
_FIELD_RE = re.compile(r"^\s*pub\s+([a-z_][a-z0-9_]*)\s*:\s*([^,]+),", re.M)
_SERDE_DEFAULT_RE = re.compile(
    r'#\[serde\(\s*default\s*(?:=\s*"[^"]*")?\s*\)\]')


def extract_truth(path: str = TRUTH):
    """抽出 (字段名 -> Rust 类型, 必填字段集)。

    返回 (None, 原因字符串) 表示抽不到 ⇒ 判据无从执行 ⇒ 调用方必须 exit 3。
    ⛔ 这是「宁可判红也不假装通过」：抽不到真源时若放行，等于退回手抄常量。
    """
    if not os.path.isfile(path):
        return None, f"真源文件不存在: {TRUTH_REL}"
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            text = fh.read()
    except OSError as e:
        return None, f"真源读取失败: {e}"
    m = _STRUCT_RE.search(text)
    if not m:
        return None, "真源里抽不到 `pub struct ExternalCliPlugin { ... }`"
    body = m.group(1)

    fields, required, pending = {}, set(), []
    for line in body.splitlines():
        s = line.strip()
        if s.startswith("#[") or s.startswith("#!"):
            pending.append(s)
            continue
        # 只认结构体直属字段；`impl` 块的方法带 `fn`/`pub fn`，不匹配
        fm = re.match(r"pub\s+([a-z_][a-z0-9_]*)\s*:\s*([^,]+),\s*$", s)
        if not fm:
            continue
        name, ty = fm.group(1), fm.group(2).strip()
        fields[name] = ty
        if not any(_SERDE_DEFAULT_RE.search(a) for a in pending):
            required.add(name)
        pending = []

    if not fields:
        return None, "真源的 struct 体里一个 `pub <field>:` 都没抽到"
    if not required:
        return None, "必填字段集为空 ⇒ C1 无从执行（serde(default) 全覆盖？）"
    return (fields, required), None


def type_kind(ty: str) -> str:
    """Rust 类型 -> 本门能判的类别。

    ⛔ 返回 "opaque" 时本门**明确报出「类型判据未覆盖」**（WARN），
      而不是静默放过 —— 盲区必须自己会说话。真正的硬拦是
      --self-test 的 TRUTH 断言：真源一改，它先红，逼人补映射。
    """
    ty = ty.strip()
    if ty.startswith("Option<"):
        return "optional_string"      # Option<PathBuf>：serde 只接字符串/null
    if ty.startswith("Vec<"):
        return "string_array"
    if ty == "String":
        return "string"
    return "opaque"


# ──────────────────────────────────────────────────────────────────────
# 探活（probe_available 等价实现）
# ──────────────────────────────────────────────────────────────────────
def strip_ansi(text: str) -> str:
    """与 nt_model_cli::strip_ansi 逐分支等价（见该函数）。"""
    out, i, n = [], 0, len(text)
    while i < n:
        ch = text[i]
        if ch == "\x1b":
            if i + 1 < n and text[i + 1] == "[":
                i += 2
                while i < n and not ("@" <= text[i] <= "~"):
                    i += 1
                i += 1  # 吃掉终止符（越界也无妨，与 Rust 同）
                continue
            i += 1  # 孤立 ESC：丢弃
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def probe_available(command: str, probe_args) -> tuple[bool, str]:
    """等价于 ExternalCliPlugin::probe_available()。

    ⛔ 绝不改用 command -v（头注已论证两者不等价）。
    ⛔ shell=False：与 Rust Command::new 同语义，不引入 shell 解释面。
    """
    args = list(probe_args) if probe_args else ["--version"]
    try:
        cp = subprocess.run(
            [command] + args,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=PROBE_TIMEOUT_S,
        )
    except FileNotFoundError:
        return False, f"spawn failed: command not found: {command}"
    except PermissionError:
        return False, f"spawn failed: not executable: {command}"
    except subprocess.TimeoutExpired:
        return False, f"timed out after {PROBE_TIMEOUT_S}s"
    except OSError as e:
        return False, f"spawn failed: {e}"

    if cp.returncode != 0:
        err = strip_ansi(cp.stderr.decode("utf-8", "replace")).strip()
        return False, f"non-zero exit {cp.returncode}: {err[:200] or '(no stderr)'}"
    out = strip_ansi(cp.stdout.decode("utf-8", "replace")).strip()
    if not out:
        return False, "empty stdout (成功退出但无输出 ⇒ Rust 侧判 false)"
    return True, ""


# ──────────────────────────────────────────────────────────────────────
# 扫描
# ──────────────────────────────────────────────────────────────────────
class Scan:
    """一次扫描的结果。fails/warns 元素为 (判据号, 文件名, 说明)。"""

    def __init__(self):
        self.fails: list[tuple[str, str, str]] = []
        self.warns: list[tuple[str, str, str]] = []
        self.entries: list[tuple[str, str]] = []   # (文件名, 一行摘要)
        self.dir_state = "OK"                        # OK / MISSING / EMPTY

    def codes(self) -> set:
        return {c for c, _, _ in self.fails}

    def files(self, code: str) -> list:
        return [f for c, f, _ in self.fails if c == code]


def scan_dir(dirpath: str, fields: dict, required: set, probe: bool) -> Scan:
    r = Scan()

    if not os.path.isdir(dirpath):
        r.dir_state = "MISSING"
        return r

    try:
        names = sorted(n for n in os.listdir(dirpath)
                       if n.endswith(".json")
                       and os.path.isfile(os.path.join(dirpath, n)))
    except OSError as e:
        r.dir_state = "ERROR"
        r.fails.append(("C6", dirpath, f"目录不可读: {e}"))
        return r

    # ── C6 防空转：目录在，但一个 descriptor 都没有 ⇒ 本门什么都没验 ──
    if not names:
        r.dir_state = "EMPTY"
        r.fails.append(("C6", dirpath,
                        "插件目录存在但 0 个 *.json ⇒ 本门是**空门**，"
                        "从没真正验过任何 descriptor（不是「没问题」）"))
        return r

    for name in names:
        path = os.path.join(dirpath, name)
        try:
            with open(path, encoding="utf-8", errors="replace") as fh:
                text = fh.read()
        except OSError as e:
            r.fails.append(("C5", name, f"读取失败: {e}"))
            continue

        # ── C5 可解析性（必须先于一切字段判据）──
        try:
            doc = json.loads(text)
        except ValueError as e:
            r.fails.append(("C5", name,
                            f"JSON 解析失败: {e} ⇒ Rust 侧 load_external_cli_plugins "
                            f"会**静默跳过**它 ⇒ 该插件隐形且零报错"))
            continue
        if not isinstance(doc, dict):
            r.fails.append(("C5", name,
                            f"顶层不是 JSON 对象（是 {type(doc).__name__}）"
                            f" ⇒ serde 反序列化必失败 ⇒ 静默跳过"))
            continue

        # ── C1 必填字段 ──
        for fld in sorted(required):
            if fld not in doc:
                r.fails.append(("C1", name,
                                f"缺必填字段 '{fld}'（Rust 侧无 serde(default)"
                                f" ⇒ 整个 descriptor 解析失败 ⇒ 静默跳过"))
            elif type_kind(fields[fld]) == "string" and (
                    not isinstance(doc[fld], str) or not doc[fld].strip()):
                r.fails.append(("C1", name,
                                f"必填字段 '{fld}' 必须是非空字符串"
                                f"，实为 {json.dumps(doc[fld], ensure_ascii=False)}"))

        # ── C2 未知字段（serde 静默忽略 ⇒ 写了等于没写）──
        unknown = sorted(set(doc) - set(fields))
        if unknown:
            r.fails.append(("C2", name,
                            f"未知字段 {', '.join(unknown)} —— serde 默认**静默忽略**"
                            f" ⇒ 写了等于没写；Rust 认识的是: "
                            f"{', '.join(sorted(fields))}"))

        # ── C4 类型 ──
        for fld, ty in sorted(fields.items()):
            if fld not in doc:
                continue
            kind, val = type_kind(ty), doc[fld]
            if kind == "string_array":
                if not isinstance(val, list):
                    r.fails.append(("C4", name,
                                    f"'{fld}' 在 Rust 里是 {ty} ⇒ 必须是数组，"
                                    f"实为 {type(val).__name__}"))
                elif any(not isinstance(x, str) for x in val):
                    r.fails.append(("C4", name,
                                    f"'{fld}' 数组里有非字符串元素"
                                    f"（Rust 是 {ty}）"))
            elif kind == "optional_string":
                if val is not None and not isinstance(val, str):
                    r.fails.append(("C4", name,
                                    f"'{fld}' 在 Rust 里是 {ty} ⇒ 必须是字符串或"
                                    f" null，实为 {type(val).__name__}"))
            elif kind == "string":
                if not isinstance(val, str):
                    r.fails.append(("C4", name,
                                    f"'{fld}' 在 Rust 里是 {ty} ⇒ 必须是字符串，"
                                    f"实为 {type(val).__name__}"))
            else:
                r.warns.append(("C4?", name,
                                f"字段 '{fld}' 的 Rust 类型 {ty} **不在本门类型映射"
                                f"内 ⇒ 类型判据未覆盖（不静默放过：显式报出盲区）"))

        # ── C3 mode ──
        mode = doc.get("mode", DEFAULT_MODE)
        if not isinstance(mode, str) or mode not in ALLOWED_MODES:
            r.fails.append(("C3", name,
                            f"mode={json.dumps(mode, ensure_ascii=False)} 不在 "
                            f"{{{', '.join(ALLOWED_MODES)}}} 里"
                            f"（缺省 = {DEFAULT_MODE}）"))
        elif mode != "interactive":
            r.warns.append(("C3", name,
                            f"mode={mode} 是**已声明但本文件没有消费路径**的形态"
                            f"（真源注释：headless 应走模型 provider）"))

        # ── C8 探活 ──
        cmd = doc.get("command")
        if probe and isinstance(cmd, str) and cmd.strip():
            ok, why = probe_available(cmd, doc.get("probe_args") or [])
            if not ok:
                r.fails.append(("C8", name, f"探活失败（{cmd}）: {why}"))
                r.entries.append((name, f"C8 探活失败: {why}"))
                continue
        elif probe and (not isinstance(cmd, str) or not cmd.strip()):
            r.entries.append((name, "C1/C8 跳过探活（command 缺失或非字符串）"))
        else:
            r.entries.append((name, f"mode={mode}"
                                   f"{' probe_args=' + json.dumps(doc.get('probe_args')) if doc.get('probe_args') else ''}"))

    return r


# ──────────────────────────────────────────────────────────────────────
# 输出
# ──────────────────────────────────────────────────────────────────────
def report(res: Scan, strict: bool, do_probe: bool, dirpath: str) -> int:
    fails = len(res.fails)
    # 2026-10-06 审计定的规矩：不标模式的话，读脚本的人看到 fails=N 会
    # 误以为门红了，而 advisory 其实放行。⇒ 如实标注**本次是否阻断**。
    #
    # ⚠️ 2026-10-08 修正（LESSONS L8「绿色≠有效」的同型措辞缺陷）：
    # 原实现按 `strict` 拼标签，于是 `--strict` 下 0 失败也会打「本次判红」，
    # 同行紧跟 `exit=0` —— 两句自相矛盾，读者无从判断真实阻断与否。
    # ⇒ 标签必须由**实际 rc** 决定，不是由 flag 决定。
    head = (f"cli-plugin-descriptors: PASS（0 项失败）" if not fails
            else f"cli-plugin-descriptors: FAIL — {fails} 项")
    rc = 1 if (strict and fails) else 0
    _tag = ("**--strict：本次判红（阻断）**" if rc == 1
            else ("**--strict 形态，本次通过不阻断**" if strict
                  else "advisory 模式（**本次不阻断**）"))
    print(f"{head}  [{_tag}  本次 exit={rc}]")
    print(f"  插件目录: {dirpath}"
          f"{'' if do_probe else '（--list：未探活）'}")

    if res.dir_state == "MISSING":
        # C7：CI runner 上就是这样。⛔ 不得静默绿。
        print("  ℹ️ C7：插件目录不存在（CI runner 的常态）⇒ advisory 绿。"
              "**这不是「验过了」**，是「这里没有可验的 descriptor」。")
        return rc
    if res.dir_state == "EMPTY":
        print("  ℹ️ C6：目录存在但 0 个 *.json ⇒ 见下方 C6 条目。")
    if res.dir_state == "ERROR":
        pass

    for name, summary in res.entries:
        print(f"  · {name} — {summary}")
    for code, name, msg in res.warns:
        print(f"  ⚠️ {code} {name}: {msg}")
    for code, name, msg in res.fails:
        print(f"  ❌ {code} {name}: {msg}")
    if res.fails:
        print(f"  ⇒ 合计 {fails} 项失败（{len(res.warns)} 项警告）。"
              + ("本次 --strict 已判红。" if strict
                 else "advisory 形态放行；加 --strict 才阻断。"))
    return rc


# ──────────────────────────────────────────────────────────────────────
# 自测：注入式负向测试（**唯一有写面的形态**）
# ──────────────────────────────────────────────────────────────────────
def _mk_fixture_dir() -> str:
    """夹具目录。**结构性地**保证落在系统临时根下。

    ⛔ 这是把「绝不碰 $HOME / 绝不落仓库」写成代码而不是写成承诺：
      任何不满足临时根的路径在这里就 raise，走不到写盘那一步。
    """
    d = tempfile.mkdtemp(prefix="nt-cli-plugin-probe-")
    root = os.path.realpath(tempfile.gettempdir())
    real = os.path.realpath(d)
    if not (real == root or real.startswith(root + os.sep)):
        shutil.rmtree(d, ignore_errors=True)
        raise RuntimeError(f"夹具目录不在系统临时根下，拒绝使用: {d}")
    return d


def _write_fixture(d: str, name: str, obj) -> str:
    p = os.path.join(d, name)
    with open(p, "w", encoding="utf-8") as fh:
        if isinstance(obj, str):
            fh.write(obj)                       # 故意写坏 JSON 用
        else:
            json.dump(obj, fh, ensure_ascii=False, indent=2)
    return p


def self_test(fields: dict, required: set) -> int:
    ok_json = {"name": "ok", "command": "/bin/echo", "args": ["a"],
               "probe_args": ["nt-probe-ok"], "mode": "interactive"}
    results = []

    def case(name, cond, detail):
        results.append((name, bool(cond), detail))

    # TRUTH 漂移哨兵：真源改了字段集 ⇒ 这里红，逼人复核判据而不是让它静默过期
    expect_fields = {"name", "command", "args", "cwd", "probe_args", "mode"}
    case("TRUTH-rust-field-set-unchanged",
         set(fields) == expect_fields,
         f"抽出 {sorted(fields)}（期望 {sorted(expect_fields)}）——"
         f"不等说明 Rust struct 改了字段 ⇒ 请复核 C1/C2/C4 后再改这里")
    case("TRUTH-required-fields-are-name-and-command",
         required == {"name", "command"},
         f"必填集 = {sorted(required)}（无 serde(default) 的字段）")

    cases = [
        # (断言名, 夹具 {文件名: 内容}, 必须出现的判据号)
        ("C1-missing-required-field",
         {"bad.json": {"name": "x", "args": []}}, "C1"),
        ("C2-unknown-field-silently-ignored",
         {"bad.json": dict(ok_json, argz=["typo"])}, "C2"),
        ("C3-mode-not-in-enum",
         {"bad.json": dict(ok_json, mode="chat")}, "C3"),
        ("C4-args-must-be-array",
         {"bad.json": dict(ok_json, args="--trust-agents")}, "C4"),
        ("C5-unparsable-json-is-invisible",
         {"bad.json": '{"name":"x", "command":'}, "C5"),
        ("C6-empty-dir-is-vacuous",
         {"notadescriptor.txt": "ignored"}, "C6"),
        ("C8-probe-failure-is-named",
         {"bad.json": dict(ok_json, command="definitely-not-a-real-cli-xyz")},
         "C8"),
    ]
    for name, files, want in cases:
        d = _mk_fixture_dir()
        try:
            for fn, content in files.items():
                _write_fixture(d, fn, content)
            res = scan_dir(d, fields, required, probe=True)
            got = res.codes()
            detail = (f"判出 {sorted(got)}，要求含 {want}"
                      + (f"；指名文件 {res.files(want)}" if res.files(want) else ""))
            case(name, want in got, detail)
        finally:
            shutil.rmtree(d, ignore_errors=True)

    # ⭐ 正例对照（L8：绿色≠有效）——
    # 没有这条，一个「把所有 descriptor 都判红」的退化实现也能通过上面 7 条
    # 负向断言（它对任何东西都红）。⇒ 必须钉一条「合规 descriptor ⇒ 零失败」。
    d = _mk_fixture_dir()
    try:
        _write_fixture(d, "ok.json", ok_json)
        res = scan_dir(d, fields, required, probe=True)
        case("POS-compliant-descriptor-must-be-green",
             not res.fails,
             f"零失败（warns={[w[0] for w in res.warns]}）—— 这条钉住门的**分辨力**"
             f"：没有它，一个「见啥都红」的假门也能通过全部负向断言")
    finally:
        shutil.rmtree(d, ignore_errors=True)

    # C7：目录不存在 ⇒ advisory 绿 + 判据自称 MISSING（不静默）
    missing = os.path.join(tempfile.gettempdir(), "nt-cli-plugin-probe-绝对不存在-xyz")
    if os.path.isdir(missing):
        shutil.rmtree(missing, ignore_errors=True)
    res = scan_dir(missing, fields, required, probe=True)
    case("C7-missing-dir-advisory-green-but-explicit",
         res.dir_state == "MISSING" and not res.fails,
         f"dir_state={res.dir_state} fails={len(res.fails)} —— 目录不存在时"
         f"**绿但自称 MISSING**（⛔ 不是「验过了」）")

    # --list 形态恒 0 且不探活：探活只在 --strict/默认里发生
    d = _mk_fixture_dir()
    try:
        _write_fixture(d, "bad.json", dict(ok_json, command="definitely-not-a-real-cli-xyz"))
        res = scan_dir(d, fields, required, probe=False)
        case("LIST-mode-does-not-probe",
             "C8" not in res.codes() and not res.fails,
             f"判出 {sorted(res.codes())} —— --list 不探活（恒 0）")
    finally:
        shutil.rmtree(d, ignore_errors=True)

    print(f"cli-plugin-descriptors self-test: {len(results)} 条断言")
    bad = 0
    for name, ok, detail in results:
        print(f"  {'✅' if ok else '❌'} {name}: {detail}")
        if not ok:
            bad += 1
    if bad:
        print(f"\nself-test: FAIL — {bad}/{len(results)} 条断言未通过")
        return 1
    print(f"\nself-test: PASS（{len(results)}/{len(results)}）"
          f" —— 夹具全部落在 {tempfile.gettempdir()}，已清理")
    return 0


# ──────────────────────────────────────────────────────────────────────
def plugins_dir() -> str:
    """等价于 external_cli_plugins::plugins_dir()。"""
    d = os.environ.get("NEOTRIX_PLUGINS_DIR")
    if d:
        return d
    return os.path.join(os.environ.get("HOME", ""),
                        ".config", "neotrix", "plugins")


def main() -> int:
    ap = argparse.ArgumentParser(
        description="外部 CLI descriptor 探活门（判据真源 = "
                    "external_cli_plugins.rs 的 struct 定义）")
    ap.add_argument("--strict", action="store_true", help="任一失败 exit 1")
    ap.add_argument("--list", action="store_true", help="只列不探活，恒 0")
    ap.add_argument("--self-test", action="store_true",
                    help="注入式负向测试（唯一有写面的形态）")
    a = ap.parse_args()

    truth, why = extract_truth()
    if truth is None:
        if a.list:
            print(f"cli-plugin-descriptors: LIST — 真源解析失败，故不列字段: {why}")
        elif a.self_test:
            print(f"cli-plugin-descriptors: self-test 无法执行 — {why} ⇒ exit 3")
        else:
            print(f"cli-plugin-descriptors: {why} ⇒ 判据无从执行 ⇒ exit 3")
        return 3 if not a.list else 0
    fields, required = truth

    if a.self_test:
        return self_test(fields, required)

    dirpath = plugins_dir()

    if a.list:
        print(f"cli-plugin-descriptors: LIST（不探活）— {dirpath}")
        print(f"  Rust 字段集: {', '.join(f'{k}: {v}' for k, v in sorted(fields.items()))}")
        print(f"  必填字段: {', '.join(sorted(required))}"
              f"（其余带 serde(default)，缺省可用）")
        if not os.path.isdir(dirpath):
            print("  ℹ️ 目录不存在 ⇒ 无可列条目（CI runner 常态）")
            return 0
        res = scan_dir(dirpath, fields, required, probe=False)
        for name, summary in res.entries:
            print(f"  · {name} — {summary}")
        if res.dir_state == "EMPTY":
            print("  ⛔ C6：0 个 *.json ⇒ 本门此刻是空门（--list 恒 0，不阻断）")
        return 0

    res = scan_dir(dirpath, fields, required, probe=True)
    return report(res, a.strict, True, dirpath)


if __name__ == "__main__":
    sys.exit(main())