#!/usr/bin/env python3
"""Generate KB embeddings for all nodes without them.

Uses OpenAI-compatible embedding API. Requires env vars:
  NEOTRIX_EMBEDDING_API_KEY (required)
  NEOTRIX_EMBEDDING_BASE_URL (default: https://api.openai.com/v1)
  NEOTRIX_EMBEDDING_MODEL (default: text-embedding-3-small)
  NEOTRIX_EMBEDDING_DIMENSION (default: 768)

Usage:
  NEOTRIX_EMBEDDING_API_KEY=sk-... python3 scripts/kb-generate-embeddings.py

Stores vectors as little-endian f32 blobs in the `embeddings` table,
matching the Rust nt_memory_embed::store_embedding() format.
"""
import sqlite3, json, time, os, struct, urllib.request, urllib.error

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
API_KEY = os.environ.get("NEOTRIX_EMBEDDING_API_KEY", "")
BASE_URL = os.environ.get("NEOTRIX_EMBEDDING_BASE_URL", "https://api.openai.com/v1")
MODEL = os.environ.get("NEOTRIX_EMBEDDING_MODEL", "text-embedding-3-small")
DIMENSION = int(os.environ.get("NEOTRIX_EMBEDDING_DIMENSION", "768"))
BATCH_SIZE = 20  # Max texts per API call
RATE_LIMIT_DELAY = 0.5  # Seconds between batches

def vector_to_blob(vec: list[float]) -> bytes:
    """Serialize Vec<f32> to little-endian byte blob (matching Rust)."""
    return struct.pack(f"<{len(vec)}f", *vec)

def embed_batch(texts: list[str]) -> list[list[float]]:
    """Call OpenAI-compatible embedding API for a batch of texts."""
    body = json.dumps({
        "input": texts,
        "model": MODEL,
        "dimensions": DIMENSION,
    }).encode("utf-8")

    req = urllib.request.Request(
        f"{BASE_URL}/embeddings",
        data=body,
        headers={
            "Authorization": f"Bearer {API_KEY}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        print(f"  ⚠️  API error {e.code}: {e.read().decode()[:200]}")
        return []
    except Exception as e:
        print(f"  ⚠️  Request failed: {e}")
        return []

    # Sort by index and extract embeddings
    indexed = [(d["index"], d["embedding"]) for d in data.get("data", [])]
    indexed.sort(key=lambda x: x[0])
    return [v for _, v in indexed]

def build_node_text(title: str, summary: str = "", content: str = "") -> str:
    """Build text for embedding from node fields (matching Rust)."""
    parts = [title]
    if summary:
        parts.append(f". {summary}")
    if content:
        parts.append(f". {content[:500]}")
    return "".join(parts)

def main():
    if not API_KEY:
        print("❌ NEOTRIX_EMbedding_API_KEY not set")
        print("  Set env var and try again, e.g.:")
        print("  NEOTRIX_EMBEDDING_API_KEY=sk-... python3 scripts/kb-generate-embeddings.py")
        return 1

    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()

    # Find nodes without embeddings
    missing = c.execute("""
        SELECT id, title, COALESCE(summary, ''), COALESCE(content, '')
        FROM nodes
        WHERE id NOT IN (SELECT node_id FROM embeddings)
        ORDER BY LENGTH(COALESCE(content, '')) DESC
    """).fetchall()

    total = len(missing)
    if total == 0:
        print("✅ All nodes already have embeddings")
        return 0

    print(f"═══ Generating embeddings for {total} nodes ═══")
    print(f"  Model: {MODEL}")
    print(f"  Dimension: {DIMENSION}")
    print(f"  Base URL: {BASE_URL}")
    print(f"  Batch size: {BATCH_SIZE}")
    print()

    processed = 0
    errors = 0

    for i in range(0, total, BATCH_SIZE):
        batch = missing[i:i + BATCH_SIZE]
        texts = [build_node_text(t[1], t[2], t[3]) for t in batch]
        node_ids = [t[0] for t in batch]

        vectors = embed_batch(texts)
        if not vectors:
            errors += len(batch)
            print(f"  ⚠️  Batch {i//BATCH_SIZE + 1} failed, skipping {len(batch)} nodes")
            time.sleep(RATE_LIMIT_DELAY * 2)
            continue

        for j, (nid, vec) in enumerate(zip(node_ids, vectors)):
            try:
                blob = vector_to_blob(vec)
                c.execute(
                    "INSERT OR REPLACE INTO embeddings (node_id, vector, dimension, model) VALUES (?, ?, ?, ?)",
                    (nid, blob, len(vec), MODEL)
                )
                processed += 1
            except Exception as e:
                errors += 1
                print(f"    ⚠️  Failed to store {nid}: {e}")

        conn.commit()

        progress = min(i + BATCH_SIZE, total)
        pct = progress * 100 // total
        print(f"  [{progress}/{total} {pct}%] batch {i//BATCH_SIZE + 1}/{(total+BATCH_SIZE-1)//BATCH_SIZE}")
        time.sleep(RATE_LIMIT_DELAY)

    conn.close()
    print(f"\n{'═' * 60}")
    print(f"  Processed: {processed}/{total}")
    print(f"  Errors: {errors}")
    print(f"{'═' * 60}")
    return 0 if errors == 0 else 1

if __name__ == "__main__":
    exit(main())
