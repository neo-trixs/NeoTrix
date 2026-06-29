#!/bin/bash
set -euo pipefail

# install-transit-daemon.sh — 一键安装 neotrix-transit 守护进程
# 用法: sudo ./install-transit-daemon.sh

echo "=== NeoTrix Transit Daemon Installer ==="

# Determine workspace root (script location relative path)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 1. Build the binary
echo "[1/5] Building neotrix-transit (stealth-net)..."
cd "$WORKSPACE_ROOT"
cargo build --release --features stealth-net -p neotrix --bin neotrix-transit
echo "  -> Done"

# 2. Install binary
echo "[2/5] Installing binary to /usr/local/bin/..."
sudo cp target/release/neotrix-transit /usr/local/bin/neotrix-transit
sudo chmod +x /usr/local/bin/neotrix-transit
echo "  -> /usr/local/bin/neotrix-transit installed"

# 3. Create config directory and default config
echo "[3/5] Creating config..."
mkdir -p "$HOME/.neotrix"
if [ ! -f "$HOME/.neotrix/config.toml" ]; then
    # Let the daemon auto-generate default config on first run
    echo "  -> Default config will be auto-generated on first run"
else
    echo "  -> Existing config found at $HOME/.neotrix/config.toml (kept)"
fi

# 4. Install launchd plist
echo "[4/5] Installing launchd plist..."
cp "$SCRIPT_DIR/com.neotrix.transit-daemon.plist" "$HOME/Library/LaunchAgents/com.neotrix.transit-daemon.plist"
chmod 644 "$HOME/Library/LaunchAgents/com.neotrix.transit-daemon.plist"
echo "  -> ~/Library/LaunchAgents/com.neotrix.transit-daemon.plist installed"

# 5. Load the launchd job
echo "[5/5] Loading launchd job..."
launchctl load "$HOME/Library/LaunchAgents/com.neotrix.transit-daemon.plist" 2>/dev/null || \
    echo "  -> Note: launchctl load failed (may need to logout/login or run manually)"

echo ""
echo "=== Installation Complete ==="
echo ""
echo "The transit daemon is now running as a launchd service."
echo ""
echo "Commands:"
echo "  launchctl start com.neotrix.transit-daemon    # Start"
echo "  launchctl stop com.neotrix.transit-daemon     # Stop"
echo "  launchctl unload ~/Library/LaunchAgents/com.neotrix.transit-daemon.plist  # Remove"
echo ""
echo "Logs: /tmp/neotrix-transit-daemon.log"
echo ""
echo "To run transit daemon manually (without launchd):"
echo "  cargo run --features stealth-net --bin neotrix-transit"
echo ""
echo "To install sudoers for pfctl (optional, for transparent proxy):"
echo '  echo "ALL ALL=(ALL) NOPASSWD: /sbin/pfctl *" | sudo tee /etc/sudoers.d/neotrix-transit'
