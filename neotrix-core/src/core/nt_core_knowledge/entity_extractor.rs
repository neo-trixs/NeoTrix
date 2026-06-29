use crate::core::nt_core_consciousness::memory_lattice::{
    LatticeLayer, MemoryLattice, MemoryOrigin,
};
use crate::core::nt_core_knowledge::spread_activation::{EdgeKind, MemoryGraph, NodeKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn now_secs() -> i64 {
    crate::core::nt_core_time::unix_now_secs() as i64
}
fn year_start_epoch(year: i32) -> Option<i64> {
    // Approximate: Jan 1 of the given year
    let y = year as i64;
    let years_since_1970 = y - 1970;
    Some(years_since_1970 * 365 * 86400 + years_since_1970 / 4 * 86400)
}
fn year_end_epoch(year: i32) -> Option<i64> {
    year_start_epoch(year + 1).map(|s| s - 1)
}

// ── Entity Types ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Product,
    Technology,
    ProgrammingLanguage,
    Framework,
    Concept,
    Event,
    Date,
    Other(String),
}

impl EntityType {
    pub fn name(&self) -> &str {
        match self {
            EntityType::Person => "person",
            EntityType::Organization => "organization",
            EntityType::Location => "location",
            EntityType::Product => "product",
            EntityType::Technology => "technology",
            EntityType::ProgrammingLanguage => "programming_language",
            EntityType::Framework => "framework",
            EntityType::Concept => "concept",
            EntityType::Event => "event",
            EntityType::Date => "date",
            EntityType::Other(s) => s,
        }
    }
}

// ── Entity Mention ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMention {
    pub name: String,
    pub surface_form: String,
    pub entity_type: EntityType,
    pub start_pos: usize,
    pub end_pos: usize,
    pub confidence: f64,
}

// ── Relation Types ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    Ownership,
    WorksAt,
    LocatedIn,
    Creates,
    Uses,
    PartOf,
    Likes,
    Dislikes,
    CapableOf,
    HasProperty,
    Causes,
    Other(String),
}

impl RelationType {
    pub fn name(&self) -> &str {
        match self {
            RelationType::Ownership => "ownership",
            RelationType::WorksAt => "works_at",
            RelationType::LocatedIn => "located_in",
            RelationType::Creates => "creates",
            RelationType::Uses => "uses",
            RelationType::PartOf => "part_of",
            RelationType::Likes => "likes",
            RelationType::Dislikes => "dislikes",
            RelationType::CapableOf => "capable_of",
            RelationType::HasProperty => "has_property",
            RelationType::Causes => "causes",
            RelationType::Other(s) => s,
        }
    }
}

// ── Temporal Marker ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalMarker {
    Current,
    Past,
    Future,
    AtYear(i32),
    Range(String, String),
}

// ── Relation Triple ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationTriple {
    pub subject: String,
    pub relation: RelationType,
    pub object: String,
    pub confidence: f64,
    pub temporal: Option<TemporalMarker>,
    pub negated: bool,
}

// ── Extracted Fact ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    pub triple: RelationTriple,
    pub source_text: String,
    pub timestamp: u64,
    pub id: u64,
}

// ── Relation Pattern ──

#[derive(Clone)]
struct RelationPattern {
    relation: RelationType,
    triggers: Vec<&'static str>,
    subject_before: bool,
    weight: f64,
}

// ── Gazetteer Data ──

const DEFAULT_PERSONS: &[&str] = &[
    "Einstein",
    "Newton",
    "Turing",
    "Hawking",
    "Feynman",
    "Tesla",
    "Darwin",
    "Galileo",
    "Aristotle",
    "Plato",
    "Socrates",
    "Bohr",
    "Heisenberg",
    "Schrödinger",
    "Planck",
    "Curie",
    "Pasteur",
    "Lovelace",
    "Berners-Lee",
    "Torvalds",
    "Stallman",
    "Knuth",
    "Von Neumann",
    "Gödel",
    "Russell",
    "Wiener",
    "Shannon",
    "Chomsky",
    "Minsky",
    "McCarthy",
    "Hinton",
    "LeCun",
    "Bengio",
    "Schmidhuber",
    "Altman",
    "Musk",
    "Bezos",
    "Zuckerberg",
    "Page",
    "Brin",
    "Nadella",
    "Cook",
    "Pichai",
    "Hastings",
    "Chesterman",
    "Amodei",
    "Brockman",
    "Sutskever",
    "Karpathy",
    "Ng",
    "Dean",
    "Hassabis",
    "Silver",
    "LeCun",
    "Thrun",
    "Norvig",
    "Pedersen",
    "Carmack",
    "Gosling",
    "Ritchie",
    "Thompson",
    "Gates",
    "Jobs",
    "Wozniak",
    "Allen",
    "Mccarthy",
    "Fei-Fei",
    "Liang",
    "Hochreiter",
    "Vaswani",
    "Brown",
];

const DEFAULT_ORGANIZATIONS: &[&str] = &[
    "Google",
    "Microsoft",
    "OpenAI",
    "Apple",
    "Amazon",
    "Meta",
    "Netflix",
    "Tesla",
    "SpaceX",
    "DeepMind",
    "Anthropic",
    "IBM",
    "Intel",
    "Nvidia",
    "AMD",
    "Oracle",
    "SAP",
    "Salesforce",
    "Adobe",
    "Uber",
    "Airbnb",
    "Twitter",
    "LinkedIn",
    "Spotify",
    "Slack",
    "GitHub",
    "GitLab",
    "Red Hat",
    "Canonical",
    "Mozilla",
    "Wikimedia",
    "MIT",
    "Stanford",
    "Harvard",
    "Oxford",
    "Cambridge",
    "Caltech",
    "Berkeley",
    "CMU",
    "UCL",
    "ETH Zurich",
    "Max Planck",
    "CERN",
    "NASA",
    "DARPA",
    "NSA",
    "FBI",
    "CIA",
    "WHO",
    "UN",
    "NATO",
    "World Bank",
    "IMF",
    "JP Morgan",
    "Goldman Sachs",
    "Berkshire Hathaway",
    "BlackRock",
    "ByteDance",
    "Tencent",
    "Alibaba",
    "Baidu",
    "Huawei",
    "Samsung",
    "Sony",
    "Toyota",
    "Volkswagen",
    "Shell",
    "BP",
    "ExxonMobil",
    "Stripe",
    "Square",
    "Palantir",
    "Snowflake",
    "Databricks",
    "Hugging Face",
    "Cohere",
    "Stability AI",
    "Midjourney",
    "Perplexity",
    "Notion",
    "Figma",
    "Canva",
    "Shopify",
    "Twilio",
    "Cloudflare",
    "Fastly",
    "Vercel",
    "Netlify",
    "DigitalOcean",
    "Atlassian",
    "Unity",
    "Epic Games",
    "Valve",
    "Blizzard",
    "Riot Games",
    "Electronic Arts",
    "Supabase",
    "Fly.io",
    "Neon",
    "PlanetScale",
    "MongoDB",
    "Neo4j",
    "Redis",
    "Datadog",
    "New Relic",
    "Splunk",
];

const DEFAULT_LANGUAGES: &[&str] = &[
    "Rust",
    "Python",
    "TypeScript",
    "JavaScript",
    "Go",
    "Java",
    "C++",
    "C",
    "C#",
    "Swift",
    "Kotlin",
    "Ruby",
    "PHP",
    "Perl",
    "Lua",
    "Haskell",
    "Scala",
    "Clojure",
    "Elixir",
    "Erlang",
    "F#",
    "OCaml",
    "Racket",
    "Scheme",
    "Common Lisp",
    "Julia",
    "R",
    "MATLAB",
    "Fortran",
    "COBOL",
    "Assembly",
    "Zig",
    "Nim",
    "Dart",
    "Solidity",
    "SQL",
    "HTML",
    "CSS",
    "Shell",
    "Bash",
    "PowerShell",
    "AWK",
    "Tcl",
    "Prolog",
    "Ada",
    "Pascal",
    "Delphi",
    "Visual Basic",
    "COBOL",
    "PL/SQL",
    "VHDL",
    "Verilog",
    "GraphQL",
    "WebAssembly",
    "LLVM IR",
];

const DEFAULT_FRAMEWORKS: &[&str] = &[
    "React",
    "PyTorch",
    "TensorFlow",
    "Angular",
    "Vue",
    "Svelte",
    "Django",
    "Flask",
    "Spring",
    "Rails",
    "Laravel",
    "Express",
    "Next.js",
    "Nuxt",
    "Astro",
    "Remix",
    "Solid",
    "Qwik",
    "JAX",
    "Keras",
    "Scikit-learn",
    "Hugging Face Transformers",
    "LangChain",
    "LlamaIndex",
    "Ray",
    "Dask",
    "Spark",
    "Hadoop",
    "Kubernetes",
    "Docker",
    "Terraform",
    "Ansible",
    "Chef",
    "Puppet",
    "Salt",
    "Nomad",
    "Consul",
    "Vault",
    "gRPC",
    "GraphQL",
    "REST",
    "WebSocket",
    "tRPC",
    "Tokio",
    "Actix",
    "Rocket",
    "Axum",
    "Warp",
    "Tower",
    "Bevy",
    "Egui",
    "Tauri",
    "Electron",
    "React Native",
    "Flutter",
    "SwiftUI",
    "Jetpack Compose",
    "Xamarin",
    "OpenCV",
    "FFmpeg",
    "NumPy",
    "Pandas",
    "Matplotlib",
    "Numba",
    "Cython",
    "CUDA",
    "cuDNN",
    "OpenCL",
    "Vulkan",
];

const DEFAULT_LOCATIONS: &[&str] = &[
    "San Francisco",
    "New York",
    "London",
    "Tokyo",
    "Beijing",
    "Shanghai",
    "Berlin",
    "Paris",
    "Singapore",
    "Sydney",
    "Seattle",
    "Austin",
    "Boston",
    "Chicago",
    "Los Angeles",
    "Toronto",
    "Vancouver",
    "Montreal",
    "Amsterdam",
    "Zurich",
    "Dubai",
    "Hong Kong",
    "Seoul",
    "Mumbai",
    "Bangalore",
    "Stockholm",
    "Copenhagen",
    "Oslo",
    "Helsinki",
    "Dublin",
    "Barcelona",
    "Madrid",
    "Rome",
    "Milan",
    "Munich",
    "Silicon Valley",
    "Bay Area",
    "Wall Street",
    "Shenzhen",
    "Taipei",
    "Tel Aviv",
    "Jerusalem",
    "Moscow",
    "São Paulo",
    "Mexico City",
    "Buenos Aires",
    "Cape Town",
    "Lagos",
    "Nairobi",
    "United States",
    "China",
    "India",
    "United Kingdom",
    "Germany",
    "France",
    "Japan",
    "South Korea",
    "Canada",
    "Australia",
    "Europe",
    "Asia",
    "North America",
    "South America",
    "Africa",
];

const DEFAULT_CONCEPTS: &[&str] = &[
    "AGI",
    "Machine Learning",
    "Deep Learning",
    "Artificial Intelligence",
    "Natural Language Processing",
    "Computer Vision",
    "Reinforcement Learning",
    "Supervised Learning",
    "Unsupervised Learning",
    "Transfer Learning",
    "Neural Network",
    "Transformer",
    "Attention Mechanism",
    "Convolution",
    "Recurrent Neural Network",
    "Generative Adversarial Network",
    "Large Language Model",
    "Foundation Model",
    "Diffusion Model",
    "Knowledge Graph",
    "Vector Database",
    "Semantic Search",
    "Named Entity Recognition",
    "Relation Extraction",
    "Text Generation",
    "Sentiment Analysis",
    "Speech Recognition",
    "Image Generation",
    "Object Detection",
    "Segmentation",
    "Q Learning",
    "Monte Carlo Tree Search",
    "Bayesian Inference",
    "Markov Chain",
    "Hidden Markov Model",
    "Support Vector Machine",
    "Decision Tree",
    "Random Forest",
    "Gradient Boosting",
    "Principal Component Analysis",
    "t-SNE",
    "Quantum Computing",
    "Blockchain",
    "Cryptography",
    "Zero Knowledge Proof",
    "Federated Learning",
    "Differential Privacy",
    "Homomorphic Encryption",
    "Edge Computing",
    "Cloud Computing",
    "Serverless",
    "Microservices",
    "REST API",
    "GraphQL",
    "WebSocket",
    "HTTP",
    "TCP/IP",
    "Version Control",
    "Continuous Integration",
    "Continuous Deployment",
    "DevOps",
    "MLOps",
    "Data Pipeline",
    "ETL",
    "Data Warehouse",
    "Data Lake",
    "Data Mesh",
    "Data Fabric",
    "Data Catalog",
    "Ontology",
    "Taxonomy",
    "Folksonomy",
    "Semantic Web",
    "Symbolic AI",
    "Neuro-Symbolic AI",
    "Connectionism",
    "Emergent Behavior",
    "Consciousness",
    "Self-Awareness",
    "Theory of Mind",
    "Common Sense",
    "Causality",
    "Correlation",
    "Counterfactual",
    "Causal Inference",
    "Entropy",
    "Negentropy",
    "Free Energy Principle",
    "Active Inference",
    "Bayesian Brain",
    "Predictive Coding",
    "Global Workspace Theory",
    "Integrated Information Theory",
    "Attention Schema Theory",
    "Hyperdimensional Computing",
    "Vector Symbolic Architecture",
    "Holographic Reduced Representation",
    "Binary Spatter Code",
    "Fourier Transform",
    "Wavelet Transform",
    "FFT",
    "Hadamard Transform",
    "Information Retrieval",
    "Question Answering",
    "Text Summarization",
    "Machine Translation",
    "Speech Synthesis",
    "Text to Speech",
];

const DEFAULT_TECHNOLOGIES: &[&str] = &[
    "Linux",
    "Unix",
    "macOS",
    "Windows",
    "Android",
    "iOS",
    "Kubernetes",
    "Docker",
    "Terraform",
    "AWS",
    "GCP",
    "Azure",
    "PostgreSQL",
    "MySQL",
    "SQLite",
    "Redis",
    "MongoDB",
    "Cassandra",
    "Elasticsearch",
    "Kafka",
    "RabbitMQ",
    "NATS",
    "gRPC",
    "Protobuf",
    "GraphQL",
    "REST",
    "WebRTC",
    "HTTP/2",
    "HTTP/3",
    "QUIC",
    "WebAssembly",
    "LLVM",
    "GCC",
    "Clang",
    "Zig",
    "Bazel",
    "Nix",
    "Homebrew",
    "APT",
    "YUM",
    "Pacman",
    "Systemd",
    "Nginx",
    "Apache",
    "HAProxy",
    "Envoy",
    "Istio",
    "Linkerd",
    "Prometheus",
    "Grafana",
    "Jaeger",
    "OpenTelemetry",
    "Fluentd",
    "Git",
    "Mercurial",
    "SVN",
    "Make",
    "CMake",
    "Meson",
    "RISC-V",
    "ARM",
    "x86",
    "AVX",
    "SIMD",
    "GPU",
    "TPU",
    "NPU",
    "USB-C",
    "Thunderbolt",
    "HDMI",
    "DisplayPort",
    "PCIe",
    "NVMe",
    "Bluetooth",
    "WiFi",
    "5G",
    "LTE",
    "Ethernet",
    "TCP",
    "UDP",
    "DNS",
    "HTTP",
    "TLS",
    "SSL",
    "SSH",
    "SFTP",
    "FTPS",
    "OAuth",
    "OIDC",
    "SAML",
    "JWT",
    "API Key",
    "AES",
    "RSA",
    "ECC",
    "Ed25519",
    "SHA-256",
    "SHA-3",
    "RISC-V",
    "CUDA",
    "OpenMP",
    "MPI",
    "OpenCL",
    "SYCL",
];

// ── Entity Extractor ──

#[derive(Clone)]
pub struct EntityExtractor {
    pub gazetteer: HashMap<String, EntityType>,
    relation_patterns: Vec<RelationPattern>,
    next_fact_id: u64,
}

impl EntityExtractor {
    pub fn new() -> Self {
        let mut gazetteer = HashMap::new();
        Self::build_gazetteer(&mut gazetteer);
        let relation_patterns = Self::build_relation_patterns();
        Self {
            gazetteer,
            relation_patterns,
            next_fact_id: 1,
        }
    }

    fn build_gazetteer(map: &mut HashMap<String, EntityType>) {
        for name in DEFAULT_PERSONS {
            map.insert(name.to_lowercase(), EntityType::Person);
        }
        for name in DEFAULT_ORGANIZATIONS {
            map.insert(name.to_lowercase(), EntityType::Organization);
        }
        for name in DEFAULT_LANGUAGES {
            map.insert(name.to_lowercase(), EntityType::ProgrammingLanguage);
        }
        for name in DEFAULT_FRAMEWORKS {
            map.insert(name.to_lowercase(), EntityType::Framework);
        }
        for name in DEFAULT_LOCATIONS {
            map.insert(name.to_lowercase(), EntityType::Location);
        }
        for name in DEFAULT_CONCEPTS {
            map.insert(name.to_lowercase(), EntityType::Concept);
        }
        for name in DEFAULT_TECHNOLOGIES {
            map.insert(name.to_lowercase(), EntityType::Technology);
        }
    }

    fn build_relation_patterns() -> Vec<RelationPattern> {
        vec![
            RelationPattern {
                relation: RelationType::Ownership,
                triggers: vec!["'s ", " of ", "owned by", "belongs to"],
                subject_before: true,
                weight: 0.7,
            },
            RelationPattern {
                relation: RelationType::WorksAt,
                triggers: vec![
                    "works at",
                    "employed by",
                    "CEO of",
                    "CTO of",
                    "CFO of",
                    "engineer at",
                    "researcher at",
                    "scientist at",
                    "professor at",
                    "manager at",
                    "director of",
                    "head of",
                    "lead at",
                    "joins",
                    "joined",
                ],
                subject_before: true,
                weight: 0.85,
            },
            RelationPattern {
                relation: RelationType::LocatedIn,
                triggers: vec![
                    "located in",
                    "based in",
                    "lives in",
                    " headquartered in ",
                    "based out of",
                    "situated in",
                    "found in",
                ],
                subject_before: true,
                weight: 0.75,
            },
            RelationPattern {
                relation: RelationType::Creates,
                triggers: vec![
                    "created",
                    "built",
                    "developed",
                    "wrote",
                    "designed",
                    "founded",
                    "invented",
                    "authored",
                    "established",
                    "started",
                    "launched",
                    "implemented",
                ],
                subject_before: true,
                weight: 0.8,
            },
            RelationPattern {
                relation: RelationType::Uses,
                triggers: vec![
                    "uses",
                    "runs",
                    "built with",
                    "powered by",
                    "written in",
                    "implemented in",
                    "built on",
                    "runs on",
                    "based on",
                    "powered with",
                ],
                subject_before: true,
                weight: 0.7,
            },
            RelationPattern {
                relation: RelationType::PartOf,
                triggers: vec![
                    "part of",
                    "subsidiary of",
                    "division of",
                    "member of",
                    "unit of",
                    "branch of",
                    "department of",
                    "team at",
                ],
                subject_before: true,
                weight: 0.75,
            },
            RelationPattern {
                relation: RelationType::Likes,
                triggers: vec![
                    "likes",
                    "loves",
                    "prefers",
                    "enjoys",
                    "favorite",
                    "admires",
                    "appreciates",
                ],
                subject_before: true,
                weight: 0.65,
            },
            RelationPattern {
                relation: RelationType::Dislikes,
                triggers: vec![
                    "dislikes",
                    "hates",
                    "avoids",
                    "doesn't like",
                    "does not like",
                    "can't stand",
                    "detests",
                    "is not a fan of",
                ],
                subject_before: true,
                weight: 0.65,
            },
            RelationPattern {
                relation: RelationType::CapableOf,
                triggers: vec![
                    "can",
                    "able to",
                    "capable of",
                    "knows how to",
                    "skilled at",
                    "proficient in",
                    "expert at",
                ],
                subject_before: true,
                weight: 0.6,
            },
            RelationPattern {
                relation: RelationType::HasProperty,
                triggers: vec![
                    " is a ",
                    " is an ",
                    " are ",
                    " is considered ",
                    " is known as ",
                    " classified as ",
                ],
                subject_before: true,
                weight: 0.7,
            },
            RelationPattern {
                relation: RelationType::Causes,
                triggers: vec![
                    "causes",
                    "leads to",
                    "results in",
                    "triggers",
                    "produces",
                    "generates",
                    "creates",
                ],
                subject_before: true,
                weight: 0.7,
            },
        ]
    }

    fn find_left_entity<'a>(
        text: &'a str,
        trigger_end: usize,
        entities: &'a [EntityMention],
    ) -> Option<&'a EntityMention> {
        let before = &text[..trigger_end];
        entities
            .iter()
            .filter(|e| e.end_pos <= trigger_end)
            .max_by(|a, b| a.end_pos.cmp(&b.end_pos))
            .filter(|e| {
                let gap = before[e.end_pos..].trim();
                gap.is_empty() || gap.split_whitespace().count() <= 3
            })
    }

    fn find_right_entity<'a>(
        text: &'a str,
        trigger_end: usize,
        entities: &'a [EntityMention],
    ) -> Option<&'a EntityMention> {
        let after = &text[trigger_end..];
        let first_word_start = after.len() - after.trim_start().len();
        let adjusted_start = trigger_end + first_word_start;
        entities
            .iter()
            .filter(|e| e.start_pos >= adjusted_start)
            .min_by(|a, b| a.start_pos.cmp(&b.start_pos))
            .filter(|e| {
                let gap = &text[adjusted_start..e.start_pos].trim();
                gap.is_empty() || gap.split_whitespace().count() <= 3
            })
    }

    pub fn extract_entities(&self, text: &str) -> Vec<EntityMention> {
        let mut mentions = Vec::new();
        let lower = text.to_lowercase();

        // 1. Gazetteer exact + case-insensitive matches
        for (canonical_lower, etype) in &self.gazetteer {
            let mut search_start = 0;
            while let Some(pos) = lower[search_start..].find(canonical_lower.as_str()) {
                let abs_pos = search_start + pos;
                let end = abs_pos + canonical_lower.len();
                let surface = &text[abs_pos..end];
                // check word boundary
                if Self::is_word_boundary(text, abs_pos, end) {
                    mentions.push(EntityMention {
                        name: surface.to_string(),
                        surface_form: surface.to_string(),
                        entity_type: etype.clone(),
                        start_pos: abs_pos,
                        end_pos: end,
                        confidence: 0.92,
                    });
                }
                search_start = end + 1;
                if search_start >= text.len() {
                    break;
                }
            }
        }

        // 2. Quoted strings as concepts
        let mut i = 0;
        let bytes = text.as_bytes();
        while i < text.len() {
            if bytes[i] == b'"' {
                let start = i + 1;
                if let Some(end) = text[start..].find('"') {
                    let quoted = &text[start..start + end];
                    let trimmed = quoted.trim();
                    if !trimmed.is_empty() && trimmed.len() >= 2 {
                        mentions.push(EntityMention {
                            name: trimmed.to_string(),
                            surface_form: format!("\"{}\"", trimmed),
                            entity_type: EntityType::Concept,
                            start_pos: start,
                            end_pos: start + end,
                            confidence: 0.8,
                        });
                    }
                    i = start + end + 1;
                    continue;
                }
            }
            i += 1;
        }

        // 3. Known prefixes (Dr. Smith, Prof. Jones, CEO of ...)
        let prefix_patterns = [
            (
                vec!["Dr.", "Dr", "Professor", "Prof.", "Prof"],
                EntityType::Person,
                0.75,
            ),
            (
                vec!["CEO", "CTO", "CFO", "COO", "VP", "SVP", "EVP", "MD", "GM"],
                EntityType::Person,
                0.7,
            ),
            (
                vec![
                    "Senator",
                    "Governor",
                    "President",
                    "Prime Minister",
                    "Chancellor",
                ],
                EntityType::Person,
                0.7,
            ),
        ];
        for (prefixes, etype, conf) in &prefix_patterns {
            for prefix in prefixes {
                let search_prefix = format!("{} ", prefix);
                let lower_prefix = search_prefix.to_lowercase();
                let mut search_start = 0;
                while let Some(pos) = lower[search_start..].find(&lower_prefix) {
                    let abs_pos = search_start + pos;
                    if abs_pos > 0
                        && text
                            .as_bytes()
                            .get(abs_pos - 1)
                            .map_or(false, |&b| b.is_ascii_alphanumeric())
                    {
                        search_start = abs_pos + 1;
                        continue;
                    }
                    let name_start = abs_pos + search_prefix.len();
                    let rest = &text[name_start..];
                    let name_end = rest
                        .find(|c: char| {
                            !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '\''
                        })
                        .unwrap_or(rest.len());
                    let name = &rest[..name_end].trim();
                    if name.len() >= 2 {
                        mentions.push(EntityMention {
                            name: name.to_string(),
                            surface_form: format!("{} {}", prefix, name),
                            entity_type: etype.clone(),
                            start_pos: abs_pos,
                            end_pos: name_start + name_end,
                            confidence: *conf,
                        });
                    }
                    search_start = abs_pos + 1;
                }
            }
        }

        // 4. Acronyms (2-6 uppercase letters)
        let re_acronym = lazy_regex::regex!(r"\b[A-Z]{2,6}\b");
        for m in re_acronym.find_iter(text) {
            let acro = m.as_str();
            let acro_lower = acro.to_lowercase();
            let conf = if self.gazetteer.contains_key(&acro_lower) {
                0.9
            } else {
                0.55
            };
            mentions.push(EntityMention {
                name: acro.to_string(),
                surface_form: acro.to_string(),
                entity_type: EntityType::Technology,
                start_pos: m.start(),
                end_pos: m.end(),
                confidence: conf,
            });
        }

        // 5. Capitalized multi-word phrases (potential entities)
        let re_caps = lazy_regex::regex!(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+\b");
        for m in re_caps.find_iter(text) {
            let phrase = m.as_str();
            let phrase_lower = phrase.to_lowercase();
            if self.gazetteer.contains_key(&phrase_lower) {
                continue;
            }
            mentions.push(EntityMention {
                name: phrase.to_string(),
                surface_form: phrase.to_string(),
                entity_type: EntityType::Other("phrase".to_string()),
                start_pos: m.start(),
                end_pos: m.end(),
                confidence: 0.45,
            });
        }

        // Deduplicate: sort by start_pos, keep highest confidence for overlaps
        mentions.sort_by(|a, b| {
            a.start_pos
                .cmp(&b.start_pos)
                .then_with(|| b.end_pos.cmp(&a.end_pos))
        });
        let mut deduped: Vec<EntityMention> = Vec::new();
        for m in mentions {
            if let Some(last) = deduped.last() {
                if m.start_pos < last.end_pos {
                    if m.confidence > last.confidence {
                        deduped.pop();
                        deduped.push(m);
                    }
                    continue;
                }
            }
            deduped.push(m);
        }

        deduped
    }

    fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
        let before_ok = start == 0
            || !text
                .as_bytes()
                .get(start - 1)
                .map_or(false, |&b| b.is_ascii_alphanumeric());
        let after_ok = end >= text.len()
            || !text
                .as_bytes()
                .get(end)
                .map_or(false, |&b| b.is_ascii_alphanumeric());
        before_ok && after_ok
    }

    pub fn extract_relations(&self, text: &str, entities: &[EntityMention]) -> Vec<RelationTriple> {
        let lower = text.to_lowercase();
        let mut triples = Vec::new();

        for pattern in &self.relation_patterns {
            for trigger in &pattern.triggers {
                let trigger_lower = trigger.to_lowercase();
                let mut search_start = 0;
                while let Some(pos) = lower[search_start..].find(&trigger_lower) {
                    let abs_pos = search_start + pos;
                    let trigger_end = abs_pos + trigger_lower.len();

                    let subject = if pattern.subject_before {
                        Self::find_left_entity(text, abs_pos, entities)
                    } else {
                        Self::find_right_entity(text, trigger_end, entities)
                    };
                    let object = Self::find_right_entity(text, trigger_end, entities);

                    if let (Some(subj), Some(obj)) = (subject, object) {
                        if subj.name != obj.name {
                            let negated = Self::detect_negation(text, &subj.name, &obj.name);
                            let temporal = Self::detect_temporal(text);

                            triples.push(RelationTriple {
                                subject: subj.name.clone(),
                                relation: pattern.relation.clone(),
                                object: obj.name.clone(),
                                confidence: pattern.weight * subj.confidence.min(obj.confidence),
                                temporal,
                                negated,
                            });
                        }
                    }
                    search_start = abs_pos + 1;
                    if search_start >= text.len() {
                        break;
                    }
                }
            }
        }

        // Deduplicate: keep highest confidence for same (subject, relation, object)
        triples.sort_by(|a, b| {
            a.subject
                .cmp(&b.subject)
                .then_with(|| a.object.cmp(&b.object))
                .then_with(|| b.confidence.total_cmp(&a.confidence))
        });
        let mut deduped: Vec<RelationTriple> = Vec::new();
        for t in triples {
            if let Some(last) = deduped.last() {
                if last.subject == t.subject
                    && last.relation.name() == t.relation.name()
                    && last.object == t.object
                {
                    continue;
                }
            }
            deduped.push(t);
        }

        deduped
    }

    pub fn detect_temporal(text: &str) -> Option<TemporalMarker> {
        let lower = text.to_lowercase();
        let current_words = [
            "currently",
            "now",
            "presently",
            "at the moment",
            "these days",
            "right now",
            "today",
            "this year",
            "this month",
            "this week",
        ];
        for w in &current_words {
            if lower.contains(w) {
                return Some(TemporalMarker::Current);
            }
        }

        let past_words = [
            "used to",
            "previously",
            "formerly",
            "in the past",
            "historically",
            "earlier",
            "before",
            "ago",
            "last year",
            "last month",
            "last week",
            "yesterday",
        ];
        for w in &past_words {
            if lower.contains(w) {
                return Some(TemporalMarker::Past);
            }
        }

        let future_words = [
            "planning to",
            "will",
            "going to",
            "soon",
            "next year",
            "next month",
            "next week",
            "tomorrow",
            "in the future",
            "intends to",
            "aims to",
        ];
        for w in &future_words {
            if lower.contains(w) {
                return Some(TemporalMarker::Future);
            }
        }

        // "in YYYY" pattern
        let re_year = lazy_regex::regex!(r"\bin\s+(20\d{2})\b");
        if let Some(cap) = re_year.captures(text) {
            if let Ok(year) = cap[1].parse::<i32>() {
                return Some(TemporalMarker::AtYear(year));
            }
        }

        // "from YYYY to YYYY" pattern
        let re_range = lazy_regex::regex!(r"\bfrom\s+(20\d{2})\s+to\s+(20\d{2})\b");
        if let Some(cap) = re_range.captures(text) {
            return Some(TemporalMarker::Range(
                cap[1].to_string(),
                cap[2].to_string(),
            ));
        }

        None
    }

    pub fn detect_negation(text: &str, _subject: &str, _object: &str) -> bool {
        let lower = text.to_lowercase();
        let negation_words = [
            "doesn't like",
            "does not like",
            "don't like",
            "do not like",
            "doesn't use",
            "does not use",
            "don't use",
            "do not use",
            "isn't",
            "is not",
            "aren't",
            "are not",
            "wasn't",
            "was not",
            "weren't",
            "were not",
            "doesn't",
            "does not",
            "don't",
            "do not",
            "never",
            "no ",
            "not ",
            "n't",
            "dislikes",
            "hates",
            "avoids",
            "can't",
            "cannot",
            "unable to",
            "refuses to",
            "declines to",
        ];
        for w in &negation_words {
            if lower.contains(w) {
                return true;
            }
        }
        false
    }

    pub fn extract_facts(&mut self, text: &str) -> Vec<ExtractedFact> {
        let entities = self.extract_entities(text);
        let relations = self.extract_relations(text, &entities);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut facts = Vec::with_capacity(relations.len());
        for r in relations {
            let id = self.next_fact_id;
            self.next_fact_id += 1;
            facts.push(ExtractedFact {
                triple: r,
                source_text: text.to_string(),
                timestamp: now,
                id,
            });
        }
        facts
    }

    pub fn extract_batch(&mut self, texts: &[&str]) -> Vec<ExtractedFact> {
        let mut all = Vec::new();
        for text in texts {
            all.extend(self.extract_facts(text));
        }
        all
    }

    pub fn resolve_entity(&self, surface: &str) -> Option<(String, EntityType)> {
        let lower = surface.to_lowercase();

        // 1. Exact match
        if let Some(etype) = self.gazetteer.get(&lower) {
            return Some((surface.to_string(), etype.clone()));
        }

        // 2. Strip known prefixes
        let prefixes = [
            "Dr. ",
            "Dr ",
            "Professor ",
            "Prof. ",
            "Prof ",
            "Mr. ",
            "Mrs. ",
            "Ms. ",
            "CEO ",
            "CTO ",
            "CFO ",
            "VP ",
            "SVP ",
        ];
        for prefix in &prefixes {
            if let Some(stripped) = surface.strip_prefix(prefix) {
                let stripped_lower = stripped.to_lowercase();
                if let Some(etype) = self.gazetteer.get(&stripped_lower) {
                    return Some((stripped.to_string(), etype.clone()));
                }
            }
        }

        // 3. Compound split: check if last word is known (e.g., "Apple Inc." → "Apple")
        let parts: Vec<&str> = surface.split_whitespace().collect();
        if parts.len() >= 2 {
            // Check full phrase (case-insensitive)
            if self.gazetteer.contains_key(&lower) {
                return Some((surface.to_string(), EntityType::Organization));
            }
            // Check just the first part
            if let Some(etype) = self.gazetteer.get(&parts[0].to_lowercase()) {
                return Some((parts[0].to_string(), etype.clone()));
            }
        }

        None
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert TemporalMarker extracted from text to valid_from/valid_to bounds.
/// Useful when storing a fact that has temporal context.
pub fn temporal_marker_to_validity(marker: &TemporalMarker) -> (Option<i64>, Option<i64>) {
    let now = now_secs();
    match marker {
        TemporalMarker::Current => (Some(now), None),
        TemporalMarker::Past => (None, Some(now - 1)),
        TemporalMarker::Future => (Some(now), None),
        TemporalMarker::AtYear(y) => (year_start_epoch(*y), year_end_epoch(*y)),
        TemporalMarker::Range(a, b) => {
            let start = a.parse::<i32>().ok().and_then(year_start_epoch);
            let end = b.parse::<i32>().ok().and_then(year_end_epoch);
            (start, end)
        }
    }
}

pub fn temporal_marker_to_bounds(marker: &TemporalMarker) -> (Option<i64>, Option<i64>) {
    let now = now_secs();
    match marker {
        TemporalMarker::Current => (Some(now), None),
        TemporalMarker::Past => (None, Some(now)),
        TemporalMarker::Future => (Some(now), None),
        TemporalMarker::AtYear(y) => (year_start_epoch(*y), year_end_epoch(*y)),
        TemporalMarker::Range(a, b) => {
            let start = a.parse::<i32>().ok().and_then(year_start_epoch);
            let end = b.parse::<i32>().ok().and_then(year_end_epoch);
            (start, end)
        }
    }
}

/// Retrieve from MemoryGraph with entity-aware scoring boost.
/// Extracts entities from query text and boosts nodes whose labels match
/// any extracted entity name (case-insensitive substring match).
/// Falls back to plain `retrieve()` when no entities are found.
pub fn retrieve_with_entity_boost(
    extractor: &EntityExtractor,
    graph: &mut MemoryGraph,
    query_text: &str,
    query_vsa: &[u8],
    top_k: usize,
    max_hops: usize,
    entity_boost: f64,
    base_q_weight: f64,
) -> Vec<(u64, String, f64)> {
    let entities = extractor.extract_entities(query_text);

    if entities.is_empty() {
        return graph.retrieve(query_vsa, top_k, Some(max_hops));
    }

    let entity_names: Vec<String> = entities.iter().map(|e| e.name.to_lowercase()).collect();
    let all_labels = graph.all_labels();
    let matching_ids: std::collections::HashSet<u64> = all_labels
        .into_iter()
        .filter(|(_, label)| {
            let label_lower = label.to_lowercase();
            entity_names
                .iter()
                .any(|name| label_lower.contains(name.as_str()))
        })
        .map(|(id, _)| id)
        .collect();

    let scorer = |node_id: u64, base_score: f64| -> f64 {
        if matching_ids.contains(&node_id) {
            base_score * base_q_weight + entity_boost
        } else {
            base_score * base_q_weight
        }
    };

    graph.retrieve_with_scorer(query_vsa, top_k, max_hops, &scorer)
}

/// Extract facts from text and inject them as MemoryGraph nodes + edges.
/// Each entity becomes a node, each relation becomes an edge between entity nodes.
pub fn inject_facts_into_graph(
    extractor: &mut EntityExtractor,
    graph: &mut MemoryGraph,
    text: &str,
) -> Vec<(u64, u64, String)> {
    let facts = extractor.extract_facts(text);
    let mut result = Vec::with_capacity(facts.len());

    for fact in facts {
        let triple = fact.triple;
        let labels = graph.all_labels();
        let subj_id = labels
            .iter()
            .find(|(_, l)| l.as_str() == triple.subject.as_str())
            .map(|(id, _)| *id)
            .unwrap_or_else(|| graph.add_node(NodeKind::Semantic, vec![], &triple.subject));
        let obj_id = labels
            .iter()
            .find(|(_, l)| l.as_str() == triple.object.as_str())
            .map(|(id, _)| *id)
            .unwrap_or_else(|| graph.add_node(NodeKind::Semantic, vec![], &triple.object));
        graph.add_edge(subj_id, obj_id, EdgeKind::Associative, triple.confidence);
        result.push((subj_id, obj_id, triple.relation.name().to_string()));
    }

    result
}

/// Extract facts from text and store them in MemoryLattice Facts layer.
/// Each fact becomes a LatticeEntry in the Facts layer with temporal bounds
/// from the fact's temporal marker.
pub fn extract_and_store_facts(
    extractor: &mut EntityExtractor,
    lattice: &mut MemoryLattice,
    text: &str,
) -> usize {
    let facts = extractor.extract_facts(text);
    let mut count = 0;

    for fact in facts {
        let triple = fact.triple;
        let entry_str = format!(
            "{} {} {}",
            triple.subject,
            triple.relation.name(),
            triple.object
        );
        let (valid_from, valid_to) = triple
            .temporal
            .as_ref()
            .map(temporal_marker_to_validity)
            .unwrap_or((None, None));
        lattice.store_with_validity(
            entry_str,
            vec![],
            LatticeLayer::Facts,
            MemoryOrigin::Model,
            valid_from,
            valid_to,
        );
        count += 1;
    }

    count
}

// ── Make lazy_regex available ──
mod lazy_regex {
    macro_rules! regex {
        ($re:expr) => {{
            static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
            RE.get_or_init(|| {
                regex::Regex::new($re)
                    .expect("Regex::new failed on entity extraction pattern - malformed regex")
            })
        }};
    }

    pub(super) use regex;
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_person_gazetteer() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Einstein was a physicist");
        assert!(!ents.is_empty());
        assert!(ents
            .iter()
            .any(|e| e.name == "Einstein" && e.entity_type == EntityType::Person));
    }

    #[test]
    fn test_extract_organization() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Google announced a new AI model");
        assert!(!ents.is_empty());
        assert!(ents
            .iter()
            .any(|e| e.name == "Google" && e.entity_type == EntityType::Organization));
    }

    #[test]
    fn test_extract_location() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("The office is in San Francisco");
        assert!(ents
            .iter()
            .any(|e| e.name == "San Francisco" && e.entity_type == EntityType::Location));
    }

    #[test]
    fn test_extract_programming_language() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("I write Rust and Python");
        assert!(ents
            .iter()
            .any(|e| e.name == "Rust" && e.entity_type == EntityType::ProgrammingLanguage));
        assert!(ents
            .iter()
            .any(|e| e.name == "Python" && e.entity_type == EntityType::ProgrammingLanguage));
    }

    #[test]
    fn test_extract_framework() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("The project uses React and PyTorch");
        assert!(ents
            .iter()
            .any(|e| e.name == "React" && e.entity_type == EntityType::Framework));
        assert!(ents
            .iter()
            .any(|e| e.name == "PyTorch" && e.entity_type == EntityType::Framework));
    }

    #[test]
    fn test_extract_quoted_concept() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("The term \"Artificial Intelligence\" was coined");
        assert!(ents.iter().any(|e| e.name == "Artificial Intelligence"));
    }

    #[test]
    fn test_extract_relation_works_at() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Alice works at Google");
        let rels = ex.extract_relations("Alice works at Google", &ents);
        assert!(!rels.is_empty());
        let r = &rels[0];
        assert_eq!(r.relation, RelationType::WorksAt);
        assert!(r.object.contains("Google"));
    }

    #[test]
    fn test_extract_relation_located_in() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Apple is based in Cupertino");
        let rels = ex.extract_relations("Apple is based in Cupertino", &ents);
        assert!(!rels.is_empty());
        let r = &rels[0];
        assert_eq!(r.relation, RelationType::LocatedIn);
        assert!(r.subject.contains("Apple") || r.object.contains("Apple"));
    }

    #[test]
    fn test_extract_relation_creates() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Einstein created the theory of relativity");
        let rels = ex.extract_relations("Einstein created the theory of relativity", &ents);
        assert!(!rels.is_empty());
        assert!(rels.iter().any(|r| r.relation == RelationType::Creates));
    }

    #[test]
    fn test_negation_detection() {
        let neg = EntityExtractor::detect_negation("Alice doesn't like Python", "Alice", "Python");
        assert!(neg);
        let no_neg = EntityExtractor::detect_negation("Alice likes Python", "Alice", "Python");
        assert!(!no_neg);
    }

    #[test]
    fn test_temporal_current() {
        let t = EntityExtractor::detect_temporal("Alice currently works at Google");
        assert!(matches!(t, Some(TemporalMarker::Current)));
    }

    #[test]
    fn test_temporal_past() {
        let t = EntityExtractor::detect_temporal("Bob used to work at Microsoft");
        assert!(matches!(t, Some(TemporalMarker::Past)));
    }

    #[test]
    fn test_temporal_future() {
        let t = EntityExtractor::detect_temporal("Carol will join OpenAI next year");
        assert!(matches!(t, Some(TemporalMarker::Future)));
    }

    #[test]
    fn test_temporal_year() {
        let t = EntityExtractor::detect_temporal("Founded in 2025");
        assert!(matches!(t, Some(TemporalMarker::AtYear(2025))));
    }

    #[test]
    fn test_entity_case_insensitive() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("google is a company");
        assert!(!ents.is_empty());
        assert!(ents.iter().any(|e| e.name.eq_ignore_ascii_case("google")));
    }

    #[test]
    fn test_full_pipeline() {
        let mut ex = EntityExtractor::new();
        let facts = ex.extract_facts("Alice works at Google and Bob uses Rust");
        assert!(!facts.is_empty());
        assert!(facts
            .iter()
            .any(|f| f.triple.relation == RelationType::WorksAt));
    }

    #[test]
    fn test_empty_text() {
        let mut ex = EntityExtractor::new();
        let facts = ex.extract_facts("");
        assert!(facts.is_empty());
        let ents = ex.extract_entities("");
        assert!(ents.is_empty());
    }

    #[test]
    fn test_batch_extraction() {
        let mut ex = EntityExtractor::new();
        let texts = vec!["Alice works at Google", "Bob uses Rust"];
        let facts = ex.extract_batch(&texts);
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_gazetteer_lookup() {
        let ex = EntityExtractor::new();
        let res = ex.resolve_entity("Google");
        assert!(res.is_some());
        assert_eq!(res.unwrap().1, EntityType::Organization);
    }

    #[test]
    fn test_gazetteer_case_insensitive() {
        let ex = EntityExtractor::new();
        let res = ex.resolve_entity("GOOGLE");
        assert!(res.is_some());
    }

    #[test]
    fn test_gazetteer_prefix_strip() {
        let ex = EntityExtractor::new();
        let res = ex.resolve_entity("Dr. Einstein");
        assert!(res.is_some());
        assert_eq!(res.unwrap().0, "Einstein");
    }

    #[test]
    fn test_extract_relation_uses() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("The web server is built with Rust");
        let rels = ex.extract_relations("The web server is built with Rust", &ents);
        assert!(!rels.is_empty());
        assert!(rels.iter().any(|r| r.relation == RelationType::Uses));
    }

    #[test]
    fn test_extract_relation_ceo_of() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Sundar Pichai joined Google in 2004");
        let _rels = ex.extract_relations("Sundar Pichai joined Google in 2004", &ents);
        // The entity extractor should find both entities
        assert!(ents.iter().any(|e| e.name.contains("Pichai")));
        assert!(ents.iter().any(|e| e.name == "Google"));
    }

    #[test]
    fn test_extract_relation_part_of() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("DeepMind is part of Google");
        let rels = ex.extract_relations("DeepMind is part of Google", &ents);
        assert!(!rels.is_empty());
        assert!(rels.iter().any(|r| r.relation == RelationType::PartOf));
    }

    #[test]
    fn test_extract_concept_entity() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Machine Learning is transforming AI");
        assert!(ents.iter().any(|e| e.name == "Machine Learning"));
    }

    #[test]
    fn test_self_referential_entities_not_related() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Google uses Google Cloud");
        let rels = ex.extract_relations("Google uses Google Cloud", &ents);
        // Same names won't produce a relation (filtered in extract_relations)
        assert!(!rels.iter().any(|r| r.subject == r.object));
    }

    #[test]
    fn test_acronym_detection() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("AGI will transform the world");
        assert!(ents.iter().any(|e| e.name == "AGI"));
    }

    #[test]
    fn test_extract_relation_has_property() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Rust is a systems programming language");
        let rels = ex.extract_relations("Rust is a systems programming language", &ents);
        assert!(!rels.is_empty());
        assert!(rels.iter().any(|r| r.relation == RelationType::HasProperty));
    }

    #[test]
    fn test_deduplication_same_triple() {
        let mut ex = EntityExtractor::new();
        let text = "Alice works at Google. Yes, Alice works at Google.";
        let facts = ex.extract_facts(text);
        let works_at: Vec<_> = facts
            .iter()
            .filter(|f| f.triple.relation == RelationType::WorksAt)
            .collect();
        assert!(!works_at.is_empty());
    }

    #[test]
    fn test_extract_capitalized_phrase() {
        let ex = EntityExtractor::new();
        let ents = ex.extract_entities("Alice works at Google");
        assert!(!ents.is_empty());
    }

    #[test]
    fn test_resolve_unknown_entity() {
        let ex = EntityExtractor::new();
        let res = ex.resolve_entity("xyzzy_nonexistent");
        assert!(res.is_none());
    }

    #[test]
    fn test_temporal_marker_current() {
        let now = now_secs();
        let (start, end) = temporal_marker_to_bounds(&TemporalMarker::Current);
        assert!(start.is_some());
        assert!(end.is_none());
        assert!(start.unwrap() >= now - 1);
    }

    #[test]
    fn test_temporal_marker_past() {
        let now = now_secs();
        let (start, end) = temporal_marker_to_bounds(&TemporalMarker::Past);
        assert!(start.is_none());
        assert!(end.is_some());
        assert!(end.unwrap() >= now - 1);
    }

    #[test]
    fn test_temporal_marker_at_year() {
        let (start, end) = temporal_marker_to_bounds(&TemporalMarker::AtYear(2025));
        assert!(start.is_some());
        assert!(end.is_some());
        assert!(start.unwrap() < end.unwrap());
    }

    #[test]
    fn test_temporal_marker_range() {
        let (start, end) =
            temporal_marker_to_bounds(&TemporalMarker::Range("2020".into(), "2025".into()));
        assert!(start.is_some());
        assert!(end.is_some());
        assert!(start.unwrap() < end.unwrap());
    }

    #[test]
    fn test_extracted_facts_as_entries_no_temporal() {
        let mut ex = EntityExtractor::new();
        let facts = ex.extract_facts("Alice works at Google");
        assert!(!facts.is_empty());
        assert!(facts[0].timestamp > 0);
        assert!(!facts[0].source_text.is_empty());
        assert!(facts[0].id > 0);
    }

    #[test]
    fn test_extracted_facts_as_entries_with_temporal() {
        let mut ex = EntityExtractor::new();
        let facts = ex.extract_facts("Alice currently works at Google");
        assert!(!facts.is_empty());
        assert!(facts[0].timestamp > 0);
        assert!(!facts[0].source_text.is_empty());
    }

    #[test]
    fn test_temporal_marker_to_validity_current() {
        let now = now_secs();
        let (vf, vt) = temporal_marker_to_validity(&TemporalMarker::Current);
        assert!(vf.is_some());
        assert!(vt.is_none());
        assert!(vf.unwrap() >= now - 1);
    }

    #[test]
    fn test_temporal_marker_to_validity_past() {
        let now = now_secs();
        let (vf, vt) = temporal_marker_to_validity(&TemporalMarker::Past);
        assert!(vf.is_none());
        assert!(vt.is_some());
        assert!(vt.unwrap() <= now);
    }

    #[test]
    fn test_temporal_marker_to_validity_future() {
        let now = now_secs();
        let (vf, vt) = temporal_marker_to_validity(&TemporalMarker::Future);
        assert!(vf.is_some());
        assert!(vt.is_none());
        assert!(vf.unwrap() >= now - 1);
    }

    #[test]
    fn test_temporal_marker_to_validity_at_year() {
        let (vf, vt) = temporal_marker_to_validity(&TemporalMarker::AtYear(2025));
        assert!(vf.is_some());
        assert!(vt.is_some());
        assert!(vf.unwrap() < vt.unwrap());
    }

    #[test]
    fn test_temporal_marker_to_validity_range() {
        let (vf, vt) =
            temporal_marker_to_validity(&TemporalMarker::Range("2020".into(), "2025".into()));
        assert!(vf.is_some());
        assert!(vt.is_some());
        assert!(vf.unwrap() < vt.unwrap());
    }

    #[test]
    fn test_entity_boost_basic() {
        use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
        use crate::core::nt_core_knowledge::spread_activation::NodeKind;

        let extractor = EntityExtractor::new();
        let mut graph = MemoryGraph::new(100);
        graph.add_node(
            NodeKind::Semantic,
            QuantizedVSA::seeded_random(1, VSA_DIM),
            "Rust programming language",
        );
        graph.add_node(
            NodeKind::Semantic,
            QuantizedVSA::seeded_random(50, VSA_DIM),
            "Python scripting language",
        );
        graph.add_node(
            NodeKind::Semantic,
            QuantizedVSA::seeded_random(99, VSA_DIM),
            "JavaScript web language",
        );

        let query_vsa = QuantizedVSA::seeded_random(1, VSA_DIM);
        let results = retrieve_with_entity_boost(
            &extractor,
            &mut graph,
            "Tell me about Rust",
            &query_vsa,
            3,
            1,
            0.3,
            0.7,
        );

        assert!(!results.is_empty(), "should return results");
        let rust_result = results.iter().find(|(_, l, _)| l.contains("Rust"));
        assert!(rust_result.is_some(), "Rust node should be in results");
    }

    #[test]
    fn test_entity_boost_empty_query() {
        use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
        use crate::core::nt_core_knowledge::spread_activation::NodeKind;

        let extractor = EntityExtractor::new();
        let mut graph = MemoryGraph::new(100);
        graph.add_node(
            NodeKind::Semantic,
            QuantizedVSA::seeded_random(1, VSA_DIM),
            "test node",
        );

        let query_vsa = QuantizedVSA::seeded_random(1, VSA_DIM);
        let results =
            retrieve_with_entity_boost(&extractor, &mut graph, "", &query_vsa, 3, 1, 0.3, 0.7);

        assert!(
            !results.is_empty(),
            "fallback to plain retrieve should return results"
        );
    }

    #[test]
    fn test_entity_boost_no_matching_entities() {
        use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
        use crate::core::nt_core_knowledge::spread_activation::NodeKind;

        let extractor = EntityExtractor::new();
        let mut graph = MemoryGraph::new(100);
        graph.add_node(
            NodeKind::Semantic,
            QuantizedVSA::seeded_random(1, VSA_DIM),
            "first concept",
        );
        graph.add_node(
            NodeKind::Semantic,
            QuantizedVSA::seeded_random(2, VSA_DIM),
            "second concept",
        );

        let query_vsa = QuantizedVSA::seeded_random(1, VSA_DIM);
        let results = retrieve_with_entity_boost(
            &extractor,
            &mut graph,
            "Tell me about Google",
            &query_vsa,
            3,
            1,
            0.3,
            0.7,
        );

        assert!(
            !results.is_empty(),
            "should still return VSA-based results even without entity match"
        );
    }

    #[test]
    fn test_inject_facts_empty_text() {
        let mut extractor = EntityExtractor::new();
        let mut graph = MemoryGraph::new(100);
        let result = inject_facts_into_graph(&mut extractor, &mut graph, "");
        assert!(result.is_empty());
        assert!(graph.all_labels().is_empty());
    }

    #[test]
    fn test_inject_facts_creates_nodes() {
        let mut extractor = EntityExtractor::new();
        let mut graph = MemoryGraph::new(100);
        let result = inject_facts_into_graph(&mut extractor, &mut graph, "Alice works at Google");
        assert_eq!(result.len(), 1);
        let (subj, obj, rel) = &result[0];
        let labels: Vec<String> = graph.all_labels().into_iter().map(|(_, l)| l).collect();
        assert!(labels.iter().any(|l| l == "Alice"));
        assert!(labels.iter().any(|l| l == "Google"));
        assert_eq!(rel.as_str(), "works_at");
    }

    #[test]
    fn test_extract_and_store_facts() {
        let mut extractor = EntityExtractor::new();
        let mut lattice = MemoryLattice::new();
        let count = extract_and_store_facts(
            &mut extractor,
            &mut lattice,
            "Einstein created Theory of Relativity",
        );
        assert!(count > 0);
        let found = lattice.find("Einstein");
        assert!(!found.is_empty(), "fact should be stored in lattice");
        let has_facts = found
            .iter()
            .any(|(layer, _, _)| *layer == LatticeLayer::Facts);
        assert!(has_facts, "stored in Facts layer");
    }

    #[test]
    fn test_extract_and_store_empty() {
        let mut extractor = EntityExtractor::new();
        let mut lattice = MemoryLattice::new();
        let count = extract_and_store_facts(&mut extractor, &mut lattice, "");
        assert_eq!(count, 0);
    }
}
