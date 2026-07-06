#!/usr/bin/env python3
"""Absorb Complete-FABLE.5-traces-2M dataset into KB via Rust bridge.

Downloads the 2M-row parquet dataset from HuggingFace, extracts E8 state
transition patterns (from the `row_json` field containing the Fable-5
reasoning trace), and writes transition counts to a JSON file that the
NeoTrix CommunityDataIngester Rust module can load at runtime.

Usage:
  # Download & process (requires internet + huggingface_hub)
  python3 scripts/absorb-fable-2m.py --download

  # Use cached parquet
  python3 scripts/absorb-fable-2m.py --cached data/complete-fable-2m.parquet

  # Just print dataset stats
  python3 scripts/absorb-fable-2m.py --stats

Output:
  data/community_transitions.json  ← consumed by Rust RuntimeCommunityLoader
"""

import argparse
import json
import sys
from pathlib import Path

# ── Optional dependencies ──────────────────────────────────────────────
try:
    import pyarrow.parquet as pq
except ImportError:
    pq = None

try:
    from tqdm import tqdm
except ImportError:
    tqdm = None

DATA_DIR = Path(__file__).parent.parent / "data"
DATA_DIR.mkdir(exist_ok=True)

DATASET_REPO = "Glint-Research/Complete-FABLE.5-traces-2M"
PARQUET_PATH = DATA_DIR / "complete-fable-2m.parquet"
OUTPUT_PATH = DATA_DIR / "community_transitions.json"


def download_parquet():
    """Download the dataset parquet file from HuggingFace."""
    try:
        from huggingface_hub import hf_hub_download
    except ImportError:
        print("❌ huggingface_hub not installed. Run: pip install huggingface_hub")
        sys.exit(1)

    print(f"⬇️  Downloading {DATASET_REPO}...")
    path = hf_hub_download(
        repo_id=DATASET_REPO,
        filename="data/train-00000-of-00001.parquet",
        repo_type="dataset",
    )
    # Copy to local data dir
    import shutil
    shutil.copy2(path, PARQUET_PATH)
    print(f"✅ Saved to {PARQUET_PATH} ({PARQUET_PATH.stat().st_size / 1e6:.1f} MB)")


def analyze_parquet(stats_only: bool):
    """Read parquet, extract transitions, write JSON."""
    if pq is None:
        print("❌ pyarrow not installed. Run: pip install pyarrow")
        sys.exit(1)

    if not PARQUET_PATH.exists():
        print(f"❌ Parquet not found at {PARQUET_PATH}. Run with --download first.")
        sys.exit(1)

    print(f"📖 Reading {PARQUET_PATH}...")
    table = pq.read_table(PARQUET_PATH)
    df = table.to_pandas()
    print(f"   Rows: {len(df)}, Columns: {list(df.columns)}")

    if stats_only:
        print("\n📊 Dataset Stats:")
        print(f"   Total rows: {len(df)}")
        for col in df.columns:
            if df[col].dtype == "object":
                non_null = df[col].notna().sum()
                print(f"   {col}: {non_null}/{len(df)} non-null, "
                      f"sample: {str(df[col].iloc[0])[:100] if non_null > 0 else 'N/A'}")
            else:
                print(f"   {col}: mean={df[col].mean():.3f}, "
                      f"min={df[col].min()}, max={df[col].max()}")
        return

    # ── Transition Extraction ──────────────────────────────────────
    # The `row_json` field contains the Fable-5 reasoning trace as JSON.
    # Each trace has a sequence of reasoning steps with E8 mode assignments.
    # We extract (from_mode, to_mode, task_type, cot_length) tuples.

    transitions: dict[str, list] = {
        "General": [],
        "Reasoning": [],
        "Math": [],
        "Coding": [],
        "Agentic": [],
        "Creative": [],
    }

    task_type_counts: dict[str, int] = {}
    total_valid = 0

    iter_rows = tqdm(df.iterrows(), total=len(df), desc="Extracting") if tqdm else df.iterrows()

    for idx, row in iter_rows:
        try:
            # Parse row_json — might be string or already dict
            raw = row.get("row_json", row.get("raw", "{}"))
            if isinstance(raw, str):
                data = json.loads(raw)
            elif isinstance(raw, dict):
                data = raw
            else:
                continue

            # Extract reasoning trajectory
            trace = data.get("reasoning_trace", data.get("trace", data.get("cot", [])))
            if not trace or not isinstance(trace, list):
                continue

            # Infer task type from metadata or content
            tt_raw = str(row.get("task_type", data.get("task_type", "General")))
            tt = tt_raw if tt_raw in transitions else "General"
            task_type_counts[tt] = task_type_counts.get(tt, 0) + 1

            # Extract mode sequence from trace
            modes = []
            for step in trace:
                if isinstance(step, dict):
                    mode = step.get("mode", step.get("e8_mode", step.get("state")))
                    if mode is not None:
                        modes.append(int(mode) % 64)
                elif isinstance(step, int):
                    modes.append(step % 64)

            if len(modes) < 2:
                continue

            # Record transitions
            for i in range(len(modes) - 1):
                from_s = modes[i]
                to_s = modes[i + 1]
                if from_s != to_s:  # skip self-loops for community data
                    transitions[tt].append([from_s, to_s])

            total_valid += 1

        except (json.JSONDecodeError, ValueError, TypeError, KeyError):
            continue

    # ── Aggregate transition counts ────────────────────────────────
    from collections import Counter

    output = {}
    for task_type, t_list in transitions.items():
        counter: Counter = Counter()
        for from_s, to_s in t_list:
            counter[(from_s, to_s)] += 1
        output[task_type] = [
            {"from": int(f), "to": int(t), "count": int(c)}
            for (f, t), c in counter.most_common()
        ]

    output["_meta"] = {
        "source": DATASET_REPO,
        "total_rows": int(len(df)),
        "valid_traces": int(total_valid),
        "task_type_distribution": task_type_counts,
        "total_transition_pairs": int(sum(
            sum(item["count"] for item in v)
            for k, v in output.items() if isinstance(v, list)
        )),
    }

    # ── Write output ───────────────────────────────────────────────
    with open(OUTPUT_PATH, "w") as f:
        json.dump(output, f, indent=2)

    print(f"\n✅ Extracted transitions saved to {OUTPUT_PATH}")
    print(f"   Valid traces: {total_valid}/{len(df)}")
    print(f"   Task type distribution: {task_type_counts}")
    print(f"   Total transition pairs: {output['_meta']['total_transition_pairs']}")


def main():
    parser = argparse.ArgumentParser(
        description="Absorb Complete-FABLE.5-traces-2M into KB"
    )
    parser.add_argument("--download", action="store_true", help="Download parquet from HF")
    parser.add_argument("--cached", type=str, help="Path to cached parquet file")
    parser.add_argument("--stats", action="store_true", help="Print dataset stats only")
    args = parser.parse_args()

    if args.cached:
        global PARQUET_PATH
        PARQUET_PATH = Path(args.cached)

    if args.download:
        download_parquet()

    if args.stats:
        analyze_parquet(stats_only=True)
    else:
        if args.download or PARQUET_PATH.exists():
            analyze_parquet(stats_only=False)
        else:
            print("💡 No parquet found. Use --download to fetch, or --cached to specify path.")
            print(f"   Expected at: {PARQUET_PATH}")


if __name__ == "__main__":
    main()
