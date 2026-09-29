#!/usr/bin/env bash
# check-doc-claims.sh — 抓「文档里的断言被代码反驳」这一类死记录。
#
# 动机（R-SCAN-3 的可执行化）：本会话反复出现同一模式的失败 ——
# 文档写下「某模块零生产消费者 / 遗留未接线」，**当天把活干完了却不回头改记录**，
# 于是留下「比没有记录更危险」的死记录：下一个 agent 据此判断会重复劳动，
# 或误以为某项未完成。已发生 6 处。
#
# 本门把其中**可被代码反驳**的一类断言变成可执行检查：
#   文档声称「X 零消费者/未接生产」，但代码里其实已有 X 的消费者 ⇒ FAIL。
#
# 边界（重要）：
#   · 只检查**可机器判定**的断言：文档里点名了具体文件/模块路径，且该路径可 grep。
#   · 「某项未验证」「某门待跑」等**时点性**断言不在本门范围 —— 那类只能靠
#     复核流程，本门判不了。
#   · 历史交接件（sessions/handoff-*.md）默认**豁免**：它们是时点记录，
#     改写等于篡改历史。豁免可用 DOC_CLAIMS_SESSIONS=1 打开。
#
# 退出码：0 = 无被反驳的断言；1 = 有（CI 会红）；2 = 用法/环境错。

set -o pipefail   # 刻意不加 -u：本门是只读 lint，`$t` 等在管道子 shell 里可能未绑定

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "[doc-claims] ERROR: 不在 git 仓库内"; exit 2
}
cd "$REPO_ROOT" || exit 2

# 「零消费者」类短语。中英文引号混排都要覆盖。
CLAIM_RE='零生产消费者|零调用方|无生产消费者|无生产调用方|未接生产|未被生产|没有任何生产|零外部消费者'

# 允许扫描的文档范围。
DOC_GLOBS=(docs/architecture/*.md TODO.md)
if [ "${DOC_CLAIMS_SESSIONS:-0}" = "1" ]; then
  DOC_GLOBS+=(sessions/handoff-*.md)
fi

fail=0
checked=0

for pat in "${DOC_GLOBS[@]}"; do
  # shellcheck disable=SC2086
  for f in $pat; do
    [ -f "$f" ] || continue
    # 逐行找含断言短语的行，再从该行抽出反引号里的路径型 token
    while IFS= read -r line; do
      case "$line" in *零生产消费者*|*零调用方*|*无生产消费者*|*无生产调用方*|*未接生产*|*未被生产*|*没有任何生产*|*零外部消费者*) ;; *) continue ;; esac
      # 时点陈述豁免。判据是**有原则的**，不是关键词打地鼠：
      #   · 该行含 ≥7 位十六进制 commit hash ⇒ 明确锚定到某时点，是史实陈述；
      #   · 或含显式撤销/回溯标记。
      # 无标记的「现在没有消费者」才是本门要抓的死记录。
      if printf '%s' "$line" | grep -qE '(^|[^0-9a-f])[0-9a-f]{7,40}([^0-9a-f]|$)'; then continue; fi
      case "$line" in *已作废*|*已闭环*|*已解决*|*原写*|*曾记为*|*已修正*|*该状态已*) continue ;; esac
      # 抽反引号 token
      targets=$(printf '%s' "$line" | tr '`' '\n' | grep -E '\.rs$|^[a-z_]+$' || true)
      for t in $targets; do
        [ -n "$t" ] || continue
        # 必须是仓内真实存在的路径，或能在代码里定位的模块名
        # 只接受 .rs 路径，或能唯一解析到一个 .rs 文件的模块名。
        # ⚠️ 曾经的 bug：用 `git ls-files "*$t*"` 做解析，`manifest` 会误配到
        #    `manifest.json` ⇒ 纯误报。必须约束到 .rs。
        if [ -f "$t" ] && [ "${t%.rs}" != "$t" ]; then
          target="$t"
        elif printf '%s' "$t" | grep -qE '^[a-z_][a-z0-9_]*$'; then
          # 短名（<5 字符）或 `a::b` 记法的碎片无法唯一解析 ⇒ 跳过，避免误报。
          [ "${#t}" -ge 5 ] || continue
          cands=$(git ls-files "**/${t}.rs" "*${t}.rs" 2>/dev/null)
          cnt=$(printf '%s\n' "$cands" | grep -c . || true)
          [ "$cnt" = "1" ] || continue
          target=$(printf '%s\n' "$cands" | head -1)
        else
          continue
        fi
        [ -f "$target" ] || continue
        checked=$((checked + 1))
        # 消费者 = 除该文件自身外，引用其模块名的其它 .rs
        mod=$(basename "$target" .rs)
        consumers=$(git grep -l -- "${mod}" -- '*.rs' 2>/dev/null | grep -v -x "$target" || true)
        if [ -n "$consumers" ]; then
            n=$(printf '%s\n' "$consumers" | wc -l | tr -d ' ')
            echo "[doc-claims] ❌ 断言被代码反驳：$f"
            echo "               行：${line:0:110}"
            echo "               声称「$(printf '%s' "$t" | tr -c '[:print:]' '?')」零消费者；以下 $n 个文件引用了 \`$mod\`："
            printf '%s\n' "$consumers" | head -3 | sed 's/^/                 /'
            fail=$((fail + 1))
        fi
      done
    done < "$f"
  done
done

if [ "$fail" -gt 0 ]; then
  echo "[doc-claims] FAIL: $fail 处「零消费者」断言被代码反驳。改文档，别等下个 agent 踩。"
  exit 1
fi
echo "[doc-claims] PASS: 无被反驳的「零消费者」断言（检查了 $checked 处可判定断言）。"
exit 0
