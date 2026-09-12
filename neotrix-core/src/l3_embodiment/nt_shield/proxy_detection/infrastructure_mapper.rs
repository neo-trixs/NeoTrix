//! Infrastructure mapper — Domain/IP/account correlation engine
//!
//! Builds a unified infrastructure graph from observations:
//! - Domain → IP relationships (shared hosting, DNS resolution)
//! - IP → Account relationships (shared infrastructure)
//! - Account → Domain relationships (email patterns)
//! - Cross-layer correlation (domain→IP→account→domain cycles)
//!
//! Detects coordinated proxy networks by finding infrastructure
//! cycles that indicate controlled account farms.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProxyDetectionConfig, ProxyDetectionError, AccountObservation, DetectionSignal, ThreatLevel};

// ── Infrastructure Graph ──────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct InfrastructureMap {
    /// domain → set of IPs observed using that domain
    pub domain_to_ips: HashMap<String, HashSet<String>>,
    /// IP → set of account IDs observed from that IP
    pub ip_to_accounts: HashMap<String, HashSet<String>>,
    /// account → email domain used
    pub account_to_email_domain: HashMap<String, String>,
    /// account → IPs used
    pub account_to_ips: HashMap<String, Vec<String>>,
    /// Detected infrastructure cycles (account→IP→account loops)
    pub cycles: Vec<_InfrastructureCycle>,
    /// Shared infrastructure groups
    pub shared_infra_groups: Vec<_SharedInfraGroup>,
    /// Overall infrastructure risk score
    pub risk_score: f64,
}

#[derive(Debug, Clone)]
pub struct _InfrastructureCycle {
    pub cycle_type: _CycleType,
    pub nodes: Vec<String>,
    pub length: usize,
    pub risk_weight: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum _CycleType {
    /// Two accounts sharing the same IP
    IpOverlap,
    /// Two accounts using the same email domain + IP
    DomainIpOverlap,
    /// Three+ accounts forming a ring via shared IPs
    MultiAccountRing,
    /// Account farm: many accounts → few IPs → single domain
    AccountFarm,
}

#[derive(Debug, Clone)]
pub struct _SharedInfraGroup {
    pub group_id: String,
    pub accounts: Vec<String>,
    pub shared_ips: Vec<String>,
    pub shared_domains: Vec<String>,
    pub infrastructure_score: f64,
    pub indicators: Vec<String>,
}

// ── Infrastructure Mapper ─────────────────────────────────────────────────
pub struct InfrastructureMapper {
    config: ProxyDetectionConfig,
    /// Accumulated graph from all observations
    graph: Arc<RwLock<InfrastructureMap>>,
}

impl InfrastructureMapper {
    pub fn new(config: ProxyDetectionConfig) -> Self {
        Self {
            config,
            graph: Arc::new(RwLock::new(InfrastructureMap {
                domain_to_ips: HashMap::new(),
                ip_to_accounts: HashMap::new(),
                account_to_email_domain: HashMap::new(),
                account_to_ips: HashMap::new(),
                cycles: Vec::new(),
                shared_infra_groups: Vec::new(),
                risk_score: 0.0,
            })),
        }
    }

    /// Build infrastructure map from all current observations.
    pub async fn build_map(
        &self,
        observations: Vec<&AccountObservation>,
    ) -> Result<InfrastructureMap, ProxyDetectionError> {
        // ── Phase 1: Build adjacency maps ────────────────────────────────
        let mut domain_to_ips: HashMap<String, HashSet<String>> = HashMap::new();
        let mut ip_to_accounts: HashMap<String, HashSet<String>> = HashMap::new();
        let mut account_to_email_domain: HashMap<String, String> = HashMap::new();
        let mut account_to_ips: HashMap<String, Vec<String>> = HashMap::new();

        for obs in &observations {
            // Domain → IPs
            let entry = domain_to_ips
                .entry(obs.email_domain.clone())
                .or_insert_with(HashSet::new);
            for ip in &obs.ip_addresses {
                entry.insert(ip.clone());
            }

            // IP → Accounts
            for ip in &obs.ip_addresses {
                ip_to_accounts
                    .entry(ip.clone())
                    .or_insert_with(HashSet::new)
                    .insert(obs.account_id.clone());
            }

            // Account → Domain
            account_to_email_domain.insert(obs.account_id.clone(), obs.email_domain.clone());

            // Account → IPs
            account_to_ips.insert(obs.account_id.clone(), obs.ip_addresses.clone());
        }

        // ── Phase 2: Detect cycles ───────────────────────────────────────
        let cycles = self.detect_cycles(
            &domain_to_ips,
            &ip_to_accounts,
            &account_to_email_domain,
            &account_to_ips,
        );

        // ── Phase 3: Find shared infrastructure groups ────────────────────
        let groups = self.find_shared_infra_groups(
            &ip_to_accounts,
            &domain_to_ips,
        );

        // ── Phase 4: Compute risk score ──────────────────────────────────
        let risk_score = self.compute_risk_score(&cycles, &groups, observations.len());

        let map = InfrastructureMap {
            domain_to_ips,
            ip_to_accounts,
            account_to_email_domain,
            account_to_ips,
            cycles,
            shared_infra_groups: groups,
            risk_score,
        };

        // Cache the result
        {
            let mut graph = self.graph.write().await;
            *graph = map.clone();
        }

        Ok(map)
    }

    /// Detect infrastructure cycles in the graph.
    fn detect_cycles(
        &self,
        domain_to_ips: &HashMap<String, HashSet<String>>,
        ip_to_accounts: &HashMap<String, HashSet<String>>,
        _account_to_email_domain: &HashMap<String, String>,
        _account_to_ips: &HashMap<String, Vec<String>>,
    ) -> Vec<_InfrastructureCycle> {
        let mut cycles = Vec::new();

        // ── Cycle Type 1: IP → Multiple accounts ─────────────────────────
        for (ip, accounts) in ip_to_accounts.iter() {
            if accounts.len() >= self.config.ip_cluster_threshold {
                let mut nodes = vec![ip.clone()];
                nodes.extend(accounts.iter().cloned());
                cycles.push(_InfrastructureCycle {
                    cycle_type: _CycleType::IpOverlap,
                    nodes,
                    length: accounts.len(),
                    risk_weight: 0.8,
                });
            }
        }

        // ── Cycle Type 2: Domain → IPs → Multiple accounts ──────────────
        for (domain, ips) in domain_to_ips.iter() {
            if ips.len() >= 3 {
                let mut accounts: HashSet<String> = HashSet::new();
                for ip in ips {
                    if let Some(accs) = ip_to_accounts.get(ip) {
                        accounts.extend(accs.iter().cloned());
                    }
                }
                if accounts.len() >= self.config.ip_cluster_threshold {
                    let mut nodes = vec![domain.clone()];
                    nodes.extend(ips.iter().cloned());
                    nodes.extend(accounts.iter().cloned());
                    cycles.push(_InfrastructureCycle {
                        cycle_type: _CycleType::DomainIpOverlap,
                        nodes,
                        length: accounts.len(),
                        risk_weight: 0.85,
                    });
                }
            }
        }

        // ── Cycle Type 3: Account Farm ──────────────────────────────────
        // Many accounts → few IPs → single email domain
        for (domain, ips) in domain_to_ips.iter() {
            let mut all_accounts: HashSet<String> = HashSet::new();
            for ip in ips {
                if let Some(accs) = ip_to_accounts.get(ip) {
                    all_accounts.extend(accs.iter().cloned());
                }
            }
            let account_count = all_accounts.len();
            let ip_count = ips.len();
            let ratio = if ip_count > 0 {
                account_count as f64 / ip_count as f64
            } else {
                0.0
            };

            // High account-to-IP ratio suggests account farming
            if ratio >= 5.0 && account_count >= self.config.ip_cluster_threshold {
                let mut nodes = vec![domain.clone()];
                nodes.extend(ips.iter().cloned());
                nodes.extend(all_accounts.iter().cloned());
                cycles.push(_InfrastructureCycle {
                    cycle_type: _CycleType::AccountFarm,
                    nodes,
                    length: account_count,
                    risk_weight: (ratio / 10.0).min(1.0),
                });
            }
        }

        cycles
    }

    /// Find groups of accounts sharing significant infrastructure.
    fn find_shared_infra_groups(
        &self,
        ip_to_accounts: &HashMap<String, HashSet<String>>,
        domain_to_ips: &HashMap<String, HashSet<String>>,
    ) -> Vec<_SharedInfraGroup> {
        let mut groups = Vec::new();
        let mut visited_ips: HashSet<String> = HashSet::new();

        for (ip, accounts) in ip_to_accounts.iter() {
            if visited_ips.contains(ip.as_str()) {
                continue;
            }
            if accounts.len() < 2 {
                continue;
            }

            // Find all IPs that share at least one account with this IP
            let mut group_accounts: HashSet<String> = accounts.clone();
            let mut group_ips: HashSet<String> = HashSet::new();
            group_ips.insert(ip.clone());

            // Transitively expand: find all accounts on any IP in the group
            let mut changed = true;
            while changed {
                changed = false;
                for (other_ip, other_accounts) in ip_to_accounts.iter() {
                    if group_ips.contains(other_ip) {
                        continue;
                    }
                    let overlap: usize = group_accounts.intersection(other_accounts).count();
                    if overlap >= 2 {
                        group_ips.insert(other_ip.clone());
                        let new_accounts: HashSet<String> = other_accounts
                            .difference(&group_accounts)
                            .cloned()
                            .collect();
                        if !new_accounts.is_empty() {
                            group_accounts.extend(new_accounts);
                            changed = true;
                        }
                    }
                }
            }

            if group_accounts.len() < 2 {
                continue;
            }

            // Find domains used by group accounts
            let mut group_domains: HashSet<String> = HashSet::new();
            for (_domain, ips) in domain_to_ips.iter() {
                if !ips.intersection(&group_ips).next().is_some() {
                    continue;
                }
                // At least one IP in this domain's set overlaps with group IPs
                let domain_ip_overlap = ips.intersection(&group_ips).count();
                if domain_ip_overlap >= 1 {
                    // Check if domain is used by multiple group accounts
                    let mut domain_account_count = 0;
                    for dip in ips {
                        if let Some(accs) = ip_to_accounts.get(dip) {
                            domain_account_count += accs.intersection(&group_accounts).count();
                        }
                    }
                    if domain_account_count >= 2 {
                        group_domains.insert(_domain.clone());
                    }
                }
            }

            // Compute indicators
            let mut indicators = Vec::new();
            if group_ips.len() >= 3 {
                indicators.push(format!("{} shared IPs", group_ips.len()));
            }
            if !group_domains.is_empty() {
                indicators.push(format!("{} shared email domains", group_domains.len()));
            }
            let ratio = group_accounts.len() as f64 / group_ips.len().max(1) as f64;
            if ratio >= 3.0 {
                indicators.push(format!("Account-to-IP ratio: {:.1}:1", ratio));
            }

            let infra_score = (group_accounts.len() as f64 / 20.0).min(1.0);

            groups.push(_SharedInfraGroup {
                group_id: format!("infra_{}", &ip[..8.min(ip.len())].replace('.', "_")),
                accounts: group_accounts.into_iter().collect(),
                shared_ips: group_ips.iter().cloned().collect(),
                shared_domains: group_domains.into_iter().collect(),
                infrastructure_score: infra_score,
                indicators,
            });

            // Mark all IPs in this group as visited
            for visited_ip in &group_ips {
                visited_ips.insert(visited_ip.clone());
            }
        }

        groups
    }

    /// Compute overall infrastructure risk score.
    fn compute_risk_score(
        &self,
        cycles: &[_InfrastructureCycle],
        groups: &[_SharedInfraGroup],
        total_accounts: usize,
    ) -> f64 {
        if total_accounts == 0 {
            return 0.0;
        }

        let mut score = 0.0_f64;

        // Cycle contribution
        let max_cycle_weight = cycles.iter()
            .map(|c| c.risk_weight)
            .fold(0.0_f64, f64::max);
        score += max_cycle_weight * 0.4;

        // Group contribution
        let max_group_score = groups.iter()
            .map(|g| g.infrastructure_score)
            .fold(0.0_f64, f64::max);
        score += max_group_score * 0.3;

        // Coverage: what fraction of accounts are in suspicious groups
        let suspicious_accounts: usize = groups.iter()
            .map(|g| g.accounts.len())
            .sum();
        let coverage = suspicious_accounts as f64 / total_accounts as f64;
        score += coverage * 0.3;

        score.min(1.0)
    }

    /// Build DetectionSignals from an InfrastructureMap.
    pub fn build_signals(&self, map: &InfrastructureMap) -> Vec<DetectionSignal> {
        let mut signals = Vec::new();

        // High-risk cycles
        for cycle in &map.cycles {
            if cycle.risk_weight >= 0.8 {
                signals.push(DetectionSignal {
                    signal_type: format!("infra_cycle_{:?}", cycle.cycle_type).to_lowercase(),
                    confidence: cycle.risk_weight,
                    threat_level: if cycle.risk_weight >= 0.9 {
                        ThreatLevel::Critical
                    } else {
                        ThreatLevel::High
                    },
                    details: format!(
                        "{:?} cycle detected: {} nodes (risk: {:.2})",
                        cycle.cycle_type, cycle.length, cycle.risk_weight
                    ),
                });
            }
        }

        // Shared infrastructure groups
        for group in &map.shared_infra_groups {
            if group.accounts.len() >= self.config.ip_cluster_threshold {
                signals.push(DetectionSignal {
                    signal_type: "shared_infrastructure_group".to_string(),
                    confidence: group.infrastructure_score,
                    threat_level: if group.infrastructure_score >= 0.7 {
                        ThreatLevel::Critical
                    } else {
                        ThreatLevel::High
                    },
                    details: format!(
                        "Shared infra group {}: {} accounts, {} IPs, {} domains [{}]",
                        group.group_id,
                        group.accounts.len(),
                        group.shared_ips.len(),
                        group.shared_domains.len(),
                        group.indicators.join(", ")
                    ),
                });
            }
        }

        signals
    }

    /// Get current graph snapshot.
    pub async fn get_graph(&self) -> InfrastructureMap {
        self.graph.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn make_observation(
        id: &str,
        created_at: u64,
        ips: Vec<&str>,
        email_domain: &str,
    ) -> AccountObservation {
        AccountObservation {
            account_id: id.into(),
            created_at,
            ip_addresses: ips.into_iter().map(String::from).collect(),
            email_domain: email_domain.into(),
            user_agent: "Mozilla/5.0".into(),
            request_pattern_hash: format!("{:064x}", created_at),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn build_map_basic() {
        let config = ProxyDetectionConfig::default();
        let mapper = InfrastructureMapper::new(config);

        let observations = vec![
            make_observation("a1", 1000, vec!["1.2.3.4", "5.6.7.8"], "temp.com"),
            make_observation("a2", 1001, vec!["1.2.3.4"], "temp.com"),
            make_observation("a3", 1002, vec!["5.6.7.8"], "temp.com"),
        ];

        let refs: Vec<&AccountObservation> = observations.iter().collect();
        let map = mapper.build_map(refs).await.unwrap();

        // All 3 accounts should be mapped
        assert_eq!(map.account_to_email_domain.len(), 3);
        // IP 1.2.3.4 should have 2 accounts
        assert!(map.ip_to_accounts.get("1.2.3.4").map_or(false, |a| a.len() >= 2));
    }

    #[tokio::test]
    async fn account_farm_detection() {
        let config = ProxyDetectionConfig {
            ip_cluster_threshold: 3,
            ..Default::default()
        };
        let mapper = InfrastructureMapper::new(config);

        // 8 accounts, 2 IPs, 1 domain → ratio 4:1
        let mut observations = Vec::new();
        for i in 0..8 {
            let ip = if i % 2 == 0 { "10.0.0.1" } else { "10.0.0.2" };
            observations.push(make_observation(
                &format!("farm_{i}"),
                1000 + i as u64,
                vec![ip],
                "farm-disposable.com",
            ));
        }

        let refs: Vec<&AccountObservation> = observations.iter().collect();
        let map = mapper.build_map(refs).await.unwrap();

        let farm_cycles: Vec<_> = map.cycles.iter()
            .filter(|c| c.cycle_type == _CycleType::AccountFarm)
            .collect();
        assert!(!farm_cycles.is_empty(), "Should detect account farm pattern");
    }

    #[test]
    fn signal_generation() {
        let config = ProxyDetectionConfig::default();
        let mapper = InfrastructureMapper::new(config);

        let map = InfrastructureMap {
            domain_to_ips: HashMap::new(),
            ip_to_accounts: HashMap::new(),
            account_to_email_domain: HashMap::new(),
            account_to_ips: HashMap::new(),
            cycles: vec![_InfrastructureCycle {
                cycle_type: _CycleType::IpOverlap,
                nodes: vec!["1.2.3.4".into(), "a1".into(), "a2".into()],
                length: 2,
                risk_weight: 0.85,
            }],
            shared_infra_groups: vec![_SharedInfraGroup {
                group_id: "test".into(),
                accounts: vec!["a1".into(), "a2".into()],
                shared_ips: vec!["1.2.3.4".into()],
                shared_domains: vec!["test.com".into()],
                infrastructure_score: 0.7,
                indicators: vec!["2 shared IPs".into()],
            }],
            risk_score: 0.75,
        };

        let signals = mapper.build_signals(&map);
        assert!(!signals.is_empty());
    }
}
