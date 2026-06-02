#!/usr/bin/env bash
# NeoTrix 一键安装脚本
# 用法: curl -fsSL https://neotrix.ai/install.sh | bash
# 支持: macOS (x64, arm64), Linux (x64), Windows (x64 via Git Bash)
#
# 环境变量:
#   VERSION=latest     — 指定版本 (默认 latest)
#   INSTALL_DIR=/usr/local/bin — 安装目录 (默认 /usr/local/bin)
#   SKIP_CHECKSUM=1    — 跳过 SHA256 校验 (不推荐)

set -euo pipefail

REPO="neotrix/neotrix"
VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log()   { echo -e "${GREEN}==>${NC} ${BOLD}$1${NC}"; }
warn()  { echo -e "${YELLOW}==>${NC} $1"; }
error() { echo -e "${RED}==>${NC} $1" >&2; exit 1; }

detect_platform() {
    local os arch
    case "$(uname -s)" in
        Darwin) os="macos" ;;
        Linux)  os="linux" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *) error "Unsupported OS: $(uname -s)" ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64) arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        *) error "Unsupported arch: $(uname -m)" ;;
    esac
    echo "${os}_${arch}"
}

get_latest_version() {
    if [ "$VERSION" != "latest" ]; then
        echo "$VERSION"
        return
    fi
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    if command -v curl &>/dev/null; then
        curl -sL "$api_url" | grep '"tag_name"' | cut -d'"' -f4
    elif command -v wget &>/dev/null; then
        wget -qO- "$api_url" | grep '"tag_name"' | cut -d'"' -f4
    else
        error "Need curl or wget"
    fi
}

# Download a file, retrying up to 3 times
download() {
    local url="$1" output="$2"
    if command -v curl &>/dev/null; then
        curl -fsSL --retry 3 "$url" -o "$output"
    elif command -v wget &>/dev/null; then
        wget -q --tries=3 "$url" -O "$output"
    else
        error "Need curl or wget"
    fi
}

install() {
    local platform="$1" version="$2"
    local archive_name="neotrix-${platform}.tar.gz"
    local url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"
    local tmpdir
    tmpdir=$(mktemp -d)
    local archive_path="${tmpdir}/${archive_name}"

    log "Downloading NeoTrix ${version} (${platform})..."
    download "$url" "$archive_path"

    if [ -z "${SKIP_CHECKSUM:-}" ]; then
        log "Verifying SHA256 checksum..."
        local checksum_url="${url}.sha256"
        local checksum_path="${tmpdir}/${archive_name}.sha256"
        download "$checksum_url" "$checksum_path"
        local expected actual
        expected=$(cut -d' ' -f1 < "$checksum_path")
        if command -v shasum &>/dev/null; then
            actual=$(shasum -a 256 "$archive_path" | cut -d' ' -f1)
        elif command -v sha256sum &>/dev/null; then
            actual=$(sha256sum "$archive_path" | cut -d' ' -f1)
        else
            warn "No SHA256 tool found, skipping checksum verification"
            actual="$expected"
        fi
        if [ "$expected" != "$actual" ]; then
            error "Checksum mismatch! Expected: $expected, Got: $actual"
        fi
        log "Checksum verified: ${expected:0:16}..."
    fi

    log "Extracting to ${INSTALL_DIR}..."
    mkdir -p "$INSTALL_DIR"
    tar xzf "$archive_path" -C "$INSTALL_DIR"
    chmod +x "$INSTALL_DIR/neotrix"

    rm -rf "$tmpdir"

    log "NeoTrix ${version} installed to ${INSTALL_DIR}/neotrix"
    echo ""
    log "Run: neotrix"
}

check_deps() {
    if ! command -v tar &>/dev/null; then
        error "Need tar"
    fi
}

main() {
    echo ""
    log "NeoTrix Installer"
    echo ""

    check_deps

    local platform
    platform=$(detect_platform)
    echo "  Platform: ${platform}"

    local version
    version=$(get_latest_version)
    echo "  Version:  ${version}"
    echo ""

    if [ "$INSTALL_DIR" = "/usr/local/bin" ] && [ "$(uname -s)" != "CYGWIN" ] && [ "$(uname -s)" != "MINGW" ] && [ "$(uname -s)" != "MSYS" ]; then
        if [ ! -w "$INSTALL_DIR" ] && [ "$(id -u)" -ne 0 ]; then
            warn "${INSTALL_DIR} is not writable — retrying with sudo..."
            exec sudo bash "$0" "$@"
        fi
    fi

    install "$platform" "$version"
}

main "$@"
