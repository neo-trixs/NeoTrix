#!/usr/bin/env python3
"""nt_mapgen — 全域精细 code map 生成器.

扫描全部源码根, 输出 `.project-map/codemap.json`:
  files[]: {path, lang, loc, area, layer, modpath, items[], bins}
  roots[]: 每个根的统计 {root, files, loc}
  meta: {generated_at, tool, coverage: {rs_total, rs_indexed}}

area 划分: neotrix-core / crates/<name> / tauri / frontend / game /
          ntos / scripts / skills / workflows / docs / other
layer: l0_substrate..l6_meta (core 内) / crate 名 / app(frontend,tauri-cmd...) /
       bin / test 由路径判定, 存多值 tags[].
modpath: Rust crate:: 路径推导 (Cargo.toml package 名 + 相对路径)。
items: pub struct/enum/trait/fn/mod/type (+ TS: export function/class/const/
       interface/type/enum)。只读不写，零依赖。

用法: python3 scripts/ops/nt_mapgen.py [--root .] [--out .project-map/codemap.json]
"""
import json
import os
import re
import sys

RS_ITEM = re.compile(
    r"^\s*pub\s+(?:unsafe\s+)?(struct|enum|trait|fn|mod|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
RS_MOD = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)")
TS_ITEM = re.compile(
    r"^\s*export\s+(?:default\s+)?(function|class|const|interface|type|enum)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
PY_ITEM = re.compile(r"^\s*(?:async\s+)?(?:def|class)\s+([A-Za-z_][A-Za-z0-9_]*)")
CARGO_NAME = re.compile(r'^\s*name\s*=\s*"([^"]+)"')

SKIP_DIRS = {"target", ".git", "node_modules", "thirdparty", "__pycache__",
             ".venv", "dist", "build", ".next", "coverage", "models"}
SKIP_FILES = {"Cargo.lock"}
LANGS = {".rs": "rs", ".ts": "ts", ".tsx": "tsx", ".js": "js", ".jsx": "jsx",
         ".py": "py", ".sh": "sh", ".md": "md", ".toml": "toml", ".yaml": "yaml",
         ".yml": "yml", ".json": "json", ".sql": "sql", ".ron": "ron", ".nt": "nt",
         ".html": "html", ".css": "css", ".scss": "scss", ".vue": "vue"}

ROOT_AREAS = [
    ("neotrix-core/src", "neotrix-core"),
    ("neotrix-core/tests", "neotrix-core"),
    ("neotrix-core/benches", "neotrix-core"),
    ("neotrix-core/examples", "neotrix-core"),
    ("crates", "crates"),
    # 2026-09-28 修正 6 条死引用（`nt_scan_surface.py` 定位）：
    #   src-tauri/src          → 桌面端随 5c02e738 归档（599 files）
    #   src-tauri/frontend     → 同上
    #   src-tauri/frontend/src → 同上
    #   games                  → 随 2bbed32c 归档，现居 neotrix-archive/games/neotrix-guixu
    #   fuzz                   → 随 2bbed32c 归档，现居 neotrix-archive/fuzz
    #   ntos/src               → 该目录从来不存在（代码里的 "ntos" 只是普通词，
    #                            见 bin/nt_douyin_ingest.rs 等）
    # 桌面端代码现居 `crates/neotrix-neobot`（已由 `("crates", "crates")` 覆盖，
    # 故此处不重复列出；单列一个 neobot 条目只为让 area 标注可读）。
    ("crates/neotrix-neobot", "neobot"),
    ("sessions", "sessions"),
    ("scripts", "scripts"),
    ("skills", "skills"),
    (".github", "workflows"),
    ("docs", "docs"),
]


def area_of(rel):
    for prefix, area in ROOT_AREAS:
        if rel == prefix or rel.startswith(prefix + "/"):
            return area, rel[len(prefix):].strip("/")
    return "other", rel


def layer_of(area, sub):
    tags = []
    m = re.search(r"(l[0-6]_[a-z_]+)", sub)
    if m:
        tags.append(m.group(1))
    parts = sub.split("/") if sub else []
    if area == "crates" and parts:
        tags.append("crate:" + parts[0])
    if area == "tauri" and parts and parts[0] == "commands":
        tags.append("tauri-cmd")
    if area == "tauri" and parts and parts[0] == "domain":
        tags.append("tauri-domain")
    if "/bin/" in sub or sub.startswith("bin/"):
        tags.append("bin")
    if "test" in sub.split("/"):
        tags.append("test")
    if "bench" in sub.split("/"):
        tags.append("bench")
    if area == "frontend":
        tags.append("app")
    return tags


_cargo_cache = {}


def crate_name(root, d):
    if d in _cargo_cache:
        return _cargo_cache[d]
    name = None
    cur = d
    while True:
        mf = os.path.join(root, cur, "Cargo.toml")
        if os.path.isfile(mf):
            try:
                with open(mf, encoding="utf-8", errors="ignore") as fh:
                    in_pkg = False
                    for line in fh:
                        s = line.strip()
                        if s == "[package]":
                            in_pkg = True
                        elif s.startswith("[") and in_pkg:
                            break
                        elif in_pkg:
                            m = CARGO_NAME.match(line)
                            if m:
                                name = m.group(1)
                                break
                    if name:
                        break
            except OSError:
                pass
        parent = os.path.dirname(cur)
        if parent == cur or parent == "":
            break
        cur = parent
    _cargo_cache[d] = name
    return name


def modpath_of(root, rel, lang):
    if lang != "rs":
        return None
    d = os.path.dirname(rel)
    cname = crate_name(root, d)
    if not cname:
        return None
    base = os.path.basename(rel)
    stem = base[:-3]
    sub = os.path.relpath(d, os.path.join(root, _crate_src_dir(root, d)))
    parts = [] if sub in (".", "") else sub.split(os.sep)
    if stem != "mod" and stem != "lib" and stem != "main":
        parts.append(stem)
    mod = cname.replace("-", "_") + ("::" + "::".join(parts) if parts else "")
    return mod


def _crate_src_dir(root, d):
    cur = d
    while True:
        if os.path.isfile(os.path.join(root, cur, "Cargo.toml")):
            # src/ 优先, 无则 bin/ 所在
            if os.path.isdir(os.path.join(root, cur, "src")):
                return os.path.join(cur, "src")
            return cur
        parent = os.path.dirname(cur)
        if parent == cur or parent == "":
            return d
        cur = parent


def items_of(path, lang):
    items = []
    try:
        with open(path, encoding="utf-8", errors="ignore") as fh:
            for line in fh:
                if lang == "rs":
                    m = RS_ITEM.match(line)
                    if m:
                        items.append(m.group(1) + " " + m.group(2))
                        continue
                    m = RS_MOD.match(line)
                    if m and len(items) < 400:
                        items.append("mod " + m.group(1))
                elif lang in ("ts", "tsx", "js", "jsx"):
                    m = TS_ITEM.match(line)
                    if m:
                        items.append(m.group(1) + " " + m.group(2))
                elif lang == "py":
                    m = PY_ITEM.match(line)
                    if m:
                        items.append("def " + m.group(1))
                if len(items) >= 400:
                    break
    except OSError:
        pass
    return items


def main():
    root = sys.argv[sys.argv.index("--root") + 1] if "--root" in sys.argv else "."
    out = sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else ".project-map/codemap.json"
    root = os.path.abspath(root)
    files = []
    for dirpath, dirnames, filenames in os.walk(root):
        keep = []
        for d in sorted(dirnames):
            if d in SKIP_DIRS:
                continue
            if d.startswith(".") and d != ".github":
                continue
            keep.append(d)
        dirnames[:] = keep
        for fn in sorted(filenames):
            ext = os.path.splitext(fn)[1]
            if ext not in LANGS or fn in SKIP_FILES:
                continue
            full = os.path.join(dirpath, fn)
            rel = os.path.relpath(full, root)
            if rel.startswith(".."):
                continue
            lang = LANGS[ext]
            try:
                with open(full, "rb") as fh:
                    loc = sum(1 for _ in fh)
                broken = False
            except OSError:
                if not os.path.islink(full):
                    continue
                loc, broken = 0, True
            area, sub = area_of(rel)
            tags = layer_of(area, sub)
            if broken:
                tags.append("symlink-broken")
            files.append({
                "path": rel,
                "lang": lang,
                "loc": loc,
                "area": area,
                "tags": layer_of(area, sub),
                "modpath": modpath_of(root, rel, lang),
                "items": items_of(full, lang) if lang in ("rs", "ts", "tsx", "js", "jsx", "py") else [],
            })
    roots = {}
    for f in files:
        r = roots.setdefault(f["area"], {"files": 0, "loc": 0})
        r["files"] += 1
        r["loc"] += f["loc"]
    rs_total = sum(1 for f in files if f["lang"] == "rs")
    doc = {
        "meta": {
            "tool": "nt_mapgen.py v1",
            "files": len(files),
            "coverage": {"rs_total": rs_total, "rs_indexed": rs_total},
        },
        "roots": roots,
        "files": files,
    }
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(doc, fh, ensure_ascii=False, indent=1, sort_keys=False)
    print(f"[mapgen] {len(files)} files -> {out}")
    for area in sorted(roots):
        print(f"  {area}: {roots[area]['files']} files, {roots[area]['loc']} loc")


if __name__ == "__main__":
    main()
