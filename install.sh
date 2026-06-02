#!/usr/bin/env bash
#
# NeoTrix Installer
#   curl -fsSL https://raw.githubusercontent.com/neotrix/neotrix/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/neotrix/neotrix/main/install.sh | bash -s -- --version 0.18.0
#
set -euo pipefail

# ======================== Configuration ========================

REPO="neotrix/neotrix"
REPO_URL="https://github.com/${REPO}"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/main"

VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.neotrix/bin}"
NEOTRIX_DIR="${NEOTRIX_DIR:-$HOME/.neotrix}"
NO_MODIFY_PATH=false

# ======================== Colors ========================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { printf "${BLUE}==>${NC} %s\n" "$*"; }
ok()    { printf "${GREEN}  ✓${NC} %s\n" "$*"; }
warn()  { printf "${YELLOW}  ⚠${NC} %s\n" "$*"; }
error() { printf "${RED}  ✗${NC} %s\n" "$*"; exit 1; }
header(){ printf "\n${MAGENTA}━━━ %s ━━━${NC}\n" "$*"; }

# ======================== Parse Flags ========================

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-modify-path) NO_MODIFY_PATH=true; shift ;;
    --version) VERSION="$2"; shift 2 ;;
    --version=*) VERSION="${1#*=}"; shift ;;
    --help)
      echo "Usage: curl -fsSL ${RAW_BASE}/install.sh | bash"
      echo "       curl -fsSL ${RAW_BASE}/install.sh | bash -s -- --version 0.18.0"
      echo "       curl -fsSL ${RAW_BASE}/install.sh | bash -s -- --no-modify-path"
      exit 0
      ;;
    *) shift ;;
  esac
done

# ======================== Platform Detection ========================

detect_platform() {
  local os arch

  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"

  case "$os" in
    darwin)  OS="macos" ;;
    linux)   OS="linux" ;;
    *)       error "Unsupported OS: $os (only macOS and Linux are supported)" ;;
  esac

  case "$arch" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)            error "Unsupported architecture: $arch (only x86_64 and arm64 are supported)" ;;
  esac

  # Rust target triple
  case "$OS" in
    macos) TARGET="${ARCH}-apple-darwin" ;;
    linux) TARGET="${ARCH}-unknown-linux-gnu" ;;
  esac

  info "Detected: ${OS}-${ARCH} (${TARGET})"
}

# ======================== Dependency Checks ========================

check_deps() {
  header "Checking dependencies"

  for cmd in curl uname; do
    if ! command -v "$cmd" &>/dev/null; then
      error "Missing required command: $cmd"
    fi
  done
  ok "curl, uname available"

  command -v brew &>/dev/null && ok "Homebrew available (install method 1)"
  command -v cargo &>/dev/null && ok "Cargo available (install method 2)"
  command -v git &>/dev/null && ok "Git available (install method 4)"
}

# ======================== Install Methods ========================

install_via_brew() {
  [[ "$OS" != "macos" ]] && return 1
  command -v brew &>/dev/null || return 1

  header "Installing via Homebrew"

  if brew tap "${REPO}" &>/dev/null; then
    brew install neotrix
  else
    info "Homebrew tap not available for ${REPO}, trying custom formula..."
    local formula_url="${RAW_BASE}/deploy/homebrew/neotrix.rb"
    if curl -fsSL "$formula_url" -o /tmp/neotrix.rb &>/dev/null; then
      brew install --formula /tmp/neotrix.rb
      rm -f /tmp/neotrix.rb
    else
      warn "Homebrew formula not found, falling through..."
      return 1
    fi
  fi

  ok "NeoTrix installed via Homebrew"
  return 0
}

install_via_cargo() {
  command -v cargo &>/dev/null || return 1

  header "Installing via cargo"

  local cargo_args=""
  if [[ "$VERSION" != "latest" ]]; then
    cargo_args="--version $VERSION"
  fi

  info "Running: cargo install neotrix $cargo_args"
  cargo install neotrix $cargo_args --root "$NEOTRIX_DIR" 2>&1 | sed 's/^/  /'

  if [[ -f "$INSTALL_DIR/neotrix" ]]; then
    ok "NeoTrix installed via cargo"
    return 0
  fi
  return 1
}

install_via_binary() {
  header "Installing via pre-built binary"

  local version_tag="$VERSION"
  if [[ "$version_tag" == "latest" ]]; then
    version_tag="latest"
  else
    version_tag="v${version_tag#v}"
  fi

  local download_url
  if [[ "$version_tag" == "latest" ]]; then
    download_url="https://github.com/${REPO}/releases/latest/download/neotrix-${TARGET}.tar.gz"
  else
    download_url="https://github.com/${REPO}/releases/download/${version_tag}/neotrix-${TARGET}.tar.gz"
  fi

  local tmpdir
  tmpdir="$(mktemp -d)"

  info "Downloading: $download_url"
  if curl -fsSL "$download_url" -o "$tmpdir/neotrix.tar.gz"; then
    mkdir -p "$INSTALL_DIR"
    tar xzf "$tmpdir/neotrix.tar.gz" -C "$tmpdir"

    if [[ -f "$tmpdir/neotrix" ]]; then
      cp "$tmpdir/neotrix" "$INSTALL_DIR/neotrix"
    elif [[ -f "$tmpdir/target/release/neotrix" ]]; then
      cp "$tmpdir/target/release/neotrix" "$INSTALL_DIR/neotrix"
    else
      local found
      found="$(find "$tmpdir" -name neotrix -type f 2>/dev/null | head -1)"
      if [[ -n "$found" ]]; then
        cp "$found" "$INSTALL_DIR/neotrix"
      else
        rm -rf "$tmpdir"
        warn "Binary not found in archive, falling through..."
        return 1
      fi
    fi

    chmod +x "$INSTALL_DIR/neotrix"
    rm -rf "$tmpdir"

    if [[ -f "$INSTALL_DIR/neotrix" ]]; then
      ok "NeoTrix binary downloaded to $INSTALL_DIR/neotrix"
      return 0
    fi
  else
    rm -rf "$tmpdir"
    if [[ "$version_tag" != "latest" ]]; then
      warn "Pre-built binary not found for ${TARGET} version ${version_tag}, falling through..."
    else
      warn "Pre-built binary not found for ${TARGET}, falling through..."
    fi
  fi

  return 1
}

install_via_source() {
  command -v git &>/dev/null || return 1
  command -v cargo &>/dev/null || return 1

  header "Installing from source"

  local tmpdir
  tmpdir="$(mktemp -d)"
  local checkout="$tmpdir/neotrix"

  info "Cloning ${REPO}..."
  if ! git clone --depth 1 "https://github.com/${REPO}.git" "$checkout" 2>&1 | sed 's/^/  /'; then
    rm -rf "$tmpdir"
    error "Failed to clone repository"
  fi

  info "Building release binary (this may take a while)..."
  (
    cd "$checkout"
    cargo build --release -p neotrix 2>&1 | sed 's/^/  /'
  )

  mkdir -p "$INSTALL_DIR"
  cp "$checkout/target/release/neotrix" "$INSTALL_DIR/neotrix"
  chmod +x "$INSTALL_DIR/neotrix"
  rm -rf "$tmpdir"

  if [[ -f "$INSTALL_DIR/neotrix" ]]; then
    ok "NeoTrix built from source"
    return 0
  fi

  error "Source build produced no binary"
}

# ======================== PATH Update ========================

update_path() {
  [[ "$NO_MODIFY_PATH" == true ]] && return 0

  header "Adding NeoTrix to PATH"

  local shell_rc=""
  local shell_name

  shell_name="$(basename "${SHELL:-bash}")"

  case "$shell_name" in
    bash) shell_rc="$HOME/.bashrc" ;;
    zsh)  shell_rc="$HOME/.zshrc" ;;
    fish) shell_rc="$HOME/.config/fish/config.fish" ;;
    *)    shell_rc="$HOME/.bashrc" ;;
  esac

  local path_line
  if [[ "$shell_name" == "fish" ]]; then
    path_line="fish_add_path $INSTALL_DIR"
  else
    path_line="export PATH=\"\$PATH:$INSTALL_DIR\""
  fi

  if [[ "$shell_name" == "fish" ]]; then
    mkdir -p "$(dirname "$shell_rc")"
  fi

  if ! grep -qF "$INSTALL_DIR" "$shell_rc" 2>/dev/null; then
    echo "" >> "$shell_rc"
    echo "# NeoTrix" >> "$shell_rc"
    echo "$path_line" >> "$shell_rc"
    ok "Added $INSTALL_DIR to PATH in $shell_rc"
    info "Run 'source $shell_rc' or restart your shell to use neotrix"
  else
    ok "PATH already configured in $shell_rc"
  fi
}

# ======================== Post-Install ========================

print_success() {
  local binary="$INSTALL_DIR/neotrix"
  local version_display=""

  if [[ -f "$binary" ]]; then
    version_display="$("$binary" --version 2>/dev/null || echo "unknown")"
  fi

  cat <<EOF

$(printf "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}")
$(printf "${GREEN}  ✅ NeoTrix installed successfully!${NC}")
$(printf "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}")

  $(printf "${CYAN}Version:${NC}")   ${version_display:-$VERSION}
  $(printf "${CYAN}Location:${NC}")  $binary
  $(printf "${CYAN}Platform:${NC}")  ${OS}-${ARCH}

  $(printf "${YELLOW}Quick start:${NC}")
    neotrix               # Launch TUI
    neotrix run "..."     # One-shot prompt
    neotrix --help        # All commands

  $(printf "${YELLOW}Configuration:${NC}")
    Config: ${NEOTRIX_DIR}/config.toml
    First run auto-launches provider setup wizard

  $(printf "${YELLOW}Documentation:${NC}")
    ${REPO_URL}

  $(printf "${YELLOW}Uninstall:${NC}")
    curl -fsSL ${RAW_BASE}/uninstall.sh | bash

EOF
}

# ======================== Cleanup Trap ========================

cleanup() {
  local exit_code=$?
  if [[ $exit_code -ne 0 ]]; then
    printf "\n${RED}Installation failed (exit code: %d).${NC}\n" $exit_code
    printf "  Report issues at: ${CYAN}${REPO_URL}/issues${NC}\n"
  fi
}
trap cleanup EXIT

# ======================== Main ========================

main() {
  printf "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
  printf "${BLUE}      NeoTrix Installer v0.18.0${NC}\n"
  printf "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n\n"

  detect_platform
  check_deps

  mkdir -p "$INSTALL_DIR"
  mkdir -p "$NEOTRIX_DIR"

  info "Installing NeoTrix to $INSTALL_DIR"

  if install_via_brew; then
    :
  elif install_via_cargo; then
    :
  elif install_via_binary; then
    :
  elif install_via_source; then
    :
  else
    error "All install methods failed. Install Rust: https://rustup.rs"
  fi

  update_path
  print_success
}

main "$@"
