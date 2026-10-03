#!/usr/bin/env bash
# ⭐ 网络出口守卫门（SSRF 候选棘轮）—— 断言「运行期可控 URL 的发起点不增长」。
#
# ## ⭐⭐ 为什么本门**只棘轮候选数**，不判定「有漏洞」
#
# 2026-10-03 实测教训（本门全部设计依据，一手可复现）：
#
# ① 我先按「文件里有没有 SSRF 守卫标记」扫 ⇒ 55 个发点文件里 **38 个「无守卫」**。
#    ⛔ 但逐个读代码后：**绝大多数根本不是 SSRF 面**。例如 `source/audio/deezer.rs:18`：
#        format!("https://api.deezer.com/search?q={}&…", urlencoding::encode(&query))
#    ⭐ **host 是源码常量**，只有 query 被 `urlencoding::encode` 插值
#    ⇒ 用户无法控制 host ⇒ **SSRF 不适用**。
#    ⛔ 把 38 当成「38 个漏洞」去修，正是 `AGENTS.md` §5 R-SCAN-1 说的
#    「照单全修 ⇒ 把 bug 修进正确代码」。
#
# ② 收窄后的判据：**URL 实参是被原样喂进去的「裸变量」**（不是常量拼接）
#    ⇒ host 才可能运行期可控。
#
# ⇒ **静态文本匹配无法解析 provenance。** 本门因此只做一件**诚实**的事：
#   **候选数不许增长**。新增即红，须当场定性并写明理由。
#   ⛔ 本门**不声称**基线内的候选有漏洞，也**不声称**它们安全。
#
# ## ⭐ 与 OpenDots 的对照（吸收来源）
# `src/browser/security.ts`（59 行单点）用**六层纵深**解决同一问题，
# 最值得抄的一条：⭐ **非 IP 输入返回 false ⇒ fail-closed**
# —— 安全判定对「不认识的东西」必须返回「不可信」。
# ⛔ 本仓现状是**同一份防护复制在 9 处 / 8 个文件**（`169.254.169.254` 计数）
# ⇒ 漂移风险真实存在；但**逐处收敛是结构改动**，须先
# `cargo clean && cargo build` 跑两遍（`AGENTS.md` §1），**不与本门同批做**。
#
# 用法：
#   bash scripts/check-net-guard.sh             # 棘轮检查
#   bash scripts/check-net-guard.sh --audit     # 只打印不判红
#   bash scripts/check-net-guard.sh --rebaseline # 重建基线（须人工核对）
#   bash scripts/check-net-guard.sh --self-test  # ⭐ 负向测试（必须真翻红）

set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 2
BASELINE="scripts/net-guard-baseline.txt"

# 发起点 + 裸变量实参。用 grep -E 一步筛出候选（⛔ 不用 rg -E：本机静默返回 0，见 AGENTS.md §4.2）
#
# ⭐⭐ 必须剥掉**注释行**（`grep -v ':[[:space:]]*//'`）：
# ⭐ 本基线文件里写了大量「定性说明」注释，其中提到
# `format!("https://<常量>/…?{query}")` ⇒ ⭐ 若不剥，**说明文字会被当成发起点**。
# ⭐⭐ 这是**本门自己的假阳性**（2026-10-03 实测：26 行注释被计入），
# ⭐ 与 `check-ext-wiring.py` 的注释假阳性**同型** —— ⭐ 而那个门我当天已修过，
# ⭐⭐ **说明我没有把上一处的教训推广到第二个门。**
scan() {
  grep -rnE '(reqwest|ureq)::(get|post|request)\(&?[a-z_][a-z0-9_]*\)' \
    neotrix-core/src crates --include='*.rs' 2>/dev/null \
  | grep -v '/tests/' \
  | grep -v ':[[:space:]]*//' \
  | sed 's/:.*//' | sort -u
}

current="$(scan)"
[ -z "$current" ] && { echo "FAIL: 扫描不到任何发起点（选择器失效？本门会假绿）"; exit 2; }

if [ "${1:-}" = "--rebaseline" ]; then
  keep=$(grep '^[[:space:]]*#' "$BASELINE" 2>/dev/null || true)
  { [ -n "$keep" ] && printf '%s\n' "$keep"; printf '%s\n' "$current"; } > "$BASELINE.tmp2"
  mv "$BASELINE.tmp2" "$BASELINE"
  echo "基线已重建: $(printf '%s\n' "$current" | wc -l | tr -d ' ') 个文件"
  exit 0
fi

if [ ! -f "$BASELINE" ]; then
  echo "FAIL: 基线缺失 $BASELINE —— 先跑 --rebaseline 并人工核对"
  exit 2
fi

cur_n=$(printf '%s\n' "$current" | wc -l | tr -d ' ')
# ⭐⭐ 基线支持 `#` 理由注释（对齐 `check-ext-wiring.py` 的既有处理）：
# ⭐ 理由：⭐ **定性理由必须与清单同处一地**，否则下一个读到
# 「这个候选没被处理」却不知道它是否已定性。
grep -v '^[[:space:]]*#' "$BASELINE" | grep -v '^[[:space:]]*$' | sort -u > "$BASELINE.tmp"
base_n=$(wc -l < "$BASELINE.tmp" | tr -d ' ')
fresh=$(comm -23 <(printf '%s\n' "$current") "$BASELINE.tmp")

echo "运行期可控 URL 候选文件: $cur_n · 基线: $base_n"
if [ -n "$fresh" ]; then
  echo ""; echo "FAIL: 新增 $(printf '%s\n' "$fresh" | wc -l | tr -d ' ') 个候选："
  printf '%s\n' "$fresh" | sed 's/^/  /'
  cat <<'TXT'

⛔ 新增候选**不等于**漏洞（见文件头：38 个「无守卫」里绝大多数 host 是常量）。
   但新增即须**当场定性**：
  · host 确为源码常量 ⇒ 加进基线并在此注明 file:line 与理由
  · host 运行期可控 ⇒ ⛔ 必须加 SSRF 守卫（照 OpenDots 六层 + fail-closed）
  · 定性不了 ⇒ 留红，别加白名单
TXT
  exit 1
fi

stale=$(comm -13 <(printf '%s\n' "$current") "$BASELINE.tmp")
[ -n "$stale" ] && echo "ℹ️ 基线中 $(printf '%s\n' "$stale" | wc -l | tr -d ' ') 条已消失（可清理，不判红）"
rm -f "$BASELINE.tmp"
echo "PASS: 无新增运行期可控 URL 发起点"
echo "⛔ 本门不判定基线内的候选是否有漏洞 —— 静态匹配无法解析 provenance。"

# ── ⭐ 负向自测：注入一个新生发起点，同一份判定必须翻红 ───────────────────────
# ⭐ 教训（2026-10-03 三次修正）：① 自测必须跑**同一份**逻辑，不能重写一遍；
#   ② 变异标记**不能包含被测子串**，否则「看着生效、实际没变」= 假通过。
if [ "${1:-}" = "--self-test" ]; then
  victim="crates/neotrix-neobot/src/nt_provider.rs"
  [ -f "$victim" ] || { echo "SELFTEST FAIL: 注入目标不存在: $victim"; exit 2; }
  tmp=$(mktemp -d) || exit 2
  cp "$victim" "$tmp/victim.bak"
  # ⭐ 变异必须是**新增**候选，不是破坏既有匹配。
  #   ⓰ 我第一版把 `reqwest::get(` 改成 `getZZ(` ⇒ 那是**移除**匹配
  #      ⇒ 目标文件退出候选集 ⇒ 集合**变小** ⇒ 生长型棘轮当然不红
  #      ⇒ **变异与被测性质正交**，自测假通过。
  #   ⇒ 改为注入一个**新的**裸变量发起点。
  sed -i.bak '1i\
    fn __nt_selftest_probe(injected_url: &str) { let _ = ureq::get(injected_url); }' "$victim"
  after=$(scan)
  cp "$tmp/victim.bak" "$victim"; rm -rf "$tmp"
  if [ "$after" = "$current" ]; then
    echo "SELFTEST FAIL: 注入后扫描结果未变（变异无效 ⇒ 本自测是假通过）"; exit 1
  fi
  # 反向：还原必须回到原状，否则门会污染工作树
  now=$(scan)
  [ "$now" = "$current" ] || { echo "SELFTEST FAIL: 还原后仍不一致（污染工作树）"; exit 1; }
  echo "SELFTEST OK: 注入后候选集变化（$cur_n → $(printf '%s\n' "$after" | wc -l | tr -d ' ')），还原后一致"
fi