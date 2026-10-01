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
    /^## ACKNOWLEDGE-/ { owner=0; date=0; dec=0; v=0; t=""; seen=1; next }
    seen==1 {
      if ($0 ~ /^tree:[[:space:]]*/) { t=$0; sub(/^tree:[[:space:]]*/,"",t);
                                     gsub(/[[:space:]]+$/,"",t) }
      if ($0 ~ /^status:[[:space:]]*void/) { v=1 }
      if ($0 ~ /^decision:/) dec=1
      if ($0 ~ /^owner:/)   owner=1
      if ($0 ~ /^date:/)    date=1
      if (t != "" && dec && owner && date) {
        # status: void ⇒ 该条已作废，不算签署（散文说失效但机器仍接受 = 两套判断打架）
        if (t == want) { found=(v?0:1); exit }
        t=""; v=0
      }
    }
    END { exit(found?0:1) }
  ' "$_f"

}

# 1) 发现 vendored 树：含来源记录或第三方声明的目录
#    ⚠️ 2026-09-30 两个漏洞（实测撞出，均由 `src/vendor/openghost/` 暴露）：
#      ① 记录名只匹配 `VENDOR.md` 精确名 ⇒ 写成 `VENDOR-OPENGHOST.md` 的
#         独立 vendored 树**完全逃过门**。
#      ② 祖先去重会把**嵌套的独立 vendored 树**当成「已被祖先覆盖」而跳过
#         —— 但祖先记录并未覆盖它（法务上两棵树是两个上游、两条授权链）。
VENDORED=$(find . -type d -name node_modules -prune -o -type d -name target -prune -o \
  -type f \( -name 'THIRD_PARTY_NOTICES.md' -o -name 'VENDOR*.md' \
           -o -name 'LICENSE.upstream*' \) -print 2>/dev/null \
  | sed 's#/[^/]*$##' | sort -u)

# 1b) **内容侧**发现信号：`vendor/` 目录即使没有来源记录也必须被发现。
# 为什么必需（2026-09-30 实测）：上一版的发现信号**就是记录文件本身** ——
# 把 `VENDOR-OPENGHOST.md` 移走，该树立刻从名单消失、门回到全绿。
# ⇒ 门只能检查它**看得见**的树，而「看得见」取决于对端是否留了记录。
# `vendor/` 是 JS/TS 生态的第三方代码约定，路径本身是强信号；
# 用它兜底可使「删记录 ⇒ 静默脱管」不可能发生。
VENDOR_DIRS=$(find . -type d -name node_modules -prune -o -type d -name target -prune -o \
      -type d -path '*/vendor/*' -print 2>/dev/null | sort -u)
VENDORED=$(printf '%s\n%s\n' "$VENDORED" "$VENDOR_DIRS" \
           | sed 's#^\./##' | sed '/^$/d' | sort -u)

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
  # 祖先去重的**新判据**（2026-09-30）：只有当祖先的来源记录里**明确提到**
  # 这棵子树时才算「已覆盖」。
  # 旧判据「路径在祖先目录下即跳过」会把**嵌套的独立 vendored 树**吃掉 ——
  # `src/vendor/openghost/` 在 vendored#1 内部，但它是**另一个上游、另一条
  # 授权链**（ANDRETRIPOL/OpenGhost），法务上不可被 dsh-tauri 的记录覆盖。
  leaf=$(basename "$tree")
  # 判据（2026-09-30 定稿）：**子树若有自己的来源记录，就绝不跳过。**
  # 「有自己的记录」= 存在 VENDOR*.md / PROVENANCE.md（真正的来源记录），
  # 而不是 THIRD_PARTY_NOTICES.md（第三方声明，11 个 packages/dsh-tauri-* 只有这个）。
  # 理由：嵌套的独立 vendored 树是**另一个上游、另一条授权链**，
  # 法务上不可能被祖先的记录覆盖 ——
  # `src/vendor/openghost/`（ANDRETRIPOL/OpenGhost）就是这种。
  #
  # ⛔ 试过并否掉的两个更简判据（都会出错，已记录以免后人重蹈）：
  #   ① 「路径在祖先目录下即跳过」→ 吃掉 openghost（漏）
  #   ② 「祖先记录提到父目录名即跳过」→ 也吃掉 openghost，
  #      因为 VENDOR.md 里有 3 处小写 "vendor"（误配）
  #   ③ 「祖先记录提到叶名」→ 又把 11 个 packages/dsh-tauri-* 全部误报
  #      （顶层 VENDOR.md 只写「packages/（14 个包）」，不逐个列名）
  # ⇒ 唯一稳的判据是**记录归属**，不是文本匹配。
  has_own_record=0
  if ls "$tree"/VENDOR*.md >/dev/null 2>&1 || [ -f "$tree/PROVENANCE.md" ]; then
    has_own_record=1
  fi
  # `vendor/` 路径本身就是「此处放第三方代码」的约定信号 ⇒ 该树**必须自带记录**，
  # 不能靠祖先兜底。否则「删掉自己的来源记录」就等于静默脱管。
  case "$tree" in
    */vendor/*|vendor)
      if [ "$has_own_record" -eq 0 ]; then
        echo "FAIL: $tree 位于 vendor/ 路径下（第三方代码约定），但**无自己的来源记录**"
        echo "      ⇒ 删掉来源记录即等于脱离本门管辖。需补 VENDOR-<NAME>.md 或 PROVENANCE.md。"
        FAIL=1
        continue
      fi
      has_own_record=1
      ;;
  esac
  skip=0
  if [ "$has_own_record" -eq 0 ]; then
    n_anc=$(echo "$COVERED" | wc -w | tr -d ' ')
    if [ "$n_anc" -gt 0 ]; then
      i=1
      while [ "$i" -le "$n_anc" ]; do
        anc=$(echo "$COVERED" | cut -d' ' -f$((i+1)))
        case "$tree" in
          "$anc"/*) skip=1; break ;;
          *) i=$((i+1)); continue ;;
        esac
      done
    fi
  fi
  if [ "$skip" -eq 1 ]; then
    echo "  (covered) $tree —— 由祖先树的来源记录覆盖（该子树无自己的来源记录）"
    continue
  fi
  # 2) 找来源记录（同一目录内；`VENDOR*.md` 以支持 VENDOR-<NAME>.md 形式）
  record=""
  for cand in $(ls "$tree"/VENDOR*.md 2>/dev/null | sort) \
              "$tree/PROVENANCE.md" "$tree/THIRD_PARTY_NOTICES.md"; do
    if [ -f "$cand" ]; then record="$cand"; break; fi
  done
  # 必须在 record 查出**之后**登记，否则下一轮迭代拿到的 anc_rec 恒为 none。
  # （第一版踩了这个顺序 bug：登记写在查找之前 ⇒ 祖先去重永久失效，
  #   表现为 11 个 packages/dsh-tauri-* 全被误报「无许可字段」。）
  COVERED="$COVERED $tree"

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
  # 表格行之外，接受 `## 许可` / `## License` 标题作为声明锚点
  # （src/vendor/openghost/VENDOR-OPENGHOST.md 用的是标题+散文，
  #   其中明确写了双条款边界，属有效声明，不该判「无许可字段」）
  if [ -z "$declared" ] && grep -qiE '^#{2,3}[[:space:]]*(许可|许可证|license)' "$record" 2>/dev/null; then
    declared="（由「$(grep -iE '^#{2,3}[[:space:]]*(许可|许可证|license)' "$record" | head -1 | sed 's/^#*[[:space:]]*//')」章节声明）"
  fi

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
