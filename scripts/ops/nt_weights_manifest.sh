#!/usr/bin/env bash
# nt_weights_manifest.sh — 权重与训练产物的清单 / 校验 / 备份
#
# 为什么需要它
# ------------
# `models/` 被 `.gitignore` 的 `models/*` 整条拦着，git 保护不到。4.2G 里
# 绝大部分是**可重新下载**的 HF 权重，但 `models/training/` 那 274M 是**训练
# 产物**（pretrain 语料、路由表、4.7M 精标语料）—— 重新生成要再跑一遍训练，
# 等于不可再生。用「备份 4.2G」的方式保护它是错配：GGUF 已量化，再压缩近乎
# 零收益、耗时以小时计，而这 274M 里的 jsonl 压完只要 58M。
#
# 所以这里分三类处置：
#
#   不可再生 (training/)     → 真备份, gzip 后 ~58M
#   可再生权重 (gguf/safet.)  → 只记**来源**(repo+revision+文件名) 与校验和,
#                               丢了按来源重下即可, 不占备份
#   校验和                   → 全部登记, 用来发现损坏与版本漂移
#
# 用法
#   nt_weights_manifest.sh write     # 生成/更新清单 (默认)
#   nt_weights_manifest.sh verify    # 校验现有清单, 不符即报
#   nt_weights_manifest.sh backup    # 把不可再生产物打成 gzip 备份
#   nt_weights_manifest.sh restore <tar.gz>   # 从备份还原
#   nt_weights_manifest.sh status    # 三类各自的现状
#
# 退出码: 0 通过 / 1 校验不符或缺文件 / 2 用法错
set -uo pipefail

REPO_ROOT="${REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
MODELS_DIR="$REPO_ROOT/models"
# 清单放在 docs/ 下而不是 models/ —— 后者被 gitignore 整条拦着, 放进去等于
# 清单自己也失去版本控制, 而"哪些权重、什么版本"恰恰是最该进版本控制的东西。
MANIFEST="$REPO_ROOT/docs/architecture/WEIGHTS-MANIFEST.sha256"
# 不可再生产物的备份落点。与 bare 仓同盘: 防误删/误操作, **不防磁盘故障** ——
# 真要跨盘, 把这个文件拷到另一块介质。
BACKUP_DIR="${NT_WEIGHTS_BACKUP_DIR:-$REPO_ROOT/../Neo/weights-backup}"
BACKUP_TAR="$BACKUP_DIR/training-$(date +%Y%m%d).tar.gz"

# 不可再生 = 训练产物。可再生 = 任何可从 HF 重新拉取的权重文件。
RENEWABLE_EXT='\.(gguf|safetensors|bin|pt|pth)$'
# 训练产物里混着 __pycache__/*.pyc, 那是可再生的编译产物, 不进备份。
EXCLUDE='--exclude=__pycache__ --exclude=*.pyc --exclude=.DS_Store'

die() { printf '  ✗ %s\n' "$*" >&2; exit "${2:-1}"; }
ok()  { printf '  ✓ %s\n' "$*"; }
hdr() { printf '\n── %s\n' "$*"; }

# 单文件 SHA-256（shasum 跨 macOS/Linux 都在）。
sha256() { shasum -a 256 "$1" | awk '{print $1}'; }

gen_manifest() {
  hdr "生成清单 → ${MANIFEST#$REPO_ROOT/}"
  mkdir -p "$(dirname "$MANIFEST")"
  : > "$MANIFEST"
  {
    echo "# WEIGHTS-MANIFEST —— models/ 的内容账本"
    echo "# 生成: $(date '+%Y-%m-%d %H:%M:%S')  机器: $(hostname)"
    echo "#"
    echo "# models/ 被 .gitignore 的 'models/*' 整条拦着, git 保护不到。"
    echo "# 本清单是**唯一**记录「盘上有什么、什么版本、损坏没有」的地方。"
    echo "#"
    echo "# 三类处置（见 scripts/ops/nt_weights_manifest.sh 文件头）:"
    echo "#   [R] 可再生  可从 HF 按下方来源重新下载, 不备份"
    echo "#   [I] 不可再生 训练产物, 已 gzip 备份 (verify 不含它们的大小比对)"
    echo "#"
    echo "# 校验: bash scripts/ops/nt_weights_manifest.sh verify"
    echo
    echo "# ─── [R] 可再生权重 ───"
    echo "# 来源见 docs/architecture/WEIGHTS-INVENTORY-2026-09-28.md"
  } >> "$MANIFEST"

  local n=0
  while IFS= read -r f; do
    rel="${f#$REPO_ROOT/}"
    printf '[R] %s  %s\n' "$(sha256 "$f")" "$rel" >> "$MANIFEST"
    n=$((n + 1))
  done < <(find "$MODELS_DIR" -type f \( -name '*.gguf' -o -name '*.safetensors' \
                -o -name '*.bin' -o -name '*.pt' -o -name '*.pth' \) 2>/dev/null | sort)
  local n_r=$n
  ok "可再生权重 $n_r 个已登记"

  {
    echo
    echo "# ─── [I] 不可再生训练产物 ───"
  } >> "$MANIFEST"
  n=0
  while IFS= read -r f; do
    rel="${f#$REPO_ROOT/}"
    case "$rel" in *__pycache__*|*.pyc|*.DS_Store) continue ;; esac
    printf '[I] %s  %s\n' "$(sha256 "$f")" "$rel" >> "$MANIFEST"
    n=$((n + 1))
  done < <(find "$MODELS_DIR/training" -type f 2>/dev/null | sort)
  local n_i=$n
  ok "不可再生产物 $n_i 个已登记"
  ok "清单共 $((n_r + n_i)) 项 [R]=$n_r [I]=${n_i}，位于 ${MANIFEST#$REPO_ROOT/}（已进版本控制）"
}

verify_manifest() {
  [ -f "$MANIFEST" ] || die "清单不存在: ${MANIFEST#$REPO_ROOT/}  先跑 write" 2
  hdr "校验清单"
  local bad=0 missing=0 n=0
  while read -r kind sum rel; do
    case "$kind" in \#*|"") continue ;; esac
    n=$((n + 1))
    f="$REPO_ROOT/$rel"
    if [ ! -f "$f" ]; then
      printf '  ✗ 缺失  %s\n' "$rel"; missing=$((missing + 1)); continue
    fi
    if [ "$(sha256 "$f")" != "$sum" ]; then
      printf '  ✗ 不符  %s\n' "$rel"; bad=$((bad + 1))
    fi
  done < "$MANIFEST"
  printf '\n  共 %d 项: ' "$n"
  if [ "$bad" -eq 0 ] && [ "$missing" -eq 0 ]; then
    ok "全部一致"; return 0
  fi
  printf '%d 缺失 / %d 被改动\n' "$missing" "$bad"
  printf '  → 缺失且属 [R]: 按 WEIGHTS-INVENTORY 的来源重下。\n'
  printf '  → 缺失或被改动且属 [I]: 只能从备份还原, 立即跑 backup 覆盖或 restore。\n'
  return 1
}

do_backup() {
  [ -d "$MODELS_DIR/training" ] || die "无 models/training/, 无需备份" 2
  hdr "备份不可再生产物 → ${BACKUP_TAR/#$HOME/~}"
  mkdir -p "$BACKUP_DIR"
  printf '  压缩中（约 274M → 预期 ~58M, 首次数十秒）...\n'
  # shellcheck disable=SC2086
  tar -czf "$BACKUP_TAR" $EXCLUDE -C "$REPO_ROOT" models/training \
    || die "tar 失败" 1
  local sz; sz=$(du -h "$BACKUP_TAR" | cut -f1)
  ok "$(basename "$BACKUP_TAR")  $sz"
  # 备份自身也要能被校验: 把它的 sha256 落到同名 .sha256
  sha256 "$BACKUP_TAR" > "$BACKUP_TAR.sha256"
  ok "备份校验和 $(basename "$BACKUP_TAR").sha256"
  printf '  ⚠ 与仓库同盘 —— 防误删, **不防磁盘故障**。跨盘请自行拷贝该文件。\n'
}

do_restore() {
  local t="${1:-}"
  [ -n "$t" ] || die "用法: $0 restore <tar.gz>" 2
  [ -f "$t" ] || die "备份不存在: $t" 2
  if [ -f "$t.sha256" ] && [ "$(sha256 "$t")" != "$(awk '{print $1}' "$t.sha256")" ]; then
    die "备份自身校验和不符, 拒绝还原: $t" 1
  fi
  hdr "还原 $t → models/training/"
  # shellcheck disable=SC2086
  tar -xzf "$t" -C "$REPO_ROOT" || die "解包失败" 1
  ok "已还原, 建议随即跑 verify"
}

do_status() {
  hdr "现状"
  local r i
  r=$(find "$MODELS_DIR" -type f \( -name '*.gguf' -o -name '*.safetensors' \
        -o -name '*.bin' \) 2>/dev/null | wc -l | tr -d ' ')
  i=$(find "$MODELS_DIR/training" -type f -not -path '*__pycache__*' -not -name '*.pyc' 2>/dev/null | wc -l | tr -d ' ')
  printf '  [R] 可再生权重   %3s 个  %s\n' "$r" "$(du -sh "$MODELS_DIR" 2>/dev/null | cut -f1)"
  printf '  [I] 不可再生     %3s 个  %s\n' "$i" "$(du -sh "$MODELS_DIR/training" 2>/dev/null | cut -f1)"
  printf '  清单: %s\n' "$([ -f "$MANIFEST" ] && echo "在 ($(grep -c '^\[' "$MANIFEST") 项)" || echo '未生成')"
  printf '  备份: %s\n' "$(ls -1 "$BACKUP_DIR"/training-*.tar.gz 2>/dev/null | tail -1 || echo '无')"
  printf '  ⚠ 备份与仓库同盘, 不防磁盘物理故障。\n'
}

case "${1:-write}" in
  write)   gen_manifest ;;
  verify)  verify_manifest ;;
  backup)  do_backup ;;
  restore) do_restore "${2:-}" ;;
  status)  do_status ;;
  *)       printf '用法: %s {write|verify|backup|restore <tar.gz>|status}\n' "$0" >&2; exit 2 ;;
esac
