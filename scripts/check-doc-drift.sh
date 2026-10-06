#!/bin/bash
# Doc-drift check — R-P232（2026-09-29 改：常量 baseline → 账本棘轮）
#
# 校验每个 nt_*.rs 模块文件（排除 mod.rs / lib.rs / main.rs）在前 3 行内
# 带内层模块文档（//!）。
#
# ## 为什么从「常量 baseline」改成「账本棘轮」（2026-09-29）
#
# 旧实现把基线写死在**注释**里：
#
#     echo "Files missing //! module docs: $COUNT (baseline 111 on 2026-09-21)"
#     if [ "$STRICT" -eq 1 ] && [ "$COUNT" -gt 0 ]; then exit 1; fi
#
# 后果：`COUNT > 0` 恒真 ⇒ **`--strict` 从设计之日起就恒红**（实测 127）。
# 而它同时**已接进 pre-commit** ⇒ 每次提交都报红。
#
# 这是 `awesome-dsh-plugin/.github/workflows/pr-gate.yml` 记的那类病：
#   "A gate that dies before posting is indistinguishable from one that never
#    needed to run."
# **恒红的门比没有门更坏 —— 它训练人忽略红色。**
#
# ⇒ 改成与 `check-truth-surface.sh` / `check-layer-deps.sh` 一致的账本棘轮：
#   - 存量进 `scripts/doc-drift-baseline.txt`（**文件列表，不是数字**）
#   - `--strict` 只拦**新增**
#   - 修好一个就 `--update-baseline` 棘轮下降
#
# ⛔ **绝不要为了让门变绿而调大 baseline。** 那是把症状变成谎言。
#    参照 N-1（2026-09-29 拆掉的 5 个恒红幻影门）。
#
# 用法：
#   bash scripts/check-doc-drift.sh                  # advisory, exit 0, 打印计数
#   bash scripts/check-doc-drift.sh --strict         # 仅当有**新增**未文档化文件时 exit 1
#   bash scripts/check-doc-drift.sh --update-baseline# 把当前存量写入账本（棘轮）
#
# ## 第二类：根文档死链（2026-09-29 新增）
#
# 原实现只扫 `neotrix-core/src` 的 `nt_*.rs` 模块文档 ⇒ **根目录 13 个 .md/.toml
# 完全在门外**。而根文档互引极密（AGENTS.md 一个文件就是 94 处引用的中枢），
# 一份被删/被改名的文档会让下一个 agent 拿着死链去查。
#
# 扫法：抽根文档正文里的仓库内路径（`docs/…` `scripts/…` `.neotrix/…` 等），
# 逐个 `-e` 验证存在。⛔ 不扫 http(s) 链接（本门无网络）。
#
# ⚠️ **两种提及必须区分**（2026-09-29 实测踩到）：
#   · 「指示去读」—— 路径该存在，不存在 = 坏链 ⇒ 报。
#   · 「告知已删」—— 路径**故意**不存在。`TODO.md` 的「已废止」清单、
#     `RUST-STANDANCES.md` 的「已完全删除」记录、`CONTRIBUTING.md` 对一条
#     失效命令的说明，都是这类。它们若被门判红，会逼人去删真实历史 ——
#     **恒红的门比没有门更坏**（它训练人忽略红色）。
# ⇒ 故跳过命中「已删/已废止/已归档/已失效/已消失/不要再」语境的行。
# 实测 17 条初始命中全部属此类，跳过后归 0。
#
# Note: bash-3.2-safe style (macOS system bash). `bash -n` before commit.

set -uo pipefail

SRC="neotrix-core/src"
BASELINE="scripts/doc-drift-baseline.txt"
# ⚠️ 扫描面此前**只有 6 份根文档** ⇒ `docs/architecture/` 下 170+ 份架构文档的
#    死链**结构性不可见**（审计实测）。现默认纳入 `docs/architecture/**/*.md`。
#    仍可用 ROOTDOCS=... 覆盖。
ROOTDOCS="${ROOTDOCS:-AGENTS.md README.md DOCUMENTATION-MAP.md CONTRIBUTING.md RUST-STANDARDS.md TODO.md $(find docs/architecture -name '*.md' 2>/dev/null | tr '\n' ' ')}"
STRICT=0
UPDATE=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --update-baseline) UPDATE=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

CUR=$(mktemp); NEW=$(mktemp); GONE=$(mktemp); BASE_C=$(mktemp)
trap 'rm -f "$CUR" "$NEW" "$GONE" "$BASE_C"' EXIT

# 收集缺 module doc 的文件（**排序**，保证同一状态同一列表）
rg --files -g 'nt_*.rs' -g '!mod.rs' -g '!lib.rs' -g '!main.rs' "$SRC" 2>/dev/null | sort > "$CUR.all"
: > "$CUR"
while IFS= read -r f; do
  [ -f "$f" ] || continue
  if ! head -n 3 "$f" | rg -q '^//!'; then
    echo "$f" >> "$CUR"
  fi
done < "$CUR.all"
rm -f "$CUR.all"
sort -u "$CUR" -o "$CUR"

TOTAL=$(grep -c . "$CUR" 2>/dev/null); TOTAL=${TOTAL:-0}

if [ "$UPDATE" -eq 1 ]; then
  if [ -f "$BASELINE" ]; then
    grep '^#' "$BASELINE" > "$BASELINE.tmp" || : > "$BASELINE.tmp"
  else
    : > "$BASELINE.tmp"
  fi
  cat "$BASELINE.tmp" "$CUR" > "$BASELINE"
  rm -f "$BASELINE.tmp"
  echo "baseline updated: $BASELINE now has $(grep -vc '^#' "$BASELINE") entries"
  exit 0
fi

# 与账本求差
if [ -f "$BASELINE" ]; then
  grep -v '^#' "$BASELINE" 2>/dev/null | sort -u > "$BASE_C" || : > "$BASE_C"
fi
comm -23 "$CUR" "$BASE_C" > "$NEW"    # 在树里、不在账本 => 新增
comm -13 "$CUR" "$BASE_C" > "$GONE"   # 在账本、树里已修好 => 已解决

N_NEW=$(grep -c . "$NEW" 2>/dev/null); N_NEW=${N_NEW:-0}
N_GONE=$(grep -c . "$GONE" 2>/dev/null); N_GONE=${N_GONE:-0}
N_BASE=$(grep -vc '^#' "$BASELINE" 2>/dev/null); N_BASE=${N_BASE:-0}

echo "=== NeoTrix doc-drift check (R-P232) ==="
echo "missing //! module docs: $TOTAL   ledger: $N_BASE   resolved since ledger: $N_GONE"
echo "NEW (not in ledger): $N_NEW"

if [ "$N_GONE" -gt 0 ]; then
  echo "--- resolved (drop from ledger via --update-baseline) ---"
  head -n 10 "$GONE"
  [ "$N_GONE" -gt 10 ] && echo "  ... and $((N_GONE-10)) more"
fi

if [ "$N_NEW" -gt 0 ]; then
  echo "--- NEW offenders (regression, not in ledger) ---"
  cat "$NEW"
fi

if [ "$STRICT" -eq 1 ] && [ "$N_NEW" -gt 0 ]; then
  echo "FAIL(strict): $N_NEW new undocumented file(s). Ledger holds $N_BASE standing."
  echo "  Fix: add a //! module doc, or accept it into the ledger via --update-baseline"
  echo "  (only if the file is genuinely archival — a new file with no doc is a real gap)."
  exit 1
fi

echo "DONE(advisory)."

# ─────────────────────────────────────────────────────────────
# 第二类：根文档死链（2026-09-29 新增）
# ─────────────────────────────────────────────────────────────
LINKTMP=$(mktemp)
trap 'rm -f "$CUR" "$NEW" "$GONE" "$BASE_C" "$LINKTMP"' EXIT

# 只取形如 `path/with.ext` 或 `dir/` 的反引号片段；排除 URL、绝对路径
for d in $ROOTDOCS; do
  [ -f "$d" ] || { echo "root-doc-missing: $d"; continue; }
  # 抽出以已知仓库前缀开头的路径 token
  rg -o '\b(docs|scripts|crates|skills|sessions|config|models|neotrix-core|src-tauri|\.neotrix|\.githooks|\.github)/[A-Za-z0-9_./-]+' "$d" 2>/dev/null \
    | sed 's/[.,;:)]*$//' | sort -u >> "$LINKTMP" || true
done
# 去重并验证存在
[ -f "$LINKTMP" ] && sort -u "$LINKTMP" -o "$LINKTMP"

# ── 剔除三类**非路径** token（2026-09-29 实测踩到，见下）────────────────
#   ① glob 前缀：  sessions/handoff-<窗口>.md  被截成 sessions/handoff-
#   ② 行号引用：  core/skills/mod.rs:25 被截成 skills/mod.rs（子串）
#   ③ 通配文档：  `LESSONS-*.md` / `docs/2-PLANS/` 这类不是具体文件
#   ⛔ 不剔除的话门会恒红，而恒红的门比没有门更坏（它训练人忽略红色）。
CANDTMP=$(mktemp)
trap 'rm -f "$CUR" "$NEW" "$GONE" "$BASE_C" "$LINKTMP" "$CANDTMP"' EXIT
while IFS= read -r p; do
  [ -n "$p" ] || continue
  # ① glob 前缀：前缀后无扩展名、且以 - 结尾（sessions/handoff-）
  case "$p" in *-) continue ;; esac
  # ② 纯 glob 片段（含 * 或 ?）
  case "$p" in *'*'*|*'?'*) continue ;; esac
  # ③ 同名多实例文件：仓库里有 N 份 `mod.rs`（crypto/ extractors/ visual/ …），
  #    文档里写 `core/skills/mod.rs:25` 这类**省略 crate 前缀的模块内相对引用**，
  #    抽出来是 `skills/mod.rs` —— 它不是仓库路径，任何前缀都不该被验存在。
  #    判据：basename 是 mod.rs / lib.rs / main.rs / mod_python.rs ⇒ 跳过。
  #    （这比「首段是不是目录」准：skills/ 确实是目录，但 skills/mod.rs 不是路径）
  case "${p##*/}" in
    mod.rs|lib.rs|main.rs|mod_python.rs) continue ;;
  esac
  # ④ 行号引用被截断：token 的 basename 出现在某根文档里、且**它前面还有
  #    另一段路径**（`core/skills/mod.rs:25` ⇒ 抽出 `skills/mod.rs`）。
  #    判据：从根文档反查是否有 `<某段>/<base>` 形式的更长路径命中真实文件。
  #    ⚠️ rg 默认带 `文件:行:` 前缀，必须 --no-filename 只取内容，否则会把
  #    `TODO.md:ops/nt_ipc_keys.py` 当成路径而误判（实测踩过）。
  if [ ! -e "$p" ]; then
    base="${p##*/}"
    if [ -n "$base" ] && [ "$base" != "$p" ]; then
      longer=$(rg --no-filename -o '[A-Za-z0-9_.-]+/'"$base" $ROOTDOCS 2>/dev/null \
               | grep -v "^$p$" | head -1)
      # 拼回仓库根验证：命中且存在 ⇒ $p 是被截断的片段
      if [ -n "$longer" ] && [ -e "$longer" ]; then
        continue
      fi
    fi
  fi
  echo "$p" >> "$CANDTMP"
done < "$LINKTMP"
mv "$CANDTMP" "$LINKTMP"
trap 'rm -f "$CUR" "$NEW" "$GONE" "$BASE_C" "$LINKTMP" "$CANDTMP"' EXIT

DEAD=0
SKIPPED=0
KNOWN=0
if [ -s "$LINKTMP" ]; then
  # DEADTMP 记「该路径被当作**指示**提及」
  DEADTMP=$(mktemp)
  DEADTMP2=$(mktemp)   # 上下文判定时的哨兵（指示语境命中即置位）
  trap 'rm -f "$CUR" "$NEW" "$GONE" "$BASE_C" "$LINKTMP" "$CANDTMP" "$DEADTMP" "$DEADTMP2"' EXIT
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    [ -e "$p" ] && continue
    # 逐个根文档找它；只要有一处是指示语境就算死链，全是「已删」语境则跳过
    is_dead=0
    for d in $ROOTDOCS; do
      [ -f "$d" ] || continue
      # ⚠️ 不再用 `head -5` 截断：某路径有多处提及时，截断会让**先出现的
      # 指示语境**掩盖后面的「已删」标注（或反之）⇒ 判定随提及顺序漂移。
      # 正确判据：**全部**提及位置都是历史告知语境才算跳过。
      #
      # ⚠️ 还要看**上下文 ±2 行**：markdown 段落/列表项常跨行，路径可能在
      # 段落首也可能在**行尾**，而「已删/已作废」标记在相邻行（实测踩过两次：
      # TODO.md:422 路径在行首标记在 423-426；TODO.md:427 路径在行尾、
      # 标记在 425-426）。只看单行或只看后 2 行都会漏判 ⇒ 窗口取 ln-2..ln+2。
      : > "$DEADTMP2"
      rg -n -F "$p" "$d" 2>/dev/null | cut -d: -f1 | while IFS= read -r ln; do
[ -n "$ln" ] || continue
      lo=$((ln-3)); [ "$lo" -lt 1 ] && lo=1
      ctx=$(sed -n "${lo},$((ln+3))p" "$d" 2>/dev/null)
      # ⑤ 探针注入点（2026-10-06）：本门判据是「文档让下一个 agent 去读一个
      #    真实存在的路径」。但**探针注入文件按设计只在探针运行期间存在**
      #    （TODO.md 记载的 check-silent-failure / check-unwrap 落点即如此），
      #    文档记下那个路径是在描述**注入契约**，不是在下指示。
      #    ⛔ 判据必须窄，双条件缺一不可：
      #       (a) 上下文提到探针/probe；(b) 路径本身形如 nt_probe_*.rs。
      #       绝不能泛化成「文件名含 probe 就跳过」—— 那会让真死链溜过去。
      #    （写成 case 的额外分支会与外层 case 的 ;; 冲突 ⇒ 提前 continue）
      if printf '%s' "$ctx" | grep -qE '探针|probe|PROBE' \
         && printf '%s' "$p" | grep -qE '(^|/)nt_probe_[a-z0-9_]*\.rs$'; then
        continue
      fi
      case "$ctx" in
        *已删*|*已废止*|*已归档*|*已失效*|*已消失*|*不要再*|*已完全*|*归档*|*不要删*|*已移除*|*全无消费者*|*仅自身测试*|*无生产*|*从未入库*|*从未存在*|*不存在*|*已作废*|*已裁决*|*永久丢失*|*正典*|*未关闭*|*未落地*|*不是实测*|*仍然敞开*)
            : ;;   # 历史告知 / 订正语境 → 不计
          *) echo DEADLINE >> "$DEADTMP2"; break ;;
      esac
    done
      if [ -s "$DEADTMP2" ]; then is_dead=1; break; fi
    done
    if [ "$is_dead" -eq 1 ]; then
      echo "$p" >> "$DEADTMP"
    else
      SKIPPED=$((SKIPPED+1))
    fi
  done < "$LINKTMP"
  BASEFILE="${BASEFILE:-scripts/doc-drift-baseline.txt}"
  if [ -s "$DEADTMP" ]; then
    while IFS= read -r p; do
      # 棘轮：已在基线里的死链只报告（KNOWN），不计入阻断；基线外的新增才拦。
      # 与 orphan-dir / dead-flag 门同一套棘轮纪律。
      if [ -f "$BASEFILE" ] && grep -Fqx "$p" "$BASEFILE" 2>/dev/null; then
        echo "root-doc-deadlink(known): $p"; KNOWN=$((KNOWN+1))
      else
        echo "root-doc-deadlink: $p"; DEAD=$((DEAD+1))
      fi
    done < "$DEADTMP"
  fi
  rm -f "$DEADTMP" "$DEADTMP2"
fi
echo "root-doc deadlinks: $DEAD  (skipped $SKIPPED intentional '已删' mentions; scanned $(grep -c . "$LINKTMP" 2>/dev/null || echo 0) path refs across: $ROOTDOCS)"

if [ "$STRICT" -eq 1 ] && [ "$DEAD" -gt 0 ]; then
  echo "FAIL(strict): $DEAD dead path reference(s) in root docs."
  echo "  These are real paths the docs tell the next agent to read."
  echo "  Either restore the file, or fix the reference."
  exit 1
fi
