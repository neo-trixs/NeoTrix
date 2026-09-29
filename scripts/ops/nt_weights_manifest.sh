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
#   nt_weights_manifest.sh verify     # 校验现有清单, 不符即报
#   nt_weights_manifest.sh provenance # 逐字节比对上游 LFS oid (= 内容 SHA-256), 离线可跑
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
# 默认落点与 bare 仓同盘（防误删，不防磁盘故障）。跨盘见下。
BACKUP_DIR="${NT_WEIGHTS_BACKUP_DIR:-/Users/neo/Downloads/Neo/weights-backup}"
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
  done < <(find "$MODELS_DIR" -type f -not -path '*/training/*' \
                -not -name '.DS_Store' -not -path '*__pycache__*' 2>/dev/null | sort)
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
  # 备份自身也要能被校验: 落标准格式 (`<hash>  <文件名>`)，`shasum -c` 可直接验。
  # 只写裸 hash 的话标准工具认不出（2026-09-29 踩过：shasum -c 报
  # "no properly formatted SHA checksum lines"）。
  (cd "$(dirname "$BACKUP_TAR")" && shasum -a 256 "$(basename "$BACKUP_TAR")") > "$BACKUP_TAR.sha256"
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
  r=$(find "$MODELS_DIR" -type f -not -path '*/training/*' -not -name '.DS_Store' \
        -not -path '*__pycache__*' 2>/dev/null | wc -l | tr -d ' ')
  i=$(find "$MODELS_DIR/training" -type f -not -path '*__pycache__*' -not -name '*.pyc' 2>/dev/null | wc -l | tr -d ' ')
  printf '  [R] 可再生权重   %3s 个  %s\n' "$r" "$(du -sh "$MODELS_DIR" 2>/dev/null | cut -f1)"
  printf '  [I] 不可再生     %3s 个  %s\n' "$i" "$(du -sh "$MODELS_DIR/training" 2>/dev/null | cut -f1)"
  printf '  清单: %s\n' "$([ -f "$MANIFEST" ] && echo "在 ($(grep -c '^\[' "$MANIFEST") 项)" || echo '未生成')"
  printf '  备份: %s\n' "$(ls -1 "$BACKUP_DIR"/training-*.tar.gz 2>/dev/null | tail -1 || echo '无')"
  printf '  ⚠ 备份与仓库同盘, 不防磁盘物理故障。\n'
  printf '  跨盘: cp %s/training-*.tar.gz* /Volumes/<你的盘>/\n' "$BACKUP_DIR"
}

# 已核实的上游来源。**键是本地相对路径，值是 "repo@revision|上游LFS-oid"**。
#
# 为什么记 oid 而不是只记 repo: HF API 的 `tree` 端点对 LFS 文件返回
# `lfs.oid`，那个值**就是文件内容的 SHA-256**。拿它当基准，比对就是确定性的 ——
# 不依赖文件名、不依赖目录结构、也不依赖 HF 页面显示的体积。
#
# 这条是踩过坑才定下来的: `minimind-3` 在 HF 上被大量镜像且 config 完全相同
# (`Qwen3ForCausalLM`/vocab 6400/hidden 768), 靠 config 或文件名都区分不了;
# 而搜索首先命中的是 5 个月前的历史 commit, 它的 model.safetensors **尺寸与本地
# 一模一样 (127834168) 但 sha256 不同** —— 只比尺寸会把错误来源记成事实。
PROVENANCE=(
  "models/qwen35-4b-uncensored/Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-Q6_K.gguf|HauhauCS/Qwen3.5-4B-Uncensored-HauhauCS-Aggressive@c09cdbcdb1fefad6d335809d445621b5f5ba0c6e|ba93c21300854075ab42655bc30dca82c7c6c958f511d1ec9ea2b3e750b4b75f|lfs"
  "models/qwen35-4b-uncensored/mmproj-Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-BF16.gguf|HauhauCS/Qwen3.5-4B-Uncensored-HauhauCS-Aggressive@c09cdbcdb1fefad6d335809d445621b5f5ba0c6e|a1e32e86ea99aa7a56f3dcfe7e63c1d0be9439d31fd07087099f15bc0fda0f22|lfs"
  "models/minimind-3/model.safetensors|jingyaogong/minimind-3@f92512d4cd6142fa9acc0d6022375049a8974bf6|3adf69402b5d22e693151cabadc12528f923c4ba6bf343738aaf13f0892162e8|lfs"
  "models/minimind-3/config.json|jingyaogong/minimind-3@f92512d4cd6142fa9acc0d6022375049a8974bf6|c8db3894798ad2218caf6a4c0156141227958904|blob"
  "models/minimind-3/tokenizer.json|jingyaogong/minimind-3@f92512d4cd6142fa9acc0d6022375049a8974bf6|e6ca86c9335a4bbbc1ffda03f479c076d4cdf4ba|blob"
)

# 逐字节比对本地文件与上游 LFS oid。
#
# 离线可跑（oid 硬编码在上面的 PROVENANCE 里），所以这是一道**不依赖网络**的门 ——
# 上游哪天改了 main 也不会误报; 只有本地文件被动过才会红。
do_provenance() {
  hdr "来源逐字节校验（对照上游 LFS oid = 内容 SHA-256）"
  local bad=0
  for entry in "${PROVENANCE[@]}"; do
    local rel="${entry%%|*}" rest="${entry#*|}"
    local src="${rest%%|*}" rest2="${rest#*|}"
    local want="${rest2%%|*}" algo="${rest2#*|}"
    local f="$REPO_ROOT/$rel"
    if [ ! -f "$f" ]; then
      printf '  ✗ 缺失  %s\n      应来自 %s\n' "$rel" "$src"; bad=$((bad + 1)); continue
    fi
    local got
    if [ "$algo" = "blob" ]; then
      # git hash-object 走的是仓库自己的对象格式, 即使 models/ 被 gitignore 也照样能算
      got=$(git hash-object "$f")
    else
      got=$(sha256 "$f")
    fi
    if [ "$got" = "$want" ]; then
      printf '  ✓ %s  [%s]\n      ← %s\n' "${rel##*/}" "$algo" "$src"
    else
      printf '  ✗ 不符  %s  [%s]\n      ← %s\n      本地 %s\n      上游 %s\n' \
        "$rel" "$algo" "$src" "$got" "$want"
      bad=$((bad + 1))
    fi
  done
  printf '\n  %d 项, ' "${#PROVENANCE[@]}"
  if [ "$bad" -eq 0 ]; then
    ok "全部与所记 revision 逐字节一致"
    printf '  丢了就按上面的 repo@revision 重下, 不会捡错版本。\n'
    return 0
  fi
  printf '%d 项不符\n' "$bad"
  return 1
}

case "${1:-write}" in
  write)      gen_manifest ;;
  verify)     verify_manifest ;;
  provenance) do_provenance ;;
  backup)     do_backup ;;
  restore)    do_restore "${2:-}" ;;
  status)     do_status ;;
  *)          printf '用法: %s {write|verify|provenance|backup|restore <tar.gz>|status}\n' "$0" >&2; exit 2 ;;
esac
