/// 外部资源种子 — 从 X 帖子提炼的自托管/开源工具库
/// 使用 KBIngester 可复用模块
use std::time::{Duration, Instant};
use neotrix::neotrix::nt_memory_kb::{nt_memory_ingest::KBIngester, NodeType, RelationType};

const GITHUB_REPOS: &[(&str, &str, &str, &str)] = &[
    ("aleksilassila", "reiverr", "media_server",
     "All-in-one UI for Jellyfin, Radarr, Sonarr, TMDB."),
    ("Suwayomi", "Suwayomi-Server", "book_library",
     "Self-hosted manga reader server. Drop-in for Tachiyomi."),
    ("timvisee", "send", "file_sharing",
     "Temporary encrypted file sharing. Firefox Send fork."),
    ("uazo", "cromite", "nt_world_browse",
     "Privacy-focused Chromium with ad blocking, fingerprint protect."),
    ("zotify-dev", "zotify", "media_downloader",
     "Spotify music and podcast downloader. CLI-based."),
    ("lllyasviel", "Fooocus", "ai_ml",
     "Local AI image generation powered by Stable Diffusion."),
    ("immich-app", "immich", "media_server",
     "Self-hosted Google Photos alternative. Auto backup, AI search."),
];

const WEB_RESOURCES: &[(&str, &str, &str, &str)] = &[
    ("Anna's Archive", "book_library",
     "https://annas-archive.gl/",
     "Shadow library aggregator searching LibGen, Sci-Hub, Z-Lib."),
    ("FMHY", "resource_directory",
     "https://fmhy.net/",
     "Free Media Heck Yeah: curated directory of free resources."),
    ("Bootleg Archive", "media_archive",
     "https://bootleg.fm/",
     "Live concert bootleg recording archive."),
];

const CATEGORIES: &[(&str, &str)] = &[
    ("media_server", "Self-hosted media services"),
    ("book_library", "Book/manga reading resources"),
    ("file_sharing", "Temporary file sharing tools"),
    ("nt_world_browse", "Privacy-focused nt_world_browses"),
    ("media_downloader", "Media downloading tools"),
    ("ai_ml", "AI and machine learning tools"),
    ("resource_directory", "Curated free resource indexes"),
    ("media_archive", "Archival media collections"),
];

const EDGE_PAIRS: &[(&str, &str, RelationType, f64, &str)] = &[
    ("reiverr", "immich", RelationType::Related, 0.7, "Both self-hosted media platforms"),
    ("Suwayomi-Server", "Tachiyomi", RelationType::ExtensionOf, 1.0, "Suwayomi extends Tachiyomi"),
    ("send", "File sharing", RelationType::SubclassOf, 0.8, "Encrypted file sharing"),
    ("cromite", "Privacy-focused web nt_world_browse", RelationType::SubclassOf, 0.9, "Privacy Chromium fork"),
    ("Fooocus", "AI image generation", RelationType::SubclassOf, 0.9, "Local SD image generation"),
    ("immich", "Self-hosted Google Photos alternative", RelationType::SubclassOf, 0.9, "Photo management server"),
    ("Anna's Archive", "Shadow library", RelationType::SubclassOf, 1.0, "Aggregates LibGen/Sci-Hub/Z-Lib"),
    ("FMHY", "Resource directory", RelationType::SubclassOf, 1.0, "Curated free resource index"),
    ("Bootleg Archive", "Live music recording", RelationType::SubclassOf, 1.0, "Concert bootleg archive"),
    ("zotify", "Media downloader", RelationType::SubclassOf, 0.9, "Spotify music downloader"),
];

fn main() {
    let mut ing = KBIngester::open(None).expect("KBIngester open");
    let overall = Instant::now();
    let before = ing.snapshot();

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 外部资源种子 — 自托管/开源工具库         ║");
    println!("╚══════════════════════════════════════════════════════╝");

    // Phase 1: GitHub repos
    println!("\n━━━ 1/3 GitHub 仓库 ({} repos) ━━━", GITHUB_REPOS.len());
    let mut repo_ok = 0u32;
    for (i, &(owner, repo, _cat, _desc)) in GITHUB_REPOS.iter().enumerate() {
        let t = Instant::now();
        let n = ing.repo(owner, repo);
        if n > 0 {
            repo_ok += 1;
            println!("  [{:>2}/{}] {}/{} +{} ({:.1}s)", i + 1, GITHUB_REPOS.len(), owner, repo, n, t.elapsed().as_secs_f64());
        } else {
            println!("  [{:>2}/{}] {}/{} ✗ ({:.1}s)", i + 1, GITHUB_REPOS.len(), owner, repo, t.elapsed().as_secs_f64());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    println!("  GitHub: {} OK", repo_ok);
    ing.log(format!("GitHub repos: {}/{} OK", repo_ok, GITHUB_REPOS.len()));

    // Phase 2: Web resource nodes + categories
    println!("\n━━━ 2/3 Web 资源 + 分类 ({}+{} items) ━━━", WEB_RESOURCES.len(), CATEGORIES.len());
    for (title, _cat, url, summary) in WEB_RESOURCES {
        ing.try_node(title, NodeType::Article, summary, Some(url), "external_resource");
    }
    for (cat, desc) in CATEGORIES {
        ing.try_concept(cat, desc, "resource_category");
    }

    // Phase 3: Edges
    println!("\n━━━ 3/3 边连接 ━━━");
    let mut edge_ok = 0u32;

    // repo → category
    for &(owner, repo, cat, _desc) in GITHUB_REPOS {
        let repo_title = format!("{}/{}", owner, repo);
        if ing.relate(&repo_title, cat, RelationType::SubclassOf, 1.0, "Resource category") { edge_ok += 1; }
        // also try short name
        if ing.relate(repo, cat, RelationType::SubclassOf, 1.0, "Resource category") { edge_ok += 1; }
    }
    // web → category
    for (title, cat, _url, _summary) in WEB_RESOURCES {
        if ing.relate(title, cat, RelationType::SubclassOf, 1.0, "Resource category") { edge_ok += 1; }
    }
    // hand-crafted pairs
    edge_ok += ing.relate_many(&EDGE_PAIRS);

    println!("  边创建: {}", edge_ok);
    ing.log(format!("Edges created: {}", edge_ok));

    let dedup_count = ing.dedup();
    ing.log(format!("Duplicates merged: {}", dedup_count));

    ing.report("外部资源种子: 最终 KB 状态", &before, overall.elapsed());
}
