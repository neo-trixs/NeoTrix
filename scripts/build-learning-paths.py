#!/usr/bin/env python3
"""
Build learning paths: connect Paper nodes → Concept nodes via keyword matching.
Extracts top keywords from paper abstracts/Content, matches against Concept titles/domains,
creates prerequisite_of edges (weight=0.5) for strong matches.

Usage: python3 build-learning-paths.py [--dry-run]
"""

import sqlite3
import os
import sys
import re

DB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")

# Domain→keyword mapping for concept matching
DOMAIN_KEYWORDS = {
    "Technology": [
        "computer", "software", "algorithm", "programming", "machine learning",
        "deep learning", "neural network", "artificial intelligence", "data",
        "code", "system", "model", "network", "computing", "language model"
    ],
    "Mathematics": [
        "mathematics", "algebra", "calculus", "geometry", "topology", "statistics",
        "probability", "optimization", "linear algebra", "graph theory", "number",
        "theorem", "proof", "equation", "function"
    ],
    "Physics": [
        "physics", "quantum", "mechanics", "thermodynamics", "electromagnetism",
        "relativity", "particle", "wave", "energy", "force", "motion", "field"
    ],
    "Biology": [
        "biology", "cell", "dna", "gene", "evolution", "protein", "organism",
        "molecular", "genetics", "ecology", "neuroscience", "neuron", "brain"
    ],
    "Earth Science": [
        "earth", "geology", "climate", "weather", "ocean", "atmosphere",
        "seismology", "volcano", "mineral", "environment"
    ],
    "Astronomy": [
        "astronomy", "star", "galaxy", "planet", "cosmology", "telescope",
        "solar", "nebula", "black hole", "universe", "space"
    ],
    "Chemistry": [
        "chemistry", "molecule", "compound", "reaction", "element", "atom",
        "chemical", "organic", "inorganic", "biochemistry"
    ],
    "Psychology": [
        "psychology", "cognition", "behavior", "perception", "memory",
        "attention", "learning", "emotion", "consciousness", "mind"
    ],
    "Philosophy": [
        "philosophy", "logic", "ethics", "metaphysics", "epistemology",
        "reasoning", "argument", "ontology", "consciousness"
    ],
    "Medicine": [
        "medicine", "disease", "clinical", "diagnosis", "treatment", "drug",
        "patient", "therapy", "health", "medical"
    ],
}

STOP_WORDS = {
    "the", "a", "an", "of", "in", "to", "and", "is", "for", "on", "that",
    "with", "by", "this", "are", "be", "from", "or", "as", "we", "it",
    "not", "can", "but", "they", "may", "these", "their", "use", "using",
    "used", "based", "also", "such", "which", "than", "its", "will", "has",
    "been", "was", "were", "been", "would", "could", "should", "both", "each",
    "all", "most", "some", "more", "very", "too", "just", "about", "how",
    "what", "when", "where", "who", "new", "two", "three", "first", "second",
    "one", "well", "have", "had", "do", "does", "did", "into", "over",
    "between", "through", "during", "before", "after", "above", "below",
    "show", "shown", "shows", "via", "per", "without", "within", "across",
    "among", "along", "while", "however", "therefore", "because", "result",
    "results", "method", "approach", "paper", "work", "study", "problem",
}


def extract_keywords(text, top_n=10):
    """Extract top-N keywords from text using TF-style scoring."""
    if not text:
        return []

    # Normalize
    text = text.lower()
    text = re.sub(r'[^a-z0-9\s]', ' ', text)

    words = text.split()
    # Filter stop words + short words
    words = [w for w in words if w not in STOP_WORDS and len(w) > 3]

    # Count
    freq = {}
    for w in words:
        freq[w] = freq.get(w, 0) + 1

    # Score by frequency
    scored = [(freq[w], w) for w in freq]
    scored.sort(reverse=True)

    return [w for _, w in scored[:top_n]]


def compute_tfidf_similarity(keywords, concept_title, concept_domain):
    """Compute simple similarity between paper keywords and a concept."""
    if not keywords:
        return 0.0

    title_lower = concept_title.lower()
    score = 0.0

    # Direct keyword match in concept title (strong signal)
    for kw in keywords:
        if kw in title_lower:
            score += 0.3
        # Partial match for compound terms
        for word in kw.split():
            if word in title_lower and len(word) > 3:
                score += 0.1

    # Domain match
    domain_text = (concept_domain or "").lower()
    domain_kws = DOMAIN_KEYWORDS.get(concept_domain, [])
    if domain_kws:
        for kw in keywords:
            if kw in domain_kws:
                score += 0.2
            # Check if any domain keyword matches a paper keyword
            for dkw in domain_kws:
                if dkw in ' '.join(keywords):
                    score += 0.15

    return min(score, 1.0)


def main():
    dry_run = "--dry-run" in sys.argv

    db = sqlite3.connect(DB_PATH)
    db.execute("PRAGMA busy_timeout=30000")
    db.execute("PRAGMA journal_mode=WAL")

    # Get all Paper nodes with content
    papers = db.execute(
        "SELECT id, title, content, domain FROM nodes WHERE node_type IN ('Paper', 'paper') AND content IS NOT NULL AND content != ''"
    ).fetchall()
    print(f"Paper nodes with content: {len(papers)}")

    # Get all Concept nodes
    concepts = db.execute(
        "SELECT id, title, domain FROM nodes WHERE node_type IN ('Concept', 'concept')"
    ).fetchall()
    print(f"Concept nodes: {len(concepts)}")

    # Check existing prerequisite_of edges
    existing = set()
    for row in db.execute("SELECT source_id, target_id FROM edges WHERE relation_type='prerequisite_of'"):
        existing.add((row[0], row[1]))

    print(f"Existing prerequisite_of edges: {len(existing)}")

    edges_created = 0
    keywords_by_paper = {}

    print("\nAnalyzing paper→concept connections...")

    for pid, ptitle, pcontent, pdomain in papers:
        text = f"{ptitle or ''} {pcontent or ''}"
        keywords = extract_keywords(text, top_n=15)
        keywords_by_paper[pid] = keywords

        if not keywords:
            continue

        best_match = None
        best_score = 0.0

        for cid, ctitle, cdomain in concepts:
            if (pid, cid) in existing:
                continue

            score = compute_tfidf_similarity(keywords, ctitle, cdomain)
            if score > 0.4 and score > best_score:
                best_match = cid
                best_score = score

        if best_match:
            if not dry_run:
                db.execute(
                    """INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at)
                       VALUES (?, ?, ?, 'prerequisite_of', ?, 'auto-mapped via keyword matching', strftime('%s','now'))""",
                    (f"auto-{pid[:8]}-{best_match[:8]}", pid, best_match, round(best_score, 2))
                )
            edges_created += 1
            progress = f"  score={best_score:.2f}: {ptitle[:50] if ptitle else '?'[:30]} → {[c[1][:40] for c in concepts if c[0]==best_match][0] if best_match else '?'}"
            print(progress[:100])

    if not dry_run:
        db.commit()

    print(f"\n{'='*60}")
    print(f"Edges created: {edges_created}")
    print(f"Dry run: {dry_run}")

    # Also create domain-level edges: Paper → Domain hub concept
    if not dry_run and edges_created > 0:
        for domain, kws in DOMAIN_KEYWORDS.items():
            hub = db.execute(
                "SELECT id FROM nodes WHERE node_type='Concept' AND title LIKE ? LIMIT 1",
                (f"{domain}%",)
            ).fetchone()
            if hub:
                # Connect papers in this domain to the hub
                matching = db.execute(
                    "SELECT id FROM nodes WHERE node_type IN ('Paper', 'paper') AND domain LIKE ?",
                    (f"%{domain}%",)
                ).fetchall()
                for m in matching:
                    db.execute(
                        """INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at)
                           VALUES (?, ?, ?, 'SubTopicOf', 0.3, 'auto-domain-hub', strftime('%s','now'))""",
                        (f"hub-{domain[:4]}-{m[0][:8]}", m[0], hub[0])
                    )
        db.commit()
        print(f"Domain hub edges also created")


if __name__ == "__main__":
    main()
