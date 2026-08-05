use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_shield_comm::{self, PERSONAS};

const DB_NAME: &str = "comm_router.db";

fn db_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|d| d.join(".neotrix").join(DB_NAME))
}

fn open_db() -> Option<rusqlite::Connection> {
    let path = db_path()?;
    std::fs::create_dir_all(path.parent()?).ok()?;
    let conn = rusqlite::Connection::open(&path).ok()?;
    conn.execute_batch(nt_shield_comm::COMM_DB_TABLES).ok()?;
    Some(conn)
}

pub struct CommCmd;

impl CommCmd {
    fn list_personas() -> CommandOutput {
        let mut out = String::from("persona 目录 (key | label | 权重 | 区域):\n");
        for p in PERSONAS.iter() {
            out.push_str(&format!(
                "  {:<16} {:<24} weight={:<4} regions={}\n",
                p.key, p.label, p.weight, p.geo_regions.join(",")
            ));
        }
        CommandOutput::ok(&out)
    }

    fn pool_stats() -> CommandOutput {
        let conn = match open_db() {
            Some(c) => c,
            None => return CommandOutput::err("无法打开 ~/.neotrix/comm_router.db"),
        };
        match nt_shield_comm::pool_stats(&conn) {
            Ok(json) => CommandOutput::ok(&serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())),
            Err(e) => CommandOutput::err(&format!("pool_stats 失败: {}", e)),
        }
    }

    fn fetch(args: &[String]) -> CommandOutput {
        let url = args.get(0).map(|s| s.as_str()).unwrap_or("");
        if url.is_empty() {
            return CommandOutput::err("Usage: /comm fetch <url> [persona_key]");
        }
        let persona_key = args.get(1).map(|s| s.as_str()).unwrap_or("");
        let conn = match open_db() {
            Some(c) => c,
            None => return CommandOutput::err("无法打开 ~/.neotrix/comm_router.db"),
        };
        let client = crate::neotrix::nt_io_http_factory::build_async_client();
        let timeout = std::time::Duration::from_secs(15);
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => return CommandOutput::err(&format!("Runtime error: {}", e)),
        };
        let result = rt.block_on(nt_shield_comm::fetch(&conn, &client, url, persona_key, &[], timeout));
        let body_preview = result.body.as_deref().unwrap_or("").chars().take(300).collect::<String>();
        let mut out = format!(
            "status={} latency={:.0}ms persona={} identity={}\n",
            result.status, result.latency_ms, result.persona_used, result.identity_id
        );
        if !result.error.is_empty() {
            out.push_str(&format!("error={}\n", result.error));
        }
        out.push_str(&format!("body[0..300]={}\n", body_preview));
        CommandOutput::ok(&out)
    }
}

impl CliCommand for CommCmd {
    fn name(&self) -> &str { "/comm" }
    fn aliases(&self) -> Vec<&str> { vec!["/persona", "/mask"] }
    fn description(&self) -> &str {
        "NT-SHIELD 通信伪装层观测: personas|pool|fetch <url> [persona_key]"
    }
    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("help");
        match subcmd {
            "personas" => Self::list_personas(),
            "pool" => Self::pool_stats(),
            "fetch" => Self::fetch(&args[1..]),
            _ => CommandOutput::ok("Usage: /comm personas|pool|fetch <url> [persona_key]"),
        }
    }
}
