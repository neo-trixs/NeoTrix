#!/usr/bin/env python3
"""Seed ArXiv paper IDs into the crawl queue for later absorption."""
import sqlite3, time, os

def main():
    db = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
    conn = sqlite3.connect(db)
    c = conn.cursor()

    # Get existing ArXiv paper IDs already in DB
    c.execute("SELECT DISTINCT substr(url, instr(url, 'abs/')+4) FROM knowledge_nodes WHERE url LIKE '%arxiv.org/abs/%'")
    existing = set()
    for r in c.fetchall():
        if r[0]:
            paper_id = r[0].strip('/')
            existing.add(paper_id)

    # Also check crawl queue
    c.execute("SELECT DISTINCT substr(url, instr(url, 'abs/')+4) FROM crawl_queue WHERE url LIKE '%arxiv%'")
    for r in c.fetchall():
        if r[0]:
            paper_id = r[0].strip('/')
            existing.add(paper_id)

    print(f"Already in DB/crawl_queue: {len(existing)} papers")
    for e in sorted(existing)[:10]:
        print(f"  {e}")

    PAPERS = [
        # AI/ML Core - Landmark Papers
        ('1706.03762', 'Attention Is All You Need - Transformer'),
        ('1810.04805', 'BERT: Pre-training of Deep Bidirectional Transformers'),
        ('2005.14165', 'Language Models are Few-Shot Learners (GPT-3)'),
        ('2103.00020', 'CLIP: Learning Transferable Visual Models From Natural Language Supervision'),
        ('2204.06125', 'Diffusion Models Beat GANs on Image Synthesis'),
        ('2212.03551', 'LLaMA: Open and Efficient Foundation Language Models'),
        ('2302.13971', 'Llama 2: Open Foundation and Fine-Tuned Chat Models'),
        ('2303.08774', 'GPT-4 Technical Report'),
        ('2203.02155', 'Training language models to follow instructions (InstructGPT)'),
        ('2307.09288', 'DPO: Direct Preference Optimization'),
        ('2203.14465', 'Toolformer: Language Models Can Teach Themselves to Use Tools'),
        ('2306.06082', 'Tree of Thoughts: Deliberate Problem Solving'),
        ('1312.6114', 'Auto-Encoding Variational Bayes (VAE)'),
        ('1406.2661', 'Generative Adversarial Networks (GAN)'),
        ('2101.08348', 'Scaling Laws for Neural Language Models'),

        # Computer Vision
        ('1512.03385', 'Deep Residual Learning for Image Recognition (ResNet)'),
        ('1409.1556', 'Very Deep Convolutional Networks for Large-Scale Image Recognition (VGG)'),
        ('1603.04467', 'TensorFlow: A System for Large-Scale Machine Learning'),

        # Systems & Engineering
        ('1609.08144', 'Google Neural Machine Translation System'),
        ('1909.07653', 'NSynth: Neural Audio Synthesis'),

        # Science & Math
        ('1803.00065', 'Neural Ordinary Differential Equations'),
        ('1905.05644', 'AlphaFold: Protein Structure Prediction'),
        ('1907.10597', 'MuZero: Grandmaster-level chess without search'),
        ('2002.08909', 'Switch Transformers: Scaling to Trillion Parameter Models'),
        ('2303.18223', 'Graph Neural Networks: A Review'),
        ('2207.01643', 'Memorization in Neural Networks'),
        ('1906.02655', 'BERT-like models for scientific text'),
        ('2003.01144', 'A Survey on Reinforcement Learning'),
        ('2103.01121', 'A Survey of Large Language Models'),
        ('1010.3003', 'The Large Scale Structure of Spacetime'),
    ]

    new_count = 0
    existing_count = 0
    now = int(time.time())

    for paper_id, title in PAPERS:
        url = f'https://arxiv.org/abs/{paper_id}'
        if paper_id in existing:
            print(f"  EXISTS: {paper_id} - {title[:40]}")
            existing_count += 1
            continue

        c.execute("""
            INSERT INTO crawl_queue (url, depth, domain, priority, status, discovered_at)
            VALUES (?, 0, 'arxiv.org', 2, 'pending', ?)
        """, (url, now))
        new_count += 1
        print(f"  ADDED: {paper_id} - {title[:40]}")

    conn.commit()
    print(f"\nResults: {new_count} new papers added, {existing_count} already existed")
    print(f"Total papers in crawl queue: ", end='')
    c.execute("SELECT count(*) FROM crawl_queue WHERE domain='arxiv.org'")
    print(c.fetchone()[0])
    conn.close()

if __name__ == "__main__":
    main()
