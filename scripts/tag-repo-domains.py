#!/usr/bin/env python3
"""Tag Repository nodes in the KB with domain metadata via URL rules and content keywords."""
import sqlite3, os

def main():
    db = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
    conn = sqlite3.connect(db)
    c = conn.cursor()

    c.execute("""SELECT id, url, title, substr(coalesce(content,''),1,800) as c
                  FROM nodes
                  WHERE node_type='Repository'
                    AND (json_extract(metadata,'$.domain') IS NULL
                         OR json_extract(metadata,'$.domain') = '')""")
    repos = c.fetchall()
    print(f"Repos to tag: {len(repos)}\n")

    # Domain rules: (url_pattern, domain) - first match wins
    URL_RULES = [
        ('github.com/denysdovhan/', 'Technology'),
        ('github.com/typicode/', 'Technology'),
        ('github.com/pmndrs/', 'Technology'),
        ('github.com/vercel/', 'Technology'),
        ('github.com/puppeteer/', 'Technology'),
        ('github.com/public-apis/', 'Technology'),
        ('github.com/tachyons-css/', 'Technology'),
        ('github.com/justjavac/', 'Technology'),
        ('github.com/jaredpalmer/', 'Technology'),
        ('github.com/codelion/', 'Technology'),
        ('github.com/ZHZisZZ/', 'Technology'),
        ('github.com/SparkyWen/', 'Technology'),
        ('github.com/BalianWang/', 'Technology'),
        ('github.com/NousResearch/', 'Technology'),
        ('github.com/OpenHands/', 'Technology'),
        ('github.com/RunanywhereAI/', 'Technology'),
        ('github.com/antvis/', 'Technology'),
        ('github.com/docling-project/', 'Technology'),
        ('github.com/langchain-ai/', 'Technology'),
        ('github.com/Wilfred/', 'Technology'),
        ('github.com/OpenCut-app/', 'Technology'),
        ('github.com/vectorize-io/', 'Technology'),
        ('github.com/hanxiao/', 'Technology'),
        ('github.com/jianshuo/', 'Technology'),
        ('github.com/jerrywu001/', 'Technology'),
        ('github.com/tripleyak/', 'Technology'),
        ('github.com/Unclecheng-li/', 'Technology'),
        ('github.com/scikit-learn/', 'Technology'),
        ('github.com/scipy/', 'Technology'),
        ('github.com/matplotlib/', 'Technology'),
        ('github.com/networkx/', 'Technology'),
        ('github.com/sympy/', 'Technology'),
        ('github.com/d2l-ai/', 'Technology'),
        ('github.com/astropy/', 'Astronomy'),
        ('github.com/sunpy/', 'Astronomy'),
        ('github.com/yt-project/', 'Technology'),
        ('github.com/rdkit/', 'Chemistry'),
        ('github.com/geopy/', 'Geography'),
        ('github.com/github/opensource.guide', 'General'),
        ('github.com/ksindi/', 'General'),
        ('github.com/KunAgent/', 'Technology'),
    ]

    CONTENT_KEYWORDS = {
        'Technology': ['machine learning', 'deep learning', 'neural', 'python', 'javascript', 'api',
                       'software', 'framework', 'library', 'database', 'web', 'app', 'application',
                       'docker', 'kubernetes', 'react', 'node', 'typescript', 'compiler', 'algorithm',
                       'data', 'analysis', 'programming', 'code', 'tool', 'cli', 'server',
                       'graph', 'visualization', 'tensor', 'pytorch', 'tensorflow'],
        'Biology': ['biology', 'dna', 'rna', 'genome', 'protein', 'cell', 'gene', 'molecular',
                    'bio', 'organism', 'evolution', 'genetic'],
        'Physics': ['physics', 'quantum', 'particle', 'mechanics', 'electromagnetic', 'relativity',
                    'wave', 'feynman', 'newton', 'einstein', 'thermodynamics'],
        'Chemistry': ['chemistry', 'molecule', 'chemical', 'reaction', 'compound', 'element',
                      'organic', 'inorganic'],
        'Astronomy': ['astronomy', 'astrophysics', 'cosmology', 'telescope', 'solar', 'stellar',
                      'galaxy', 'planet', 'star', 'nebula', 'black hole'],
        'Mathematics': ['math', 'algebra', 'calculus', 'geometry', 'theorem', 'matrix', 'vector',
                        'statistics', 'probability', 'differential', 'linear'],
        'Geography': ['geography', 'map', 'gis', 'spatial', 'earth', 'coordinate', 'location'],
        'Music': ['music', 'audio', 'sound', 'synthesizer', 'dsp', 'signal', 'melody', 'rhythm'],
        'Art': ['art', 'design', 'graphic', 'illustration', 'typography', 'color', 'font',
                'animation', '3d', 'render', 'drawing'],
        'Education': ['education', 'learning', 'tutorial', 'course', 'curriculum', 'textbook',
                      'courseware'],
    }

    updated = 0
    for nid, url, title, content_preview in repos:
        url_lower = url.lower()
        title_lower = title.lower()
        content_lower = content_preview.lower()

        domain = None
        for pattern, d in URL_RULES:
            if pattern in url_lower or pattern in title_lower:
                domain = d
                break

        if not domain and content_preview:
            domain_scores = {}
            for d, keywords in CONTENT_KEYWORDS.items():
                score = sum(1 for kw in keywords if kw in content_lower)
                if score > 0:
                    domain_scores[d] = score
            if domain_scores:
                domain = max(domain_scores, key=domain_scores.get)

        if not domain:
            domain = 'Technology'

        c.execute("""
            UPDATE nodes SET metadata=json_set(coalesce(metadata,'{}'),'$.domain',?)
            WHERE id=?
        """, (domain, nid))
        updated += 1

    conn.commit()

    # Final breakdown
    c.execute("""SELECT json_extract(metadata,'$.domain') as d, count(*)
                  FROM nodes WHERE node_type='Repository' GROUP BY d ORDER BY 2 DESC""")
    print("Final repo domain breakdown:")
    for r in c.fetchall():
        print(f"  {r[0]}: {r[1]}")
    print(f"\nTotal tagged: {updated}")

    conn.close()

if __name__ == "__main__":
    main()
