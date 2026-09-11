# 跨层引用扫描报告

**扫描时间**: 2026-09-11 18:20:59
**扫描范围**: neotrix-core/src/ 下6层目录
**排除文件**: *test*.rs, *facade*.rs

---

## 扫描结果

### 跨层引用违规 (190 处)

| 文件路径 | 行号与Import |
|----------|--------------|
| `l1_action/nt_io/nt_io_provider/factory.rs` | `530:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow => true,` |
| `l1_action/nt_io/nt_io_provider/factory.rs` | `531:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::RequireConfirmation => {` |
| `l1_action/nt_io/nt_io_provider/factory.rs` | `535:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Deny => {` |
| `l1_action/nt_io/nt_io_neocodex/agent.rs` | `110:        Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>>,` |
| `l1_action/nt_io/nt_io_neocodex/agent.rs` | `233:            tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>,` |
| `l2_perception/nt_world/nt_world_urlhaus.rs` | `157:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_urlhaus.rs` | `165:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_urlhaus.rs` | `236:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_usgs.rs` | `284:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_usgs.rs` | `293:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_usgs.rs` | `300:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_bgpview.rs` | `124:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_bgpview.rs` | `228:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");` |
| `l2_perception/nt_world/nt_world_gdelt.rs` | `137:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_gdelt.rs` | `148:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_gdelt.rs` | `158:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_ucdp.rs` | `207:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_ucdp.rs` | `216:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_ucdp.rs` | `223:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_aoi.rs` | `204:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_aoi.rs` | `213:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_aoi.rs` | `220:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/osint/mod.rs` | `951:        let (html, host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)?;` |
| `l2_perception/nt_world/osint/mod.rs` | `952:        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(&html);` |
| `l2_perception/nt_world/osint/mod.rs` | `966:        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(&html, url);` |
| `l2_perception/nt_world/crawl/unified.rs` | `284:                    crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Source,` |
| `l2_perception/nt_world/crawl/unified.rs` | `733:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Source,` |
| `l2_perception/nt_world/nt_world_gdacs.rs` | `143:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_gdacs.rs` | `152:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_gdacs.rs` | `159:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_ofac.rs` | `135:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_ofac.rs` | `144:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_ofac.rs` | `151:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_polymarket.rs` | `113:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_polymarket.rs` | `122:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_polymarket.rs` | `129:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_opencorporates.rs` | `125:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_opencorporates.rs` | `233:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");` |
| `l2_perception/nt_world/nt_world_adsb.rs` | `129:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_adsb.rs` | `138:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_adsb.rs` | `145:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_edgar.rs` | `359:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_edgar.rs` | `371:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_edgar.rs` | `381:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l2_perception/nt_world/nt_world_urlhaus.rs` | `267:pub fn urlhaus_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_urlhaus.rs` | `268:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(URLHAUS_HOST, "443")` |
| `l2_perception/nt_world/nt_world_urlhaus.rs` | `270:pub fn urlhaus_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_usgs.rs` | `338:pub fn usgs_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_usgs.rs` | `339:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(USGS_HOST, "443")` |
| `l2_perception/nt_world/nt_world_usgs.rs` | `341:pub fn usgs_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_bgpview.rs` | `151:pub fn bgpview_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_bgpview.rs` | `152:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(BGPVIEW_HOST, "443")` |
| `l2_perception/nt_world/nt_world_bgpview.rs` | `154:pub fn bgpview_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_gdelt.rs` | `209:pub const GDELT_HOST: &str = crate::l3_embodiment::nt_shield::nt_shield_sandbox::INTEL_GDELT_HOST;` |
| `l2_perception/nt_world/nt_world_gdelt.rs` | `211:pub fn gdelt_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_gdelt.rs` | `212:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(GDELT_HOST, "443")` |
| `l2_perception/nt_world/nt_world_ucdp.rs` | `274:pub fn ucdp_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_ucdp.rs` | `275:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(UCDP_HOST, "443")` |
| `l2_perception/nt_world/nt_world_ucdp.rs` | `277:pub fn ucdp_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_aoi.rs` | `269:pub fn aoi_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_aoi.rs` | `270:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(AOI_HOST, "443")` |
| `l2_perception/nt_world/nt_world_aoi.rs` | `272:pub fn aoi_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/osint/securitytrails.rs` | `225:pub fn securitytrails_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/osint/securitytrails.rs` | `226:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(SECURITYTRAILS_API_HOST, "443")` |
| `l2_perception/nt_world/osint/securitytrails.rs` | `228:pub fn securitytrails_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/osint/fofa.rs` | `549:pub fn fofa_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/osint/fofa.rs` | `550:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(FOFA_API_HOST, "443")` |
| `l2_perception/nt_world/osint/fofa.rs` | `553:pub fn fofa_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/osint/shodan.rs` | `173:pub fn shodan_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/osint/shodan.rs` | `174:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(SHODAN_API_HOST, "443")` |
| `l2_perception/nt_world/osint/shodan.rs` | `176:pub fn shodan_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/osint/zoomeye.rs` | `143:pub fn zoomeye_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/osint/zoomeye.rs` | `144:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(ZOOMEYE_API_HOST, "443")` |
| `l2_perception/nt_world/osint/zoomeye.rs` | `146:pub fn zoomeye_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/osint/censys.rs` | `141:pub fn censys_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/osint/censys.rs` | `142:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(CENSYS_API_HOST, "443")` |
| `l2_perception/nt_world/osint/censys.rs` | `144:pub fn censys_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_gdacs.rs` | `194:pub fn gdacs_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_gdacs.rs` | `195:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(GDACS_HOST, "443")` |
| `l2_perception/nt_world/nt_world_gdacs.rs` | `197:pub fn gdacs_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_ofac.rs` | `180:pub fn ofac_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_ofac.rs` | `181:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(OFAC_HOST, "443")` |
| `l2_perception/nt_world/nt_world_ofac.rs` | `183:pub fn ofac_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_polymarket.rs` | `158:pub fn polymarket_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_polymarket.rs` | `159:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(POLYMARKET_HOST, "443")` |
| `l2_perception/nt_world/nt_world_polymarket.rs` | `161:pub fn polymarket_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_opencorporates.rs` | `156:pub fn opencorporates_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_opencorporates.rs` | `157:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(OPENCORPORATES_HOST, "443")` |
| `l2_perception/nt_world/nt_world_opencorporates.rs` | `159:pub fn opencorporates_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_adsb.rs` | `175:pub fn adsb_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_adsb.rs` | `176:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(ADSB_HOST, "443")` |
| `l2_perception/nt_world/nt_world_adsb.rs` | `178:pub fn adsb_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l2_perception/nt_world/nt_world_edgar.rs` | `434:pub fn edgar_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {` |
| `l2_perception/nt_world/nt_world_edgar.rs` | `435:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(EDGAR_HOST, "443")` |
| `l2_perception/nt_world/nt_world_edgar.rs` | `438:pub fn edgar_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {` |
| `l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs` | `260:fn to_anthropic_response(resp: &crate::l1_action::nt_io::nt_io_provider::types::LlmResponse, model: &str) -> AnthropicResponse {` |
| `l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs` | `463:        let resp = crate::l1_action::nt_io::nt_io_provider::types::LlmResponse {` |
| `l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs` | `466:            usage: crate::l1_action::nt_io::nt_io_provider::types::Usage {` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_core.rs` | `373:        let _ = kb.insert_or_get_node(&title, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Session, Some(summary), None, Some("neotrix"));` |
| `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | `438:            let _ = kb_ref.insert_or_get_node(&title, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Session, Some(&summary), None, Some("neotrix"));` |
| `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | `1074:            crate::l1_action::nt_io::nt_io_provider::gateway::run_periodic_re_evaluation();` |
| `l5_cognition/nt_mind/nt_mind_background_loop/mod.rs` | `134:        if let Ok(kb) = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | `679:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch};` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | `680:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | `761:            if let Ok(clusters) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::list_clusters(&conn) {` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_daily_intel.rs` | `54:                let _ = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::kv_set(` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `1476:                &crate::l1_action::nt_memory::nt_memory_kb::nt_memory_galaxy_hygiene::GalaxyHygieneConfig::default(),` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `1594:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `1600:            crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),` |
| `l5_cognition/nt_mind/nt_mind_background_loop/knowledge_pipeline.rs` | `123:            crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)?;` |
| `l5_cognition/nt_mind/nt_mind_background_loop/knowledge_pipeline.rs` | `138:            crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_async(url)` |
| `l5_cognition/nt_mind/nt_mind/web_miner.rs` | `203:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)` |
| `l5_cognition/nt_mind/nt_mind/web_miner.rs` | `238:        let (xml_text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)` |
| `l5_cognition/nt_mind/nt_mind/web_miner.rs` | `289:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_with_headers(` |
| `l5_cognition/nt_mind/nt_mind/self_evolver.rs` | `215:        let (body, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)` |
| `l5_cognition/nt_mind/nt_mind/consciousness/hypercube_bridge.rs` | `97:    fn coord_from_kb_node(node: &crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::KnowledgeNode) -> HyperCoord {` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `1959:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionPatternType::RecurringError` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `1961:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionPatternType::StrategyDiscovery` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `1968:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionRecord {` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/loop_impl/seal_loop.rs` | `1000:            crate::l1_action::nt_io::nt_io_provider::gateway::register_gateway_for_re_evaluation(&gw);` |
| `l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs` | `516:        let mut ledger = crate::l1_action::nt_memory::evidence_ledger::SessionLedger::new(session_id);` |
| `l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs` | `518:            crate::l1_action::nt_memory::evidence_ledger::EvidenceType::Observation,` |
| `l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs` | `203:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)` |
| `l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs` | `238:        let (xml_text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)` |
| `l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs` | `289:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_with_headers(` |
| `l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs` | `32:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&url) {` |
| `l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs` | `78:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&url) {` |
| `l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs` | `110:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&search_url) {` |
| `l5_cognition/nt_mind/nt_mind/co_evolution.rs` | `586:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.into())).expect("open kb");` |
| `l5_cognition/nt_mind/nt_mind/evolution/agent_capability/mod.rs` | `133:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::default(),` |
| `l5_cognition/nt_mind/nt_mind/evolution/self_evolver.rs` | `215:        let (body, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)` |
| `l5_cognition/nt_mind/nt_mind/evolution/co_evolution.rs` | `586:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.into())).expect("open kb");` |
| `l5_cognition/nt_mind/nt_mind/reason/attention_router.rs` | `657:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Insight,` |
| `l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs` | `426:            // let verification_score = crate::l1_action::nt_io::nt_io_standalone::verify_answer(gold, response);` |
| `l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs` | `440:                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,` |
| `l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs` | `480:                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,` |
| `l5_cognition/nt_mind/nt_mind/auto_crystallizer.rs` | `74:        let now = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_diversity::now_unix_secs() as u64;` |
| `l5_cognition/nt_mind/foundation/seal_pipeline.rs` | `97:    inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate,` |
| `l5_cognition/nt_mind/foundation/seal_pipeline.rs` | `103:            inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),` |
| `l5_cognition/nt_mind/foundation/seal_pipeline.rs` | `127:            inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),` |
| `l5_cognition/nt_mind/foundation/knowledge_store.rs` | `72:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html)` |
| `l5_cognition/nt_mind/foundation/knowledge_store.rs` | `76:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::is_safe_fetch_url(url)` |
| `l5_cognition/nt_mind/foundation/knowledge_store.rs` | `88:        let item = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::claim_next_crawl_url(&conn)` |
| `l5_cognition/nt_mind/foundation/l1_wrappers.rs` | `37:    inner: crate::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager,` |
| `l5_cognition/nt_mind/foundation/l1_wrappers.rs` | `43:            inner: crate::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager::new(session_id),` |
| `l5_cognition/nt_mind/foundation/l1_wrappers.rs` | `102:    inner: std::sync::Mutex<crate::l1_action::nt_io::nt_io_user_avatar::DistillationEngine>,` |
| `l5_cognition/nt_mind/nt_mind_skill_engine.rs` | `328:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.content);` |
| `l5_cognition/nt_mind/nt_mind_skill_engine.rs` | `330:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(body);` |
| `l5_cognition/nt_mind/nt_mind_skill_engine.rs` | `332:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.description);` |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | `10:use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | `33:// pub use crate::l1_action::nt_act::nt_l1_shared_types::{` |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | `70:    fn get_snapshot(&self) -> crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | `21:// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;` |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | `67:pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | `1580:impl crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::EvolutionLoopProvider for EvolutionLoop {` |
| `l5_cognition/nt_core/nt_core_parallel/contract.rs` | `195:    kb: Option<crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase>,` |
| `l5_cognition/nt_core/nt_core_parallel/contract.rs` | `210:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None).ok();` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `2252:            crate::l2_perception::nt_world::nt_world_exploration_engine::ExplorationConfig::default(` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `2255:            crate::l2_perception::nt_world::nt_world_exploration_engine::ExplorationEngine::new(` |
| `l5_cognition/mod.rs` | `16:/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `1451:            let auditor = crate::l3_embodiment::nt_shield::nt_shield_audit::create_reasoning_trace_auditor();` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `1620:        self_tests.register(crate::l3_embodiment::nt_shield::nt_shield::browser_security::create_browser_security_self_test());` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `1621:        self_tests.register(crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test());` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `3104:        registry.register(crate::l3_embodiment::nt_shield::nt_shield::browser_security::create_browser_security_self_test());` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `3105:        registry.register(crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test());` |
| `l5_cognition/nt_mind/nt_mind_skill_engine.rs` | `1558:                                crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::scan_skill_content(&skill.content);` |
| `l5_cognition/nt_mind/nt_mind_skill_engine.rs` | `1559:                            let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::InspectionResult::Allow);` |
| `l5_cognition/nt_mind/nt_mind_skill_engine.rs` | `1590:                            crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::scan_skill_content(&skill.content);` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `3100:        //     crate::l4_emotion::nt_feel::nt_core_fep_iit::bridge::FEPIITBridge::new(),` |
| `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | `717:            self_improvement: crate::l6_meta::coordination::self_improvement::SelfImprovementLoop::new(),` |
| `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | `1028:    self_improvement: crate::l6_meta::coordination::self_improvement::SelfImprovementLoop,` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | `940:        use crate::l6_meta::coordination::self_improvement::SystemMetrics;` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `197:                    //         let input = crate::l6_meta::nt_meta::nt_core_intra_reflection::ReflectionInput {` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `205:                    //             crate::l6_meta::nt_meta::nt_core_intra_reflection::analyze(` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `266:        let (infos, _problems): (Vec<_>, Vec<_>) = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::harness_infos_from_registry_export(&json);` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `3079:            let mut cm = crate::l6_meta::nt_repair::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();` |
| `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `3102:        registry.register(Box::new(crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));` |
| `l5_cognition/mod.rs` | `22:/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。` |
| `l5_cognition/traits.rs` | `137:// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations` |
| `l6_meta/memory/evolution_harness.rs` | `120:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,` |
| `l6_meta/memory/evolution_harness.rs` | `200:impl crate::l5_cognition::traits::EvolutionHarnessApi for EvolutionHarness {` |
| `l6_meta/memory/evolution_harness.rs` | `205:    fn harness_infos_from_registry_export(json: &str) -> (Vec<crate::l5_cognition::traits::RegistryNodeInfo>, Vec<String>) {` |
| `l6_meta/memory/evolution_harness.rs` | `207:        let infos = l6_infos.into_iter().map(|i| crate::l5_cognition::traits::RegistryNodeInfo {` |
| `l6_meta/healing/nt_mind_consciousness_gold_standard.rs` | `229:impl crate::l5_cognition::traits::GoldStandardApi for ConsciousnessGoldStandard {}` |
| `l6_meta/healing/nt_mind_eval_harness.rs` | `1267:impl crate::l5_cognition::traits::EvalHarnessApi for EvalHarness {` |
| `l6_meta/healing/nt_mind_eval_harness.rs` | `1268:    fn generate_regression_test(&self, candidate: &str) -> crate::l5_cognition::traits::RegressionCase {` |
| `l6_meta/healing/nt_mind_eval_harness.rs` | `1270:        crate::l5_cognition::traits::RegressionCase {` |
| `l6_meta/healing/nt_mind_consciousness_monitor.rs` | `399:impl crate::l5_cognition::traits::ConsciousnessMonitorApi for ConsciousnessMonitor {` |
| `l6_meta/healing/nt_mind_consciousness_monitor.rs` | `408:    fn get_report(&self) -> crate::l5_cognition::traits::ConsciousnessAwarenessReport {` |
| `l6_meta/healing/nt_mind_consciousness_monitor.rs` | `410:        crate::l5_cognition::traits::ConsciousnessAwarenessReport {` |

---

## 统计摘要

- **总违规数**: 190

### 各层违规分布

| 源层 | 违规数 |
|------|--------|
| l1_action | 5 |
| l2_perception | 90 |
| l3_embodiment | 3 |
| l4_emotion | 0 |
| l5_cognition | 81 |
| l6_meta | 11 |

---

## 修复建议

1. **Facade 模式**: 使用层内 facade 文件作为对外接口
2. **Trait 抽象**: 通过 trait 定义跨层依赖，减少直接模块引用
3. **依赖注入**: 将跨层依赖通过参数传入，降低耦合
