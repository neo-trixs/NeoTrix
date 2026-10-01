#!/usr/bin/env bash
# Provenance checker — verify external build inputs against provenance/external-inputs.json.
# Absorbed: grok-bot-0.18-reconstructed "pinned build input" pattern.
#
# Usage:
#   bash neotrix-core/src/l3_embodiment/nt_shield/safety_tools/provenance_check.sh
#   bash neotrix-core/src/l3_embodiment/nt_shield/safety_tools/provenance_check.sh --url-check
#
# Exit codes: 0 = all pass, 1 = any FAIL **or the gate could not run**.
set -euo pipefail

BOLD='\033[1m'
GREEN='\033[92m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'
log()   { echo -e "${GREEN}==>${NC} ${BOLD}$1${NC}"; }
warn()  { echo -e "${YELLOW}==>${NC} $1"; }
error() { echo -e "${RED}==>${NC} $1" >&2; }
# ⛔ fatal 必须**退出**。2026-09-30 修：原实现里三处致命条件用的都是
# `error`（只 echo，不退出），叠加 `set -e` 对 `||` 列表末位成功不生效、
# 以及 `done < <(python3 …)` 的进程替换退出码不传播 ⇒ 清单缺失时脚本
# 一路跑到底、打印 traceback，最终 `FAIL=0` → **exit 0**。
# 即「供应链溯源门在完全没检查任何东西时报告通过」。
# 一个 fail-open 的供应链门比没有门更坏：它让人以为已验证。
fatal() { error "$1"; exit 1; }

URL_CHECK=0
[ "${1:-}" = "--url-check" ] && URL_CHECK=1

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/provenance/external-inputs.json"
EXAMPLE="$ROOT/provenance/external-inputs.example.json"

command -v python3 >/dev/null || fatal "python3 required"
[ -f "$MANIFEST" ] || fatal "manifest missing: $MANIFEST
       这不是可以忽略的告警 —— 本门是 release.yml 上传二进制前的供应链检查。
       需要一份 external-inputs.json（schema 见同目录 external-inputs.example.json）。
       ⛔ 不要为了让本脚本返回 0 而删掉这行检查。"

# sha256 helper: Linux (sha256sum) or macOS (shasum -a 256)
sha256_of() {
    if command -v sha256sum >/dev/null; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        return 1
    fi
}

log "Provenance check: $MANIFEST"

PASS=0
FAIL=0

# TSV 先落临时文件再消费：`python3 -c … | while read` 会让 python 的退出码
# 被 while 吞掉（管道），`done < <(…)` 则根本不传播子 shell 状态 ——
# 两种写法都无法察觉「解析失败」。落盘后显式检查 $? 才算真的检查过。
TSV="$(mktemp)"
trap 'rm -f "$TSV"' EXIT

if ! python3 -c '
import json, sys
data = json.load(open(sys.argv[1]))
inputs = data["inputs"]
if not isinstance(inputs, list):
    raise SystemExit("schema: inputs must be a list")
for i in inputs:
    row = [i.get("name",""), i.get("kind",""), i.get("version",""),
           i.get("sha256",""), i.get("purpose","")]
    print("\t".join(row))
' "$MANIFEST" > "$TSV"; then
    fatal "manifest 无法解析（schema 漂移或 JSON 非法）：$MANIFEST
       本门不会在无法读取清单时放行。"
fi

# Emit one entry per input as TSV: name|kind|version|sha256|purpose
while IFS=$'\t' read -r name kind version sha purpose; do

    status="PASS"
    detail=""

    # 1. Structural: required fields non-empty
    if [ -z "$name" ] || [ -z "$kind" ] || [ -z "$version" ] || [ -z "$sha" ]; then
        status="FAIL"
        detail="missing required field"
    fi

    # 2. Hash verification when a real hash is anchored
    if [ "$status" = "PASS" ]; then
        case "$sha" in
            unpinned:*)
                detail="honest-unpinned (${sha#unpinned:})"
                ;;
            *)
                # Real hash: verify if a locally cached artifact exists at
                # provenance/cache/<name-safe> else mark deferred (no fetch in CI gate).
                cache="$ROOT/provenance/cache/$(echo "$name" | tr '/' '_')"
                if [ -f "$cache" ]; then
                    actual="$(sha256_of "$cache" || true)"
                    if [ "$actual" != "$sha" ]; then
                        status="FAIL"
                        detail="hash mismatch expected=$sha got=${actual:-none}"
                    else
                        detail="local cache verified"
                    fi
                else
                    detail="real hash anchored; cache absent (deferred to fetch-time verify)"
                fi
                ;;
        esac
    fi

    # 3. Optional live reachability probe
    if [ "$status" = "PASS" ] && [ "$URL_CHECK" = "1" ] && [ "$kind" = "github-action" ]; then
        owner="${name%%/*}"
        repo="${name#*/}"
        code="$(curl -s -o /dev/null -w '%{http_code}' "https://github.com/$owner/$repo" || echo net-err)"
        case "$code" in
            200) detail="$detail; url 200" ;;
            *) status="FAIL"; detail="$detail; url probe got $code" ;;
        esac
    fi

    if [ "$status" = "PASS" ]; then
        PASS=$((PASS + 1))
        echo "  PASS  $name@$version — $detail"
    else
        FAIL=$((FAIL + 1))
        warn "  FAIL  $name@$version — $detail"
    fi
done < "$TSV"

TOTAL=$((PASS + FAIL))
if [ "$TOTAL" -eq 0 ]; then
    # 原来这里是 `error "no inputs parsed from manifest — schema drift?"`，
    # 而 error 只 echo 不退出 ⇒ **这条检测完全失效**，空清单一路 exit 0。
    fatal "manifest 解析出 0 条输入（schema 漂移？）——空清单等于没检查，本门不放行。
       参考 schema：$EXAMPLE"
fi

log "Provenance: $PASS passed, $FAIL failed, $TOTAL total"
[ "$FAIL" -eq 0 ] || { error "$FAIL input(s) failed provenance check"; exit 1; }
log "Provenance gate: PASS"
