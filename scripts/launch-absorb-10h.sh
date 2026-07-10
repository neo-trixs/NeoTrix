#!/usr/bin/env bash
# NeoTrix 10-Hour Continuous Absorption Launcher
# Kills old pipeline, fixes KB, injects seeds, launches continuous absorption.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  NeoTrix 10-Hour Continuous Absorption Pipeline            ║"
echo "║  $(date)                        ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── 1. Kill existing pipeline ──
if [ -f "$HOME/.neotrix/auto-absorb.pid" ]; then
    OLD_PID=$(cat "$HOME/.neotrix/auto-absorb.pid")
    echo "Killing old pipeline (PID $OLD_PID)..."
    kill "$OLD_PID" 2>/dev/null || true
    rm -f "$HOME/.neotrix/auto-absorb.pid"
    sleep 2
fi

# ── 2. KB structural fixes ──
echo ""
echo "═══ Phase 1: KB Structural Fixes ═══"
neotrix /kb fix structural
echo ""

# ── 3. Inject crawl queue seeds ──
echo "═══ Phase 2: Inject Crawl Queue Seeds ═══"
neotrix /kb seed crawl-queue
echo ""

# ── 4. KB stats before absorption ──
echo "═══ Phase 3: Pre-Absorption KB Snapshot ═══"
python3 -c "
import sqlite3, os
db = sqlite3.connect(os.path.expanduser('~/.neotrix/knowledge.db'))
print(f'  Nodes:       {db.execute(\"SELECT COUNT(*) FROM nodes\").fetchone()[0]}')
print(f'  Edges:       {db.execute(\"SELECT COUNT(*) FROM edges\").fetchone()[0]}')
print(f'  Empty:       {db.execute(\"SELECT COUNT(*) FROM nodes WHERE content IS NULL OR content=\\\"\\\"\").fetchone()[0]}')
print(f'  Orphaned:    {db.execute(\"SELECT COUNT(*) FROM nodes n WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.source_id=n.id OR e.target_id=n.id)\").fetchone()[0]}')
print(f'  Crawl pend:  {db.execute(\"SELECT COUNT(*) FROM crawl_queue WHERE status=\\\"pending\\\"\").fetchone()[0]}')
print(f'  Embeddings:  {db.execute(\"SELECT COUNT(*) FROM embeddings\").fetchone()[0]}')
db.close()
"
echo ""

# ── 5. Launch absorption (continuous, no sleep between cycles) ──
echo "═══ Phase 4: Launch 10-Hour Continuous Absorption ═══"
echo "  Mode: continuous (no sleep between cycles)"
echo "  Crawl queue batch: 100/cycle"
echo "  ArXiv fill: 30/cycle"
echo "  Wikipedia: 10/cycle"
echo "  Max runtime: ~10 hours (auto-shutdown)"
echo ""

nohup python3 scripts/neotrix-auto-absorb.py \
    --interval 1 \
    > /tmp/neotrix-absorb-10h.log 2>&1 &

NEW_PID=$!
echo "$NEW_PID" > "$HOME/.neotrix/auto-absorb.pid"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Pipeline started! PID: $NEW_PID"
echo "║  Log:   tail -f /tmp/neotrix-absorb-10h.log"
echo "║  Log:   tail -f ~/.neotrix/auto-absorb-log.jsonl"
echo "║  KB:    ~/.neotrix/knowledge.db"
echo "║  Stop:  kill $NEW_PID"
echo "╚══════════════════════════════════════════════════════════════╝"
