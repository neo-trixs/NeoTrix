#!/usr/bin/env python3
"""NeoTrix Product Quantization (PQ) trainer.

Trains a PQ codebook from the existing `embeddings` table and writes compressed
codes to `embeddings_pq` + codebook to `pq_codebook`. This turns the empty
schema shells into a working ANN index: M sub-spaces × K centroids.

Usage:
  python3 scripts/kb-embed-pq.py                     # train M=24, K=256
  python3 scripts/kb-embed-pq.py --m 12 --k 256      # smaller codebook
  python3 scripts/kb-embed-pq.py --report            # print PQ status only
"""
import sqlite3
import struct
import os
import sys
import time
import argparse
import numpy as np

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))


def blob_to_vector(blob):
    return np.frombuffer(blob, dtype="<f4")


def load_vectors(conn, limit=None):
    q = "SELECT node_id, vector FROM embeddings"
    if limit:
        q += f" LIMIT {int(limit)}"
    rows = conn.execute(q).fetchall()
    ids = [r[0] for r in rows]
    vecs = np.stack([blob_to_vector(r[1]) for r in rows])
    return ids, vecs


def kmeans(data, k, iters=12, seed=42):
    rng = np.random.default_rng(seed)
    n, d = data.shape
    centroids = data[rng.choice(n, k, replace=False)].copy()
    for _ in range(iters):
        dists = ((data[:, None, :] - centroids[None, :, :]) ** 2).sum(axis=2)
        labels = dists.argmin(axis=1)
        for j in range(k):
            mask = labels == j
            if mask.sum() > 0:
                centroids[j] = data[mask].mean(axis=0)
    return centroids


def report(conn):
    total = conn.execute("SELECT COUNT(*) FROM embeddings").fetchone()[0]
    pq = conn.execute("SELECT COUNT(*) FROM embeddings_pq").fetchone()[0]
    cb = conn.execute("SELECT COUNT(*) FROM pq_codebook").fetchone()[0]
    print(f"PQ: {pq}/{total} vectors quantized, {cb} codebooks")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--m", type=int, default=24, help="number of sub-spaces (must divide 384)")
    ap.add_argument("--k", type=int, default=256, help="centroids per sub-space")
    ap.add_argument("--limit", type=int, default=0, help="train on subset only")
    ap.add_argument("--report", action="store_true")
    args = ap.parse_args()

    conn = sqlite3.connect(KB)
    if args.report:
        return report(conn)

    total = conn.execute("SELECT COUNT(*) FROM embeddings").fetchone()[0]
    if total < args.k * 4:
        print(f"Not enough vectors to train PQ: {total} < {args.k * 4}. Need more embeddings first.")
        return 1

    dim = 384
    if args.m <= 0 or dim % args.m != 0:
        print(f"--m must be a positive divisor of {dim}")
        return 1

    print(f"Loading {total} embeddings ...")
    ids, vecs = load_vectors(conn, args.limit or None)
    n = len(ids)
    sub_dim = dim // args.m
    print(f"  {n} vectors, {dim} dim, {args.m} sub-spaces x {args.k} centroids")

    # Train per-subspace codebooks
    t0 = time.time()
    codewords_all = []
    reshaped = vecs.reshape(n, args.m, sub_dim)
    for s in range(args.m):
        sub = reshaped[:, s, :]
        centroids = kmeans(sub, args.k, iters=10)
        codewords_all.append(centroids)
        print(f"  subspace {s + 1}/{args.m} trained ({100.0 * (s + 1) / args.m:.0f}%)", flush=True)

    codeword_blob = b"".join(
        np.ascontiguousarray(cw, dtype="<f4").tobytes() for cw in codewords_all
    )
    now = int(time.time())

    # Persist codebook
    cur = conn.execute(
        "INSERT INTO pq_codebook "
        "(m, ks, sub_dim, codewords, dimension, model, trained_at, num_vectors) "
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        (args.m, args.k, sub_dim, codeword_blob, dim, "all-MiniLM-L6-v2", now, n),
    )
    codebook_id = cur.lastrowid

    # Assign codes for all vectors
    print("  encoding all vectors ...")
    with conn:
        for i in range(0, n, 5000):
            batch = np.arange(i, min(i + 5000, n))
            for idx in batch:
                node_id = ids[idx]
                codes = []
                for s in range(args.m):
                    sub = reshaped[idx, s, :]
                    dists = ((codewords_all[s] - sub) ** 2).sum(axis=1)
                    codes.append(int(dists.argmin()))
                codes_blob = struct.pack(f"<{args.m}B", *codes)
                conn.execute(
                    "INSERT OR REPLACE INTO embeddings_pq (node_id, pq_codes, codebook_id) "
                    "VALUES (?, ?, ?)",
                    (node_id, codes_blob, codebook_id),
                )
    conn.commit()
    elapsed = time.time() - t0
    print(f"Done: {n} vectors quantized in {elapsed:.1f}s -> codebook #{codebook_id} "
          f"(M={args.m}, K={args.k}, {dim}/{args.m} per code)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
