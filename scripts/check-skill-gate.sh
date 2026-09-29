#!/usr/bin/env bash
# check-skill-gate.sh — Skill 市场入库门禁 (P0-2 三段式 + awesome 领养清单)
#
# 吸收源（思想，非代码）:
# - yibie/awesome-autoresearch "Curation is not endorsement" + 领养前检查表
#   （loop 可跑否 / 有无实证 / 数字有源否 / 试用成本 / 有无 license）
# - sindresorhus/awesome 治理机制（inclusion 规则 + lint 门禁）
# - repowise PR Bot 哲学：绿则静默，有问题才列清单
#
# 用法: check-skill-gate.sh [--strict] [--adopt] [skills/index.json]
#   默认 advisory 模式：只报告，exit 0（不挡现有存量）。
#   --strict：门禁项任一失败即 exit 1（供 CI 新 skill 增量门）。
#   --adopt：只查**待接入**的 SKILL.md（frontmatter 缺失），供新 skill 增量门用。
#
# 2026-09-28 升级：门此前**只看 index.json，看不见磁盘上真实存在的 SKILL.md**。
# 而真正有判别力的门在文件侧：frontmatter（官方 Agent Skills 规范的发现依据）
# 覆盖不足。故门改为**双侧**校验：
#   ① index 侧：description / triggers / exclusions / output_contract / license
#   ② 文件侧：SKILL.md 存在 + frontmatter 完整（name/description/when_to_use）
#
# 2026-09-29 修正三处**已失效的数字**（实测，见 ABSORPTION-AGENT-ARCH2-2026-09-29.md §4）。
# 这三处是「门记录里的数字本身会陈旧」的活样本 —— 它们不在 nt_manifest.py 的
# 覆盖范围内（那门查 `file:line` 有效性，**不查数字**）：
#   - 上面写的「67 个真实 SKILL.md」  -> 实测磁盘 **68** 个
#   - 「只有 24/67 有 frontmatter」   -> 实测 **34/68**
#   - 「58 条存量」                   -> 门自身报的是 **59**（= categories.*.skills 之和，
#                                        即 SkillLoader::list_skills() 能看到的那部分；
#                                        skill_index 有 60 条，差 1 条是顶层 skills/SKILL.md）
# 修法不是把数字改对就完事 —— 是**每次改这个文件都重跑一遍**：
#   find skills -name SKILL.md | wc -l                                    # 磁盘
#   python3 -c "import json;d=json.load(open('skills/index.json'));\
#     print(len(d['skill_index']), sum(len(v.get('skills',[])) for v in d['categories'].values()))"
#   bash scripts/check-skill-gate.sh | head -2
set -euo pipefail

STRICT=0
ADOPT=0
INDEX="skills/index.json"
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --adopt) ADOPT=1 ;;
    *) INDEX="$arg" ;;
  esac
done

if [ ! -f "$INDEX" ]; then
  echo "skill-gate: index not found: $INDEX"
  exit 2
fi

python3 - "$INDEX" "$STRICT" "$ADOPT" <<'PYEOF'
import json, os, re, sys

index_path, strict, adopt_only = sys.argv[1], sys.argv[2] == "1", sys.argv[3] == "1"
with open(index_path) as f:
    index = json.load(f)

fails, total, three_part = [], 0, 0
LIC_MISSING = []
FM_MISSING = []       # SKILL.md 缺 frontmatter
FM_FIELD_MISSING = [] # frontmatter 有但缺必填字段
NO_FILE = []          # index 指向的 SKILL.md 不存在
BROKEN_LINKS = []     # SKILL.md 内的相对链接指向不存在的路径

# ---- 文件侧 frontmatter 解析（不引第三方依赖，容忍 YAML 不完整）----
FM_RE = re.compile(r"\A---\r?\n(.*?)\r?\n---", re.S)
REQUIRED_FM = ("name", "description")

def parse_frontmatter(path):
    """Return (has_frontmatter, {key: value} of top-level scalars)."""
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            head = fh.read(4096)
    except OSError:
        return (False, {})
    m = FM_RE.match(head)
    if not m:
        return (False, {})
    fields = {}
    for line in m.group(1).splitlines():
        mm = re.match(r"^([A-Za-z_][A-Za-z0-9_-]*)\s*:\s*(.*)$", line)
        if mm:
            fields[mm.group(1)] = mm.group(2).strip().strip("'\"")
    return (True, fields)

skills_root = os.path.dirname(os.path.abspath(index_path)) or "."

for cat_name, cat in index.get("categories", {}).items():
    for skill_name, s in cat.get("skills", {}).items():
        total += 1
        label = f"{cat_name}/{skill_name}"
        desc = (s.get("description") or "").strip()
        triggers = s.get("triggers") or []
        exclusions = s.get("exclusions") or []
        contract = (s.get("output_contract") or "").strip()
        if not desc:
            fails.append(f"{label}: FAIL empty description")
        if not [t for t in triggers if str(t).strip()]:
            fails.append(f"{label}: FAIL empty triggers")
        missing = []
        if not [t for t in exclusions if str(t).strip()]:
            missing.append("exclusions")
        if not contract:
            missing.append("output_contract")
        if missing:
            fails.append(f"{label}: GATE missing {','.join(missing)}")
        else:
            three_part += 1
        # awesome 领养清单：license（仅 advisory）
        if "license" not in s:
            LIC_MISSING.append(label)

        # ---- 文件侧：解析 skill_index 指向的真实 SKILL.md ----
        # ---- 文档侧：SKILL.md 里的相对链接是否指向真实路径 ----
        # 2026-09-28 加：`skills/SKILL.md` 曾列 `src-tauri/`（已随 5c02e738 删除）
        # 一年无人发现。分类表里的死链是**导航腐烂**的入口 —— 照着点进去才发现
        # 目录不存在。判据同上：**路径不存在 ≠ 该面为空**。
        idx_entry = index.get("skill_index", {}).get(label) \
                    or index.get("skill_index", {}).get(skill_name)
        rel = (idx_entry or {}).get("file")
        if not rel:
            continue
        path = os.path.join(skills_root, rel)
        if not os.path.isfile(path):
            NO_FILE.append(label)
            continue
        has_fm, fields = parse_frontmatter(path)
        if not has_fm:
            FM_MISSING.append(label)
        else:
            absent = [k for k in REQUIRED_FM if not fields.get(k)]
            if absent:
                FM_FIELD_MISSING.append(f"{label}: {','.join(absent)}")

# frontmatter 缺失属**存量欠账**，advisory；--adopt 只在"新接入"语境报错。
if adopt_only:
    fails.extend(f"{l}: ADOPT no-frontmatter" for l in FM_MISSING)
else:
    for f in FM_FIELD_MISSING:
        fails.append(f"{f}: FAIL frontmatter missing fields")

# ---- 文档侧：扫**整棵 skills 树**的 SKILL.md 相对链接 ----
# 2026-09-28：首版只在 index 覆盖的条目里查，而 `skills/SKILL.md`（顶层导航，
# 含全部分类的链接表）**不在 index.json 里** ⇒ 循环漏掉它，死链照样过。
# 教训与 nt_scan_surface 同源：检查的覆盖面本身也会漂。
import glob as _glob
for _md in _glob.glob(os.path.join(skills_root, "**", "SKILL.md"), recursive=True):
    try:
        with open(_md, encoding="utf-8", errors="replace") as _fh:
            _body = _fh.read()
    except OSError:
        continue
    _dir = os.path.dirname(_md)
    for _lm in re.finditer(r"\[([^\]]+)\]\(([^)]+)\)", _body):
        _t = _lm.group(2).split("#")[0].strip()
        if not _t or _t.startswith(("http://", "https://", "mailto:")):
            continue
        if not os.path.exists(os.path.normpath(os.path.join(_dir, _t))):
            BROKEN_LINKS.append(
                f"{os.path.relpath(_md, skills_root)} -> {_t}")

print(f"skill-gate: {total} skills, {three_part} three-part complete, "
      f"{len(fails)} gate findings, {len(LIC_MISSING)} missing license(advisory)")
print(f"skill-gate: file-side — {len(NO_FILE)} index-path missing, "
      f"{len(FM_MISSING)}/{total} SKILL.md without frontmatter (advisory), "
      f"{len(FM_FIELD_MISSING)} frontmatter incomplete, "
      f"{len(BROKEN_LINKS)} broken doc links")
for line in fails[:20]:
    print("  " + line)
if len(fails) > 20:
    print(f"  ... and {len(fails) - 20} more")
if FM_MISSING:
    _fm_ex = ", ".join(FM_MISSING[:3])
    print(f"  advisory: {len(FM_MISSING)} SKILL.md lack frontmatter ⇒ "
          f"在 Agent Skills 规范的 agent 里不可自动发现 (e.g. {_fm_ex})")
if BROKEN_LINKS:
    _bl_ex = "; ".join(BROKEN_LINKS[:5])
    print(f"  FAIL: {len(BROKEN_LINKS)} SKILL.md 内的相对链接指向不存在的路径 (e.g. {_bl_ex})")
    print("       ⇒ 导航腐烂：照着点进去才发现目录不存在。路径不存在 ≠ 该面为空。")
if NO_FILE:
    _nf_ex = ", ".join(NO_FILE[:3])
    print(f"  FAIL: {len(NO_FILE)} index entries point to missing files "
          f"(e.g. {_nf_ex})")
if LIC_MISSING:
    _lic_ex = ", ".join(LIC_MISSING[:3])
    print(f"  advisory: {len(LIC_MISSING)} skills lack license field "
          f"(e.g. {_lic_ex})")

if strict and fails:
    print("skill-gate: STRICT mode → failing")
    sys.exit(1)
print("skill-gate: OK" if not fails else "skill-gate: advisory only (use --strict to enforce)")
PYEOF
