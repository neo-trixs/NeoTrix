#!/usr/bin/env python3
"""
NeoTrix Auto-Absorption Pipeline — 4-source continuous KB ingestion.

Architecture:
  Cycle (repeat until ~10h total):
    [Phase 1] System Snapshot (KB stats, Rust build state)
    [Phase 2] Crawl Queue Processing (pending URLs from KB)
    [Phase 3] ArXiv Content Fill (Paper nodes with empty content)
    [Phase 4] GitHub Trending Discovery (new/watched repos)
    [Phase 5] Wikipedia Random Topic Absorption (new Concept nodes)
    [Phase 6] KB Internal Analysis (content distiller + panorama)
    [Phase 7] Meta-Cognition Summary (defects, patterns, todo)
    └─ sleep CYCLE_INTERVAL, repeat

Meta-Cognition Logging:
  - Writes every event to ~/.neotrix/auto-absorb-log.jsonl
  - Records defects to KB kv_store under meta_cognition namespace
  - Generates evolution_records for recurring defect patterns

Usage:
  python3 scripts/neotrix-auto-absorb.py [--cycles N] [--interval SEC] [--daemon]
    --cycles N    Run N cycles then exit (default: infinite)
    --interval S  Seconds between cycles (default: 600 = 10min)
    --daemon      Fork to background, detach from terminal
"""

import sys, os, json, time, sqlite3, hashlib, re, html, signal, traceback, random, urllib.parse

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from nt_api_client import AccessPipeline
from nt_normalizer import normalize_text, strip_markdown, normalize_lang

# ── Constants ──
KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
LOG_PATH = os.path.expanduser("~/.neotrix/auto-absorb-log.jsonl")
PID_PATH = os.path.expanduser("~/.neotrix/auto-absorb.pid")
DUPLICATE_BLACKLIST_NS = "dedup_blacklist"
SESSION_START = int(time.time())
STARTED_AT = SESSION_START
CYCLE_COUNT = 0
SHUTDOWN = False

# ── Signal Handling ──
def _handle_sigterm(signum, frame):
    global SHUTDOWN
    SHUTDOWN = True
    log("INFO", "signal", f"Received signal {signum}, shutting down gracefully after current phase")

signal.signal(signal.SIGTERM, _handle_sigterm)
signal.signal(signal.SIGINT, _handle_sigterm)

# ── Logging ──
def log(level, phase, message, extra=None):
    entry = {
        "ts": int(time.time()),
        "level": level,
        "cycle": CYCLE_COUNT,
        "phase": phase,
        "message": message,
        "uptime_sec": int(time.time()) - STARTED_AT,
    }
    if extra:
        entry["data"] = extra
    os.makedirs(os.path.dirname(LOG_PATH), exist_ok=True)
    with open(LOG_PATH, "a") as f:
        f.write(json.dumps(entry) + "\n")
        f.flush()
    prefix = f"[{time.strftime('%H:%M:%S')}][C{CYCLE_COUNT}][{phase}]"
    if level == "ERROR":
        print(f"  {prefix} ❌ {message}", flush=True)
    elif level == "WARN":
        print(f"  {prefix} ⚠️  {message}", flush=True)
    elif level == "INFO":
        print(f"  {prefix} {message}", flush=True)
    elif level == "OK":
        print(f"  {prefix} ✅ {message}", flush=True)

_defect_seq = 0
def record_defect(defect_type, source, description, severity=0.5, extra=None):
    """Record a defect to both JSONL log and KB kv_store."""
    global _defect_seq
    log("ERROR", "defect", f"{defect_type}: {description}", extra=extra)
    try:
        db = _get_db()
        now = int(time.time())
        _defect_seq += 1
        uuid_str = f"df-{now:x}-{_defect_seq:04x}"
        key = f"auto_absorb_defect_{CYCLE_COUNT}_{uuid_str}"
        val = json.dumps({
            "defect_type": defect_type,
            "category": defect_type,
            "source": source,
            "description": description,
            "severity": severity,
            "ts": now,
            "timestamp": now,
            "cycle": CYCLE_COUNT,
            "uptime_sec": int(time.time()) - STARTED_AT,
        })
        _safe_execute(db, "INSERT OR IGNORE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                      ("meta_cognition", key, val, now))
        db.commit()
    except Exception as e:
        log("WARN", "defect", f"Failed to persist defect to KB: {e}")

_db_conn = None
def _get_db():
    """Lazy singleton DB connection with WAL mode."""
    global _db_conn
    if _db_conn is None:
        _db_conn = sqlite3.connect(KB_PATH, timeout=120)
        _db_conn.execute("PRAGMA journal_mode=WAL")
        _db_conn.execute("PRAGMA busy_timeout=60000")
        _db_conn.execute("PRAGMA synchronous=NORMAL")
    return _db_conn

def _safe_execute(db, sql, params=None):
    """Execute SQL with retry on lock."""
    for i in range(5):
        try:
            if params:
                return db.execute(sql, params)
            return db.execute(sql)
        except sqlite3.OperationalError as e:
            if "locked" in str(e) and i < 4:
                time.sleep(1.0 * (i + 1))
                continue
            raise

def _safe_fetchone(db, sql, params=None):
    try:
        c = _safe_execute(db, sql, params)
        return c.fetchone()
    except Exception:
        return None

def _safe_fetchall(db, sql, params=None):
    try:
        c = _safe_execute(db, sql, params)
        return c.fetchall()
    except Exception:
        return []

def generate_uuid():
    return "nt-" + hashlib.md5((str(time.time()) + str(random.random())).encode()).hexdigest()[:20]

def _get_duplicate_blacklist(db):
    rows = _safe_fetchall(db, "SELECT key, value FROM kv_store WHERE namespace=? ORDER BY updated_at DESC LIMIT 500", (DUPLICATE_BLACKLIST_NS,))
    return {r[0] for r in rows}

def _add_to_duplicate_blacklist(db, url):
    _safe_execute(db, "INSERT OR IGNORE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                  (DUPLICATE_BLACKLIST_NS, url, "1", int(time.time())))
    db.commit()

# ── Phase Implementations ──

def phase_snapshot(db):
    """Record KB and system snapshot."""
    info = {}
    for row in _safe_fetchall(db, "SELECT node_type, COUNT(*) as cnt FROM nodes GROUP BY node_type ORDER BY cnt DESC"):
        info[f"nodes_{row[0]}"] = row[1]
    info["total_nodes"] = _safe_fetchone(db, "SELECT COUNT(*) FROM nodes")[0]
    info["total_edges"] = _safe_fetchone(db, "SELECT COUNT(*) FROM edges")[0]
    info["crawl_pending"] = _safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='pending'")[0]
    info["crawl_failed"] = _safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='failed'")[0]
    info["crawl_done"] = _safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='done'")[0]
    info["empty_content"] = _safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE content IS NULL OR content = ''")[0]
    info["embeddings"] = _safe_fetchone(db, "SELECT COUNT(*) FROM embeddings")[0]
    log("OK", "snapshot", f"KB: {info['total_nodes']} nodes, {info['total_edges']} edges, {info['empty_content']} empty, {info['crawl_pending']} pending crawl")
    try:
        cn = _safe_fetchone(db, "SELECT COUNT(*) FROM knowledge_nodes")[0]
        co = _safe_fetchone(db, "SELECT COUNT(*) FROM knowledge_edges")[0]
        info["legacy_nodes"] = cn
        info["legacy_edges"] = co
        log("INFO", "snapshot", f"Legacy tables: {cn} knowledge_nodes, {co} knowledge_edges")
    except Exception:
        pass
    now = int(time.time())
    val = json.dumps({"snapshot": info, "cycle": CYCLE_COUNT, "ts": now})
    _safe_execute(db, "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                  ("meta_cognition", f"auto_absorb_snapshot_{CYCLE_COUNT}", val, now))
    db.commit()
    return info


def phase_crawl_queue(db, pipeline, batch_size=50):
    """Process pending crawl_queue URLs."""
    urls = _safe_fetchall(db, "SELECT id, url, COALESCE(domain, 'web') FROM crawl_queue WHERE status='pending' ORDER BY CASE WHEN url LIKE '%github.com%' THEN 1 ELSE 0 END, id LIMIT ?", (batch_size,))
    if not urls:
        log("INFO", "crawlq", "No pending crawl queue items")
        return 0
    log("INFO", "crawlq", f"Processing {len(urls)} crawl queue items")
    ok, fail = 0, 0
    for qid, url, source in urls:
        if SHUTDOWN: break
        try:
            if "arxiv.org" in url:
                arxiv_id = extract_arxiv_id(url)
                if arxiv_id is None:
                    # Not a valid paper URL (archive page, status page, etc.)
                    _safe_execute(db, "UPDATE crawl_queue SET status='skipped', last_attempt=? WHERE id=?", (int(time.time()), qid))
                    log("INFO", "crawlq", f"Skipped non-paper arXiv URL: {url[:60]}")
                else:
                    meta = pipeline.fetch_arxiv(arxiv_id)
                    if meta:
                        _store_arxiv_as_node(db, meta, url)
                        _safe_execute(db, "UPDATE crawl_queue SET status='done', last_attempt=? WHERE id=?", (int(time.time()), qid))
                        ok += 1
                        log("OK", "crawlq", f"arXiv {arxiv_id}")
                    else:
                        _safe_execute(db, "UPDATE crawl_queue SET status='failed', last_attempt=? WHERE id=?", (int(time.time()), qid))
                        fail += 1
                        record_defect("API_FAIL", "arxiv", f"Failed to fetch {arxiv_id}", severity=0.4)
            elif "wikipedia.org" in url:
                title = extract_wiki_title(url)
                if title:
                    meta = pipeline.fetch_wikipedia(title)
                    if meta:
                        _store_wiki_as_node(db, meta, url)
                        _safe_execute(db, "UPDATE crawl_queue SET status='done', last_attempt=? WHERE id=?", (int(time.time()), qid))
                        ok += 1
                        log("OK", "crawlq", f"Wikipedia {title}")
                    else:
                        _safe_execute(db, "UPDATE crawl_queue SET status='failed', last_attempt=? WHERE id=?", (int(time.time()), qid))
                        fail += 1
                else:
                    _safe_execute(db, "UPDATE crawl_queue SET status='failed', last_attempt=? WHERE id=?", (int(time.time()), qid))
                    fail += 1
            elif "github.com" in url:
                parts = urllib.parse.urlparse(url).path.strip("/").split("/")
                if len(parts) >= 2:
                    owner, repo = parts[0], parts[1]
                    meta = pipeline.fetch_github(owner, repo)
                    if meta:
                        _store_github_as_node(db, meta, f"{owner}/{repo}")
                        _safe_execute(db, "UPDATE crawl_queue SET status='done', last_attempt=? WHERE id=?", (int(time.time()), qid))
                        ok += 1
                        log("OK", "crawlq", f"GitHub {owner}/{repo}")
                    else:
                        _safe_execute(db, "UPDATE crawl_queue SET status='failed', last_attempt=? WHERE id=?", (int(time.time()), qid))
                        fail += 1
                        record_defect("API_FAIL", "github", f"Failed to fetch {owner}/{repo}", severity=0.3)
                else:
                    _safe_execute(db, "UPDATE crawl_queue SET status='failed', last_attempt=? WHERE id=?", (int(time.time()), qid))
                    fail += 1
            else:
                _safe_execute(db, "UPDATE crawl_queue SET status='failed', last_attempt=? WHERE id=?", (int(time.time()), qid))
                fail += 1
            db.commit()
            time.sleep(1.5)
        except Exception as e:
            log("ERROR", "crawlq", f"Error processing {url}: {e}")
            record_defect("PROCESSING_ERROR", "crawlq", f"{url}: {e}", severity=0.6)
            _safe_execute(db, "UPDATE crawl_queue SET status='failed', last_attempt=? WHERE id=?", (int(time.time()), qid))
            db.commit()
            fail += 1
    log("OK", "crawlq", f"Done: {ok} ok, {fail} failed")
    return ok


def phase_arxiv_fill(db, pipeline, limit=30):
    """Fill Paper/article nodes with empty content from arxiv URLs."""
    nodes = _safe_fetchall(db,
        """SELECT id, node_type, title, url FROM nodes
           WHERE (content IS NULL OR content = '')
             AND url LIKE '%arxiv.org%'
             AND url NOT LIKE '%/archive/%'
             AND url NOT LIKE '%status.arxiv%'
           ORDER BY created_at LIMIT ?""",
        (limit,))
    if not nodes:
        log("INFO", "arxiv", "No empty arXiv nodes to fill")
        return 0
    log("INFO", "arxiv", f"Filling {len(nodes)} empty arXiv nodes")
    ok, fail, skipped = 0, 0, 0
    for node_id, node_type, title, url in nodes:
        if SHUTDOWN: break
        arxiv_id = extract_arxiv_id(url)
        if not arxiv_id:
            skipped += 1
            continue
        try:
            meta = pipeline.fetch_arxiv(arxiv_id)
            if meta:
                _update_node_from_arxiv(db, node_id, meta)
                ok += 1
                log("OK", "arxiv", f"Filled {arxiv_id}")
            else:
                fail += 1
                record_defect("API_FAIL", "arxiv", f"Failed to fill {arxiv_id}", severity=0.4)
            db.commit()
            time.sleep(3.5)
        except Exception as e:
            log("ERROR", "arxiv", f"Error filling {arxiv_id}: {e}")
            record_defect("FILL_ERROR", "arxiv", f"{arxiv_id}: {e}", severity=0.5)
            fail += 1
    log("OK", "arxiv", f"Filled: {ok} ok, {fail} failed, {skipped} skipped")
    return ok


WIKI_TOPICS = [
    # ── Artificial Intelligence & ML (70) ──
    "Artificial intelligence", "Machine learning", "Deep learning", "Neural network",
    "Transformer (deep learning architecture)", "Large language model", "Reinforcement learning",
    "Computer vision", "Natural language processing", "Robotics",
    "Supervised learning", "Unsupervised learning", "Semi-supervised learning",
    "Self-supervised learning", "Transfer learning", "Multi-task learning",
    "Federated learning", "Active learning", "Meta-learning", "Few-shot learning",
    "Zero-shot learning", "Continual learning", "Lifelong learning",
    "Generative adversarial network", "Variational autoencoder", "Diffusion model",
    "Autoregressive model", "Normalizing flow", "Energy-based model",
    "Convolutional neural network", "Recurrent neural network", "Long short-term memory",
    "Graph neural network", "Attention (machine learning)", "Mixture of experts",
    "Random forest", "Decision tree", "Support vector machine", "k-nearest neighbors",
    "Bayesian network", "Hidden Markov model", "Conditional random field",
    "Principal component analysis", "t-distributed stochastic neighbor embedding",
    "K-means clustering", "DBSCAN", "Hierarchical clustering",
    "Gradient descent", "Stochastic gradient descent", "Adam (optimizer)",
    "Backpropagation", "Batch normalization", "Layer normalization",
    "Dropout (neural networks)", "Activation function", "Loss function",
    "Overfitting", "Regularization (mathematics)", "Cross-validation (statistics)",
    "Hyperparameter optimization", "Neural architecture search",
    "Knowledge distillation", "Model compression", "Quantization (signal processing)",
    "Edge computing", "TinyML", "Federated learning",
    "Reinforcement learning from human feedback", "Constitutional AI",
    "Automated machine learning", "Feature engineering", "Feature selection",
    # ── AI Safety & Ethics (15) ──
    "AI alignment", "AI safety", "Interpretability (artificial intelligence)",
    "Explainable artificial intelligence", "Machine unlearning",
    "Algorithmic fairness", "Bias (artificial intelligence)", "AI governance",
    "Responsible AI", "Value learning", "Inverse reinforcement learning",
    "Cooperative AI", "Multi-agent system", "Swarm intelligence",
    "Emergent behavior",
    # ── Cognitive Science & Neuroscience (35) ──
    "Cognitive science", "Neuroscience", "Cognitive psychology",
    "Cognitive architecture", "Working memory", "Long-term memory",
    "Attention", "Consciousness", "Phenomenology",
    "Global workspace theory", "Integrated information theory",
    "Predictive coding", "Free energy principle", "Active inference",
    "Bayesian brain", "Neural oscillation", "Synaptic plasticity",
    "Hebbian theory", "Spiking neural network", "Neuromorphic computing",
    "Neuroplasticity", "Mirror neuron", "Theory of mind",
    "Embodied cognition", "Situated cognition", "Distributed cognition",
    "Extended mind thesis", "Dual-process theory", "System 1 and System 2",
    "Cognitive load", "Metacognition", "Executive functions",
    "Decision-making", "Problem solving", "Creativity",
    # ── Mathematics: Algebra & Number Theory (40) ──
    "Linear algebra", "Abstract algebra", "Group theory", "Ring theory",
    "Field theory (mathematics)", "Galois theory", "Representation theory",
    "Lie group", "Lie algebra", "Category theory", "Topos theory",
    "Homological algebra", "Commutative algebra", "Number theory",
    "Algebraic number theory", "Analytic number theory", "Arithmetic geometry",
    "Elliptic curve", "Modular form", "L-function",
    "Cryptography", "Post-quantum cryptography", "Lattice-based cryptography",
    "Homomorphic encryption", "Secure multi-party computation",
    "Zero-knowledge proof", "Blockchain", "Distributed ledger",
    # ── Mathematics: Analysis (35) ──
    "Calculus", "Real analysis", "Complex analysis", "Functional analysis",
    "Harmonic analysis", "Fourier analysis", "Wavelet",
    "Measure theory", "Integration theory", "Lebesgue integration",
    "Differential equation", "Partial differential equation",
    "Ordinary differential equation", "Stochastic differential equation",
    "Numerical analysis", "Finite element method", "Monte Carlo method",
    "Optimization", "Convex optimization", "Linear programming",
    "Nonlinear programming", "Dynamic programming", "Integer programming",
    "Calculus of variations", "Optimal control", "Stochastic process",
    "Brownian motion", "Markov chain", "Martingale (probability theory)",
    "Information theory", "Entropy (information theory)", "Kullback-Leibler divergence",
    "Mutual information", "Fisher information", "Rényi entropy",
    # ── Mathematics: Geometry & Topology (30) ──
    "Geometry", "Differential geometry", "Riemannian geometry",
    "Symplectic geometry", "Algebraic geometry", "Complex geometry",
    "Topology", "Algebraic topology", "Differential topology",
    "Geometric topology", "Knot theory", "Homotopy theory",
    "Cohomology", "Sheaf theory", "Manifold",
    "Vector bundle", "Characteristic class", "Index theorem",
    "Graph theory", "Combinatorics", "Enumerative combinatorics",
    "Matroid theory", "Set theory", "Model theory",
    "Proof theory", "Type theory", "Lambda calculus",
    "Computability theory", "Computational complexity theory", "Automata theory",
    # ── Mathematics: Probability & Statistics (25) ──
    "Probability theory", "Statistics", "Bayesian inference",
    "Frequentist inference", "Hypothesis testing", "Confidence interval",
    "Regression analysis", "Time series analysis", "Survival analysis",
    "Nonparametric statistics", "Multivariate statistics", "Spatial statistics",
    "Statistical learning theory", "Probably approximately correct learning",
    "VC dimension", "Rademacher complexity", "Concentration inequality",
    "Empirical risk minimization", "Structural risk minimization",
    "Minimum description length", "Akaike information criterion",
    "Bayesian information criterion", "Cross-validation", "Bootstrap (statistics)",
    "Markov chain Monte Carlo",
    # ── Physics (70) ──
    "Physics", "Classical mechanics", "Quantum mechanics",
    "Quantum field theory", "Quantum electrodynamics", "Quantum chromodynamics",
    "String theory", "M-theory", "Loop quantum gravity",
    "General relativity", "Special relativity", "Cosmology",
    "Astrophysics", "Particle physics", "Standard Model",
    "Supersymmetry", "Grand Unified Theory", "Theory of everything",
    "Statistical mechanics", "Thermodynamics", "Condensed matter physics",
    "Solid-state physics", "Semiconductor", "Superconductivity",
    "Topological insulator", "Quantum Hall effect", "Spintronics",
    "Atomic physics", "Molecular physics", "Optics",
    "Photonics", "Laser", "Nonlinear optics",
    "Plasma physics", "Nuclear physics", "Nuclear fusion",
    "Nuclear fission", "Particle accelerator", "Dark matter",
    "Dark energy", "Inflation (cosmology)", "Black hole",
    "Neutron star", "Gravitational wave", "Big Bang",
    "Quantum entanglement", "Quantum decoherence", "Bell's theorem",
    "Quantum information", "Quantum computing", "Quantum algorithm",
    "Quantum error correction", "Topological quantum computing",
    "AdS/CFT correspondence", "Holographic principle",
    "Chaos theory", "Complex system", "Dynamical system",
    "Fluid dynamics", "Aerodynamics", "Electromagnetism",
    "Classical electromagnetism", "Maxwell's equations", "Photon",
    "Gauge theory", "Yang-Mills theory", "Renormalization",
    "Lattice gauge theory", "Perturbation theory", "Scattering theory",
    # ── Computer Science: Systems (35) ──
    "Computer science", "Operating system", "Compiler",
    "Computer architecture", "Parallel computing", "Distributed computing",
    "Cloud computing", "Grid computing", "Cluster computing",
    "Computer network", "Internet", "World Wide Web",
    "Network protocol", "TCP/IP", "HTTP",
    "Database", "Relational database", "NoSQL",
    "SQL", "Query optimization", "Transaction processing",
    "Data warehouse", "Data lake", "Big data",
    "Software engineering", "Programming language theory",
    "Formal verification", "Model checking", "Theorem proving",
    "Compiler optimization", "Program analysis", "Static program analysis",
    "Type system", "Garbage collection (computer science)", "Memory management",
    # ── Computer Science: AI & ML Applied (25) ──
    "Data mining", "Web mining", "Text mining",
    "Information retrieval", "Search engine", "Web search engine",
    "Recommender system", "Collaborative filtering", "Content-based filtering",
    "Anomaly detection", "Outlier detection", "Fraud detection",
    "Speech recognition", "Speaker recognition", "Music information retrieval",
    "Image segmentation", "Object detection", "Facial recognition system",
    "Autonomous vehicle", "Self-driving car", "Simultaneous localization and mapping",
    "Computer graphics", "Rendering (computer graphics)", "Ray tracing (graphics)",
    "Scientific visualization",
    # ── Engineering & Applied (25) ──
    "Control theory", "Control engineering", "PID controller",
    "Robotics", "Robot kinematics", "Robot control",
    "Cybernetics", "Systems theory", "Operations research",
    "Queueing theory", "Reliability engineering", "Fault tolerance",
    "Signal processing", "Digital signal processing", "Image processing",
    "Computer vision", "Pattern recognition", "Biometrics",
    "Human–computer interaction", "User interface", "User experience",
    "Information visualization", "Data visualization", "Virtual reality",
    "Augmented reality",
    # ── Biology & Life Sciences (25) ──
    "Biology", "Molecular biology", "Cell biology",
    "Genetics", "Genomics", "Bioinformatics",
    "Evolution", "Evolutionary biology", "Population genetics",
    "Ecology", "Ecosystem", "Neuroscience",
    "Computational neuroscience", "Systems biology", "Synthetic biology",
    "Biochemistry", "Biophysics", "Structural biology",
    "Immunology", "Microbiology", "Virology",
    "Epigenetics", "CRISPR gene editing", "Protein folding",
    "Drug discovery",
    # ── Philosophy & Linguistics (20) ──
    "Philosophy", "Philosophy of mind", "Philosophy of science",
    "Philosophy of mathematics", "Logic", "Modal logic",
    "Epistemology", "Ontology", "Metaphysics",
    "Ethics", "Philosophy of artificial intelligence",
    "Linguistics", "Computational linguistics", "Semantics",
    "Pragmatics", "Syntax", "Morphology (linguistics)",
    "Phonetics", "Phonology", "Natural language understanding",
    # ── Advanced & Frontier Topics (30) ──
    "E8 (mathematics)", "Exceptional Lie algebra", "Hypercomplex number",
    "Clifford algebra", "Geometric algebra", "Spinor",
    "Twistor theory", "Noncommutative geometry", "Quantum group",
    "Von Neumann algebra", "Operator algebra", "C-star-algebra",
    "K-theory", "Motivic cohomology", "Higher category theory",
    "Infinity-category", "Homotopy type theory", "Univalent foundations",
    "Vector symbolic architecture", "Hyperdimensional computing",
    "Binary spatter code", "Holographic reduced representation",
    "Frequency hypervector", "Multiply-add permute", "Sparse distributed memory",
    "Liquid state machine", "Echo state network", "Reservoir computing",
    "Neuromorphic engineering", "Memristor", "Physical neural network",
]

def _fetch_random_wikipedia(pipeline):
    """Fetch a random Wikipedia article via API list=random."""
    try:
        import urllib.request, urllib.error
        api_url = "https://en.wikipedia.org/w/api.php?action=query&list=random&rnnamespace=0&rnlimit=1&format=json"
        req = urllib.request.Request(api_url, headers={'User-Agent': 'NeoTrix/1.0'})
        with urllib.request.urlopen(req, timeout=15) as resp:
            data = json.loads(resp.read().decode())
        pages = data.get('query', {}).get('random', [])
        if pages:
            title = pages[0].get('title', '')
            if title:
                return pipeline.fetch_wikipedia(title)
        return None
    except Exception as e:
        log("WARN", "wiki", f"Random Wikipedia fetch failed: {e}")
        return None

def phase_wikipedia_discovery(db, pipeline, count=5):
    """Discover new Wikipedia articles via curated topics then random fallback."""
    log("INFO", "wiki", f"Discovering up to {count} Wikipedia topics")
    existing = set()
    for row in _safe_fetchall(db, "SELECT title FROM nodes WHERE domain='en.wikipedia.org' AND node_type='Concept'"):
        existing.add(row[0].lower())
    candidates = [t for t in WIKI_TOPICS if t.lower() not in existing]
    ok = 0
    if candidates:
        selected = random.sample(candidates, min(count, len(candidates)))
        for topic in selected:
            if SHUTDOWN: break
            try:
                meta = pipeline.fetch_wikipedia(topic)
                if meta:
                    _store_wiki_as_node(db, meta, None)
                    ok += 1
                    log("OK", "wiki", f"Added '{topic}'")
                else:
                    record_defect("API_FAIL", "wikipedia", f"Failed to fetch '{topic}'", severity=0.3)
                time.sleep(1.0)
            except Exception as e:
                log("ERROR", "wiki", f"Error fetching '{topic}': {e}")
                record_defect("FETCH_ERROR", "wikipedia", f"{topic}: {e}", severity=0.4)
        db.commit()
    # Fallback: random Wikipedia articles when curated topics exhausted
    remaining = count - ok
    if remaining > 0:
        log("INFO", "wiki", f"Curated topics exhausted, fetching {remaining} random Wikipedia articles")
        for _ in range(remaining):
            if SHUTDOWN: break
            try:
                meta = _fetch_random_wikipedia(pipeline)
                if meta:
                    title = meta.get("title", "")
                    if title.lower() not in existing:
                        _store_wiki_as_node(db, meta, None)
                        ok += 1
                        existing.add(title.lower())
                        log("OK", "wiki", f"Random: '{title}'")
                    time.sleep(1.5)
                else:
                    time.sleep(3.0)
            except Exception as e:
                log("ERROR", "wiki", f"Random Wikipedia error: {e}")
                time.sleep(3.0)
        db.commit()
    log("OK", "wiki", f"Added {ok} new Wikipedia articles")
    return ok


def _get_github_skip_set(db):
    rows = _safe_fetchall(db, "SELECT key FROM kv_store WHERE namespace='github_skip'")
    return {r[0] for r in rows}

def _add_to_github_skip(db, owner, repo):
    key = f"{owner}/{repo}"
    _safe_execute(db, "INSERT OR IGNORE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                  ("github_skip", key, str(int(time.time())), int(time.time())))
    db.commit()

def phase_wikipedia_concept_fill(db, pipeline, limit=8):
    """Fill empty Concept/Article nodes with content from Wikipedia."""
    nodes = _safe_fetchall(db,
        """SELECT id, COALESCE(NULLIF(title,''), url) AS fill_target FROM nodes
           WHERE (content IS NULL OR content = '')
             AND (COALESCE(NULLIF(title,''), url) != '' AND COALESCE(NULLIF(title,''), url) IS NOT NULL)
             AND node_type IN ('Concept', 'Insight', 'Article')
             AND (domain = '' OR domain IS NULL OR domain = 'en.wikipedia.org')
           ORDER BY RANDOM() LIMIT ?""",
        (limit,))
    if not nodes:
        log("INFO", "wikifill", "No empty Concept/Insight/Article nodes to fill")
        return 0
    log("INFO", "wikifill", f"Filling up to {len(nodes)} empty Concept/Insight/Article nodes via Wikipedia")
    ok = 0
    for node_id, fill_target in nodes:
        if SHUTDOWN: break
        try:
            meta = pipeline.fetch_wikipedia(fill_target)
            if meta and meta.get("extract"):
                extract = normalize_text(meta.get("extract", ""))
                existing = _safe_fetchone(db, "SELECT content FROM nodes WHERE id=?", (node_id,))
                if existing and (existing[0] is None or existing[0] == ""):
                    full_content = extract
                    if meta.get("content"):
                        full_content = normalize_text(meta["content"])
                    _safe_execute(db, "UPDATE nodes SET content=?, summary=COALESCE(NULLIF(summary,''), ?), domain='en.wikipedia.org', updated_at=? WHERE id=?",
                                  (full_content, extract[:200], int(time.time()), node_id))
                    ok += 1
                    log("OK", "wikifill", f"Filled '{fill_target[:50]}'")
            else:
                log("INFO", "wikifill", f"No Wikipedia content for '{fill_target[:50]}'")
            db.commit()
            time.sleep(1.2)
        except Exception as e:
            log("ERROR", "wikifill", f"Error filling '{fill_target[:30]}': {e}")
            db.commit()
    log("OK", "wikifill", f"Filled {ok}/{len(nodes)} empty nodes with Wikipedia content")
    return ok


def phase_github_trending(db, pipeline, count=5):
    """Discover new GitHub repos via trending scrape + curated fallback."""
    log("INFO", "github", f"Discovering up to {count} GitHub repos")
    existing_urls = set()
    for row in _safe_fetchall(db, "SELECT url FROM nodes WHERE domain LIKE '%github.com%'"):
        if row[0]:
            existing_urls.add(row[0].rstrip("/"))
    skip_set = _get_github_skip_set(db)
    ok = 0
    # Phase 1: Try GitHub trending page via simple scrape
    if ok < count:
        try:
            import urllib.request, urllib.error
            trending_url = "https://github.com/trending?since=weekly"
            req = urllib.request.Request(trending_url, headers={'User-Agent': 'NeoTrix/1.0'})
            with urllib.request.urlopen(req, timeout=15) as resp:
                html = resp.read().decode('utf-8', errors='replace')
            repos = re.findall(r'href="/([^/"]+/[^/"]+)"', html)
            seen = set()
            for full_name in repos:
                if SHUTDOWN or ok >= count:
                    break
                if full_name in seen or '/' not in full_name:
                    continue
                seen.add(full_name)
                url = f"https://github.com/{full_name}"
                if url in existing_urls or full_name in skip_set:
                    continue
                parts = full_name.split("/")
                if len(parts) >= 2:
                    meta = pipeline.fetch_github(parts[0], parts[1])
                    if meta:
                        _store_github_as_node(db, meta, full_name)
                        existing_urls.add(url)
                        ok += 1
                        log("OK", "github", f"Added {full_name} ({meta.get('stars',0)}★ trending)")
                    else:
                        _add_to_github_skip(db, parts[0], parts[1])
                    time.sleep(2.0)
        except Exception as e:
            log("WARN", "github", f"Trending page scrape failed: {e}")
    # Phase 2: Curated fallback
    if ok < count:
        curated = [
            ("neotrix", "neotrix"), ("opencode", "opencode"),
            ("rust-lang", "rust"), ("python", "cpython"),
            ("torvalds", "linux"), ("microsoft", "vscode"),
            ("facebook", "react"), ("tensorflow", "tensorflow"),
            ("pytorch", "pytorch"), ("openai", "openai-cookbook"),
            ("langchain-ai", "langchain"), ("ggerganov", "llama.cpp"),
            ("deepseek-ai", "DeepSeek-V3"), ("unslothai", "unsloth"),
            ("nvidia", "cuda"), ("astral-sh", "ruff"),
            ("denoland", "deno"), ("tauri-apps", "tauri"),
            ("zed-industries", "zed"), ("surrealdb", "surrealdb"),
        ]
        candidates = [(o,r) for (o,r) in curated if f"https://github.com/{o}/{r}" not in existing_urls and f"{o}/{r}" not in skip_set]
        for owner, repo in candidates[:count - ok]:
            if SHUTDOWN: break
            try:
                meta = pipeline.fetch_github(owner, repo)
                if meta:
                    _store_github_as_node(db, meta, f"{owner}/{repo}")
                    ok += 1
                    log("OK", "github", f"Added {owner}/{repo} ({meta.get('stars',0)}★ curated)")
                else:
                    _add_to_github_skip(db, owner, repo)
                time.sleep(2.0)
            except Exception as e:
                log("ERROR", "github", f"Error fetching {owner}/{repo}: {e}")
                _add_to_github_skip(db, owner, repo)
    db.commit()
    log("OK", "github", f"Added {ok} new GitHub repos ({len(existing_urls)} total known)")
    return ok


def phase_kb_analysis(db):
    """Run internal KB content analysis and record patterns."""
    log("INFO", "analysis", "Starting KB content analysis")

    # 1. Empty content analysis
    empty_by_type = _safe_fetchall(db,
        "SELECT node_type, COUNT(*) as cnt FROM nodes WHERE content IS NULL OR content = '' GROUP BY node_type ORDER BY cnt DESC LIMIT 10")
    log("INFO", "analysis", "Empty content by type:")
    for t, c in empty_by_type:
        log("INFO", "analysis", f"  {t}: {c} empty")

    # 2. Domain coverage
    domains = _safe_fetchall(db,
        "SELECT domain, COUNT(*) as cnt FROM nodes WHERE domain != '' GROUP BY domain ORDER BY cnt DESC LIMIT 10")
    log("INFO", "analysis", "Top domains:")
    for d, c in domains:
        log("INFO", "analysis", f"  {d}: {c} nodes")

    # 3. Orphaned nodes (no edges)
    orphaned = _safe_fetchone(db,
        "SELECT COUNT(*) FROM nodes n WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.source_id = n.id OR e.target_id = n.id)")[0]
    log("INFO", "analysis", f"Orphaned nodes: {orphaned}")

    # 4. Duplicate detection (same URL) — blacklist known dups after first report
    dups = _safe_fetchall(db,
        "SELECT url, COUNT(*) as cnt FROM nodes WHERE url != '' GROUP BY url HAVING cnt > 1 ORDER BY cnt DESC LIMIT 10")
    blacklist = _get_duplicate_blacklist(db)
    new_dups = 0
    if dups:
        for d, c in dups:
            if d in blacklist:
                log("INFO", "analysis", f"Skipped blacklisted duplicate: {d[:80]}")
                continue
            log("WARN", "analysis", f"Duplicate URL: {d[:80]} ({c}x)")
            record_defect("DUPLICATE_NODE", "kb", f"URL {d[:80]} has {c} copies", severity=0.3)
            _add_to_duplicate_blacklist(db, d)
            new_dups += 1
    if new_dups == 0 and dups:
        log("INFO", "analysis", "All known duplicates suppressed (0 new)")

    # 5. Record analysis to kv_store
    now = int(time.time())
    report = {
        "empty_by_type": dict(empty_by_type),
        "top_domains": dict(domains),
        "orphaned": orphaned,
        "duplicates": len(dups),
        "cycle": CYCLE_COUNT,
        "ts": now,
    }
    _safe_execute(db, "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                  ("meta_cognition", f"auto_absorb_analysis_{CYCLE_COUNT}", json.dumps(report), now))
    db.commit()
    return report


def phase_metacognition_summary(db):
    """Summarize defects found this cycle and generate meta-cognition record."""
    log("INFO", "metacog", "Generating meta-cognition summary")

    # Collect defects from this session only (key format: auto_absorb_defect_{CYCLE_COUNT}_...)
    defect_prefix = f"auto_absorb_defect_{CYCLE_COUNT}_"
    defects = _safe_fetchall(db,
        "SELECT key, value FROM kv_store WHERE namespace='meta_cognition' AND key LIKE ? ORDER BY updated_at DESC",
        (f"{defect_prefix}%",))
    by_type = {}
    for key, val in defects:
        try:
            d = json.loads(val)
            dt = d.get("defect_type", "UNKNOWN")
            by_type.setdefault(dt, []).append(d)
        except Exception:
            pass

        
    # Generate evolution pattern candidates
    patterns = []
    for dtype, instances in by_type.items():
        if len(instances) >= 3:
            patterns.append({
                "pattern_type": "RecurringError",
                "defect_type": dtype,
                "count": len(instances),
                "severity_avg": sum(i.get("severity", 0.5) for i in instances) / len(instances),
                "description": f"{dtype} occurred {len(instances)} times this cycle",
            })
            log("WARN", "metacog", f"Recurring pattern: {dtype} ({len(instances)}x)")

    if by_type:
        log("INFO", "metacog", f"Defect summary: {len(by_type)} types, {sum(len(v) for v in by_type.values())} total")

    # Store summary
    now = int(time.time())
    summary = {
        "defects_by_type": {k: len(v) for k, v in by_type.items()},
        "total_defects": sum(len(v) for v in by_type.values()),
        "patterns_detected": len(patterns),
        "patterns": patterns,
        "cycle": CYCLE_COUNT,
        "ts": now,
    }
    _safe_execute(db, "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                  ("meta_cognition", f"auto_absorb_metacog_{CYCLE_COUNT}", json.dumps(summary), now))
    db.commit()
    return summary


# ── KB Edge Helper ──

def _ensure_edge(db, src_id, tgt_id, rel_type, weight=0.5):
    """Create an edge between two nodes if it doesn't already exist."""
    if not src_id or not tgt_id or src_id == tgt_id:
        return False
    eid = f"auto_{src_id[:16]}_{tgt_id[:16]}_{rel_type[:8]}"
    now = int(time.time())
    try:
        _safe_execute(db,
            "INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            (eid, src_id, tgt_id, rel_type, weight, now))
        return True
    except Exception:
        return False


# ── KB Storage Helpers ──

def _node_exists(db, url):
    row = _safe_fetchone(db, "SELECT id FROM nodes WHERE url=? LIMIT 1", (url,))
    return row[0] if row else None

def _store_arxiv_as_node(db, meta, url):
    arxiv_id = meta.get("arxiv_id", "")
    existing = _node_exists(db, url)
    if existing:
        _update_node_from_arxiv(db, existing, meta)
        return existing
    node_id = "nt-" + hashlib.md5(url.encode()).hexdigest()[:20]
    title = normalize_text(meta.get("title", ""))
    abstract = normalize_text(meta.get("abstract", ""))
    authors = meta.get("authors", [])
    categories = meta.get("categories", [])
    summary = f"[arXiv {arxiv_id}] {', '.join(categories[:3])}" if categories else f"[arXiv {arxiv_id}]"
    if authors:
        summary = f"{authors[0]}{' et al.' if len(authors) > 1 else ''} — {summary}"
    content = abstract
    if authors:
        content += f"\n\nAuthors: {', '.join(authors)}"
    if categories:
        content += f"\n\nCategories: {', '.join(categories)}"
    if meta.get("published"):
        content += f"\n\nPublished: {meta['published']}"
    content = normalize_text(content)
    metadata = json.dumps({
        "arxiv_id": arxiv_id, "authors": authors, "categories": categories,
        "published": meta.get("published", ""), "doi": meta.get("doi", ""),
    })
    now = int(time.time())
    try:
        _safe_execute(db,
            "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (node_id, "Paper", title, summary, content, url, "arxiv.org", "en", 1.0, 0.7, now, now, metadata))
        # Edge: Paper → related Concept based on first category
        if categories and len(categories) > 0:
            cat_concept = _safe_fetchone(db, "SELECT id FROM nodes WHERE node_type='Concept' AND (title=? OR title LIKE ?) LIMIT 1",
                (categories[0].replace('.', ' ').title(), categories[0].split('.')[0] + '%'))
            if cat_concept:
                _ensure_edge(db, node_id, cat_concept[0], "about_topic", 0.7)
        return node_id
    except Exception as e:
        record_defect("DB_ERROR", "arxiv_store", f"Store error: {e}", severity=0.5)
        return None

def _update_node_from_arxiv(db, node_id, meta):
    title = normalize_text(meta.get("title", ""))
    abstract = normalize_text(meta.get("abstract", ""))
    authors = meta.get("authors", [])
    categories = meta.get("categories", [])
    arxiv_id = meta.get("arxiv_id", "")
    summary = f"[arXiv {arxiv_id}] {', '.join(categories[:3])}" if categories else f"[arXiv {arxiv_id}]"
    if authors:
        summary = f"{authors[0]}{' et al.' if len(authors) > 1 else ''} — {summary}"
    content = abstract
    if authors:
        content += f"\n\nAuthors: {', '.join(authors)}"
    if categories:
        content += f"\n\nCategories: {', '.join(categories)}"
    content = normalize_text(content)
    metadata = json.dumps({
        "arxiv_id": arxiv_id, "authors": authors, "categories": categories,
        "published": meta.get("published", ""),
    })
    now = int(time.time())
    try:
        _safe_execute(db,
            "UPDATE nodes SET title=COALESCE(NULLIF(?, ''), title), summary=COALESCE(NULLIF(?, ''), summary), content=COALESCE(NULLIF(?, ''), content), metadata=?, updated_at=? WHERE id=?",
            (title, summary, content, metadata, now, node_id))
    except Exception as e:
        record_defect("DB_ERROR", "arxiv_update", f"Update error: {e}", severity=0.5)

def _store_wiki_as_node(db, meta, url):
    title = meta.get("title", "")
    url = url or f"https://en.wikipedia.org/wiki/{title.replace(' ', '_')}"
    existing = _node_exists(db, url)
    if existing:
        return existing
    node_id = "nt-" + hashlib.md5(url.encode()).hexdigest()[:20]
    extract = normalize_text(meta.get("extract", ""))
    summary = normalize_text(meta.get("description", "")) or title
    if not summary:
        summary = extract[:200] if extract else title
    content = extract
    if meta.get("content"):
        content = normalize_text(meta['content'])
    if meta.get("categories"):
        categories = ", ".join(meta["categories"])
        content += f"\n\nCategories: {categories}"
    metadata = json.dumps({
        "source": "wikipedia", "page_id": meta.get("page_id", ""),
        "categories": meta.get("categories", []),
    })
    now = int(time.time())
    try:
        _safe_execute(db,
            "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (node_id, "Concept", title, summary, content, url, "en.wikipedia.org", "en", 1.0, 0.6, now, now, metadata))
        # Edge: Concept → broader category via category graph
        cats = meta.get("categories", [])
        for cat in cats[:3]:
            parent = _safe_fetchone(db, "SELECT id FROM nodes WHERE node_type='Concept' AND (title=? OR title LIKE ?) LIMIT 1",
                (cat.rstrip(), cat.rstrip()[:60] + '%'))
            if parent:
                _ensure_edge(db, node_id, parent[0], "sub_topic_of", 0.5)
                _ensure_edge(db, node_id, parent[0], "about_topic", 0.6)
        return node_id
    except Exception as e:
        record_defect("DB_ERROR", "wiki_store", f"Store error: {e}", severity=0.5)
        return None

def _store_github_as_node(db, meta, full_name):
    url = f"https://github.com/{full_name}"
    existing = _node_exists(db, url)
    if existing:
        return existing
    node_id = "nt-" + hashlib.md5(url.encode()).hexdigest()[:20]
    title = full_name.split("/")[1]
    stars = meta.get("stars", 0)
    language = meta.get("language", "")
    topics = meta.get("topics", [])
    description = normalize_text(meta.get("description", ""))
    summary = f"{title} — {description[:150]}" if description else title
    if stars:
        summary += f" ({stars}★)"
    content = description
    if meta.get("readme"):
        content += "\n\n" + strip_markdown(normalize_text(meta["readme"]))
    if topics:
        content += f"\n\nTopics: {', '.join(topics[:10])}"
    if language:
        content += f"\n\nLanguage: {language}"
    metadata = json.dumps({
        "stars": stars, "language": language, "topics": topics,
        "owner": full_name.split("/")[0],
    })
    now = int(time.time())
    lang_norm = normalize_lang(language) if language else "en"
    try:
        _safe_execute(db,
            "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (node_id, "Repository", title, summary, content, url, "github.com", lang_norm, 1.0, min(1.0, stars / 100000), now, now, metadata))
        # Edge: Repository → owner Organization/Person
        owner = full_name.split("/")[0]
        owner_node = _safe_fetchone(db, "SELECT id FROM nodes WHERE node_type IN ('Organization','Person') AND (title=? OR url LIKE ?) LIMIT 1",
            (owner, f"%github.com/{owner}%"))
        if owner_node:
            _ensure_edge(db, node_id, owner_node[0], "developed_by", 0.8)
        # Edge: Repository → related Concept via topics
        for topic in (topics or [])[:5]:
            topic_node = _safe_fetchone(db, "SELECT id FROM nodes WHERE node_type='Concept' AND title=? LIMIT 1",
                (topic.title(),))
            if topic_node:
                _ensure_edge(db, node_id, topic_node[0], "about_topic", 0.6)
        return node_id
    except Exception as e:
        record_defect("DB_ERROR", "github_store", f"Store error: {e}", severity=0.5)
        return None


# ── URL Helpers ──

def extract_arxiv_id(url):
    if not url:
        return None
    # Skip non-paper URLs
    if '/archive/' in url or 'status.arxiv.org' in url:
        return None
    # Format 1: 1706.03762 (new, 4+4 or 4+5 digits)
    m = re.search(r'arxiv\.org/(?:abs|pdf)/(\d{4}\.\d{4,5})(?:v\d+)?', url)
    if m:
        return m.group(1)
    # Format 2: math.HO/0405323 (old, category/7digits)
    m = re.search(r'arxiv\.org/(?:abs|pdf)/([a-z\-]+\.[A-Za-z]+/\d{7})(?:v\d+)?', url)
    if m:
        return m.group(1)
    # Format 3: hep-th/9901001 (old, category/7digits)
    m = re.search(r'arxiv\.org/(?:abs|pdf)/([a-z\-]+/\d{7})(?:v\d+)?', url)
    return m.group(1) if m else None

def extract_wiki_title(url):
    if not url:
        return None
    m = re.search(r'wikipedia\.org/wiki/(.+)', url)
    if m:
        return urllib.parse.unquote(m.group(1)).replace("_", " ")
    return None

def extract_github_owner_repo(url):
    parts = urllib.parse.urlparse(url).path.strip("/").split("/")
    if len(parts) >= 2:
        return parts[0], parts[1]
    return None, None


# ── Main Loop ──

def run_cycle(db, pipeline):
    global CYCLE_COUNT
    CYCLE_COUNT += 1

    log("INFO", "start", f"═══ Starting Cycle {CYCLE_COUNT} ═══")

    # Phase 1: Snapshot
    snapshot = phase_snapshot(db)

    # Phase 2: Crawl Queue
    try:
        phase_crawl_queue(db, pipeline, batch_size=30)
    except Exception as e:
        log("ERROR", "crawlq", f"Phase failed: {e}")
        record_defect("PHASE_FAIL", "crawlq", str(e), severity=0.7)

    # Phase 2b: Seed crawl queue if low
    try:
        pending = _safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='pending'")[0]
        if pending < 10:
            log("INFO", "seed", f"Crawl queue low ({pending} pending), seeding more URLs")
            _seed_crawl_queue_startup(db)
    except Exception as e:
        log("WARN", "seed", f"Seed failed: {e}")

    # Phase 3: ArXiv Content Fill
    try:
        phase_arxiv_fill(db, pipeline, limit=20)
    except Exception as e:
        log("ERROR", "arxiv", f"Phase failed: {e}")
        record_defect("PHASE_FAIL", "arxiv", str(e), severity=0.7)

    # Phase 4: Wikipedia Discovery
    try:
        phase_wikipedia_discovery(db, pipeline, count=5)
    except Exception as e:
        log("ERROR", "wiki", f"Phase failed: {e}")
        record_defect("PHASE_FAIL", "wiki", str(e), severity=0.6)

    # Phase 5: Wikipedia Concept Fill (fill empty nodes with Wikipedia content)
    try:
        phase_wikipedia_concept_fill(db, pipeline, limit=8)
    except Exception as e:
        log("ERROR", "wikifill", f"Phase failed: {e}")
        record_defect("PHASE_FAIL", "wikifill", str(e), severity=0.6)

    # Phase 6: GitHub Trending
    try:
        phase_github_trending(db, pipeline, count=3)
    except Exception as e:
        log("ERROR", "github", f"Phase failed: {e}")
        record_defect("PHASE_FAIL", "github", str(e), severity=0.6)

    # Phase 6: KB Analysis
    analysis = None
    try:
        analysis = phase_kb_analysis(db)
    except Exception as e:
        log("ERROR", "analysis", f"Phase failed: {e}")
        record_defect("PHASE_FAIL", "analysis", str(e), severity=0.5)

    # Phase 7: Meta-Cognition Summary
    try:
        metacog = phase_metacognition_summary(db)
    except Exception as e:
        log("ERROR", "metacog", f"Phase failed: {e}")
        record_defect("PHASE_FAIL", "metacog", str(e), severity=0.5)

    # Final snapshot for this cycle
    end_snapshot = phase_snapshot(db)

    delta = {
        "nodes_gained": end_snapshot.get("total_nodes", 0) - snapshot.get("total_nodes", 0),
        "edges_gained": end_snapshot.get("total_edges", 0) - snapshot.get("total_edges", 0),
        "empty_filled": snapshot.get("empty_content", 0) - end_snapshot.get("empty_content", 0),
    }
    log("OK", "end", f"═══ Cycle {CYCLE_COUNT} done: +{delta['nodes_gained']} nodes, +{delta['edges_gained']} edges, -{delta['empty_filled']} empty ═══")
    return delta


def generate_todo_list(db):
    """Generate evolution todo list from defects found in current session."""
    log("INFO", "todo", "Generating evolution todo list from meta-cognition defects")

    defect_prefix = f"auto_absorb_defect_{CYCLE_COUNT}_"
    defects_raw = _safe_fetchall(db,
        "SELECT value FROM kv_store WHERE namespace='meta_cognition' AND key LIKE ? ORDER BY updated_at DESC",
        (f"{defect_prefix}%",))

    # Aggregate by defect_type
    by_type = {}
    for row in defects_raw:
        try:
            d = json.loads(row[0])
            dt = d.get("defect_type", "UNKNOWN")
            by_type.setdefault(dt, []).append(d)
        except Exception:
            pass

        
    todo_entries = []

    # Generate todo items from defect patterns
    if "API_FAIL" in by_type:
        sources = {}
        for d in by_type["API_FAIL"]:
            s = d.get("source", "unknown")
            sources[s] = sources.get(s, 0) + 1
        for src, cnt in sorted(sources.items(), key=lambda x: -x[1]):
            if cnt >= 3:
                todo_entries.append({
                    "priority": "HIGH" if cnt >= 5 else "MEDIUM",
                    "title": f"API_FAIL: {src} failed {cnt} times - add retry/fallback",
                    "defect_type": "API_FAIL",
                    "count": cnt,
                    "source": src,
                })

    if "DB_ERROR" in by_type:
        todo_entries.append({
            "priority": "HIGH",
            "title": f"DB_ERROR: {len(by_type['DB_ERROR'])} SQLite errors - check lock contention + schema",
            "defect_type": "DB_ERROR",
            "count": len(by_type["DB_ERROR"]),
        })

    if "DUPLICATE_NODE" in by_type:
        todo_entries.append({
            "priority": "MEDIUM",
            "title": f"DUPLICATE_NODE: {len(by_type['DUPLICATE_NODE'])} duplicate URLs detected - enhance dedup pipeline",
            "defect_type": "DUPLICATE_NODE",
            "count": len(by_type["DUPLICATE_NODE"]),
        })

    if "PROCESSING_ERROR" in by_type:
        todo_entries.append({
            "priority": "HIGH",
            "title": f"PROCESSING_ERROR: {len(by_type['PROCESSING_ERROR'])} processing errors - review crawl queue handling",
            "defect_type": "PROCESSING_ERROR",
            "count": len(by_type["PROCESSING_ERROR"]),
        })

    if "FILL_ERROR" in by_type:
        todo_entries.append({
            "priority": "MEDIUM",
            "title": f"FILL_ERROR: {len(by_type['FILL_ERROR'])} content fill errors - check ArXiv HTML fallback",
            "defect_type": "FILL_ERROR",
            "count": len(by_type["FILL_ERROR"]),
        })

    # KB structural issues
    empty_count = _safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE content IS NULL OR content = ''")[0]
    if empty_count > 1000:
        todo_entries.append({
            "priority": "HIGH",
            "title": f"{empty_count} nodes still have empty content - need bulk fill pipeline",
            "defect_type": "EMPTY_CONTENT",
            "count": empty_count,
        })

    orphaned = _safe_fetchone(db, "SELECT COUNT(*) FROM nodes n WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.source_id = n.id OR e.target_id = n.id)")[0]
    if orphaned > 100:
        todo_entries.append({
            "priority": "MEDIUM",
            "title": f"{orphaned} orphaned nodes (no edges) - need edge discovery pipeline",
            "defect_type": "ORPHANED_NODES",
            "count": orphaned,
        })

    dup_urls = _safe_fetchone(db, "SELECT COUNT(*) FROM (SELECT url, COUNT(*) as cnt FROM nodes WHERE url != '' GROUP BY url HAVING cnt > 1)")[0]
    if dup_urls > 0:
        todo_entries.append({
            "priority": "MEDIUM",
            "title": f"{dup_urls} duplicate URLs detected - run dedup merge",
            "defect_type": "DUPLICATE_URL",
            "count": dup_urls,
        })

    crawl_failed = _safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='failed'")[0]
    if crawl_failed > 50:
        todo_entries.append({
            "priority": "LOW",
            "title": f"{crawl_failed} failed crawl queue items - review and retry",
            "defect_type": "CRAWL_FAILED",
            "count": crawl_failed,
        })

    # Sort by priority
    priority_order = {"HIGH": 0, "MEDIUM": 1, "LOW": 2}
    todo_entries.sort(key=lambda x: (priority_order.get(x["priority"], 99), -x.get("count", 0)))

    # Store in KB
    now = int(time.time())
    todo_json = json.dumps({"entries": todo_entries, "generated_at": now, "uptime_sec": int(time.time()) - STARTED_AT})
    _safe_execute(db, "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                  ("meta_cognition", "auto_absorb_todo_list", todo_json, now))
    _safe_execute(db, "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                  ("meta_cognition", "latest_todo_list", todo_json, now))
    db.commit()

    return todo_entries


CRAWL_SEED_OFFSET = 0

def _seed_crawl_queue_startup(db):
    """Seed crawl queue URLs from Wikipedia, rotating through WIKI_TOPICS each cycle."""
    global CRAWL_SEED_OFFSET
    pending = _safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='pending'")[0]
    if pending > 10:
        return 0
    log("INFO", "seed", f"Crawl queue low ({pending} pending). Seeding fresh URLs...")
    now = int(time.time())
    seeded = 0
    import urllib.request, urllib.error, urllib.parse
    batch_size = 60
    total = len(WIKI_TOPICS)
    for i in range(batch_size):
        idx = (CRAWL_SEED_OFFSET + i) % total
        topic = WIKI_TOPICS[idx]
        url = f"https://en.wikipedia.org/wiki/{topic.replace(' ', '_')}"
        qid = "seed-" + hashlib.md5(url.encode()).hexdigest()[:16]
        try:
            cursor = db.execute("SELECT status FROM crawl_queue WHERE id=?", (qid,))
            row = cursor.fetchone()
            if row is None:
                _safe_execute(db,
                    "INSERT INTO crawl_queue (id, url, depth, priority, status, discovered_at) VALUES (?, ?, ?, ?, 'pending', ?)",
                    (qid, url, 1, 10, now))
                seeded += 1
            elif row[0] in ('failed', 'done'):
                _safe_execute(db,
                    "UPDATE crawl_queue SET status='pending', last_attempt=NULL WHERE id=?",
                    (qid,))
                seeded += 1
        except Exception:
            pass
    CRAWL_SEED_OFFSET = (CRAWL_SEED_OFFSET + batch_size) % total
    db.commit()
    log("INFO", "seed", f"Seeded {seeded} Wikipedia URLs into crawl queue (offset={CRAWL_SEED_OFFSET})")
    return seeded


def main():
    global SHUTDOWN
    import argparse
    parser = argparse.ArgumentParser(description="NeoTrix Auto-Absorption Pipeline")
    parser.add_argument("--cycles", type=int, default=0, help="Number of cycles to run (0 = infinite)")
    parser.add_argument("--interval", type=int, default=600, help="Seconds between cycles (default: 600)")
    parser.add_argument("--daemon", action="store_true", help="Fork to background")
    parser.add_argument("--generate-todo", action="store_true", help="Generate todo list from existing defects and exit")
    args = parser.parse_args()

    if args.generate_todo:
        db = _get_db()
        todos = generate_todo_list(db)
        print(f"\n{'='*70}")
        print(f"  NeoTrix Evolution Todo List — Generated at {time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"{'='*70}")
        for i, t in enumerate(todos, 1):
            icon = "🔴" if t["priority"] == "HIGH" else "🟡" if t["priority"] == "MEDIUM" else "🟢"
            print(f"\n  {icon} [{t['priority']}] #{i}: {t['title']}")
            print(f"     Type: {t['defect_type']} | Count: {t.get('count', '?')}")
        print(f"\n{'='*70}")
        print(f"  Total: {len(todos)} todo items")
        db.close()
        return

    if args.daemon:
        pid = os.fork()
        if pid > 0:
            with open(PID_PATH, "w") as f:
                f.write(str(pid))
            print(f"Daemon started (PID {pid}). Log: {LOG_PATH}")
            print(f"Monitor: tail -f {LOG_PATH}")
            print(f"Stop:    kill {pid}")
            sys.exit(0)

    # Write PID
    with open(PID_PATH, "w") as f:
        f.write(str(os.getpid()))

    print(f"NeoTrix Auto-Absorption Pipeline")
    print(f"  KB:   {KB_PATH}")
    print(f"  Log:  {LOG_PATH}")
    print(f"  PID:  {os.getpid()}")
    print()

    db = _get_db()
    pipeline = AccessPipeline(cache_capability=False)
    cap = pipeline.probe()
    print(f"Capability probe:")
    print(f"  ArXiv:    {'✅' if cap.get('arxiv_api') else '❌'}")
    print(f"  GitHub:   {pipeline.github.rate_limit_status()}")
    print(f"  Wiki:     {'✅' if cap.get('wikipedia') else '❌'}")
    print(f"  Tor:      {'✅' if cap.get('tor') else '❌'}")
    print(f"  Proxy:    {'✅' if cap.get('proxy_available') else '❌'}")
    print(f"  External: {cap.get('external_ip', '?')}")
    print()

    cycles_goal = args.cycles or float('inf')
    cycle = 0
    start_time = time.time()

    # Seed crawl queue on startup
    _seed_crawl_queue_startup(db)

    while cycle < cycles_goal and not SHUTDOWN:
        cycle_start = time.time()
        try:
            run_cycle(db, pipeline)
        except Exception as e:
            log("ERROR", "main", f"Cycle failed: {e}\n{traceback.format_exc()}")
            record_defect("CYCLE_FAIL", "main", str(e), severity=0.9)
        cycle += 1

        # Check runtime
        runtime_sec = time.time() - start_time
        runtime_hours = runtime_sec / 3600
        log("INFO", "main", f"Runtime: {runtime_hours:.1f}h / goals ({cycles_goal} cycles or ~10h)")

        if runtime_hours >= 10:
            log("INFO", "main", "10-hour target reached. Generating final todo list and shutting down.")
            break

        if not SHUTDOWN and cycle < cycles_goal:
            log("INFO", "main", f"Sleeping {args.interval}s before next cycle...")
            # Sleep in short increments to check SHUTDOWN
            for _ in range(args.interval // 5):
                if SHUTDOWN:
                    break
                time.sleep(5)

    # Generate final todo list
    log("INFO", "main", "Generating final evolution todo list from all defects...")
    try:
        todos = generate_todo_list(db)
        log("OK", "main", f"Generated {len(todos)} evolution todo items")
        # Print summary
        print(f"\n{'='*70}")
        print(f"  NeoTrix Evolution Todo List — {time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"  Total runtime: {(time.time() - start_time) / 3600:.1f}h")
        print(f"  Cycles completed: {CYCLE_COUNT}")
        print(f"{'='*70}")
        for i, t in enumerate(todos, 1):
            icon = "🔴" if t["priority"] == "HIGH" else "🟡" if t["priority"] == "MEDIUM" else "🟢"
            print(f"  {icon} P{t['priority']:6s} | {t['title']}")
        print(f"{'='*70}")
        print(f"  Total: {len(todos)} todo items")
    except Exception as e:
        log("ERROR", "main", f"Failed to generate todo list: {e}")

    # Final snapshot
    try:
        fs = phase_snapshot(db)
        log("OK", "main", f"Final KB: {fs['total_nodes']} nodes, {fs['total_edges']} edges")
    except Exception:
        pass



    db.close()
    os.remove(PID_PATH)
    log("INFO", "main", "Shutdown complete")


if __name__ == "__main__":
    import random
    main()
