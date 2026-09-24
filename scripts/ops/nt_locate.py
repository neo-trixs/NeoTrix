#!/usr/bin/env python3
"""nt_locate — 选择器/组件名 → 代码点（文件:行）精准定位.

吸收 Agentation 思想：人类点选（selector/component/sourceFile）必须
落地到可微操作的精确代码点，而不是模糊描述。本工具分层定位：

  L1 sourceFile 直达：basename 全仓找文件，行内再定 token 行
  L2 组件/类 token grep：选择器拆 token，按覆盖度排名
  L3 兜底：selector 原串全文搜

输出按置信排序的 file:line:hit 行（给 agent 做微操作前读上下文用）。
只读不写，零依赖（rg 优先，fallback grep）。

用法:
  python3 scripts/ops/nt_locate.py --selector ".sidebar > button.primary" --root .
  python3 scripts/ops/nt_locate.py --component CrystalIterState --source-file handlers_crystal.rs --root .
  python3 scripts/ops/nt_locate.py --selftest
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys

RG = shutil.which("rg")
TEXT_EXTS = {".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".vue", ".svelte",
            ".html", ".css", ".scss", ".json", ".toml", ".yaml", ".yml", ".md",
            ".sh", ".sql", ".ron", ".nt"}
DEFAULT_INDEX = ".project-map/codemap.json"


def load_index(root):
    for cand in (os.path.join(root, DEFAULT_INDEX), DEFAULT_INDEX):
        if os.path.isfile(cand):
            try:
                with open(cand, encoding="utf-8") as fh:
                    return json.load(fh), cand
            except (OSError, ValueError):
                pass
    return None, None


def locate_index(doc, selector="", component="", source_file="", limit=15):
    """L0: 索引直查（毫秒级）。basename 全等 > item/modpath 包含 > token 覆盖。"""
    hits = []
    seen = set()

    def add(score, path, text):
        if path in seen:
            return
        seen.add(path)
        hits.append((score, path, 0, text[:160]))

    files = doc.get("files", [])
    toks = ([component] if component else []) + tokens_of(selector)
    toks = [t for t in toks if t][:6]
    base = os.path.basename(source_file) if source_file else ""

    if base:
        for f in files:
            if os.path.basename(f["path"]) == base:
                add(100, f["path"], "<index-file> area=%s loc=%d" % (f["area"], f["loc"]))
                # 行内定点：items 命中
                for tok in ([component] + toks)[:4]:
                    if not tok:
                        continue
                    low = tok.lower()
                    for item in f.get("items", []):
                        if low in item.lower():
                            add(95, f["path"], "<index-item> " + item)
                            break
    if component:
        low = component.lower()
        for f in files:
            if any(low in (it or "").lower() for it in f.get("items", [])):
                add(90, f["path"], "<index-item> %s" % component)
            elif f.get("modpath") and low in f["modpath"].lower():
                add(80, f["path"], "<index-mod> " + (f["modpath"] or ""))
    if toks and len(hits) < limit:
        for f in files:
            blob = " ".join(f.get("items", [])) + " " + (f.get("modpath") or "") + " " + f["path"]
            low = blob.lower()
            c = sum(1 for t in toks if t.lower() in low)
            if c >= max(1, len(toks) - 1):
                add(50 + c, f["path"], "<index-cover %d/%d>" % (c, len(toks)))
    hits.sort(key=lambda h: -h[0])
    return [{"score": s, "file": f, "line": ln, "hit": t} for s, f, ln, t in hits[:limit]]


def audit_index(doc, root):
    """完整性审计：文件系统 vs 索引。返回 (missing, extra, rs_cov)。"""
    have = set()
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames
                       if d not in ("target", ".git", "node_modules", "thirdparty",
                                    "__pycache__", ".venv", "dist", "build", "models")
                       and not (d.startswith(".") and d != ".github")]
        for fn in filenames:
            if os.path.splitext(fn)[1] in TEXT_EXTS:
                have.add(os.path.relpath(os.path.join(dirpath, fn), root))
    indexed = set(f["path"] for f in doc.get("files", []))
    missing = sorted(have - indexed)
    extra = sorted(indexed - have)
    rs_have = sorted(p for p in have if p.endswith(".rs"))
    rs_hit = [p for p in rs_have if p in indexed]
    return missing, extra, (len(rs_hit), len(rs_have))


def tokens_of(selector):
    parts = re.split(r"[.#>\s+~:\[\]()\"'=,]+", selector or "")
    return [t for t in parts if len(t) >= 2]


def run_grep(pattern, root, files_with_matches=False):
    if RG:
        cmd = [RG, "-n", "--no-heading", "--glob", "!target/**",
               "--glob", "!.git/**", "--glob", "!node_modules/**",
               "--glob", "!thirdparty/**", pattern, root]
        if files_with_matches:
            cmd[3:3] = ["-l"]
    else:
        cmd = ["grep", "-rn", "--exclude-dir=target", "--exclude-dir=.git",
               pattern, root]
    try:
        p = subprocess.run(cmd, capture_output=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired):
        return []
    return p.stdout.decode("utf-8", errors="ignore").splitlines()


def locate(selector="", component="", source_file="", root=".", limit=15):
    hits = []  # (score, file, line, text)
    seen = set()

    def add(score, file, line, text):
        key = (file, line)
        if key in seen:
            return
        seen.add(key)
        hits.append((score, file, line, text.strip()[:160]))

    # L1: sourceFile 直达
    if source_file:
        base = os.path.basename(source_file)
        for line in run_grep(re.escape(base), root, files_with_matches=True):
            f = line.strip()
            if not f or not os.path.splitext(f)[1] in TEXT_EXTS:
                pass
            if base in os.path.basename(f):
                add(100, f, 0, f"<file-match> {base}")
                # 行内再定：用 component/selector token 定行
                for tok in ([component] + tokens_of(selector))[:4]:
                    if not tok:
                        continue
                    for gl in run_grep(re.escape(tok), f):
                        m = re.match(r"^(.*?):(\d+):(.*)$", gl)
                        if m:
                            add(90, m.group(1), int(m.group(2)), m.group(3))

    # L2: token 覆盖度排名
    toks = ([component] if component else []) + tokens_of(selector)
    toks = [t for t in toks if t][:6]
    if toks and len(hits) < limit:
        per_tok = {}
        for tok in toks:
            files = set()
            for gl in run_grep(re.escape(tok), root, files_with_matches=True):
                f = gl.strip()
                if f:
                    files.add(f)
            per_tok[tok] = files
        from collections import Counter
        cov = Counter()
        for tok, files in per_tok.items():
            for f in files:
                cov[f] += 1
        for f, c in cov.most_common(limit):
            if c >= max(1, len(toks) - 1):
                add(50 + c, f, 0, f"<token-cover {c}/{len(toks)}>")

    # L3: 原串兜底
    if selector and len(hits) < 3:
        for gl in run_grep(re.escape(selector), root)[:limit]:
            m = re.match(r"^(.*?):(\d+):(.*)$", gl)
            if m:
                add(10, m.group(1), int(m.group(2)), m.group(3))

    hits.sort(key=lambda h: -h[0])
    return [{"score": s, "file": f, "line": ln, "hit": t} for s, f, ln, t in hits[:limit]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selector", default="")
    ap.add_argument("--component", default="")
    ap.add_argument("--source-file", default="")
    ap.add_argument("--root", default=".")
    ap.add_argument("--limit", type=int, default=15)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--index", default="",
                    help="codemap.json 路径（默认自动找 .project-map/codemap.json）；"
                         "--index=off 强制走 grep")
    ap.add_argument("--audit", action="store_true",
                    help="完整性审计：文件系统 vs 索引覆盖率")
    args = ap.parse_args()

    if args.audit:
        doc, found = load_index(args.root)
        if not doc:
            print("no index found; run: python3 scripts/ops/nt_mapgen.py", flush=True)
            raise SystemExit(1)
        missing, extra, (hit, total) = audit_index(doc, os.path.abspath(args.root))
        print(f"[audit] index={found} rs_coverage={hit}/{total} "
              f"missing={len(missing)} extra={len(extra)}", flush=True)
        for p in missing[:20]:
            print(f"  MISSING {p}", flush=True)
        for p in extra[:20]:
            print(f"  EXTRA {p}", flush=True)
        raise SystemExit(0 if not missing else 1)

    if args.selftest:
        root = os.getcwd()
        r = locate(component="CrystalIterState",
                   source_file="handlers_crystal.rs", root=root)
        assert r, "selftest: no hits"
        top = r[0]
        assert top["file"].endswith("handlers_crystal.rs"), top
        assert top["score"] >= 90, top
        print(f"[selftest] OK top={top['file']}:{top['line']} score={top['score']}", flush=True)
        return

    if not (args.selector or args.component or args.source_file):
        print("need --selector/--component/--source-file", flush=True)
        raise SystemExit(2)
    # 索引优先（毫秒级），无索引/命中不足回落 grep
    use_index = args.index != "off"
    doc = None
    if use_index:
        if args.index:
            try:
                with open(args.index, encoding="utf-8") as fh:
                    doc = json.load(fh)
            except (OSError, ValueError):
                doc = None
        else:
            doc, _ = load_index(args.root)
    out = locate_index(doc, args.selector, args.component,
                       args.source_file, args.limit) if doc else []
    if len(out) < 3:
        out = out + [h for h in locate(args.selector, args.component,
                                       args.source_file, args.root, args.limit)
                     if h["file"] not in {x["file"] for x in out}][:args.limit]
        out = out[:args.limit]
    if args.json:
        print(json.dumps(out, ensure_ascii=False, indent=1))
    else:
        for h in out:
            ln = f":{h['line']}" if h["line"] else ""
            print(f"[{h['score']}] {h['file']}{ln} :: {h['hit']}", flush=True)
        if not out:
            print("(no hits)", flush=True)


if __name__ == "__main__":
    main()
