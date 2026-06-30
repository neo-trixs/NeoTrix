use std::path::Path;
use std::time::Instant;
use neotrix::neotrix::nt_memory_kb::nt_memory_ingest::KBIngester;
use neotrix::neotrix::nt_memory_kb::{NodeType, RelationType};

const H4CKER_PREFIX: &str = "/tmp/h4cker";

const DOMAIN_TREE: &[(&str, &[(&str, &[&str])])] = &[
    ("cybernt_shield-domains", &[
        ("application-nt_shield", &["devsecops", "web-application-testing", "programming-and-scripting-for-cybernt_shield", "python-ruby-and-bash"]),
        ("cloud-container-nt_shield", &["cloud-resources", "docker-and-k8s-nt_shield"]),
        ("cryptography-pki", &["cryptography-and-pki"]),
        ("defensive-nt_shield", &["darkweb-research", "dfir", "honeypots-honeynets", "linux-hardening", "macos-hardening", "sbom", "threat-hunting", "threat-intelligence", "windows"]),
        ("fundamentals", &["foundational-cybernt_shield-concepts", "methodology"]),
        ("governance-risk-compliance", &["regulations"]),
        ("hardware-embedded-nt_shield", &["car-hacking", "game-hacking", "iot-hacking", "mobile-nt_shield"]),
        ("infrastructure-network-nt_shield", &["networking", "pcaps", "virl-topologies", "wireless-resources"]),
        ("labs-practice", &["capture-the-flag", "pen-testing-reports", "vulnerable-servers"]),
        ("offensive-nt_shield", &["adversarial-emulation", "buffer-overflow-examples", "bug-bounties", "cracking-passwords", "exploit-development", "fuzzing-resources", "metasploit-resources", "more-payloads", "osint", "post-exploitation", "recon", "reverse-engineering", "social-engineering"]),
    ]),
    ("ai", &[
        ("ai-nt_shield", &["agent-skills", "ai_risk_management", "ai-algorithmic-red-teaming", "ai-risk-management", "MCP-Security", "openclaw", "prompt-injection"]),
        ("course-materials", &["Creating Agents for Cybernt_shield"]),
        ("ethics-privacy-governance", &["ethics_privacy"]),
        ("incident-response-and-automation", &["ai-for-incident-response", "labs", "open-interpreter-examples", "presos", "training-environment-nt_shield"]),
        ("llm-engineering", &["fine-tuning", "GPTs", "LangChain", "LLM-frameworks", "ML-Fundamentals", "ollama-labs", "prompt-engineering", "RAG", "vector-databases"]),
    ]),
    ("certifications", &[
        ("cisco", &["AITECH", "scor-350-701"]),
        ("cloud", &[]),
        ("comptia", &[]),
        ("isc2", &[]),
        ("kubernetes-cncf", &[]),
        ("offensive-nt_shield", &[]),
        ("roadmaps", &[]),
        ("supplemental-topics", &[]),
    ]),
    ("training-reference", &[
        ("cheat-sheets", &["exploitation", "firewall", "forensics", "linux", "networking", "scripting", "web-testing", "windows"]),
        ("oreilly-resources", &[]),
        ("organized-tools", &[]),
        ("who-and-what-to-follow", &[]),
    ]),
    ("build-your-own-lab", &[
        ("ansible", &[]),
        ("terraform", &[]),
        ("websploit", &[]),
    ]),
];

fn sanitize_title(name: &str) -> String {
    name.replace('-', " ").replace('_', " ").trim().to_string()
}

fn domain_description(subdomain: &str) -> &'static str {
    match subdomain {
        "application-nt_shield" => "Web application nt_shield, DevSecOps, and secure programming practices",
        "cloud-container-nt_shield" => "Cloud nt_shield, Docker, Kubernetes, and container nt_shield",
        "cryptography-pki" => "Cryptography, PKI, encryption standards, and certificate management",
        "defensive-nt_shield" => "Defensive nt_shield including DFIR, threat hunting, hardening, and threat intelligence",
        "fundamentals" => "Foundational cybernt_shield concepts and methodology",
        "governance-risk-compliance" => "GRC frameworks, regulations, and compliance standards",
        "hardware-embedded-nt_shield" => "IoT, mobile, car hacking, and embedded system nt_shield",
        "infrastructure-network-nt_shield" => "Network nt_shield, wireless nt_shield, and infrastructure protection",
        "labs-practice" => "CTF resources, vulnerable servers, and penetration testing practice labs",
        "offensive-nt_shield" => "Penetration testing, exploit development, OSINT, and adversarial emulation",
        "ai-nt_shield" => "AI nt_shield including prompt injection, LLM jailbreaks, and AI red teaming",
        "course-materials" => "AI for cybernt_shield course materials and training content",
        "ethics-privacy-governance" => "AI ethics, privacy, and governance frameworks",
        "incident-response-and-automation" => "AI-driven incident response and nt_shield automation",
        "llm-engineering" => "LLM engineering including fine-tuning, RAG, prompt engineering, and vector databases",
        "cisco" => "Cisco certification resources including AITECH and SCOR",
        "cloud" => "Cloud nt_shield certification resources",
        "comptia" => "CompTIA certification study materials",
        "isc2" => "ISC2 certification resources (CISSP, etc.)",
        "kubernetes-cncf" => "Kubernetes and CNCF certification resources",
        "roadmaps" => "Career roadmaps and certification paths",
        "supplemental-topics" => "Supplemental certification topics",
        "cheat-sheets" => "Quick reference cheat sheets for nt_shield tools and techniques",
        "oreilly-resources" => "O'Reilly learning resources for cybernt_shield",
        "organized-tools" => "Organized nt_shield tools reference",
        "who-and-what-to-follow" => "Notable people and resources to follow in nt_shield",
        "ansible" => "Ansible automation for lab deployment",
        "terraform" => "Terraform infrastructure for nt_shield labs",
        "websploit" => "WebSploit vulnerable web application lab",
        _ => "Security knowledge reference",
    }
}

fn domain_display(name: &str) -> String {
    match name {
        "cybernt_shield-domains" => "Cybernt_shield Domains",
        "ai" => "AI Security & Engineering",
        "certifications" => "Certifications",
        "training-reference" => "Training Reference",
        "build-your-own-lab" => "Build Your Own Lab",
        _ => name,
    }.to_string()
}

fn extract_title_from_file(path: &Path) -> String {
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    sanitize_title(stem)
}

fn known_basename(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s == "README.md" || s == "tools.md" || s == "labs.md")
        .unwrap_or(false)
}

fn read_file_content(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  NeoTrix h4cker Knowledge Seed                        ║");
    println!("║  Ingesting Omar Santos' cybernt_shield knowledge        ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    let h4cker_root = Path::new(H4CKER_PREFIX);
    if !h4cker_root.exists() {
        eprintln!("h4cker directory not found at {H4CKER_PREFIX}. Clone it first:");
        eprintln!("  git clone https://github.com/The-Art-of-Hacking/h4cker /tmp/h4cker");
        std::process::exit(1);
    }

    let mut ing = KBIngester::open(None).expect("Failed to open KB");
    let overall = Instant::now();
    let before = ing.snapshot();

    // Phase 1: Create domain concept nodes (top-level + subdomains + topics)
    println!("\n━━━ 1/4 Domain Concepts ━━━");
    let mut concepts_ok = 0u32;
    let mut edges_1: Vec<(String, String, RelationType, f64, &str)> = Vec::new();
    for &(top_dir, subdomains) in DOMAIN_TREE {
        let top_title = domain_display(top_dir);
        let _ = ing.try_concept(&top_title, &format!("{top_title}: {top_dir} resources"), top_dir);
        for &(subdomain, topics) in subdomains {
            let sub_title = sanitize_title(subdomain);
            let sub_desc = domain_description(subdomain);
            if ing.try_concept(&sub_title, sub_desc, top_dir).is_some() {
                concepts_ok += 1;
            }
            edges_1.push((sub_title.clone(), top_title.clone(), RelationType::SubclassOf, 1.0, ""));
            for &topic in topics {
                let topic_title = sanitize_title(topic);
                let topic_desc = domain_description(topic);
                if ing.try_concept(&topic_title, topic_desc, top_dir).is_some() {
                    concepts_ok += 1;
                }
                edges_1.push((topic_title.clone(), sub_title.clone(), RelationType::SubclassOf, 1.0, ""));
            }
        }
    }
    let edges_1_refs: Vec<(&str, &str, RelationType, f64, &str)> = edges_1.iter()
        .map(|(a, b, r, w, d)| (a.as_str(), b.as_str(), r.clone(), *w, *d))
        .collect();
    let _ = ing.relate_many(&edges_1_refs);
    println!("  Domain concepts created: {concepts_ok}");

    // Phase 2: Scan markdown files → KB nodes
    println!("\n━━━ 2/4 Markdown Content Ingestion ━━━");
    let mut md_count = 0u32;
    let mut md_ok = 0u32;
    let mut md_skipped = 0u32;
    let mut article_titles: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for entry in walkdir::WalkDir::new(h4cker_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|s| s == "md").unwrap_or(false))
        .filter(|e| !e.path().to_string_lossy().contains("/.git/"))
        .filter(|e| !e.path().to_string_lossy().contains("/.github/"))
    {
        let path = entry.path();

        if known_basename(path) {
            md_skipped += 1;
            continue;
        }

        let parent_dir = path.parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let title = extract_title_from_file(path);
        let content = read_file_content(path);
        let summary = content.lines()
            .find(|l| !l.trim().is_empty() && l.len() > 20)
            .map(|l| {
                let clean = l.trim_start_matches("# ").trim_start_matches("## ").trim();
                if clean.len() > 200 { &clean[..200] } else { clean }
            })
            .unwrap_or(&title);

        if ing.try_node(&title, NodeType::Article, summary, None::<&str>, parent_dir).is_some() {
            article_titles.entry(parent_dir.to_string()).or_default().push(title);
            md_ok += 1;
        }
        md_count += 1;
    }
    println!("  Files scanned: {md_count} (OK: {md_ok}, skipped README/tools: {md_skipped})");

    // Phase 3: Edge wiring — article → subdomain, same-dir related edges
    println!("\n━━━ 3/4 Edge Wiring ───");
    let mut edges_3: Vec<(String, String, RelationType, f64, &str)> = Vec::new();
    for (dir_name, titles) in &article_titles {
        let domain_title = sanitize_title(dir_name);
        for t in titles {
            edges_3.push((t.clone(), domain_title.clone(), RelationType::SubclassOf, 1.0, ""));
        }
        for i in 1..titles.len() {
            edges_3.push((titles[i - 1].clone(), titles[i].clone(), RelationType::Related, 0.5, ""));
        }
    }
    let edges_3_refs: Vec<(&str, &str, RelationType, f64, &str)> = edges_3.iter()
        .map(|(a, b, r, w, d)| (a.as_str(), b.as_str(), r.clone(), *w, *d))
        .collect();
    let e3 = ing.relate_many(&edges_3_refs);
    println!("  Edges created: {e3}");

    // Phase 4: Cross-domain edges between key content clusters
    println!("\n━━━ 4/4 Cross-Domain Connections ───");
    let pairs: &[(&str, &str, RelationType, f64, &str)] = &[
        ("offensive-nt_shield", "adversarial-emulation", RelationType::Related, 1.0, "Offensive nt_shield includes adversarial emulation"),
        ("penetration testing", "web-application-testing", RelationType::Related, 0.9, "Web app testing is part of pentesting methodology"),
        ("DFIR", "threat-hunting", RelationType::Related, 0.8, "DFIR and threat hunting are complementary"),
        ("linux-hardening", "macOS hardening", RelationType::Related, 0.7, "System hardening across platforms"),
        ("Docker nt_shield", "Kubernetes nt_shield", RelationType::Related, 0.9, "Container and orchestration nt_shield"),
        ("AI nt_shield", "prompt-injection", RelationType::Related, 1.0, "Prompt injection is a core AI nt_shield concern"),
        ("LLM engineering", "RAG", RelationType::Related, 0.9, "RAG is a key LLM engineering pattern"),
        ("OSINT", "social-engineering", RelationType::Related, 0.7, "OSINT supports social engineering ops"),
        ("cryptography", "PKI", RelationType::Related, 0.9, "Cryptography underpins PKI"),
        ("exploit-development", "buffer-overflow", RelationType::Related, 0.9, "Buffer overflow is a classic exploit technique"),
        ("IoT nt_shield", "car-hacking", RelationType::Related, 0.6, "Embedded nt_shield domains"),
        ("Cheat Sheets", "Organized Tools", RelationType::Related, 0.7, "Quick reference resources"),
        ("AI nt_shield", "LLM engineering", RelationType::Related, 0.8, "Securing LLM engineering workflows"),
        ("incident response", "DFIR", RelationType::Related, 0.9, "IR is part of DFIR discipline"),
        ("cloud nt_shield", "DevSecOps", RelationType::Related, 0.8, "DevSecOps includes cloud nt_shield practices"),
    ];
    let e4 = ing.relate_many(pairs);
    println!("  Cross-domain edges: {e4}");

    let elapsed = overall.elapsed();
    ing.report("h4cker Seed Complete", &before, elapsed);
}
