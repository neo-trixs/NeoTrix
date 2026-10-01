#!/bin/bash
# npm 依赖许可门 —— 填补「许可防护真空」的 npm 侧。
#
# # ⭐ 为什么必须有这道门（子代理只读审计的结论，非推测）
#
# Rust 侧有**真门**：`deny.toml` 的 allow 列表穷举 15 项、`exceptions = []`，
# AGPL/GPL/SSPL/Elastic/BUSL 全部不在 allow 里，且有两条阻断 job
# （`deny.yml` 的 `bans licenses sources` leg + `security-audit.yml` 的 `check all`）。
# ⇒ **npm 侧是唯一真空**，而本项目是**商用**产品。
#
# `check-license.sh` 抓不到 npm 依赖，三条**独立**的原因：
#   1. `:78` `:89` 都带 `-name node_modules -prune` ⇒ pnpm 装的包对它**完全不可见**；
#      全脚本 250 行**无任何依赖清单解析**。
#   2. deny 名单只作用于 `LICENSE*` 文件（`:221` 的 `rg -g 'LICENSE*'`），
#      `$record` / `$declared` **都不在搜索范围**内。
#   3. `declared` 只被提取、**从不与 DENY 比对**。
#
# ⇒ 实测后果：`neobot-ui/src/vendor/openghost` 的记录里写着
#   「⛔ 非商用保留（禁取）…明文含 rebrand」，而该树**零 FAIL 通过**。
#
# # ⭐ 为什么用「基线棘轮」而不是解析 lockfile
#
# ⓔ **`pnpm-lock.yaml` 不含任何 license 字段**（实测零命中）⇒ 锁文件
# **在结构上无法回答许可问题**。而 `node_modules` 被 gitignore
# ⇒ 许可真源若放那里，它就是**下一个会静默回退的载体**
# （`.neotrix/absorption-cache.json` 刚刚就是这么把 AGPL 藏起来的）。
# ⇒ 数据源必须在**版本控制里**：`config/node-license-baseline.txt`。
#   本仓已有同款范式：`config/.gitleaks-baseline.txt`。
#
# # 判据
#
# |  | 判据 | 拦住什么 |
# |---|------|---------|
# | A | 枚举 `neobot-ui/node_modules` 每个包的 `(name, version, license)` | — |
# | B ⭐ | license 命中 **deny 名单** ⇒ FAIL，**零例外通道** | AGPL/GPL/SSPL/Elastic/BUSL/PolyForm-NC/Commons-Clause/CC-BY-NC |
# | C ⭐ | license **缺失 / `NOASSERTION`** ⇒ FAIL | ⛔ 静默跳过 —— 这正是 `nt_absorption_audit.py:109` 犯过的错 |
# | D ⭐ | **基线外**的包 ⇒ FAIL | 新引入的包不被自动放行 |
# | E ⭐ | `node_modules` **不存在** ⇒ FAIL | ⛔「没扫成」与「没问题」必须可区分（`security-scan.yaml:80-84` 同款判据） |
# | F | 基线里有、但实测**已不装** ⇒ 提示（不 FAIL） | 基线腐化 |
#
# ⛔ **C 不可省**：「缺 license 就跳过」是最容易静默通过失败的写法。
# ⛔ **E 不可省**：门在没装依赖时若 exit 0，就是一个**永远绿的假门**。
# ⛔ **B 的例外通道是零**：⛔ 授权不是 agent 能给的（见 `LICENSE-EXCEPTIONS.md`），
#    商用例外只能由人签署。
#
# 用法：
#   bash scripts/check-license-js.sh            # 检查
#   bash scripts/check-license-js.sh --strict   # 同上（`gate-registry` 枚举需要该标志）
#   bash scripts/check-license-js.sh --update   # ⛔ 显式重生成基线（人工审过才用）
#
# 退出码：0 = PASS，1 = FAIL，2 = 用法错。
set -uo pipefail   # ⛔ 不用 -e：断言要自己控制流（`scripts/probes/_lib.sh` 的教训）

cd "$(dirname "$0")/.." || exit 2

UI="apps/neobot-desktop/neobot-ui"
NM="$UI/node_modules"
BASE="config/node-license-baseline.txt"

STRICT=0; UPDATE=0
for a in "$@"; do
  case "$a" in
    --strict)  STRICT=1 ;;
    --update)  UPDATE=1 ;;
    *) echo "用法: $0 [--strict|--update]" >&2; exit 2 ;;
  esac
done

# ⛔ deny 名单。SPDX 表达式形式；`OR`/`AND`/`WITH` 里的每个 token 都要过筛。
#    ⛔ 不含中文标记（那是 `check-license.sh` 的缺口）：本门只读 SPDX 字段，
#       散文里的「非商用」不在 `package.json` 的 license 里。
DENY='AGPL|GPL|SSPL|BUSL|Business Source|Elastic-2\.0|Elastic License|PolyForm Noncommercial|Commons Clause|CC-BY-NC|NonCommercial|No Commercial Use|UNLICENSED|NOASSERTION'

# ── 枚举 ────────────────────────────────────────────────────────────
if [ ! -d "$NM" ]; then
  # ⛔ E：不是「通过」，是「没扫成」。exit 0 会让门永远绿。
  echo "⛔ $NM 不存在 ⇒ **没扫成**，不是「没有问题」。"
  echo "   先装依赖：pnpm --dir $UI install --frozen-lockfile"
  exit 1
fi

enumerate() {
  python3 - "$NM" <<'PY'
import json, os, sys
root = sys.argv[1]
out = {}
for dp, _dn, fn in os.walk(root):
    if 'package.json' not in fn:
        continue
    try:
        d = json.load(open(os.path.join(dp, 'package.json'), encoding='utf-8'))
    except Exception:
        continue
    n, v, l = d.get('name'), d.get('version'), d.get('license')
    if not n or not v:
        continue
    if isinstance(l, dict):          # 老式 {type,url} 或 {type:[..]}
        l = l.get('type')
        if isinstance(l, list): l = l[0] if l else ''
    out[f'{n}\t{v}'] = l if isinstance(l, str) else ''
for k in sorted(out):
    print(k + '\t' + out[k])
PY
}

ACTUAL=$(enumerate)

if [ "$UPDATE" -eq 1 ]; then
  mkdir -p config
  {
    echo '# npm 依赖许可基线（scripts/check-license-js.sh 的棘轮）'
    echo '# 格式：name<TAB>version<TAB>license（SPDX，来自 node_modules/<pkg>/package.json）'
    echo '# ⛔ 新增包**不会**被自动放行：基线外 ⇒ 门红 ⇒ 须人工审后加进本文件。'
    echo '# ⛔ deny 名单即使写进基线也 FAIL —— 基线不是绕过通道。'
    printf '%s\n' "$ACTUAL"
  } > "$BASE"
  echo "♻️  基线已重生成：${BASE}（$(printf '%s\n' "$ACTUAL" | wc -l | tr -d ' ') 个包）"
  exit 0
fi

if [ ! -f "$BASE" ]; then
  echo "⛔ 基线不存在：$BASE"
  echo "   ⛔ 绝不能在无基线时 exit 0 —— 那等于「没有许可约束」。"
  echo "   生成：bash scripts/check-license-js.sh --update（须人工审过每个包）"
  exit 1
fi

BASE_MAP=$(grep -v '^#' "$BASE" | grep -v '^[[:space:]]*$' | awk -F'\t' '{print $1"\t"$2"\t"$3}')

FAIL=0
declare -a b_no_lic=() b_deny=() b_new=()

while IFS=$'\t' read -r name ver lic; do
  [ -z "$name" ] && continue
  # B：deny 名单
  if printf '%s' "$lic" | rg -qi "$DENY"; then
    b_deny+=("$name@$ver  →  $lic")
    continue
  fi
  # C：缺 license（⛔ 不静默跳过）
  if [ -z "$lic" ]; then
    b_no_lic+=("$name@$ver  →  <无 license 字段>")
    continue
  fi
  # D：基线外
  if ! printf '%s\n' "$BASE_MAP" | rg -q -F "$name"$'\t'"$ver"$'\t'; then
    b_new+=("$name@$ver  →  $lic  （基线外，须人工审后入基线）")
  fi
done <<< "$ACTUAL"

# F：基线里有、但没装（腐化提示，不 FAIL）
GHOST=0
while IFS=$'\t' read -r name ver _l; do
  [ -z "$name" ] && continue
  if ! printf '%s\n' "$ACTUAL" | rg -q -F "$name"$'\t'"$ver"$'\t'; then
    GHOST=$((GHOST+1))
  fi
done <<< "$BASE_MAP"

N=$(printf '%s\n' "$ACTUAL" | grep -c . || true)

echo "npm 依赖许可门（$N 个包，strict=${STRICT}）"
echo

if [ "${#b_deny[@]}" -gt 0 ]; then
  echo "⛔ B deny 名单命中 ${#b_deny[@]} 个（**零例外通道**）："
  for x in "${b_deny[@]}"; do echo "     ⛔ $x"; done
  FAIL=1
fi
if [ "${#b_no_lic[@]}" -gt 0 ]; then
  echo "⛔ C 缺 license 字段 ${#b_no_lic[@]} 个（⛔ 不静默跳过）："
  for x in "${b_no_lic[@]}"; do echo "     ⛔ $x"; done
  FAIL=1
fi
if [ "${#b_new[@]}" -gt 0 ]; then
  echo "⛔ D 基线外的新包 ${#b_new[@]} 个："
  for x in "${b_new[@]}"; do echo "     ⛔ $x"; done
  echo "     处置：⛔ 商用例外只能由人签署（见 .neotrix/LICENSE-EXCEPTIONS.md）"
  echo "          确认无误后：bash scripts/check-license-js.sh --update"
  FAIL=1
fi
[ "$GHOST" -gt 0 ] && echo "ℹ️  F 基线里 ${GHOST} 个包当前未安装（基线可能腐化，不判失败）"

echo
if [ "$FAIL" -eq 0 ]; then
  echo "✅ PASS: $N 个包的许可均在基线内且不在 deny 名单。"
else
  echo "FAIL: npm 侧存在 deny 命中 / 缺 license / 基线外新包。"
fi
exit "$FAIL"
