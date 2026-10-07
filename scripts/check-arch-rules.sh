#!/bin/bash
# check-arch-rules.sh — 架构规则注册表门（裁定 ①A）
#
# # 为什么需要这个门
#
# 裁定 ①A 定的是「**规则开集可注册，但每条必须可证伪 + 有消费者 + 进豁免基线**」。
# ⛔ 规则本身在 `nt_arch_rules.rs` 里（`RuleRegistry::register` 的三条准入，
#    9 个测试守住）。但那些是 **Rust 单元测试**，只有跑 `cargo test` 才生效。
# ⇒ 缺口：一个能**改规则注册表**的人（或脚本）可以让规则绕过三条准入，
#   而 CI 若不跑 cargo test 就看不见。
#
#   ⭐ 这就是 anonrouter/confidential-content-plane 的核心手法：
#   **「声明一个边界」要由机器强制，而不是靠信任文档。**
#   那个仓的 `content-plane-closure.test.ts` 从入口算模块图，
#   控制面模块一旦可达即红。本门做同一件事的轻量版：
#   **规则集 + 基线**必须一致，任何"新增规则未登记"或"基线项已消失"都判红。
#
# # 判据（三条，双判据防空转）
#
#   ① 规则源可读且非空           —— 文件存在 + 有实际条目
#   ② 每条规则三字段齐全         —— falsifiable_by / serves / id 唯一
#   ③ 基线双向一致               —— 规则与基线的差集必须为空
#
#   ⚠️ 空判据必须判红（防空转）：若 ①②③ 的 pattern 一条都命中不到，
#      门会"永远绿"而毫无作用 —— 本门对每条判据都要求**至少一条真命中**。
#
# # 只读性（R-SCAN-4：本门自身先审有无写操作再跑）
#
#   只有 read / echo / grep / awk / sort / comm / find -r。没有写文件、没有 git 写操作。
#   bash-3.2-safe（无关联数组、无 ${var,,}）。
#
# 用法：bash scripts/check-arch-rules.sh [--strict]
# 退出： 0 = 一致；1 = FAIL（有规则未登记 / 基线项已消失 / 规则三字段缺失）

set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

RULES=".neotrix/arch-rules.tsv"
BASELINE=".neotrix/arch-rules-baseline.txt"
SRC="neotrix-core/src/l6_meta/coordination/nt_arch_rules.rs"
STRICT=0
[ "${1:-}" = "--strict" ] && STRICT=1

FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }

# ── 判据① 规则源与基线文件都在，且有实际条目 ────────────────────────
if [ ! -f "$RULES" ]; then
  fail "规则源不存在: $RULES"
elif [ ! -f "$BASELINE" ]; then
  fail "豁免基线不存在: $BASELINE（①A 裁定要求每条违反都登记）"
elif ! grep -qE '^[A-Z]' "$RULES"; then
  # 防"空文件永远绿"：规则源必须真有条目
  fail "规则源无实际条目（空判据 ⇒ 门形同虚设）"
else
  N=$(grep -cE '^[A-Z]' "$RULES")
  echo "ok: 规则源 $RULES 有 $N 条规则"
fi

if [ -f "$RULES" ]; then
  # ── 判据② 三字段齐全（每行 id \t falsifiable_by \t serves）──────
  BAD=$(awk -F'\t' 'NF>0 && $1!~/^#/ && $1!="" {
      if (NF < 3) print "  行 " NR ": 字段不足（需 id<TAB>falsifiable_by<TAB>serves）";
      else if ($2 == "" || $2 == "-") print "  行 " NR ": falsifiable_by 为空 ⇒ 不可证伪";
      else if ($3 == "" || $3 == "-") print "  行 " NR ": serves 为空 ⇒ 没有消费者";
    }' "$RULES")
  if [ -n "$BAD" ]; then
    fail "规则字段缺失（①A 三条准入）:
$BAD"
  else
    echo "ok: 每条规则的 falsifiable_by / serves 均非空"
  fi

  # id 唯一
  DUP=$(grep -E '^[A-Z]' "$RULES" | cut -f1 | sort | uniq -d)
  if [ -n "$DUP" ]; then
    fail "规则 id 重复: $DUP"
  else
    echo "ok: 规则 id 唯一"
  fi

  # ── 判据③ 基线双向一致 ──────────────────────────────────────────
  if [ -f "$BASELINE" ]; then
    # 规则里声明"当前违反"的 id（行尾 `VIOLATES: R1,R2`），应逐个登记在基线。
    # ⛔ 必须排除注释行（`^[^#]*`）—— 规则表头部注释里就写了 "VIOLATES:" 这个词，
    #    否则会把说明文字当成声明（实测踩过：抓出 "R2` 声明本规则当前..." 这种垃圾）。
    grep -E '^[^#]*VIOLATES:' "$RULES" \
      | sed 's/.*VIOLATES:[[:space:]]*//' | tr ',' '\n' \
      | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$' | sort -u > /tmp/.nt_arch_decl.$$
    # 规则集自身（全部 id）
    grep -E '^[A-Z]' "$RULES" | cut -f1 | sort -u > /tmp/.nt_arch_all.$$
    # 基线：只取行首 token（规则 id）；`#` 之后是理由，忽略；注释行与空行跳过
    grep -vE '^[[:space:]]*#|^[[:space:]]*$' "$BASELINE" \
      | sed 's/[[:space:]].*$//' | grep -v '^$' | sort -u > /tmp/.nt_arch_base.$$
    # 双向①：规则声明的违反项必须都在基线
    MISSING=$(comm -23 /tmp/.nt_arch_decl.$$ /tmp/.nt_arch_base.$$)
    # 双向②：基线里的 id 必须仍在规则集内（基线不得腐化）
    GONE=$(comm -13 /tmp/.nt_arch_all.$$ /tmp/.nt_arch_base.$$)
    rm -f /tmp/.nt_arch_decl.$$ /tmp/.nt_arch_all.$$ /tmp/.nt_arch_base.$$
    [ -n "$GONE" ] && fail "基线登记的规则 id 已不存在（基线腐化）:
$(echo "$GONE" | sed 's/^/    /')"
    [ -n "$MISSING" ] && fail "规则声明违反但未登记进基线:
$(echo "$MISSING" | sed 's/^/    /')"
    [ -z "$GONE" ] && [ -z "$MISSING" ] && echo "ok: 规则与基线双向一致"
    # 双向②：规则声明的违反项必须都在基线
    [ -n "$GONE" ] && fail "基线登记的规则 id 已不存在（基线腐化）:
$(echo "$GONE" | sed 's/^/    /')"
    [ -n "$MISSING" ] && fail "规则声明违反但未登记进基线:
$(echo "$MISSING" | sed 's/^/    /')"
  fi

  # 规则源与实现双向：每个 rule id 应能在 Rust 源码里被找到
  # ⛔ 这是**双判据**：文件在 + pattern 真命中。命中不到 = 断言失效 = 判红。
  if [ -f "$SRC" ]; then
    NOREF=$(grep -E '^[A-Z]' "$RULES" | cut -f1 | while read -r id; do
      grep -q "$id" "$SRC" || echo "  $id"
    done)
    if [ -n "$NOREF" ]; then
      fail "规则 id 在实现里查不到（规则成了装饰）:
$(echo "$NOREF" | sed 's/^/    /')"
    else
      echo "ok: 每条规则 id 在 $SRC 中有实现引用"
    fi
  else
    fail "实现文件不存在: $SRC"
  fi
fi

# ── 判据⑥ R2 执行器：层名只有一个定义处 ──────────────────────────
#
# ⭐ 裁定 ②B 归一时，层名的**权威定义处**是权威门自己的层序表
# （scripts/check-layer-deps.sh:84-88 的 5 行 check_layer）。
# ⛔ 我方此前把同一份层名复制到 Rust 的 `ArchLayer::REAL_DIRS`
#    ⇒ **两处字符串定义 = 归一前的"四套词汇"的成因**。
#
# ⇒ 本判据**双向**校验：
#   ① 权威门层序表里的 7 个层名，与 Rust `REAL_DIRS` 的 7 个**逐字相同**
#   ② 别处不得再有第三份层名清单（`nt_review_runner.rs` 曾有一份，已删）
#
# ⛔ 双判据防空转：若两个 pattern 都命中不到 ⇒ 判红（断言失效）
LAYER_SRC="scripts/check-layer-deps.sh"
# ⛔ 真源是 nt_review_types.rs（ArchLayer::REAL_DIRS 所在处），
#    **不是** $SRC（那是 nt_arch_rules.rs，规则注册表所在处）。
#    两者是不同的文件 —— 初版指错导致 R2 判据永久判红。
ARCH_TYPES="neotrix-core/src/l6_meta/nt_core_self_review/nt_review_types.rs"
if [ ! -f "$LAYER_SRC" ]; then
  fail "权威层序表不存在: $LAYER_SRC"
else
  # ① 权威门声明的层名（check_layer 的第 1 个参数）
  grep -E '^check_layer "l[0-9]_' "$LAYER_SRC" \
    | sed 's/check_layer "\([^"]*\)".*/\1/' | sort -u > /tmp/.nt_layers_canonical.$$
  CANON=$(grep -cE '^check_layer "l[0-9]_' "$LAYER_SRC")
  if [ "$CANON" -lt 5 ]; then
    fail "权威层序表只解析出 $CANON 个层（期望 ≥5）⇒ 判据失效"
  else
    # Rust 侧的 7 个（REAL_DIRS 表）
    # ⚠️ ⛔ 不要用 `tr -d '("'`：它会把**每个词的首字符**也删掉
    #    （实测把 l1_action 变成 1_action，comm 于是判"全不一致"）。
    # ⇒ 用 sed 只剥掉首尾括号。
    grep -oE '\(\"l[0-9]_[a-z]+\"' "$ARCH_TYPES" | sed 's/^(\"//; s/\"$//' | sort -u > /tmp/.nt_layers_rust.$$
    # ⚠️ 口径修正：权威门只声明 **5 层**（l1_action..l5_cognition）——
    #   `l0_substrate` 是基座，它不需要禁任何层（没有比它更低的层）。
    # ⇒ 正确判据是「权威门的每个层名都必须在 Rust REAL_DIRS 里」，
    #   **不是**「两者逐字相同」（那会要求基座层出现在 check_layer 里，语义错）。
    MISSING=$(comm -23 /tmp/.nt_layers_canonical.$$ /tmp/.nt_layers_rust.$$)
    rm -f /tmp/.nt_layers_canonical.$$ /tmp/.nt_layers_rust.$$
    if [ -n "$MISSING" ]; then
      fail "权威门声明的层在 Rust REAL_DIRS 里查不到（R2：层名必须只定义一次）:
$(echo "$MISSING" | sed 's/^/    /')"
    else
      echo "ok: 权威层序表的 $CANON 个层名全部见于 Rust REAL_DIRS（基座 l0_substrate 无需禁层，权威门不声明它）"
    fi
  fi
  # ② 第三份清单：**清单**的特征是单文件含 ≥3 个层名（而非仅提到某个层名）。
  #    ⚠️ 口径修正：仅"提到 l0_substrate"不足以判违规 ——
  #    nt_review_runner.rs 的注释与 tests.rs 的测试路径都合法提到层名。
  # ⛔ 只统计**代码**里的层名，跳过 `//` 与 `///` 行 —— 实测 nt_review_runner.rs
  #    的 l1_action/l2_perception 出现处全是**文档注释**（解释判据为何这样写），
  #    tests.rs 的 7 个层名是**测试断言真目录名**（归一 ②B 时刚写的）。
  #    把它们算成"第三份清单"是判据自身的缺陷，不是代码的。
  THIRD=$(for f in $(grep -rl 'l[0-9]_[a-z]' neotrix-core/src/l6_meta/nt_core_self_review/ 2>/dev/null); do
    n=$(grep -vE '^[[:space:]]*//' "$f" \
        | grep -oE 'l[0-9]_(substrate|action|perception|embodiment|emotion|cognition|meta)' \
        | sort -u | wc -l | tr -d ' ')
    [ "$n" -ge 3 ] && echo "$f"
    # ⚛ 测试文件（tests.rs / *_test.rs / 含 #[cfg(test)] 的文件）**天然会列出全部层名**
    #   —— 它们要断言"每个层都能被归类"。⇒ 排除测试文件，
    #   否则本判据会永久判红，而那是判据自身的缺陷。
  done | grep -vE 'nt_review_types\.rs|(^|/)tests\.rs$|_test\.rs$|/tests/' | sed 's/^/    /')
  if [ -n "$THIRD" ]; then
    fail "发现第三份层名清单（含 ≥3 个层名且非真源，R2 违规）:
$THIRD"
  else
    echo "ok: 无第三份层名清单（唯一真源 ArchLayer::REAL_DIRS）"
  fi
fi

# ── 已知违反项登记（供对抗评分消费）──────────────────────────────
# ⭐ 本门同时校验「已知违反项」清单存在且非空 —— 它是 rank_layer_debts 的输入，
#   没有它，红蓝对抗就无债可打分（而"无债"会被读成"无问题"）。
VIOL=".neotrix/arch-known-violations.tsv"
if [ ! -f "$VIOL" ]; then
  fail "已知违反项清单不存在: ${VIOL} （红蓝对抗无输入 ⇒ 误读为「架构无债」）"
elif ! grep -qE '^[a-z]' "$VIOL"; then
  fail "已知违反项清单为空（防：空清单 ⇒ 对抗评分报「全部健康」）"
else
  echo "ok: 已知违反项清单有 $(grep -cE '^[a-z]' "$VIOL") 条"
fi

if [ "$FAIL" -ne 0 ]; then
  echo ""
  echo "FAIL: 架构规则注册表不一致。处理三选一："
  echo "  ① 若规则合法 ⇒ 把它当前违反的 id 登记进 $BASELINE"
  echo "  ② 若规则已不适用 ⇒ 从 $RULES 删除并同步基线"
  echo "  ③ 若规则不满足①A 三条（可证伪/有消费者/进基线）⇒ 它不得注册"
  [ "$STRICT" -eq 1 ] && exit 1
  exit 1
fi
echo "PASS: 架构规则注册表一致（开集 + 三条准入 + 双向基线）"
exit 0