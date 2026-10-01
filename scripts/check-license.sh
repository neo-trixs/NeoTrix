#!/bin/bash
# check-license.sh — 外部代码**来源与许可**门（G6 的直接解法）
#
# 为什么需要它（2026-09-30 实测触发，非理论）：
#   "check-supply-iocs.sh" 的 advisory B1 只扫 markers
#   （Multica / BSL-1.1 / FSL- / PolyForm / Commons Clause）**且只 WARN**。
#   于是本仓 1:1 vendored 的 "apps/neobot-desktop/frontend/" 带了
#   "LICENSE.details" = 「No Commercial Secondary Development … these additional
#   terms prevail」，而 "VENDOR.md" 的许可字段只写了「MIT License」。
#   ⇒ 记录**低报了实际条款**，而该树正在被持续修改（即 terms 所称 secondary
#   development）。既有门 100% 漏掉这一类。
#
# 本门只做**记录一致性**检查，不做法律判断：
#   1. 每个 vendored 树必须有来源记录（VENDOR.md 之类）且含许可字段
#   2. 树内若存在「附加条款」文件，来源记录**必须体现**它（不能只写基础 SPDX）
#   3. 声明的许可不得落在 deny 名单
#   4. deny 名单命中 ⇒ FAIL（不是 WARN）
#
# ⛔ 本门**不判断**「NeoTrix 是否构成 commercial secondary development」——
#   那是业务/法务决定。本门只保证**记录不撒谎**。
#   记录修正后若判定为不可接受，正确动作是移除该 vendored 树，不是改门。
#
# 只读性（R-SCAN-4：本门自身先审有无写操作再跑）：
#   全脚本只有 read / echo / rg / awk / find -type f。无 rm/cp/mv/tee/重定向写文件。
# bash-3.2-safe（无关联数组、无 ${var,,}）。"bash -n" 通过后方可提交。
#
# 用法： bash scripts/check-license.sh
# 退出： 0 = 记录一致；1 = FAIL（有树缺来源记录 / 低报条款 / 命中 deny）

set -u
cd "$(dirname "$0")/.." || exit 2
FAIL=0

# 非许可、禁止 vendoring 的标识（大小写不敏感按字面 rg）
DENY='PolyForm Noncommercial|PolyForm Shield|Sustainable Use License|BSL-1\.1|Business Source License|FSL-1\.1|AGPL-3\.0|AGPL-3\.0-only|SSPL-1\.0|Commons Clause|No Commercial Secondary Development|No Commercial Use|Elastic License|Commons-Clause'

# 「附加条款」载体文件名。存在即表示基础 SPDX 之外还有限制。
ADDITIONAL_PAT='^LICENSE\.(details|additional|extra|addendum|terms)$|^LICENSE-[a-z]*terms'

echo "=== NeoTrix 外部来源与许可门（G6） ==="

EXCEPTIONS=".neotrix/LICENSE-EXCEPTIONS.md"

# ack_for <tree> <file>：该树是否有字段齐全的人工签署。
# 「字段齐全」= tree/decision/owner/date 都在同一 ACKNOWLEDGE 段内。
# 少任一字段 ⇒ 视为未签署（宁缺勿错：半个签名等于没有签名）。
ack_for() {
  _tree=$1; _f=$2
  [ -f "$_f" ] || return 1
  awk -v want="$_tree" '
    /^```/            { fence = !fence; seen=0; next }   # 跳过围栏：格式说明不是签署
    fence==1          { next }
    /^## ACKNOWLEDGE-/ { owner=0; date=0; dec=0; t=""; seen=1; next }
    seen==1 {
      if ($0 ~ /^tree:[[:space:]]*/) { t=$0; sub(/^tree:[[:space:]]*/,"",t);
                                     gsub(/[[:space:]]+$/,"",t) }
      if ($0 ~ /^decision:/) dec=1
      if ($0 ~ /^owner:/)   owner=1
      if ($0 ~ /^date:/)    date=1
      if (t != "" && dec && owner && date) {
        if (t == want) { found=1; exit }
        t=""
      }
    }
    END { exit(found?0:1) }
  ' "$_f"

}

# 1) 发现 vendored 树：含来源记录或第三方声明的目录
VENDORED=$(find . -type d -name node_modules -prune -o -type d -name target -prune -o \
  -type f \( -name 'THIRD_PARTY_NOTICES.md' -o -name 'VENDOR.md' \
           -o -name 'LICENSE.upstream*' \) -print 2>/dev/null \
  | sed 's#/[^/]*$##' | sort -u)

if [ -z "$VENDORED" ]; then
  echo "clean: 未发现 vendored 树（无 THIRD_PARTY_NOTICES/VENDOR/LICENSE.upstream）。"
  exit 0
fi

echo "vendored 树（已按祖先覆盖去重）："
echo "$VENDORED" | sed 's/^/  /'
echo

# 已被祖先树覆盖的子树跳过：子目录里的 THIRD_PARTY_NOTICES.md 是「第三方声明」
# 而非「本仓引入该树的来源记录」。实测：本仓 packages/dsh-tauri-* 共 11 个子树
# 全部由顶层 VENDOR.md 一条记录覆盖（该记录显式列出 packages/ 14 个包）。
# ⇒ 不做祖先去重会产生 11/12 假阳性；门一旦噪音就会被忽略（宁缺勿错）。
COVERED=""
for tree in $VENDORED; do
  [ -d "$tree" ] || continue
  tree=${tree#./}
  skip=0
  for anc in $COVERED; do
    case "$tree" in
      "$anc"/*) skip=1; break ;;
    esac
  done
  if [ "$skip" -eq 1 ]; then
    echo "  (covered) $tree —— 由祖先树的来源记录覆盖"
    continue
  fi
  COVERED="$COVERED $tree"

  # 2) 找来源记录
  record=""
  for cand in VENDOR.md VENDOR-PROVENANCE.md PROVENANCE.md THIRD_PARTY_NOTICES.md; do
    if [ -f "$tree/$cand" ]; then record="$tree/$cand"; break; fi
  done

  # 3) 树内是否存在附加条款文件
  addl=$(find "$tree" -maxdepth 1 -type f 2>/dev/null | sed 's#.*/##' \
         | grep -E "$ADDITIONAL_PAT" || true)

  if [ -z "$record" ]; then
    echo "FAIL: $tree 存在 vendored 证据但无来源记录（需 VENDOR.md 或 THIRD_PARTY_NOTICES.md）"
    FAIL=1
    continue
  fi

  # 4) 记录里的许可字段（中文表格式："| 许可 | ... |" 或 "license"）
  declared=$(grep -iE '^\|?\s*(许可|许可证|license)\s*\|' "$record" 2>/dev/null \
            | head -1 | sed 's/^[^|]*|//; s/^[^:]*://' || true)

  if [ -z "$declared" ]; then
    echo "FAIL: $tree 的来源记录 $record 无许可字段"
    FAIL=1
    continue
  fi

  # 5) 附加条款必须被记录体现 —— 本门存在的首要原因
  if [ -n "$addl" ]; then
    # 记录里必须提到附加条款文件名，或提到其中的限制用语
    if ! printf '%s' "$declared" | grep -qiE "$ADDITIONAL_PAT|additional|附加|非商用|non-?commercial"; then
      echo "FAIL: $tree 存在附加条款文件 [$(echo "$addl" | tr '\n' ' ')]，"
      echo "      但 $record 的许可字段只写了基础 SPDX："
      echo "        $declared"
      echo "      ⇒ 记录低报了实际条款。必须补全后重跑（法律判断不在本门范围）。"
      FAIL=1
    else
      echo "ok: $tree 附加条款已被记录体现"
    fi
  else
    echo "ok: $tree 无附加条款文件，声明=$declared"
  fi

  # 6) deny 名单（对记录与树内许可证文件都查）
  hits=$(rg -l -i --no-messages -g 'LICENSE*' -e "$DENY" "$tree" 2>/dev/null | head -5)
  if [ -n "$hits" ]; then
    # 6a) 唯一放行通道 = 人工签署的例外记录。
    #     为什么不是直接删掉这条 deny：删 deny = 门变绿 = 问题被隐藏，
    #     正是本仓「改门让检查通过」的失败模式。签署把决定权显式落到人。
    if ack_for "$tree" "$EXCEPTIONS"; then
      echo "WARN: $tree 命中 deny 条款，但已有**人工签署**的例外记录："
      rg -A2 -i "^tree:.*${tree//\//\\/}\$" "$EXCEPTIONS" 2>/dev/null \
        | head -3 | sed 's/^/      /'
      echo "      ⇒ 已记录在 ${EXCEPTIONS}（非静默放行）。到期需重审。"
    else
      echo "FAIL: $tree 命中禁止 vendoring 的许可条款："
      echo "$hits" | sed 's/^/      /'
      echo "      处理三选一（详见 $EXCEPTIONS 头部）："
      echo "        ① 移除该 vendored 树（架构级决定）"
      echo "        ② 取得上游书面授权"
      echo "        ③ 由项目所有者在 $EXCEPTIONS 签署并写明条件与重审日期"
      echo "      ⛔ 改本门 / 删 deny 名单让检查变绿 = 不可接受。"
      FAIL=1
    fi
  fi
done

echo
if [ "$FAIL" -eq 0 ]; then
  echo "PASS: 所有 vendored 树的来源与许可记录一致。"
  exit 0
fi
echo "FAIL: 见上。记录一致性是最低要求，不构成法律意见。"
exit 1
