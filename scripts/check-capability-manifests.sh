#!/bin/bash
# check-capability-manifests.sh — 能力 manifest 一致性门（报告式，默认 exit 0）
#
# 吸收源（思想，非代码）: QwenLM/Qwen-MM-Plugins (Apache-2.0) 的
# `scripts/check_manifests.py` + `CLAUDE.md` "Release invariants"：
# 每个能力的 plugin-versions.json ↔ harness manifests ↔ MCP 包引用 ↔
# 服务器 __version__ 必须一致，"Never move a published tag"。
# NeoTrix 的对等物是 skills/index.json（manifest）↔ SKILL.md frontmatter
# （能力声明）——此前只有 skill-gate 查"门禁字段是否齐"，没人查
# "两边写的是不是同一个能力"。本门补这一刀。
#
# 检查项：
#   M1  skill_index 指向的 SKILL.md 存在
#   M2  frontmatter name == index key 后缀（`cat/skill` 的 skill 部分）
#   M3  frontmatter description == index description（逐字）
#   M4  frontmatter name 全库唯一
#   M5  index.json 有 version + generated
#
# 用法: bash scripts/check-capability-manifests.sh [--strict] [skills/index.json]
#   默认    : exit 0, 打印 mismatch 清单（供日常可见）
#   --strict: 有 FAIL 时 exit 1（供 CI 新 skill 增量门；存量 mismatch 先清零再接）
#
# ⚠️ 基线说明：本门 2026-09-28 新立，无历史基线。首次跑出的 mismatch 数
#    即存量（多为"index 描述是中文、frontmatter 描述是英文"的双语漂移），
#    不要当场"修齐"——先记录，后续按 skill 归属逐个认领（批量改描述是 churn）。

set -uo pipefail

STRICT=0
INDEX="skills/index.json"
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    -h|--help) sed -n '2,26p' "$0"; exit 0 ;;
    *) INDEX="$arg" ;;
  esac
done

if [ ! -f "$INDEX" ]; then
  echo "capability-manifests: index not found: $INDEX"
  exit 2
fi

python3 - "$INDEX" "$STRICT" <<'PYEOF'
import json, os, re, sys
from collections import Counter

index_path, strict = sys.argv[1], sys.argv[2] == "1"
with open(index_path, encoding="utf-8") as f:
    index = json.load(f)

fails, warns = [], []
FM_RE = re.compile(r"\A---\r?\n(.*?)\r?\n---", re.S)

def frontmatter(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            head = fh.read(4096)
    except OSError as e:
        return (False, {}, str(e))
    m = FM_RE.match(head)
    if not m:
        return (False, {}, "no frontmatter")
    fields = {}
    for line in m.group(1).splitlines():
        mm = re.match(r"^([A-Za-z_][A-Za-z0-9_-]*)\s*:\s*(.*)$", line)
        if mm:
            fields[mm.group(1)] = mm.group(2).strip().strip("'\"")
    return (True, fields, "")

# M5
for k in ("version", "generated"):
    if k not in index:
        fails.append(f"M5: index.json missing '{k}'")

root = os.path.dirname(os.path.abspath(index_path)) or "."
names_seen = Counter()
checked = 0

for label, loc in index.get("skill_index", {}).items():
    rel = loc.get("file", "")
    p = os.path.join(root, rel)
    want_name = label.split("/")[-1]
    # M1
    if not os.path.isfile(p):
        fails.append(f"M1: {label}: file missing: {rel}")
        continue
    has_fm, fm, why = frontmatter(p)
    if not has_fm:
        warns.append(f"{label}: no frontmatter ({why}) — invisible to spec-compliant agents")
        continue
    checked += 1
    got_name = fm.get("name", "")
    names_seen[got_name] += 1
    # M2
    if got_name != want_name:
        fails.append(f"M2: {label}: frontmatter name={got_name!r} != key suffix {want_name!r}")
    # M3
    cat, _, sk = label.partition("/")
    idx_desc = ((index.get("categories", {}).get(cat, {}).get("skills", {}).get(sk, {}).get("description")) or "").strip()
    fm_desc = (fm.get("description") or "").strip()
    if idx_desc and fm_desc and idx_desc != fm_desc:
        fails.append(f"M3: {label}: description drift (index {len(idx_desc)}ch vs file {len(fm_desc)}ch)")

# M4
for name, n in names_seen.items():
    if n > 1:
        fails.append(f"M4: duplicate frontmatter name {name!r} x{n}")

print(f"capability-manifests: checked={checked} fails={len(fails)} warns={len(warns)}")
for w in warns[:10]:
    print(f"  WARN {w}")
if len(warns) > 10:
    print(f"  ... and {len(warns) - 10} more warns")
for fl in fails[:30]:
    print(f"  FAIL {fl}")
if len(fails) > 30:
    print(f"  ... and {len(fails) - 30} more fails")

sys.exit(1 if (strict and fails) else 0)
PYEOF
