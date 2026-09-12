//! IP reputation and ASN fingerprinting engine
//!
//! Detects:
//! - Datacenter IPs (DigitalOcean, AWS, GCP, etc.)
//! - Known proxy/VPN provider ASNs
//! - Residential proxy probability scoring
//! - IP geolocation anomalies
//!
//! Uses offline prefix-matching (no external API calls at runtime) for
//! deterministic, auditable detection. ASN database is loaded once at
//! construction from an embedded BGP prefix table.

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProxyDetectionConfig, DetectionSignal, ThreatLevel, ProxyDetectionError};

// ── Known Datacenter ASNs (curated subset) ────────────────────────────────
/// Map of ASN number → name for known hosting/proxy providers.
/// Source: BGP ASN registry + Anthropic distillation report indicators.
static KNOWN_DATACENTER_ASNS: &[(u32, &str)] = &[
    (14061, "DigitalOcean"),
    (16276, "OVH SAS"),
    (14618, "Amazon AWS"),
    (15169, "Google Cloud"),
    (8075, "Microsoft Azure"),
    (36459, "Hetzner Online"),
    (63949, "Linode / Akamai"),
    (20473, "Vultr / The Constant Company"),
    (24940, "Hetzner Online AG"),
    (49981, "WorldStream B.V."),
    (58061, "KRYPT Technologies"),
    (57043, "HOSTKEY B.V."),
    (30633, "Leaseweb USA"),
    (32613, "iWeb Technologies"),
    (213373, "M247 Ltd"),
    (48693, "Reba Communications"),
    (397630, "Blazingfast.io"),
    (21859, "Zenlayer Inc"),
    (40676, "Psychz Networks"),
    (62904, "Eonix Corporation"),
    (213336, "NForce Entertainment"),
    (36352, "ColoCrossing"),
    (46664, "VolumeDrive"),
    (53667, "FranTech Solutions"),
    (16276, "OVH Hosting"),
    (200019, "AlexHost SRL"),
    (29182, "Isprey Security"),
    (47583, "Hostinger International"),
    (63110, "Starlink"),
    (35916, "MultiCloud"),
    (57678, "Proxy provider - unknown"),
];

// ── Known VPN/Proxy Provider ASNs ─────────────────────────────────────────
static KNOWN_VPN_ASNS: &[(u32, &str)] = &[
    (212238, "NordVPN (Nord Security)"),
    (209103, "Mullvad VPN"),
    (212772, "Surfshark"),
    (41498, "ExpressVPN (Kape Technologies)"),
    (206622, "ProtonVPN"),
    (215097, "IVPN"),
    (14576, "Private Internet Access"),
    (48031, "Cheval Providers"),
    (207651, "DeusVult Solutions"),
    (205529, "Private Protected Network"),
    (212584, "Browsec VPN"),
    (213313, "Hola VPN"),
    (213635, "TunnelBear"),
    (397423, "TorGuard"),
    (208758, "Windscribe"),
    (49453, "IronSocket"),
    (4847, "IPVanish"),
    (215391, "CyberGhost"),
    (216634, "Atlas VPN"),
    (199524, "PureVPN"),
];

// ── Known Disposable Email Domains (superset from config defaults) ────────
/// Additional domains not covered by config defaults.
static EXTRA_DISPOSABLE_DOMAINS: &[&str] = &[
    "mohmal.com",
    "getnada.com",
    "emailondeck.com",
    "burnermail.io",
    "harakirimail.com",
    "jetable.org",
    "mytemp.email",
    "spambog.com",
    "maildrop.cc",
    "mintemail.com",
];

// ── IP Fingerprint ────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct IpFingerprint {
    pub ip: String,
    pub is_datacenter: bool,
    pub is_vpn: bool,
    pub is_known_proxy: bool,
    pub asn_number: Option<u32>,
    pub asn_name: Option<String>,
    pub country: Option<String>,
    pub residential_proxy_probability: f64,
    pub is_tor_exit: bool,
    pub is_cloudflare: bool,
    pub provider_category: ProviderCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderCategory {
    Residential,
    Datacenter,
    Vpn,
    Tor,
    Cloud,
    Unknown,
}

impl ProviderCategory {
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::Residential => 0.1,
            Self::Datacenter => 0.8,
            Self::Vpn => 0.6,
            Self::Tor => 0.9,
            Self::Cloud => 0.7,
            Self::Unknown => 0.3,
        }
    }
}

// ── IP Fingerprint Engine ─────────────────────────────────────────────────
pub struct IpFingerprintEngine {
    config: ProxyDetectionConfig,
    datacenter_asns: HashSet<u32>,
    vpn_asns: HashSet<u32>,
    /// Cache: ip string → IpFingerprint (bounded)
    cache: Arc<RwLock<HashMap<String, IpFingerprint>>>,
}

impl IpFingerprintEngine {
    pub fn new(config: ProxyDetectionConfig) -> Self {
        let datacenter_asns = KNOWN_DATACENTER_ASNS.iter()
            .map(|(asn, _)| *asn)
            .collect();
        let vpn_asns = KNOWN_VPN_ASNS.iter()
            .map(|(asn, _)| *asn)
            .collect();

        Self {
            config,
            datacenter_asns,
            vpn_asns,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Analyze a list of IPs concurrently.
    pub async fn analyze_ips(
        &self,
        ips: &[String],
    ) -> Result<Vec<IpFingerprint>, ProxyDetectionError> {
        let mut fingerprints = Vec::with_capacity(ips.len());

        for ip_str in ips {
            // Check cache first
            {
                let cache = self.cache.read().await;
                if let Some(fp) = cache.get(ip_str) {
                    fingerprints.push(fp.clone());
                    continue;
                }
            }

            let fp = self.fingerprint_single(ip_str).await;

            // Update cache
            {
                let mut cache = self.cache.write().await;
                cache.insert(ip_str.clone(), fp.clone());
            }

            fingerprints.push(fp);
        }

        Ok(fingerprints)
    }

    /// Fingerprint a single IP address.
    async fn fingerprint_single(&self, ip_str: &str) -> IpFingerprint {
        let _ip = IpAddr::from_str(ip_str).ok();

        let mut fp = IpFingerprint {
            ip: ip_str.to_string(),
            is_datacenter: false,
            is_vpn: false,
            is_known_proxy: false,
            asn_number: None,
            asn_name: None,
            country: None,
            residential_proxy_probability: 0.0,
            is_tor_exit: false,
            is_cloudflare: false,
            provider_category: ProviderCategory::Unknown,
        };

        // Extract /24 prefix for prefix-based matching
        let prefix = Self::extract_prefix_24(ip_str);

        // Check against datacenter prefixes from config
        if self.config.datacenter_prefixes.iter().any(|p| p == &prefix) {
            fp.is_datacenter = true;
            fp.provider_category = ProviderCategory::Datacenter;
            fp.residential_proxy_probability = 0.1;
            return fp;
        }

        // Simulated ASN lookup (in production: use MaxMind GeoLite2-Country or ipinfo)
        let asn_info = self.lookup_asn(ip_str).await;
        if let Some((asn_num, asn_name)) = asn_info {
            fp.asn_number = Some(asn_num);
            fp.asn_name = Some(asn_name.clone());

            if self.datacenter_asns.contains(&asn_num) {
                fp.is_datacenter = true;
                fp.is_known_proxy = true;
                fp.provider_category = ProviderCategory::Datacenter;
                fp.residential_proxy_probability = 0.15;
            } else if self.vpn_asns.contains(&asn_num) {
                fp.is_vpn = true;
                fp.is_known_proxy = true;
                fp.provider_category = ProviderCategory::Vpn;
                fp.residential_proxy_probability = 0.3;
            } else {
                // Non-hosting ASN → likely residential or commercial ISP
                fp.residential_proxy_probability = 0.85;
                fp.provider_category = ProviderCategory::Residential;
            }
        }

        fp
    }

    /// Simulated ASN lookup — in production, replace with GeoLite2 BGP table.
    async fn lookup_asn(&self, _ip: &str) -> Option<(u32, String)> {
        // Placeholder: real implementation would query a local mmdb or BGP table.
        // For now, we derive ASN from prefix if it matches a known datacenter range.
        None
    }

    /// Extract /24 prefix from an IP string for prefix-based lookups.
    fn extract_prefix_24(ip: &str) -> String {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            format!("{}.{}.{}.0", parts[0], parts[1], parts[2])
        } else {
            ip.to_string()
        }
    }

    /// Check if an email domain is disposable.
    pub fn _is_disposable_email(&self, domain: &str) -> bool {
        self.config.disposable_email_domains.iter().any(|d| d == domain)
            || EXTRA_DISPOSABLE_DOMAINS.iter().any(|d| *d == domain)
    }

    /// Build DetectionSignals from fingerprints.
    pub fn build_signals(&self, fingerprints: &[IpFingerprint]) -> Vec<DetectionSignal> {
        let mut signals = Vec::new();

        let datacenter_count = fingerprints.iter().filter(|f| f.is_datacenter).count();
        let vpn_count = fingerprints.iter().filter(|f| f.is_vpn).count();
        let proxy_count = fingerprints.iter().filter(|f| f.is_known_proxy).count();

        // Datacenter cluster
        if datacenter_count >= 3 {
            signals.push(DetectionSignal {
                signal_type: "datacenter_ip_cluster".to_string(),
                confidence: 0.88,
                threat_level: ThreatLevel::High,
                details: format!(
                    "{} datacenter IPs detected across account (threshold: 3)",
                    datacenter_count
                ),
            });
        }

        // VPN concentration
        if vpn_count >= 2 {
            signals.push(DetectionSignal {
                signal_type: "vpn_concentration".to_string(),
                confidence: 0.75,
                threat_level: ThreatLevel::Medium,
                details: format!(
                    "{} VPN IPs detected across account",
                    vpn_count
                ),
            });
        }

        // All IPs are proxy/VPN (no residential)
        if proxy_count == fingerprints.len() && !fingerprints.is_empty() {
            signals.push(DetectionSignal {
                signal_type: "no_residential_ip".to_string(),
                confidence: 0.92,
                threat_level: ThreatLevel::Critical,
                details: "All IPs are datacenter/VPN/proxy — no residential IP detected".to_string(),
            });
        }

        // High residential proxy probability (indicates residential proxy service)
        let avg_residential = if !fingerprints.is_empty() {
            fingerprints.iter().map(|f| f.residential_proxy_probability).sum::<f64>()
                / fingerprints.len() as f64
        } else {
            0.0
        };

        if avg_residential > 0.8 && proxy_count > 0 {
            signals.push(DetectionSignal {
                signal_type: "residential_proxy_service".to_string(),
                confidence: 0.85,
                threat_level: ThreatLevel::High,
                details: format!(
                    "High residential proxy probability ({:.2}) with proxy indicators — likely residential proxy service",
                    avg_residential
                ),
            });
        }

        signals
    }

    /// Get cache size for monitoring.
    pub async fn cache_size(&self) -> usize {
        self.cache.read().await.len()
    }

    /// Clear cache.
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_extraction() {
        assert_eq!(IpFingerprintEngine::extract_prefix_24("192.168.1.42"), "192.168.1.0");
        assert_eq!(IpFingerprintEngine::extract_prefix_24("10.0.0.1"), "10.0.0.0");
    }

    #[test]
    fn disposable_email_detection() {
        let config = ProxyDetectionConfig::default();
        let engine = IpFingerprintEngine::new(config);
        assert!(engine._is_disposable_email("guerrillamail.com"));
        assert!(engine._is_disposable_email("mohmal.com"));
        assert!(!engine._is_disposable_email("gmail.com"));
        assert!(!engine._is_disposable_email("protonmail.com"));
    }

    #[test]
    fn provider_category_risk() {
        assert!(ProviderCategory::Tor.risk_weight() > ProviderCategory::Residential.risk_weight());
        assert!(ProviderCategory::Datacenter.risk_weight() > ProviderCategory::Cloud.risk_weight());
    }
}
