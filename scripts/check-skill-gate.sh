#!/usr/bin/env bash
# check-skill-gate.sh — Skill 市场入库门禁 (P0-2 三段式 + awesome 领养清单)
#
# 吸收源（思想，非代码）:
# - yibie/awesome-autoresearch "Curation is not endorsement" + 领养前检查表
#   （loop 可跑否 / 有无实证 / 数字有源否 / 试用成本 / 有无 license）
# - sindresorhus/awesome 治理机制（inclusion 规则 + lint 门禁）
# - repowise PR Bot 哲学：绿则静默，有问题才列清单
#
# 用法: check-skill-gate.sh [--strict] [skills/index.json]
#   默认 advisory 模式：只报告，exit 0（不挡现有 58 条存量）。
#   --strict：门禁项任一失败即 exit 1（供 CI 新 skill 增量门）。
set -euo pipefail

STRICT=0
INDEX="skills/index.json"
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    *) INDEX="$arg" ;;
  esac
done

if [ ! -f "$INDEX" ]; then
  echo "skill-gate: index not found: $INDEX"
  exit 2
fi

python3 - "$INDEX" "$STRICT" <<'PYEOF'
import json, sys

index_path, strict = sys.argv[1], sys.argv[2] == "1"
with open(index_path) as f:
    index = json.load(f)

fails, warns, total, three_part = [], [], 0, 0
LIC_MISSING = []

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
        # awesome 领养清单：license（仅 advisory，不进门禁）
        if "license" not in s:
            LIC_MISSING.append(label)
        # 证据门（awesome-autoresearch: runnable check / results log）
        # index schema 暂无 evidence 字段 → 仅统计口径预留位
        _ = warns  # 占位：后续 evidence 字段落地后在此加 WARN

print(f"skill-gate: {total} skills, {three_part} three-part complete, "
      f"{len(fails)} gate findings, {len(LIC_MISSING)} missing license(advisory)")
for line in fails[:20]:
    print("  " + line)
if len(fails) > 20:
    print(f"  ... and {len(fails) - 20} more")
if LIC_MISSING:
    print(f"  advisory: {len(LIC_MISSING)} skills lack license field "
          f"(e.g. {', '.join(LIC_MISSING[:3])})")

if strict and fails:
    print("skill-gate: STRICT mode → failing")
    sys.exit(1)
print("skill-gate: OK" if not fails else "skill-gate: advisory only (use --strict to enforce)")
PYEOF
