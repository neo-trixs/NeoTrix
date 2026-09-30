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

# `--strict` 为**约定兼容**，当前是 no-op：本门只有阻断语义，没有 advisory 模式
# ——「有断言被代码反驳」在语义上不可能是 advisory（要么记录对，要么记录错）。
# 接受它是为了让 scripts/check-gate-satisfiable.sh 的门发现逻辑
# （`grep -q -- "--strict" scripts/check-*.sh`）把本门纳入统计，
# 否则它就是元门看不见的门。
case "${1:-}" in
  --strict|"") ;;
  -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
  *) echo "用法: bash $0 [--strict]"; exit 2 ;;
esac

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
      # 2026-09-30 修两个抽取 bug —— 这是本门长期「检查了 0 处」的根因：
      #  ① `:line` 后缀未剥离 ⇒ `l5_cognition/.../registry.rs:462` 因不以 `.rs`
      #     结尾被 `\.rs$` 丢掉，而「点路径 + 行号」恰恰是本仓最常见的写法；
      #  ② `^[a-z_]+$` 排除 CamelCase ⇒ 文档最常点名的**类型**
      #     （`DecisionEngine`）全部抽不到。
      # 放宽不引入误报：下游对每个 token 做仓内解析，解析不到就 `continue`。
      targets=$(printf '%s' "$line" \
        | tr '`' '\n' \
        | sed -E 's/:[0-9]+(-[0-9]+)?$//' \
        | grep -E '\.rs$|^[A-Za-z_][A-Za-z0-9_]*$' || true)
      for t in $targets; do
        [ -n "$t" ] || continue
        # ⛔ 「宁缺勿错」：basename 冲突时**无法验证**，故不报。
        # 实测踩到：`l5_cognition/nt_core/capability/registry.rs` 与
        # `crates/nt-core-capability-tree/src/registry.rs` 同名 ⇒ 消费者判据
        # （只用 basename）把后者 34 个文件算成前者的消费者。
        # 这与 AGENTS.md L15「同名 ≠ 同一符号」同源。
        # ⇒ 宁可不报，也不误报。
        dupes=$(git ls-files "**/$(basename "$t")" 2>/dev/null | grep -c . || true)
        if [ "${dupes:-0}" -gt 1 ]; then
          echo "[doc-claims] ⏭  跳过（同名文件 ${dupes} 个，basename 无法定位消费者）: $t"
          continue
        fi

        # 必须是仓内真实存在的路径，或能在代码里定位的模块名
        # 只接受 .rs 路径，或能唯一解析到一个 .rs 文件的模块名。
        # ⚠️ 曾经的 bug：用 `git ls-files "*$t*"` 做解析，`manifest` 会误配到
        #    `manifest.json` ⇒ 纯误报。必须约束到 .rs。
        if [ -f "$t" ] && [ "${t%.rs}" != "$t" ]; then
          target="$t"
        elif [ "${t%.rs}" != "$t" ] && [ -f "neotrix-core/src/$t" ]; then
          # 2026-09-30 修第三个抽取 bug。文档写的是**层相对路径**
          # （`l5_cognition/nt_core/capability/registry.rs`），而仓根在
          # `neotrix-core/src/` 之下 ⇒ 上面 `-f "$t"` 判否，下面又因含 `/`
          # 匹配不上短名分支 ⇒ token 被静默丢弃。
          # 这与 2026-09-30 B 方案「第二棵树路径→层路径」的迁移同类：
          # **路径的表达方式决定门能不能看见东西。**
          target="neotrix-core/src/$t"
        elif printf '%s' "$t" | grep -qE '^[A-Za-z_][A-Za-z0-9_]*$'; then
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
        # 2026-09-30 修消费者判据的两个误报源（修完抽取立刻暴露的）：
        #
        #  ① 裸子串把**声明**当消费：`nt_decision_engine` 的唯一「消费者」是
        #     `mod nt_decision_engine;` —— 声明不等于调用。这正是本仓
        #     §6.2「导出 ≠ 调用」那条教训本身。
        #  ② 裸子串把**同后缀标识符**当消费：`registry` 裸 grep 命中 283 个文件，
        #     但真 `use` 只有 51，且仓内有 **126 个含 registry 的不同标识符变体**
        #     （skill_registry / CapabilityRegistry / …）⇒ 通配匹配是噪声源。
        #
        # ⇒ 消费者必须是**引用形式**（`use …X` 或 `X::`）且**不是 mod 声明**。
        # ⚠️ 2026-09-30 第四个正则方言地雷：`git grep -E` 走 **POSIX ERE，
        # `\b` 未定义** ⇒ 含 `\b` 的正则静默返回**空**（不是报错）。
        # 实测 `\bnt_core_telemetry\b` → 0 条，`(^|[^A-Za-z0-9_])nt_core_telemetry` → 7 条。
        # 本会话已踩四次同类坑（rg -E、两处抽取、此处）⇒ 规律：
        # **换引擎/换上下文后，边界写法必须重测，不能照抄。**
        consumers=$(
          git grep -n -E "(^|[^A-Za-z0-9_])(use[^;]*[^A-Za-z0-9_]${mod}([^A-Za-z0-9_]|$)|[^A-Za-z0-9_]${mod}::)" -- '*.rs' 2>/dev/null \
          | grep -v -E "^[0-9]+:[[:space:]]*(pub(\([^)]\))?[[:space:]]+)?mod[[:space:]]+${mod}[[:space:]]*;" \
          | grep -v "^$target:" \
          | cut -d: -f1 | sort -u || true
        )
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

# ---------------------------------------------------------------------------
# 空跑不许绿灯（2026-09-30）
#
# 实测：本门长期报「PASS: 检查了 0 处可判定断言」。
# 诊断：短语表其实**命中 17 行**（零生产消费者 4 / 无生产消费者 7 /
# 无生产调用方 2 / 零调用方 1 / 未接生产 1 / 没有任何生产 1 / 零外部消费者 1），
# 但 0 行能抽出可判定的目标 —— 因为本门假设「断言会在**同一行用反引号点名
# 文件或模块**」，而真实文档用散文与表格叙述。
#
# ⇒ 「检查了 0 处」与「抽取逻辑坏了」在输出上**完全同形**。
# 这正是本会话反复在打的「假绿灯」：门看起来在工作，实际什么也没查。
# 一个查了 0 项却报 PASS 的门比没有门更危险 —— 它制造虚假信心。
#
# 因此：0 断言 = FAIL，除非显式豁免。
# ---------------------------------------------------------------------------
if [ "$checked" -eq 0 ] && [ "${DOC_CLAIMS_ALLOW_EMPTY:-0}" != "1" ]; then
  echo "[doc-claims] FAIL: 检查了 0 处可判定断言 ⇒ 本门在空跑。"
  echo "  「0 处」与「抽取逻辑坏了」在本门输出上完全同形，无法区分。"
  echo "  修法二选一："
  echo "    a) 修抽取逻辑，让它认得文档实际使用的表述方式（当前假设同行反引号，"
  echo "       而真实文档用散文/表格叙述）；"
  echo "    b) 若本仓确实已无此类断言，显式豁免：DOC_CLAIMS_ALLOW_EMPTY=1"
  echo "  ⛔ 不要因为它报了 PASS 就认为文档与代码一致 —— 它什么都没查。"
  exit 1
fi

echo "[doc-claims] PASS: 无被反驳的「零消费者」断言（检查了 $checked 处可判定断言）。"
exit 0
