use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::{
    BridgeHealth, ConsciousnessAbility, CuriositySignal, Domain, GraceMode, IntentionVsa, VsaLight,
    VsaOrigin, VsaTagged, WorldEffect,
};

fn control_socket_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.neotrix/neotrix-proxy.sock", home)
}
const DAEMON_BIN_NAME: &str = "neotrix-proxy-daemon";
const FINGERPRINT_PROFILES: &[&str] = &[
    "ChromeMac",
    "FirefoxMac",
    "SafariMac",
    "ChromeLinux",
    "EdgeWin",
];
const PROXY_PORT: u16 = 11080;

pub struct NetworkBridge {
    pub vsa: VsaLight,
    pub proxy_running: bool,
    pub current_fingerprint: String,
    pub fingerprints_tested: Vec<String>,
    pub connection_successes: u64,
    pub connection_failures: u64,
    pub total_actuations: u64,
    pub last_socket_check_ms: i64,
    daemon_path: Option<PathBuf>,
    last_quality: f64,
}

impl NetworkBridge {
    pub fn new() -> Self {
        Self {
            vsa: VsaLight::new(4096),
            proxy_running: false,
            current_fingerprint: "ChromeMac".into(),
            fingerprints_tested: vec!["ChromeMac".into()],
            connection_successes: 0,
            connection_failures: 0,
            total_actuations: 0,
            last_socket_check_ms: 0,
            daemon_path: None,
            last_quality: 0.5,
        }
    }

    pub fn with_daemon_path(mut self, path: PathBuf) -> Self {
        self.daemon_path = Some(path);
        self
    }

    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    fn seed_for(&self, tag: &str, extra: u64) -> u64 {
        let mut s: u64 = 0xe8e8_2024_0608_0000u64.wrapping_add(extra);
        for b in tag.bytes() {
            s = s.wrapping_mul(31).wrapping_add(b as u64);
        }
        s
    }

    fn send_control_command(&self, cmd: &str) -> Result<String, String> {
        let mut stream =
            UnixStream::connect(control_socket_path())
                .map_err(|e| format!("socket connect: {}", e))?;
        stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
        stream
            .write_all(format!("{}\n", cmd).as_bytes())
            .map_err(|e| format!("socket write: {}", e))?;
        let mut reader = BufReader::new(&mut stream);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|e| format!("socket read: {}", e))?;
        Ok(response.trim().to_string())
    }

    fn proxy_process_alive(&self) -> bool {
        Command::new("ps")
            .args(["-ax", "-o", "comm="])
            .output()
            .ok()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .any(|l| l.contains(DAEMON_BIN_NAME))
            })
            .unwrap_or(false)
    }

    fn resolve_daemon(&mut self) -> Result<PathBuf, String> {
        if let Some(ref path) = self.daemon_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }
        if let Ok(path) = std::env::var("NEOTRIX_PROXY_DAEMON_PATH") {
            let p = PathBuf::from(&path);
            if p.exists() {
                self.daemon_path = Some(p.clone());
                return Ok(p);
            }
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let sibling = dir.join(DAEMON_BIN_NAME);
                if sibling.exists() {
                    self.daemon_path = Some(sibling.clone());
                    return Ok(sibling);
                }
            }
        }
        Err(format!("{} not found", DAEMON_BIN_NAME))
    }

    fn measure_connection_quality(&mut self) -> f64 {
        let start = SystemTime::now();
        let addr = format!("127.0.0.1:{}", PROXY_PORT);
        match TcpStream::connect_timeout(
            &addr
                .parse()
                .unwrap_or_else(|_| ([127, 0, 0, 1], PROXY_PORT).into()),
            Duration::from_secs(5),
        ) {
            Ok(mut stream) => {
                let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
                let test = b"HEAD / HTTP/1.1\r\nHost: proxy-check\r\n\r\n";
                if stream.write_all(test).is_ok() {
                    self.connection_successes += 1;
                    let elapsed = SystemTime::now()
                        .duration_since(start)
                        .unwrap_or_default()
                        .as_secs_f64();
                    let latency_score = 1.0 - (elapsed / 5.0).clamp(0.0, 1.0);
                    self.last_quality = 0.7 * self.last_quality + 0.3 * latency_score;
                } else {
                    self.connection_failures += 1;
                    self.last_quality = 0.7 * self.last_quality + 0.3 * 0.0;
                }
            }
            Err(_) => {
                self.connection_failures += 1;
                self.last_quality = 0.7 * self.last_quality + 0.3 * 0.0;
            }
        }
        self.last_quality
    }

    fn check_dns_health(&self) -> f64 {
        // DNS bypass 已移除，所有域名走系统 DNS 正常解析
        0.0
    }

    fn check_socket_healthy(&self) -> bool {
        UnixStream::connect(control_socket_path()).is_ok()
    }

}

impl Default for NetworkBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessAbility for NetworkBridge {
    fn domain(&self) -> Domain {
        Domain::Network
    }

    fn sense(&mut self) -> Vec<VsaTagged> {
        self.last_socket_check_ms = self.now_ms();
        let socket_ok = self.check_socket_healthy();
        self.proxy_running = self.proxy_process_alive() && socket_ok;

        let quality = self.measure_connection_quality();
        let dns_health = self.check_dns_health();

        vec![
            VsaTagged {
                vector: self.vsa.seeded_vector(self.seed_for("network_quality", 1)),
                origin: VsaOrigin::World(crate::types::Sensory::NetworkEvent),
                timestamp_ms: self.now_ms(),
                negentropy_contribution: quality,
            },
            VsaTagged {
                vector: self.vsa.seeded_vector(self.seed_for("network_proxy", 2)),
                origin: VsaOrigin::World(crate::types::Sensory::NetworkEvent),
                timestamp_ms: self.now_ms(),
                negentropy_contribution: if self.proxy_running { 0.3 } else { 0.0 },
            },
            VsaTagged {
                vector: self.vsa.seeded_vector(self.seed_for("network_dns", 3)),
                origin: VsaOrigin::World(crate::types::Sensory::NetworkEvent),
                timestamp_ms: self.now_ms(),
                negentropy_contribution: dns_health * 0.2,
            },
            VsaTagged {
                vector: {
                    let base = self.vsa.seeded_vector(self.seed_for("network_fp", 4));
                    let fp = self.current_fingerprint.as_bytes();
                    let len = fp.len().max(1);
                    base.into_iter()
                        .enumerate()
                        .map(|(i, b)| b.wrapping_add(fp[i % len]))
                        .collect()
                },
                origin: VsaOrigin::World(crate::types::Sensory::NetworkEvent),
                timestamp_ms: self.now_ms(),
                negentropy_contribution: 0.1,
            },
        ]
    }

    fn actuate(&mut self, intention: &IntentionVsa) -> Result<WorldEffect, String> {
        self.total_actuations += 1;
        let start = SystemTime::now();

        let result = match intention.action.as_str() {
            "set_mode" => {
                let mode = intention
                    .parameters
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("direct");
                match mode {
                    "stealth" | "direct" => {
                        self.send_control_command(&format!("mode={}", mode))?;
                        Ok(format!("proxy mode set to {}", mode))
                    }
                    other => Err(format!("unknown proxy mode: {}", other)),
                }
            }

            "rotate_fingerprint" => {
                let profile = intention
                    .parameters
                    .get("profile")
                    .and_then(|v| v.as_str())
                    .unwrap_or("random");
                if profile == "random" {
                    let resp = self.send_control_command("fingerprint=rotate")?;
                    if let Some(extracted) = resp.strip_prefix("OK: rotated to ") {
                        self.current_fingerprint = extracted.trim().to_string();
                    }
                } else if FINGERPRINT_PROFILES.contains(&profile) {
                    let _ = self.send_control_command("fingerprint=rotate")?;
                    self.current_fingerprint = profile.to_string();
                } else {
                    return Err(format!("unknown fingerprint: {}", profile));
                }
                if !self.fingerprints_tested.contains(&self.current_fingerprint) {
                    self.fingerprints_tested
                        .push(self.current_fingerprint.clone());
                }
                Ok(format!("rotated to {}", self.current_fingerprint))
            }

            "rotate_ip" => {
                let resp = self.send_control_command("ip=rotate")?;
                if resp.starts_with("OK: rotated IP") {
                    Ok(resp)
                } else {
                    Err(resp)
                }
            }

            "upstream_add" => {
                let url = intention
                    .parameters
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "missing url parameter".to_string())?;
                let resp = self.send_control_command(&format!("upstream=add,{}", url))?;
                if resp.starts_with("OK: added") {
                    Ok(resp)
                } else {
                    Err(resp)
                }
            }

            "upstream_clear" => {
                let resp = self.send_control_command("upstream=clear")?;
                Ok(resp)
            }

            "upstream_list" => {
                let resp = self.send_control_command("upstream=list")?;
                Ok(resp)
            }

            "upstream_reload" => {
                let resp = self.send_control_command("upstream=reload")?;
                Ok(resp)
            }

            "start_proxy" => {
                if self.proxy_process_alive() {
                    self.proxy_running = true;
                    return Ok(WorldEffect {
                        domain: Domain::Network,
                        description: "proxy already running".into(),
                        success: true,
                        latency_ms: 0,
                    });
                }
                let path = self.resolve_daemon()?;
                let log_dir = std::env::var("HOME").map(|h| format!("{}/.neotrix", h)).unwrap_or_else(|_| "/tmp/.neotrix".to_string());
                let _ = std::fs::create_dir_all(&log_dir);
                let out = std::fs::File::create(format!("{}/{}.out.log", log_dir, DAEMON_BIN_NAME))
                    .map_err(|e| format!("log create: {}", e))?;
                let err = std::fs::File::create(format!("{}/{}.err.log", log_dir, DAEMON_BIN_NAME))
                    .map_err(|e| format!("err log create: {}", e))?;
                Command::new(&path)
                    .arg("--mode=stealth")
                    .stdout(Stdio::from(out))
                    .stderr(Stdio::from(err))
                    .stdin(Stdio::null())
                    .spawn()
                    .map_err(|e| format!("spawn daemon: {}", e))?;
                self.proxy_running = true;
                Ok("proxy daemon started".into())
            }

            "stop_proxy" => {
                let _ = Command::new("pkill").args(["-x", DAEMON_BIN_NAME]).output();
                self.proxy_running = false;
                Ok("proxy daemon stopped".into())
            }

            other => Err(format!("unknown network action: {}", other)),
        };

        let latency_ms = SystemTime::now()
            .duration_since(start)
            .unwrap_or_default()
            .as_millis() as u64;

        match result {
            Ok(desc) => Ok(WorldEffect {
                domain: Domain::Network,
                description: desc,
                success: true,
                latency_ms,
            }),
            Err(e) => Ok(WorldEffect {
                domain: Domain::Network,
                description: e,
                success: false,
                latency_ms,
            }),
        }
    }

    fn curiosity_signals(&self) -> Vec<CuriositySignal> {
        let mut signals = Vec::new();

        for profile in FINGERPRINT_PROFILES {
            if !self.fingerprints_tested.contains(&profile.to_string()) {
                signals.push(CuriositySignal {
                    domain: Domain::Network,
                    query: format!("test fingerprint profile {}", profile),
                    novelty_estimate: 0.9,
                    potential_negentropy: 0.15,
                });
            }
        }

        if self.connection_failures > self.connection_successes.saturating_add(3) {
            signals.push(CuriositySignal {
                domain: Domain::Network,
                query: format!(
                    "diagnose high proxy failure: {} failures / {} successes",
                    self.connection_failures, self.connection_successes
                ),
                novelty_estimate: 0.6,
                potential_negentropy: 0.4,
            });
            signals.push(CuriositySignal {
                domain: Domain::Network,
                query: "rotate IP address via upstream proxy pool to bypass rate limiting".into(),
                novelty_estimate: 0.7,
                potential_negentropy: 0.5,
            });
        }

        if self.connection_failures > 5 {
            signals.push(CuriositySignal {
                domain: Domain::Network,
                query: "high failure rate detected, consider adding more upstream proxies for IP rotation".into(),
                novelty_estimate: 0.5,
                potential_negentropy: 0.3,
            });
        }

        if !self.proxy_running && self.total_actuations > 0 {
            signals.push(CuriositySignal {
                domain: Domain::Network,
                query: "proxy daemon stopped unexpectedly, investigate restart".into(),
                novelty_estimate: 0.5,
                potential_negentropy: 0.3,
            });
        }

        signals
    }

    fn grace_mode(&self) -> GraceMode {
        GraceMode::FallbackDefault
    }

    fn health(&self) -> BridgeHealth {
        BridgeHealth {
            domain: Domain::Network,
            available: self.proxy_running || self.check_dns_health() > 0.5,
            last_seen_ms: self.last_socket_check_ms,
            error_count: self.connection_failures,
            total_actuations: self.total_actuations,
        }
    }

    fn probe_available(&self) -> bool {
        true
    }

    fn negentropy_estimate(&self) -> f64 {
        let total = self.connection_successes + self.connection_failures;
        if total == 0 {
            return self.last_quality;
        }
        let rate = self.connection_successes as f64 / total as f64;
        0.6 * rate + 0.3 * self.last_quality + 0.1 * self.check_dns_health()
    }
}
