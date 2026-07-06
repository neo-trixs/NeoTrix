#!/usr/bin/env python3
"""Inject 350 seed URLs into crawl_queue for the auto-absorption pipeline."""
import sqlite3, os, hashlib, time

KB = os.path.expanduser("~/.neotrix/knowledge.db")
NOW = int(time.time())

GITHUB_REPOS = [
    "topoteretes/cognee","DeusData/codebase-memory-mcp","diegosouzapw/OmniRoute",
    "ogulcancelik/herdr","msitarzewski/agency-agents","stablyai/orca",
    "usestrix/strix","alibaba/page-agent","browser-use/video-use",
    "callesthio/OpenMontage","openai/codex-plugin-cc","n8n-io/n8n",
    "langgenius/dify","getcursor/cursor","continuedev/continue",
    "sourcegraph/cody","TabbyML/tabby","aider-ai/aider","yohasebe/mcp-tools",
    "executeautomation/mcp-playwright","modelcontextprotocol/servers",
    "anthropics/anthropic-cookbook","openai/openai-cookbook","neotrix/neotrix",
    "deepseek-ai/DeepSeek-V3","deepseek-ai/DeepSeek-R1","meta-llama/llama",
    "ggerganov/llama.cpp","oobabooga/text-generation-webui",
    "comfyanonymous/ComfyUI","AUTOMATIC1111/stable-diffusion-webui",
    "langchain-ai/langchain","run-llama/llama_index","chatanywhere/GPT_API_free",
    "lobehub/lobe-chat","siyuan-note/siyuan","logseq/logseq",
    "zotero/zotero","jina-ai/jina","weaviate/weaviate","qdrant/qdrant",
    "milvus-io/milvus","pingcap/tidb","cockroachdb/cockroach",
    "yugabyte/yugabyte-db","timescale/timescaledb","influxdata/influxdb",
    "apache/spark","apache/flink","apache/kafka","apache/hadoop",
    "redis/redis","memcached/memcached","nginx/nginx","apache/httpd",
    "nodejs/node","denoland/deno","bun-sh/bun","rust-lang/rust",
    "golang/go","python/cpython","dotnet/runtime","apple/swift",
    "kotlin/kotlin-spec","openjdk/jdk","v8/v8","llvm/llvm-project",
    "tensorflow/tensorflow","pytorch/pytorch","jax-ml/jax",
    "google-deepmind/alphafold","NVIDIA/Megatron-LM","microsoft/DeepSpeed",
    "facebookresearch/llama","huggingface/transformers","huggingface/diffusers",
]

WIKI_TOPICS = [
    "Artificial_intelligence","Machine_learning","Deep_learning","Natural_language_processing",
    "Computer_vision","Robotics","Reinforcement_learning","Transformer",
    "GPT","BERT","Diffusion_model","Variational_autoencoder",
    "Generative_adversarial_network","Graph_neural_network","Neural_radiance_field",
    "Quantum_computing","Neuromorphic_computing","Edge_computing",
    "Federated_learning","Differential_privacy","Homomorphic_encryption",
    "Semantic_Web","Knowledge_graph","Ontology","RDF","SPARQL",
    "Vector_database","Embedding","Information_retrieval","Semantic_search",
    "Question_answering","Machine_translation","Speech_recognition","Text-to-speech",
    "Autonomous_vehicle","Self-driving_car","SLAM","Simultaneous_localization_and_mapping",
    "Multi-agent_system","Swarm_intelligence","Evolutionary_algorithm","Genetic_algorithm",
    "Ant_colony_optimization","Particle_swarm_optimization","Simulated_annealing",
    "Bayesian_network","Markov_chain","Monte_Carlo_method","Kalman_filter",
    "Particle_filter","Causal_inference","Instrumental_variable","Structural_equation_modeling",
    "Decision_theory","Game_theory","Mechanism_design","Social_choice_theory",
    "Computational_complexity_theory","P=NP","NP-completeness","Boolean_satisfiability",
    "Model_checking","Formal_verification","Theorem_proving","SAT_solver",
    "SMT_solver","Constraint_satisfaction","Answer_set_programming","Logic_programming",
    "Prolog","Datalog","Lambda_calculus","Type_theory","Category_theory",
    "Homotopy_type_theory","Univalent_foundations","Proof_assistant","Coq","Isabelle",
    "Lean","Agda","Idris","Haskell","OCaml","F#","Scala","Elixir","Erlang",
    "Clojure","Scheme","Common_Lisp","Racket","Julia","MATLAB","R",
    "Wolfram_Language","Mathematica","Maple","SageMath","SymPy","NumPy","SciPy",
    "Pandas","Scikit-learn","XGBoost","LightGBM","CatBoost","FastAI",
    "Keras","PyTorch_Lightning","Weights_and_Biases","MLflow","Kubeflow",
    "DVC","Git_LFS","Docker","Kubernetes","Terraform","Ansible","Puppet",
    "Chef","Nix","NixOS","Guix","GNU_Guix","Flatpak","Snap","AppImage",
    "WebAssembly","Emscripten","Blazor","Yew","Leptos","Dioxus","Tauri",
    "Electron","Flutter","React_Native","Ionic","Capacitor","Cordova",
    "SwiftUI","Jetpack_Compose","Xamarin","MAUI","WASM","SIMD","AVX",
    "ARM","RISC-V","MIPS","x86","AMD64","AArch64","OpenCL","SYCL",
    "Vulkan","DirectX","Metal","CUDA","ROCm","oneAPI","OpenMP","MPI",
    "GPGPU","FPGA","ASIC","TPU","NPU","VPU","Tensor_Processing_Unit",
    "Vision_Processing_Unit","Neural_Processing_Unit","DPU",
    "CXL","Compute_Express_Link","PCI_Express","NVLink","CXL_memory_pooling",
    "CXL_type_3_device","SmartNIC","DPU_data_processing_unit","IPU",
]

ARXIV_IDS = [
    "2303.08774","2306.07951","2307.09288","2310.06825","2312.00752",
    "2401.14295","2402.06196","2403.08763","2404.14294","2405.20323",
    "2406.07550","2407.21783","2408.05634","2409.12271","2410.01131",
    "2411.15124","2412.04434","2501.00342","2502.01671","2503.02827",
    "2005.14165","2103.00020","2106.09685","2203.02155","2205.10625",
    "2301.00234","2302.13971","2303.12712","2304.08485","2305.18290",
    "2306.15243","2307.08621","2308.08155","2309.05401","2310.03714",
    "2311.04217","2312.11805","2401.05329","2402.04625","2403.05530",
    "2404.05834","2405.04604","2406.07691","2407.04873","2408.03314",
    "2409.04723","2107.03374","2201.11903","2204.06125","2210.03629",
]

def run():
    db = sqlite3.connect(KB, timeout=60)
    db.execute("PRAGMA journal_mode=WAL")
    db.execute("PRAGMA busy_timeout=30000")
    c = db.cursor()

    # Get existing URLs to avoid duplicates
    existing = set()
    for row in c.execute("SELECT url FROM crawl_queue"):
        existing.add(row[0])
    for row in c.execute("SELECT url FROM nodes WHERE url IS NOT NULL AND url != ''"):
        existing.add(row[0])

    total_injected = 0
    by_source = {}

    def inject(url, source):
        nonlocal total_injected
        if url in existing:
            return
        qid = "nt-cq-" + hashlib.md5(url.encode()).hexdigest()[:20]
        c.execute(
            "INSERT OR IGNORE INTO crawl_queue (id, url, domain, status, discovered_at) VALUES (?, ?, ?, 'pending', ?)",
            (qid, url, source.split("-")[0], NOW)
        )
        if c.rowcount > 0:
            total_injected += 1
            by_source[source] = by_source.get(source, 0) + 1

    # GitHub repos
    for repo in GITHUB_REPOS:
        inject(f"https://github.com/{repo}", "github-trending")

    # Wikipedia topics
    for topic in WIKI_TOPICS:
        inject(f"https://en.wikipedia.org/wiki/{topic}", "wikipedia-seed")

    # ArXiv papers
    for aid in ARXIV_IDS:
        inject(f"https://arxiv.org/abs/{aid}", "arxiv-seed")

    db.commit()

    print(f"Crawl Queue Seed Injection")
    print(f"  Total injected: {total_injected}")
    for src, cnt in sorted(by_source.items(), key=lambda x: -x[1]):
        print(f"  {src:20s}: {cnt}")
    total_attempted = len(GITHUB_REPOS) + len(WIKI_TOPICS) + len(ARXIV_IDS)
    print(f"  Duplicates skipped: {total_attempted - total_injected}")
    pending_q = c.execute("SELECT COUNT(*) FROM crawl_queue WHERE status='pending'").fetchone()[0]
    print(f"  Queue now has: {pending_q} pending")
    db.close()

if __name__ == "__main__":
    run()
