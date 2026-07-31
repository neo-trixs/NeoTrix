// ── absorb-crawl: drain crawl_queue into KB ──
use std::path::PathBuf;
use clap::Parser;
use neotrix::neotrix::l2_world_impl::nt_memory_kb_bridge::KnowledgeBase;
use neotrix::neotrix::l2_world_impl::nt_world_absorber::{UnifiedAbsorber, AbsorbSource, AbsorberConfig};
use neotrix::neotrix::nt_memory_kb::nt_memory_store::{claim_next_crawl_url, mark_crawl_complete};

#[derive(Parser)]
#[command(name = "absorb-crawl")]
struct Args {
    #[arg(long)]
    db: Option<PathBuf>,
    #[arg(long, default_value = "10")]
    max: usize,
}

fn main() {
    let args = Args::parse();
    let db_path = args.db.unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".neotrix").join("knowledge.db")
    });
    let conn = rusqlite::Connection::open(&db_path).expect("open KB");
    let kb = KnowledgeBase::open(Some(db_path)).expect("open KB bridge");
    let absorber = UnifiedAbsorber::new(kb, AbsorberConfig::default()).expect("create absorber");
    let mut done = 0usize;
    loop {
        if done >= args.max { break; }
        let item = match claim_next_crawl_url(&conn) {
            Ok(Some(i)) => i,
            _ => { eprintln!("[absorb] no pending"); break; }
        };
        eprintln!("[absorb] ({}/{}) {}", done + 1, args.max, item.url);
        match absorber.absorb(&AbsorbSource::from_url(&item.url).unwrap_or(AbsorbSource::WebPage(item.url.clone()))) {
            Ok(r) => { eprintln!("  ok: nodes={} edges={}", r.nodes_created, r.edges_created); mark_crawl_complete(&conn, &item.id, true, None).ok(); done += 1; }
            Err(e) => { eprintln!("  fail: {e}"); mark_crawl_complete(&conn, &item.id, false, Some(&e)).ok(); }
        }
    }
    eprintln!("[absorb] done: {done} processed");
}
