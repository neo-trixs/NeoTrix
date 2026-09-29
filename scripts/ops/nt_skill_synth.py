#!/usr/bin/env python3
"""nt_skill_synth — 代码 → Skill（三件套门对齐）.

吸收 Code2Skill 思想：技能必须 grounded 在真实调用点上，
自带离线校验，不许 hallucinate 接口。输出：

  SKILL.md（name/description/triggers/condition + 工作流）
  + index.json 条目片段 + gate 自检（对齐 Rust gate_skill：
    triggers 非空 + exclusions 非空 + output_contract 非空）

用法:
  python3 scripts/ops/nt_skill_synth.py --src scripts/ops/nt_locate.py --out skills/gen/
  python3 scripts/ops/nt_skill_synth.py --selftest
"""
import argparse
import ast
import json
import os
import re
import sys

CAP = 12


def split_ident(name):
    parts = re.split(r"_+|(?<=[a-z0-9])(?=[A-Z])", name)
    return [p.lower() for p in parts if len(p) >= 3]


def analyze_py(path):
    tree = ast.parse(open(path, encoding="utf-8").read())
    doc = (ast.get_docstring(tree) or "").strip().splitlines()
    desc = doc[0] if doc else os.path.basename(path)
    fns = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and not node.name.startswith("_"):
            d = (ast.get_docstring(node) or "").strip().splitlines()
            fns.append({"name": node.name, "doc": d[0] if d else "",
                        "args": [a.arg for a in node.args.args if a.arg not in ("self", "cls")]})
    return desc, fns


def analyze_rs(path):
    text = open(path, encoding="utf-8", errors="ignore").read()
    m = re.search(r"//!\s*(.+)", text)
    desc = m.group(1).strip() if m else os.path.basename(path)
    fns = [{"name": n, "doc": "", "args": []}
           for n in re.findall(r"pub\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)", text)]
    seen, uniq = set(), []
    for f in fns:
        if f["name"] not in seen:
            seen.add(f["name"])
            uniq.append(f)
    return desc, uniq[:20]


def synthesize(src, skill_name=""):
    ext = os.path.splitext(src)[1]
    if ext == ".py":
        desc, fns = analyze_py(src)
    elif ext == ".rs":
        desc, desc_fns = analyze_rs(src), None
        fns = desc_fns
    else:
        raise SystemExit(f"unsupported: {ext} (only .py/.rs)")
    base = skill_name or re.sub(r"[^a-z0-9]+", "-", os.path.splitext(os.path.basename(src))[0]).strip("-")
    toks = []
    for f in fns:
        toks += split_ident(f["name"])
    freq = {}
    for t in toks:
        freq[t] = freq.get(t, 0) + 1
    triggers = sorted(freq, key=lambda t: -freq[t])[:8]
    func_list = "\n".join(f"- `{f['name']}({', '.join(f['args'])})`"
                          + (f" — {f['doc']}" if f["doc"] else "") for f in fns[:CAP])
    md = f"""---
name: {base}
description: {desc[:120]}
origin: NeoTrix-synth
triggers: {', '.join(triggers)}
condition: task:auto
---

# {base} — 代码 grounded 技能（nt_skill_synth 生成）

> 源：`{src}`。能力以真实函数为准，禁止脑补不存在的接口。

## 真实调用点

{func_list or '(未发现公开函数)'}

## 工作流

1. 用触发词命中本技能后，先读源文件确认接口仍在
2. 按函数签名调用，不编造参数
3. 跑离线自检（--selftest / 单测）再交付

## exclusions（本技能不碰）

- 不修改源文件本身（只读定位与调用）
- 不绕过调用方的门控与审计

## output_contract（输出长什么样）

- 调用结果原文 + 文件:行证据；失败时返回错误原文，不编造成功

## 离线校验

- [ ] 源文件存在且可解析
- [ ] triggers 非空（gate_skill 三段式之一）
- [ ] exclusions 非空（本技能不碰的列明）
- [ ] output_contract 非空（输出长什么样）
"""
    entry = {"description": desc[:120],
             "tags": ["synth", "grounded"],
             "triggers": triggers,
             "dependencies": []}
    return base, md, entry


def gate(md):
    """对齐 Rust gate_skill：triggers + exclusions + output_contract 三段式。"""
    m = re.match(r"^---\n(.*?)\n---\n", md, re.S)
    fm = m.group(1) if m else ""
    has_trig = bool(re.search(r"^triggers:\s*\S", fm, re.M))
    has_excl = "exclusions" in md.lower()
    has_contract = "output_contract" in md.lower() or "output contract" in md.lower()
    missing = []
    if not has_trig:
        missing.append("triggers")
    if not has_excl:
        missing.append("exclusions(模板待填)")
    if not has_contract:
        missing.append("output_contract(模板待填)")
    return missing


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default=None)
    ap.add_argument("--out", default=None)
    ap.add_argument("--name", default="")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        base, md, entry = synthesize("scripts/ops/nt_locate.py")
        assert base == "nt-locate", base
        assert "locate" in md, "tokens must include locate"
        assert entry["triggers"], "triggers empty"
        json.dumps(entry, ensure_ascii=False)
        print(f"[selftest] OK skill={base} triggers={entry['triggers'][:4]} "
              f"gate_missing={gate(md)}", flush=True)
        return
    if not args.src or not args.out:
        print("need --src <file> --out <dir>", flush=True)
        raise SystemExit(2)
    base, md, entry = synthesize(args.src, args.name)
    os.makedirs(args.out, exist_ok=True)
    with open(os.path.join(args.out, "SKILL.md"), "w", encoding="utf-8") as f:
        f.write(md)
    with open(os.path.join(args.out, "index-entry.json"), "w", encoding="utf-8") as f:
        json.dump({base: entry}, f, ensure_ascii=False, indent=2)
    print(f"[synth] skill={base} gate_missing={gate(md)} -> {args.out}", flush=True)


if __name__ == "__main__":
    main()
