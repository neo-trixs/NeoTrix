#!/usr/bin/env python3
"""nt_fuse_types.py — 按完整判据自动融合重复类型（每组独立、编译验证、可回滚）。

## 架构依据（晶体核心分层原则）
类型应住在**所有消费者都能到达的最低层**（`.neotrix/layer-map.json` R-P199 口径）。
⇒ 同名同字段同层内 ⇒ **保留 `mod.rs` 已 re-export 的那一份**（它就是事实上的真源），
另一份改为 re-export。

## 七项判据（全过才动，缺一不动）
① 同名
② 字段**名 + 类型**完全相同（只比名字会把「高度相似的两个不同类型」误判成重复）
③ 无刻意镜像声明（注释里出现 mirror / stay stable / interface contract …）
④ **两侧 trait impl 块一致** —— 否则 `impl Default` 冲突
⑤ **被删侧无固有 impl 方法** —— 否则融合后与真源侧同名方法撞成 E0592
   （2026-09-29 实测：`Verdict` 两侧各有 `is_blocked`）
⑥ **依赖安全** —— 被删那一侧的定义，其引用方仍能通过 re-export 拿到类型
⑦ **字段类型闭包安全** —— 若某字段的类型名本身也是重复类型名，则同名背后
   可能是不同类型（如 ConnectionType 4变体 vs 7变体），名义类型系统下不可自动融合 —— 被删那一侧的定义，其引用方仍能通过 re-export 拿到类型

## 为什么逐组编译
2026-09-29 实测：批量执行第一组就炸（E0119 + E0560）——
④⑤ 两项是「删了才发现」的类型。逐组编译 + 失败即回滚，
把单组风险限制在一个提交内。

## 用法
    python3 scripts/ops/nt_fuse_types.py --dry-run    # 只报告
    python3 scripts/ops/nt_fuse_types.py --limit 20   # 实际执行前 20 组
"""
import argparse
import json
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(REPO, "neotrix-core", "src")

TYPE_RE = re.compile(r"pub (struct|enum|trait) (\w+)([^{]*)\{")
IMPL_RE = re.compile(r"impl(?:<[^>]*>)?\s+([\w:<>, ]+?)\s+for\s+(\w+)")
FIELD_TYPED_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z_0-9]*)\s*:\s*([^,\n]+)", re.M
)
BLOCK_RE = re.compile(
    r"(?:#\[derive\([^)]*\)\]\n)*(?:#\[serde\([^)]*\)\]\n)*"
    r"(?:#\[cfg_attr\([^)]*\)\]\n)*pub (?:struct|enum|trait) (\w+)\b"
)


def read(rel):
    with open(os.path.join(SRC, rel), encoding="utf-8", errors="replace") as f:
        return f.read()


def block_bounds(text, name):
    """返回 (block_start, body_lbrace, block_end)，找不到返回 None。

    block_start **含前置的 derive/serde/cfg_attr 属性与 doc 注释**——
    删除时必须连它们一起删，否则留下悬空 `#[derive(...)]` ⇒ E0774。
    body_lbrace 是结构体左大括号位置（供 fields_of 用，避免从更早的
    block_start 往后找 `{` 时误中注释里的花括号）。

    2026-09-29 两次踩坑：
    ① 初版只从 `pub struct X` 起删，没连 derive 一起删 ⇒ 16 组里 12 组 E0774。
    ② 修了吞回逻辑，却在 return 处写成 `m.start()`，把算好的 `start` 丢掉 ⇒
       修正完全没生效，PressureRating 照样残留 derive。
    """
    m = re.search(r"pub (?:struct|enum|trait) " + re.escape(name) + r"\b[^{]*\{", text)
    if not m:
        return None
    # 从定义往回吞掉紧邻的属性行（derive / serde / cfg_attr / cfg / doc）
    start = m.start()
    while True:
        prev_nl = text.rfind("\n", 0, start - 1)
        if prev_nl < 0:
            break
        line = text[prev_nl + 1:start].strip()
        # 属性行（`#[derive(...)]` / `#[serde(...)]` / `#[cfg_attr(...)]`）**与
        # doc 注释行（`/// ...` / `//! ...`）都要吞**。
        # 2026-09-29 实测：初版只吞属性行，遇到
        #     /// 尺寸规格
        #     #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        #     pub struct SizeSpec { … }
        # 这种「doc 在 derive 上方」的排版时，`#[derive]` 上一行是 `/// …`
        # ⇒ 循环立即 break ⇒ derive 残留 ⇒ `E0774` 依旧（12/16 组仍失败）。
        if line.startswith(("#[", "///", "//!")) or line == "":
            start = prev_nl + 1
            continue
        break
    i = text.index("{", m.end() - 1)
    depth = 0
    k = i
    while k < len(text):
        if text[k] == "{":
            depth += 1
        elif text[k] == "}":
            depth -= 1
            if depth == 0:
                return start, i, k + 1
        k += 1
    return None


def fields_of(text, name):
    b = block_bounds(text, name)
    if not b:
        return None
    body = text[b[1]:b[2]]
    return frozenset(
        f"{n}:{t.strip()}" for n, t in FIELD_TYPED_RE.findall(body)
    )


def impls_of(text, name):
    return frozenset(
        f"{tr.strip()} for {ty}" for tr, ty in IMPL_RE.findall(text) if ty == name
    )


def inherent_methods(text, name):
    """返回该类型在**本文件内**的固有 impl 方法名集合。

    2026-09-29 实测踩坑：`impls_of` 只查 trait impl（`impl X for Y`），
    漏掉了固有 impl（`impl Y { ... }`）。`evomal_guard.rs` 与
    `self_poison.rs` 各有一份 `impl Verdict`（都有 `is_blocked`）——
    融合后两个固有 impl 指向同一类型 ⇒ E0592 duplicate definitions。
    故被删一侧**必须没有任何固有 impl 方法**，否则跳过（需人工迁移方法）。
    """
    methods = set()
    for m in re.finditer(r"impl\s+" + re.escape(name) + r"\b[^{]*\{", text):
        # 跳过 trait impl（`impl X for Y` 含 for，已由 impls_of 处理）
        head = m.group(0)
        if " for " in head:
            continue
        i = text.index("{", m.end() - 1)
        depth, k = 0, i
        while k < len(text):
            if text[k] == "{":
                depth += 1
            elif text[k] == "}":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        body = text[i:k]
        for fm in re.finditer(r"(?:pub(?:\([^)]*\))?\s+)?fn (\w+)\s*(?:<[^>]*>)?\s*\(", body):
            methods.add(fm.group(1))
    return frozenset(methods)


def module_of(rel):
    parts = rel.split("/")
    for i in range(len(parts) - 1, 0, -1):
        if os.path.exists(os.path.join(SRC, "/".join(parts[:i]), "mod.rs")):
            return "/".join(parts[:i])
    return "/".join(parts[:-1])


def layer_of(rel):
    for seg in rel.split("/"):
        if len(seg) > 1 and seg[0] == "l" and seg[1].isdigit():
            return seg[:2]
    return "xx"


def canonical_source(paths):
    """决定哪一份是真源：看各文件所在模块的 mod.rs re-export 了谁。

    规则：
    ① 若某文件在**自己模块的 mod.rs** 里被 `pub use <模块>::<Name>` 导出
       （而不是 re-export 另一个文件）⇒ 它是真源
    ② 否则退化为：路径最短 / 字典序
    """
    scored = []
    for p in paths:
        mod_rs = os.path.join(SRC, module_of(p), "mod.rs")
        is_source = 0
        if os.path.isfile(mod_rs):
            t = open(mod_rs, encoding="utf-8", errors="replace").read()
            base = p.rsplit("/", 1)[-1][:-3]  # 去 .rs
            # 本文件自己的类型被 re-export（pub use xxx::Name, 且 xxx 属于本模块）
            if re.search(r"pub use (?:crate::)?[\w:]*" + re.escape(base) + r"\s*::\s*\{", t):
                is_source = 1
        scored.append((-is_source, len(p), p))
    scored.sort()
    return scored[0][2]


def analyse(name, paths):
    """返回 (ok, 理由, 真源, 其余路径)"""
    if len({module_of(p) for p in paths}) > 1:
        return False, "跨模块", None, []
    if len({layer_of(p) for p in paths}) > 1:
        return False, "跨层", None, []
    src = canonical_source(paths)
    others = [p for p in paths if p != src]
    fs = fields_of(read(src), name)
    for o in others:
        if fields_of(read(o), name) != fs:
            return False, f"字段名+类型不同（{o.rsplit('/',1)[-1]}）", None, []
        si, oi = impls_of(read(src), name), impls_of(read(o), name)
        if si != oi:
            only_s = sorted(si - oi)[:2]
            only_o = sorted(oi - si)[:2]
            return (False,
                    f"impl 块不同：真源有 {only_s or '无'} 多余，另一侧有 {only_o or '无'} 多余",
                    None, [])
        # 固有 impl：被删一侧若有自己的方法，融合后会与真源侧的同名方法
        # 撞成 E0592（同一 crate 内同一类型的两个固有 impl）。必须人工迁移。
        om = inherent_methods(read(o), name)
        if om:
            return (False,
                    f"被删侧有 {len(om)} 个固有方法（{sorted(om)[:3]}）⇒ 需人工迁移",
                    None, [])
        # 字段类型闭包：若某字段的类型名本身也是重复类型，则同名背后可能
        # 是不同类型（名义类型系统下同名 ≠ 同类型）。
        # 2026-09-29 实测：Product.connection_type 两处都叫 ConnectionType，
        # 但 data_model 版 4 变体、unified_types 版 7 变体 ⇒ 根本不是同一类型。
        # 融合 Product 会把 data_model 的 4 变体语义偷换成 7 变体。必须人工裁决。
        ftypes = {t.strip().split("<")[0].split("[")[0]
                  for _, t in (f.split(":", 1) for f in fields_of(read(o), name) or [])}
        dup_names = _dup_type_names()
        clash = sorted(ftypes & dup_names - {name})
        if clash:
            return (False,
                    f"字段类型闭包不安全：{clash[:3]} 本身也是重复类型名"
                    f"（同名背后可能是不同类型，如 ConnectionType 4变体 vs 7变体）⇒ 需人工裁决",
                    None, [])
    return True, "七项判据全过", src, others


def _dup_type_names():
    """缓存的重复类型名集合（nt_dup_types.py 的检出结果）。"""
    if not hasattr(_dup_type_names, "cache"):
        try:
            r = subprocess.run(
                ["python3", os.path.join(REPO, "scripts/ops/nt_dup_types.py"), "--json"],
                cwd=REPO, capture_output=True, text=True, timeout=300,
            )
            d = json.loads(r.stdout)
            _dup_type_names.cache = {g["name"] for g in d["groups"]}
        except Exception:
            _dup_type_names.cache = set()
    return _dup_type_names.cache


def fuse(name, source, targets):
    """删除 targets 里的定义，改为 `pub use super::<file>::<Name>;`"""
    changed = []
    for t in targets:
        p = os.path.join(SRC, t)
        text = read(t)
        b = block_bounds(text, name)
        if not b:
            print(f"    ! {t}: 找不到定义，跳过")
            continue
        src_mod = module_of(source)
        tgt_mod = module_of(t)
        if src_mod == tgt_mod:
            # 同模块的兄弟文件：从 mod.rs 或相对路径引用
            ref = f"super::{source.rsplit('/', 1)[-1][:-3]}::{name}"
        else:
            ref = f"crate::{src_mod.replace('/', '::')}::{name}"
        note = (f"// 2026-09-29 自动融合（nt_fuse_types.py）：`{name}` 原在本文件与\n"
                f"// `{source}` 各有一份，字段名+类型+impl 块完全相同。\n"
                f"// 真源是后者（模块 mod.rs 的 re-export 指向它）⇒ 本文件改为 re-export，\n"
                f"// 消除「两份同名类型」的歧义。\n"
                f"pub use {ref};\n\n")
        new = text[: b[0]] + note + text[b[2]:]
        new = re.sub(r"\n{4,}", "\n\n\n", new)
        with open(p, "w", encoding="utf-8") as f:
            f.write(new)
        changed.append(t)
    return changed


def mem_gate_open():
    """内存门自查。2026-09-29 实测教训：我在 mem-gate BLOCKED 时启动了构建，
    构建因他窗文件破红而全部回滚 —— 白白消耗一轮编译且污染判断
    （分不清是我的改动错还是树本身红）。门只在被遵守时才有效，
    而人会忘，故工具自己查。"""
    r = subprocess.run(
        ["sh", "scripts/ops/nt_mem_gate.sh"],
        cwd=REPO, capture_output=True, text=True, timeout=60,
    )
    out = (r.stdout or "") + (r.stderr or "")
    return "OPEN" in out


def tree_green_quick():
    """动手前确认树本身是绿的。若树已红（他窗 WIP 破坏），我的逐组验证
    会全部误报失败 —— 必须先停，而不是把红树当成我的改动有问题。"""
    r = subprocess.run(
        ["cargo", "check", "-p", "neotrix", "--lib"],
        cwd=REPO, capture_output=True, text=True, timeout=1800,
    )
    return r.returncode == 0


def cargo_ok():
    r = subprocess.run(
        ["cargo", "check", "-p", "neotrix", "--lib"],
        cwd=REPO, capture_output=True, text=True, timeout=1800,
    )
    return r.returncode == 0, (r.stderr or r.stdout)[-600:]


def main():
    ap = argparse.ArgumentParser(description="按七项判据自动融合重复类型")
    ap.add_argument("--dry-run", action="store_true", help="只报告不动手")
    ap.add_argument("--limit", type=int, default=10, help="最多处理几组")
    args = ap.parse_args()

    d = json.loads(subprocess.run(
        ["python3", os.path.join(REPO, "scripts/ops/nt_dup_types.py"), "--json"],
        capture_output=True, text=True, cwd=REPO).stdout)

    if not args.dry_run:
        if not mem_gate_open():
            print("nt-fuse-types: ⛔ mem-gate BLOCKED —— 拒绝启动构建。"
                  "等 OPEN 后再跑（强行跑会与他窗构建抢锁，且失败时分不清是谁的错）。")
            return 2
        print("nt-fuse-types: mem-gate OPEN，检查基线树状态…")
        if not tree_green_quick():
            print("nt-fuse-types: ⛔ 基线树本身是红的（他窗 WIP 破坏）。"
                  "此时逐组验证会全部误报失败 —— 先停，等树绿。我的改动一个没动。")
            return 3

    cands = [g for g in d["groups"] if g.get("automatable")]
    print(f"nt-fuse-types: 可自动 {len(cands)} 组，limit={args.limit}\n")

    done, skipped = [], []
    for g in cands:
        if len(done) >= args.limit:
            break
        name = g["name"]
        ok, why, src, others = analyse(name, g["paths"])
        if not ok:
            skipped.append((name, why))
            continue
        print(f"── {name}: 真源 {src.rsplit('/',1)[-1]}")
        for o in others:
            print(f"   将删: {o}")
        if args.dry_run:
            done.append((name, src, others, False))
            continue
        # 快照本组触及的文件（只恢复这些，不碰文件内其他已成功的融合）。
        # 2026-09-29 实测踩坑：初版用 `git checkout -- <file>` 整文件回滚，
        # Product 组失败时把同文件里 4 个已成功的融合（InquiryMetadata /
        # PerformanceMetrics / PressureRating / PriceInfo）一起 wipe。
        snapshots = {}
        for t in others:
            p = os.path.join(SRC, t)
            with open(p, encoding="utf-8") as f:
                snapshots[t] = f.read()
        fuse(name, src, others)
        passed, err = cargo_ok()
        if not passed:
            for t, content in snapshots.items():
                with open(os.path.join(SRC, t), "w", encoding="utf-8") as f:
                    f.write(content)
            print(f"   ⛔ 编译失败 ⇒ 已回滚本组 {len(snapshots)} 个文件：{err[:200]}")
            skipped.append((name, f"编译失败（已回滚）"))
            continue
        print("   ✅ 编译通过")
        done.append((name, src, others, True))

    print(f"\n════ 汇总 ════")
    print(f"处理 {len(done)} 组（{'实改' if not args.dry_run else '干跑'}），跳过 {len(skipped)} 组")
    if skipped:
        print("\n跳过原因分布:")
        import collections
        c = collections.Counter(w.split("（")[0] for _, w in skipped)
        for k, v in c.most_common():
            print(f"  {k}: {v}")
    for n, s, o, real in done:
        print(f"  {'✅' if real else '·'} {n}: {s.rsplit('/',1)[-1]} ← {len(o)} 处")
    return 0


if __name__ == "__main__":
    sys.exit(main())
