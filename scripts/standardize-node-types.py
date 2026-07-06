#!/usr/bin/env python3
"""
Standardize node_type casing across ~86K nodes.
Maps known lowercase types to capitalized forms.
Usage: python3 standardize-node-types.py [--dry-run]
"""

import sqlite3
import os
import sys

DB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")

# Canonical mapping: lowercase → correct casing
TYPE_MAP = {
    "paper": "Paper",
    "concept": "Concept",
    "repository": "Repository",
    "resource": "Resource",
    "article": "Article",
    "insight": "Insight",
    "book": "Book",
    "image": "Image",
    "course": "Course",
    "source": "Source",
    "theory": "Theory",
    "algorithm": "Algorithm",
    "reference": "Reference",
    "organization": "Organization",
    "person": "Person",
    "textbook": "Textbook",
    "evolution_pattern": "EvolutionPattern",
    "conversation_evolution": "ConversationEvolution",
    "external": "External",
}

# Types already capitalized (should stay as-is)
CAPITALIZED_TYPES = {
    "Paper", "Concept", "Repository", "Resource", "Article",
    "Insight", "Book", "Image", "Course", "Source", "Theory",
    "Algorithm", "Reference", "Organization", "Person",
    "Textbook", "EvolutionPattern", "ConversationEvolution",
    "External", "Guide", "Framework", "MeetingNote",
}


def main():
    dry_run = "--dry-run" in sys.argv

    db = sqlite3.connect(DB_PATH)
    db.execute("PRAGMA busy_timeout=30000")
    db.execute("PRAGMA journal_mode=WAL")

    # Count current state
    c = db.execute("SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type ORDER BY COUNT(*) DESC")
    print(f"{'Node Type':<30} {'Count':>8} {'Action':>12}")
    print("-" * 52)

    total_updates = 0
    updates_by_type = {}

    for row in c.fetchall():
        ntype, count = row
        canonical = TYPE_MAP.get(ntype.lower())

        if ntype in CAPITALIZED_TYPES:
            action = "keep"
        elif canonical and canonical != ntype:
            action = f"→ {canonical}"
            total_updates += count
            updates_by_type[ntype] = (canonical, count)
        else:
            action = "keep"

        print(f"{ntype:<30} {count:>8} {action:>12}")

    print(f"\nTotal nodes to update: {total_updates}")
    for old, (new, cnt) in sorted(updates_by_type.items()):
        print(f"  {old} ({cnt}) → {new}")

    if dry_run:
        print("\nDRY RUN — skipping writes")
        db.close()
        return

    if total_updates == 0:
        print("\nNothing to update. Exiting.")
        db.close()
        return

    # Confirm
    print(f"\nProceeding with {total_updates} updates...")

    # Perform updates
    db.execute("BEGIN TRANSACTION")
    for old_type, (new_type, count) in updates_by_type.items():
        db.execute(
            "UPDATE nodes SET node_type = ? WHERE node_type = ?",
            (new_type, old_type)
        )
        print(f"  {old_type} ({count}) → {new_type}: OK")

    # Rebuild FTS5 index after type changes
    try:
        db.execute("INSERT INTO nodes_fts(nodes_fts) VALUES('rebuild')")
        print("  FTS5 index rebuilt")
    except Exception as e:
        print(f"  FTS5 rebuild (may fail in WAL mode): {e}")

    db.commit()
    print(f"\nDone! Updated {total_updates} nodes.")


if __name__ == "__main__":
    main()
