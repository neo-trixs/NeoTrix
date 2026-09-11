# Cross-Layer Import Scan — neotrix-core 6-Layer Architecture

**Scan date**: 2026-09-11 19:26
**Scope**: `neotrix-core/src/l{1..6}_*/`
**Exclusions**: test modules (`*test*`), facade files (`*facade*`)

---

## Summary

| Direction | Count |
|-----------|-------|
| `l1_action → l3_embodiment` | 3 |
| `l1_action → l5_cognition` | 2 |
| `l2_perception → l1_action` | 62 |
| `l2_perception → l3_embodiment` | 75 |
| `l3_embodiment → l1_action` | 3 |
| `l5_cognition → l1_action` | 115 |
| `l5_cognition → l2_perception` | 3 |
| `l5_cognition → l3_embodiment` | 12 |
| `l5_cognition → l4_emotion` | 1 |
| `l5_cognition → l6_meta` | 21 |
| `l6_meta → l1_action` | 1 |
| `l6_meta → l5_cognition` | 16 |
| **Total** | **314** |

## `l1_action → l3_embodiment` (3)

```
l1_action/nt_io/nt_io_provider/factory.rs:530:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow => true,
l1_action/nt_io/nt_io_provider/factory.rs:531:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::RequireConfirmation => {
l1_action/nt_io/nt_io_provider/factory.rs:535:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Deny => {
```

## `l1_action → l5_cognition` (2)

```
l1_action/nt_io/nt_io_neocodex/agent.rs:110:        Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>>,
l1_action/nt_io/nt_io_neocodex/agent.rs:233:            tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>,
```

## `l2_perception → l1_action` (62)

```
l2_perception/nt_world/crawl/unified.rs:284:                    crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Source,
l2_perception/nt_world/crawl/unified.rs:733:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Source,
l2_perception/nt_world/nt_world_adsb.rs:129:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_adsb.rs:138:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_adsb.rs:145:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_adsb.rs:245:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_aoi.rs:204:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_aoi.rs:213:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_aoi.rs:220:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_aoi.rs:374:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_bgpview.rs:124:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_bgpview.rs:228:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_edgar.rs:359:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_edgar.rs:371:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_edgar.rs:381:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_edgar.rs:618:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
l2_perception/nt_world/nt_world_edgar.rs:632:        let hits = kb.search_permission_aware("Apple", 10, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::Public).expect("search");
l2_perception/nt_world/nt_world_edgar.rs:652:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
l2_perception/nt_world/nt_world_gdacs.rs:143:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_gdacs.rs:152:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_gdacs.rs:159:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_gdacs.rs:263:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_gdelt.rs:137:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_gdelt.rs:148:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_gdelt.rs:158:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_gdelt.rs:346:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
l2_perception/nt_world/nt_world_gdelt.rs:359:        let hits = kb.search_permission_aware("healthcare", 10, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::Public).expect("search");
l2_perception/nt_world/nt_world_gdelt.rs:379:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
l2_perception/nt_world/nt_world_ofac.rs:135:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_ofac.rs:144:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_ofac.rs:151:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_ofac.rs:250:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_opencorporates.rs:125:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_opencorporates.rs:233:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_polymarket.rs:113:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_polymarket.rs:122:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_polymarket.rs:129:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_polymarket.rs:227:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_ucdp.rs:207:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_ucdp.rs:216:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_ucdp.rs:223:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_ucdp.rs:352:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_urlhaus.rs:157:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_urlhaus.rs:165:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_urlhaus.rs:236:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_urlhaus.rs:358:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/nt_world_usgs.rs:284:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_usgs.rs:293:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_usgs.rs:300:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
l2_perception/nt_world/nt_world_usgs.rs:409:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
l2_perception/nt_world/osint/mod.rs:1094:        let (html, host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)?;
l2_perception/nt_world/osint/mod.rs:1095:        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(&html);
l2_perception/nt_world/osint/mod.rs:1109:        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(&html, url);
l2_perception/nt_world/osint/mod.rs:1176:        let r = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::run_crawl_cycle(conn, max)?;
l2_perception/nt_world/osint/mod.rs:1189:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::discover_from_seed(conn, topic)
l2_perception/nt_world/osint/mod.rs:1200:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::upsert_crawl_queue(conn, url, depth, domain, priority, ts)
l2_perception/nt_world/osint/mod.rs:1203:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes(conn)
l2_perception/nt_world/osint/mod.rs:1206:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes_by_type(conn, node_type)
l2_perception/nt_world/osint/mod.rs:1273:        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(
l2_perception/nt_world/osint/mod.rs:1286:        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html);
l2_perception/nt_world/osint/mod.rs:1296:        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(html, "");
l2_perception/nt_world/osint/mod.rs:1305:        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(html, "");
```

## `l2_perception → l3_embodiment` (75)

```
l2_perception/nt_world/nt_world_adsb.rs:175:pub fn adsb_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_adsb.rs:176:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(ADSB_HOST, "443")
l2_perception/nt_world/nt_world_adsb.rs:178:pub fn adsb_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_adsb.rs:179:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![adsb_egress_rule()], false)
l2_perception/nt_world/nt_world_aoi.rs:269:pub fn aoi_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_aoi.rs:270:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(AOI_HOST, "443")
l2_perception/nt_world/nt_world_aoi.rs:272:pub fn aoi_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_aoi.rs:273:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![aoi_egress_rule()], false)
l2_perception/nt_world/nt_world_bgpview.rs:151:pub fn bgpview_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_bgpview.rs:152:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(BGPVIEW_HOST, "443")
l2_perception/nt_world/nt_world_bgpview.rs:154:pub fn bgpview_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_bgpview.rs:155:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![bgpview_egress_rule()], false)
l2_perception/nt_world/nt_world_edgar.rs:434:pub fn edgar_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_edgar.rs:435:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(EDGAR_HOST, "443")
l2_perception/nt_world/nt_world_edgar.rs:438:pub fn edgar_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_edgar.rs:439:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![edgar_egress_rule()], false)
l2_perception/nt_world/nt_world_edgar.rs:599:        with_deny.rules.push(crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::deny(EDGAR_HOST, "443"));
l2_perception/nt_world/nt_world_gdacs.rs:194:pub fn gdacs_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_gdacs.rs:195:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(GDACS_HOST, "443")
l2_perception/nt_world/nt_world_gdacs.rs:197:pub fn gdacs_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_gdacs.rs:198:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![gdacs_egress_rule()], false)
l2_perception/nt_world/nt_world_gdelt.rs:209:pub const GDELT_HOST: &str = crate::l3_embodiment::nt_shield::nt_shield_sandbox::INTEL_GDELT_HOST;
l2_perception/nt_world/nt_world_gdelt.rs:211:pub fn gdelt_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_gdelt.rs:212:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(GDELT_HOST, "443")
l2_perception/nt_world/nt_world_gdelt.rs:215:pub fn gdelt_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_gdelt.rs:216:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::intel_egress_policy()
l2_perception/nt_world/nt_world_gdelt.rs:328:        with_deny.rules.push(crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::deny(GDELT_HOST, "443"));
l2_perception/nt_world/nt_world_ofac.rs:180:pub fn ofac_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_ofac.rs:181:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(OFAC_HOST, "443")
l2_perception/nt_world/nt_world_ofac.rs:183:pub fn ofac_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_ofac.rs:184:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![ofac_egress_rule()], false)
l2_perception/nt_world/nt_world_opencorporates.rs:156:pub fn opencorporates_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_opencorporates.rs:157:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(OPENCORPORATES_HOST, "443")
l2_perception/nt_world/nt_world_opencorporates.rs:159:pub fn opencorporates_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_opencorporates.rs:160:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![opencorporates_egress_rule()], false)
l2_perception/nt_world/nt_world_polymarket.rs:158:pub fn polymarket_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_polymarket.rs:159:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(POLYMARKET_HOST, "443")
l2_perception/nt_world/nt_world_polymarket.rs:161:pub fn polymarket_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_polymarket.rs:162:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![polymarket_egress_rule()], false)
l2_perception/nt_world/nt_world_ucdp.rs:274:pub fn ucdp_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_ucdp.rs:275:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(UCDP_HOST, "443")
l2_perception/nt_world/nt_world_ucdp.rs:277:pub fn ucdp_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_ucdp.rs:278:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![ucdp_egress_rule()], false)
l2_perception/nt_world/nt_world_urlhaus.rs:267:pub fn urlhaus_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_urlhaus.rs:268:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(URLHAUS_HOST, "443")
l2_perception/nt_world/nt_world_urlhaus.rs:270:pub fn urlhaus_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_urlhaus.rs:271:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![urlhaus_egress_rule()], false)
l2_perception/nt_world/nt_world_urlhaus.rs:273:pub fn cisa_kev_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_urlhaus.rs:274:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(CISA_KEV_HOST, "443")
l2_perception/nt_world/nt_world_urlhaus.rs:276:pub fn cisa_kev_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_urlhaus.rs:277:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![cisa_kev_egress_rule()], false)
l2_perception/nt_world/nt_world_usgs.rs:338:pub fn usgs_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/nt_world_usgs.rs:339:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(USGS_HOST, "443")
l2_perception/nt_world/nt_world_usgs.rs:341:pub fn usgs_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/nt_world_usgs.rs:342:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![usgs_egress_rule()], false)
l2_perception/nt_world/osint/censys.rs:141:pub fn censys_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/osint/censys.rs:142:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(CENSYS_API_HOST, "443")
l2_perception/nt_world/osint/censys.rs:144:pub fn censys_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/osint/censys.rs:145:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![censys_egress_rule()], false)
l2_perception/nt_world/osint/fofa.rs:549:pub fn fofa_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/osint/fofa.rs:550:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(FOFA_API_HOST, "443")
l2_perception/nt_world/osint/fofa.rs:553:pub fn fofa_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/osint/fofa.rs:554:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(
l2_perception/nt_world/osint/securitytrails.rs:225:pub fn securitytrails_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/osint/securitytrails.rs:226:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(SECURITYTRAILS_API_HOST, "443")
l2_perception/nt_world/osint/securitytrails.rs:228:pub fn securitytrails_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/osint/securitytrails.rs:229:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![securitytrails_egress_rule()], false)
l2_perception/nt_world/osint/shodan.rs:173:pub fn shodan_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/osint/shodan.rs:174:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(SHODAN_API_HOST, "443")
l2_perception/nt_world/osint/shodan.rs:176:pub fn shodan_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/osint/shodan.rs:177:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![shodan_egress_rule()], false)
l2_perception/nt_world/osint/zoomeye.rs:143:pub fn zoomeye_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
l2_perception/nt_world/osint/zoomeye.rs:144:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(ZOOMEYE_API_HOST, "443")
l2_perception/nt_world/osint/zoomeye.rs:146:pub fn zoomeye_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
l2_perception/nt_world/osint/zoomeye.rs:147:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![zoomeye_egress_rule()], false)
```

## `l3_embodiment → l1_action` (3)

```
l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs:260:fn to_anthropic_response(resp: &crate::l1_action::nt_io::nt_io_provider::types::LlmResponse, model: &str) -> AnthropicResponse {
l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs:463:        let resp = crate::l1_action::nt_io::nt_io_provider::types::LlmResponse {
l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs:466:            usage: crate::l1_action::nt_io::nt_io_provider::types::Usage {
```

## `l5_cognition → l1_action` (115)

```
l5_cognition/nt_core/nt_core_parallel/contract.rs:195:    kb: Option<crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase>,
l5_cognition/nt_core/nt_core_parallel/contract.rs:210:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None).ok();
l5_cognition/nt_mind/evolution/evolution_loop.rs:1580:impl crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::EvolutionLoopProvider for EvolutionLoop {
l5_cognition/nt_mind/evolution/evolution_loop.rs:1581:    fn get_snapshot(&self) -> crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::ProjectSnapshotLite {
l5_cognition/nt_mind/evolution/evolution_loop.rs:1583:        crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::ProjectSnapshotLite {
l5_cognition/nt_mind/evolution/evolution_loop.rs:1600:    fn self_diagnose(&mut self) -> (Vec<String>, Vec<crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue>) {
l5_cognition/nt_mind/evolution/evolution_loop.rs:1602:        let l1_issues: Vec<crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue> = pq.into_vec().into_iter().map(|di| {
l5_cognition/nt_mind/evolution/evolution_loop.rs:1603:            crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue {
l5_cognition/nt_mind/evolution/evolution_loop.rs:1604:                issue: crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::Issue {
l5_cognition/nt_mind/evolution/evolution_loop.rs:1611:                    ActionPlan::AddTestStub { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::AddTestStub { file: file.clone() },
l5_cognition/nt_mind/evolution/evolution_loop.rs:1612:                    ActionPlan::RunCargoFix => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::RunCargoFix,
l5_cognition/nt_mind/evolution/evolution_loop.rs:1613:                    ActionPlan::RemoveTodo { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::RemoveTodo { file: file.clone() },
l5_cognition/nt_mind/evolution/evolution_loop.rs:1614:                    ActionPlan::SplitLargeFile { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::SplitLargeFile { file: file.clone() },
l5_cognition/nt_mind/evolution/evolution_loop.rs:1615:                    ActionPlan::ReviewUnsafe { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ReviewUnsafe { file: file.clone() },
l5_cognition/nt_mind/evolution/evolution_loop.rs:1616:                    ActionPlan::ReplaceUnwrap { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ReplaceUnwrap { file: file.clone() },
l5_cognition/nt_mind/evolution/evolution_loop.rs:1617:                    ActionPlan::HumanDecision { reason, options } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::HumanDecision { reason: reason.clone(), options: options.clone() },
l5_cognition/nt_mind/evolution/evolution_loop.rs:1618:                    ActionPlan::NoAction { reason } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::NoAction { reason: reason.clone() },
l5_cognition/nt_mind/evolution/evolution_loop.rs:1619:                    ActionPlan::AutoFix(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::AutoFix(s.clone()),
l5_cognition/nt_mind/evolution/evolution_loop.rs:1620:                    ActionPlan::ManualReview(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ManualReview(s.clone()),
l5_cognition/nt_mind/evolution/evolution_loop.rs:1621:                    ActionPlan::Skip(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::Skip(s.clone()),
l5_cognition/nt_mind/evolution/evolution_loop.rs:21:// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;
l5_cognition/nt_mind/evolution/evolution_loop.rs:67:pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
l5_cognition/nt_mind/evolution/self_diagnose.rs:10:use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
l5_cognition/nt_mind/evolution/self_diagnose.rs:33:// pub use crate::l1_action::nt_act::nt_l1_shared_types::{
l5_cognition/nt_mind/evolution/self_diagnose.rs:486:        let snapshot = crate::l1_action::nt_act::nt_act_types::ProjectSnapshot {
l5_cognition/nt_mind/evolution/self_diagnose.rs:70:    fn get_snapshot(&self) -> crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
l5_cognition/nt_mind/foundation/knowledge_store.rs:106:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::mark_crawl_complete(&conn, id, success, error)
l5_cognition/nt_mind/foundation/knowledge_store.rs:112:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes_by_domain(&conn)
l5_cognition/nt_mind/foundation/knowledge_store.rs:118:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::enqueue_seed_urls(&conn, urls)
l5_cognition/nt_mind/foundation/knowledge_store.rs:123:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html)
l5_cognition/nt_mind/foundation/knowledge_store.rs:127:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::is_safe_fetch_url(url)
l5_cognition/nt_mind/foundation/knowledge_store.rs:72:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html)
l5_cognition/nt_mind/foundation/knowledge_store.rs:76:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::is_safe_fetch_url(url)
l5_cognition/nt_mind/foundation/knowledge_store.rs:88:        let item = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::claim_next_crawl_url(&conn)
l5_cognition/nt_mind/foundation/l1_wrappers.rs:102:    inner: std::sync::Mutex<crate::l1_action::nt_io::nt_io_user_avatar::DistillationEngine>,
l5_cognition/nt_mind/foundation/l1_wrappers.rs:108:            inner: std::sync::Mutex::new(crate::l1_action::nt_io::nt_io_user_avatar::DistillationEngine::new()),
l5_cognition/nt_mind/foundation/l1_wrappers.rs:37:    inner: crate::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager,
l5_cognition/nt_mind/foundation/l1_wrappers.rs:43:            inner: crate::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager::new(session_id),
l5_cognition/nt_mind/foundation/seal_pipeline.rs:103:            inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
l5_cognition/nt_mind/foundation/seal_pipeline.rs:127:            inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
l5_cognition/nt_mind/foundation/seal_pipeline.rs:145:    inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:151:            inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
l5_cognition/nt_mind/foundation/seal_pipeline.rs:158:        crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::compute_entropy(prompt, context)
l5_cognition/nt_mind/foundation/seal_pipeline.rs:171:            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Increasing => EntropyTrend::Increasing,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:172:            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Decreasing => EntropyTrend::Decreasing,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:173:            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Stable => EntropyTrend::Stable,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:179:            inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
l5_cognition/nt_mind/foundation/seal_pipeline.rs:197:    inner: crate::l1_action::nt_act::actions::sandbox::ActionSandbox,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:203:            inner: crate::l1_action::nt_act::actions::sandbox::ActionSandbox::new(),
l5_cognition/nt_mind/foundation/seal_pipeline.rs:211:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::Approved => SandboxVerdict::Approved,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:212:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::Denied => SandboxVerdict::Denied,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:213:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::RequiresApproval => SandboxVerdict::RequiresApproval,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:219:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::Approved => SandboxVerdict::Approved,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:220:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::Denied => SandboxVerdict::Denied,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:221:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::RequiresApproval => SandboxVerdict::RequiresApproval,
l5_cognition/nt_mind/foundation/seal_pipeline.rs:235:            inner: crate::l1_action::nt_act::actions::sandbox::ActionSandbox::new(),
l5_cognition/nt_mind/foundation/seal_pipeline.rs:97:    inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate,
l5_cognition/nt_mind/nt_mind/auto_crystallizer.rs:74:        let now = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_diversity::now_unix_secs() as u64;
l5_cognition/nt_mind/nt_mind/co_evolution.rs:586:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.into())).expect("open kb");
l5_cognition/nt_mind/nt_mind/consciousness/hypercube_bridge.rs:97:    fn coord_from_kb_node(node: &crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::KnowledgeNode) -> HyperCoord {
l5_cognition/nt_mind/nt_mind/evolution/agent_capability/mod.rs:133:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::default(),
l5_cognition/nt_mind/nt_mind/evolution/co_evolution.rs:586:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.into())).expect("open kb");
l5_cognition/nt_mind/nt_mind/evolution/self_evolver.rs:215:        let (body, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)
l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs:516:        let mut ledger = crate::l1_action::nt_memory::evidence_ledger::SessionLedger::new(session_id);
l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs:518:            crate::l1_action::nt_memory::evidence_ledger::EvidenceType::Observation,
l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs:110:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&search_url) {
l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs:32:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&url) {
l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs:78:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&url) {
l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs:203:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)
l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs:238:        let (xml_text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)
l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs:289:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_with_headers(
l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs:363:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url_str)
l5_cognition/nt_mind/nt_mind/reason/attention_router.rs:657:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Insight,
l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs:426:            // let verification_score = crate::l1_action::nt_io::nt_io_standalone::verify_answer(gold, response);
l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs:440:                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs:480:                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs:499:                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/loop_impl/seal_loop.rs:1000:            crate::l1_action::nt_io::nt_io_provider::gateway::register_gateway_for_re_evaluation(&gw);
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:1959:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionPatternType::RecurringError
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:1961:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionPatternType::StrategyDiscovery
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:1968:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionRecord {
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2297:        let corpus = match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_resource_ingest::corpus_archive_path() {
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2308:        let kb = match crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2325:        match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_cortex_sync::digest_sample(
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2336:        match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_cortex_sync::prune_external(
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3087:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3132:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_write_guard::WriteGuardAudit,
l5_cognition/nt_mind/nt_mind/self_evolver.rs:215:        let (body, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)
l5_cognition/nt_mind/nt_mind/web_miner.rs:203:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)
l5_cognition/nt_mind/nt_mind/web_miner.rs:238:        let (xml_text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)
l5_cognition/nt_mind/nt_mind/web_miner.rs:289:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_with_headers(
l5_cognition/nt_mind/nt_mind/web_miner.rs:363:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url_str)
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1476:                &crate::l1_action::nt_memory::nt_memory_kb::nt_memory_galaxy_hygiene::GalaxyHygieneConfig::default(),
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1594:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1600:            crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1603:            crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1606:            crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox::new(),
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2081:            crate::l1_action::nt_act::nt_act_autonomy::cross_session_memory::CrossSessionMemorySelfTest
l5_cognition/nt_mind/nt_mind_background_loop/handlers_core.rs:373:        let _ = kb.insert_or_get_node(&title, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Session, Some(summary), None, Some("neotrix"));
l5_cognition/nt_mind/nt_mind_background_loop/handlers_daily_intel.rs:54:                let _ = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::kv_set(
l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:679:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch};
l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:680:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{
l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:761:            if let Ok(clusters) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::list_clusters(&conn) {
l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:769:                crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::list_clusters(&conn)
l5_cognition/nt_mind/nt_mind_background_loop/knowledge_pipeline.rs:123:            crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)?;
l5_cognition/nt_mind/nt_mind_background_loop/knowledge_pipeline.rs:138:            crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_async(url)
l5_cognition/nt_mind/nt_mind_background_loop/mod.rs:134:        if let Ok(kb) = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
l5_cognition/nt_mind/nt_mind_background_loop/run.rs:1074:            crate::l1_action::nt_io::nt_io_provider::gateway::run_periodic_re_evaluation();
l5_cognition/nt_mind/nt_mind_background_loop/run.rs:438:            let _ = kb_ref.insert_or_get_node(&title, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Session, Some(&summary), None, Some("neotrix"));
l5_cognition/nt_mind/nt_mind_skill_engine.rs:1652:                last_indexed_at: Some(crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now()),
l5_cognition/nt_mind/nt_mind_skill_engine.rs:1653:                created_at: crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now(),
l5_cognition/nt_mind/nt_mind_skill_engine.rs:1654:                updated_at: crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now(),
l5_cognition/nt_mind/nt_mind_skill_engine.rs:328:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.content);
l5_cognition/nt_mind/nt_mind_skill_engine.rs:330:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(body);
l5_cognition/nt_mind/nt_mind_skill_engine.rs:332:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.description);
```

## `l5_cognition → l2_perception` (3)

```
l5_cognition/mod.rs:16:/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2252:            crate::l2_perception::nt_world::nt_world_exploration_engine::ExplorationConfig::default(
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2255:            crate::l2_perception::nt_world::nt_world_exploration_engine::ExplorationEngine::new(
```

## `l5_cognition → l3_embodiment` (12)

```
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3104:        registry.register(crate::l3_embodiment::nt_shield::nt_shield::browser_security::create_browser_security_self_test());
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3105:        registry.register(crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test());
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1451:            let auditor = crate::l3_embodiment::nt_shield::nt_shield_audit::create_reasoning_trace_auditor();
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1620:        self_tests.register(crate::l3_embodiment::nt_shield::nt_shield::browser_security::create_browser_security_self_test());
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1621:        self_tests.register(crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test());
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1626:            crate::l3_embodiment::nt_shield::nt_shield_audit::CohGuard::new(
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1633:            crate::l3_embodiment::nt_shield::nt_shield_audit::ReasoningTraceGuard::default(),
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2025:            crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test()
l5_cognition/nt_mind/nt_mind_skill_engine.rs:1558:                                crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::scan_skill_content(&skill.content);
l5_cognition/nt_mind/nt_mind_skill_engine.rs:1559:                            let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::InspectionResult::Allow);
l5_cognition/nt_mind/nt_mind_skill_engine.rs:1590:                            crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::scan_skill_content(&skill.content);
l5_cognition/nt_mind/nt_mind_skill_engine.rs:1591:                        let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::InspectionResult::Allow);
```

## `l5_cognition → l4_emotion` (1)

```
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3100:        //     crate::l4_emotion::nt_feel::nt_core_fep_iit::bridge::FEPIITBridge::new(),
```

## `l5_cognition → l6_meta` (21)

```
l5_cognition/mod.rs:22:/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3079:            let mut cm = crate::l6_meta::nt_repair::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();
l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3102:        registry.register(Box::new(crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1143:            let mut inspector = crate::l6_meta::nt_meta::auto_inspector::AutoInspector::new();
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1615:            crate::l6_meta::memory::evolution_harness::EvolutionHarness::new(
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1616:                crate::l6_meta::memory::transcendent_loop::LoopConfig::default(),
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1656:        //     crate::l6_meta::nt_repair::nt_mind_causal_trace::CausalTraceSelfTest,
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1788:            let mut cm = crate::l6_meta::nt_repair::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1815:            self_tests.register(Box::new(crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:197:                    //         let input = crate::l6_meta::nt_meta::nt_core_intra_reflection::ReflectionInput {
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2042:        //     crate::l6_meta::nt_repair::nt_mind_causal_trace::CausalTraceSelfTest
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2054:            crate::l6_meta::memory::meta_observer::MetaObserverSelfTest
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:205:                    //             crate::l6_meta::nt_meta::nt_core_intra_reflection::analyze(
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:266:        let (infos, _problems): (Vec<_>, Vec<_>) = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::harness_infos_from_registry_export(&json);
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:272:        let mut harness = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::new_harness();
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:276:        let actionable = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::harness_actionable_suggestions(&report, 0.7);
l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:962:            let hexagram_states: Vec<crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::E8HexagramState> = Vec::new();
l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:940:        use crate::l6_meta::coordination::self_improvement::SystemMetrics;
l5_cognition/nt_mind/nt_mind_background_loop/run.rs:1028:    self_improvement: crate::l6_meta::coordination::self_improvement::SelfImprovementLoop,
l5_cognition/nt_mind/nt_mind_background_loop/run.rs:717:            self_improvement: crate::l6_meta::coordination::self_improvement::SelfImprovementLoop::new(),
l5_cognition/traits.rs:137:// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
```

## `l6_meta → l1_action` (1)

```
l6_meta/memory/evolution_harness.rs:120:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
```

## `l6_meta → l5_cognition` (16)

```
l6_meta/healing/nt_mind_consciousness_gold_standard.rs:229:impl crate::l5_cognition::traits::GoldStandardApi for ConsciousnessGoldStandard {}
l6_meta/healing/nt_mind_consciousness_monitor.rs:399:impl crate::l5_cognition::traits::ConsciousnessMonitorApi for ConsciousnessMonitor {
l6_meta/healing/nt_mind_consciousness_monitor.rs:408:    fn get_report(&self) -> crate::l5_cognition::traits::ConsciousnessAwarenessReport {
l6_meta/healing/nt_mind_consciousness_monitor.rs:410:        crate::l5_cognition::traits::ConsciousnessAwarenessReport {
l6_meta/healing/nt_mind_eval_harness.rs:1267:impl crate::l5_cognition::traits::EvalHarnessApi for EvalHarness {
l6_meta/healing/nt_mind_eval_harness.rs:1268:    fn generate_regression_test(&self, candidate: &str) -> crate::l5_cognition::traits::RegressionCase {
l6_meta/healing/nt_mind_eval_harness.rs:1270:        crate::l5_cognition::traits::RegressionCase {
l6_meta/healing/nt_mind_eval_harness.rs:1278:    fn run_regression_test(&self, case: &crate::l5_cognition::traits::RegressionCase) -> crate::l5_cognition::traits::RegressionResult {
l6_meta/healing/nt_mind_eval_harness.rs:1286:        crate::l5_cognition::traits::RegressionResult {
l6_meta/memory/evolution_harness.rs:200:impl crate::l5_cognition::traits::EvolutionHarnessApi for EvolutionHarness {
l6_meta/memory/evolution_harness.rs:205:    fn harness_infos_from_registry_export(json: &str) -> (Vec<crate::l5_cognition::traits::RegistryNodeInfo>, Vec<String>) {
l6_meta/memory/evolution_harness.rs:207:        let infos = l6_infos.into_iter().map(|i| crate::l5_cognition::traits::RegistryNodeInfo {
l6_meta/memory/evolution_harness.rs:220:        infos: &[crate::l5_cognition::traits::RegistryNodeInfo],
l6_meta/memory/evolution_harness.rs:237:        kb: &crate::l5_cognition::kb_facade::KnowledgeBase,
l6_meta/memory/evolution_harness.rs:253:    ) -> Vec<crate::l5_cognition::traits::RegistrySuggestion> {
l6_meta/memory/evolution_harness.rs:260:            .map(|s| crate::l5_cognition::traits::RegistrySuggestion {
```

---

## Methodology

- `grep -n` for `use crate::l{N}::`, `use crate::neotrix::l{N}::`, `mod l{N}::`, `crate::l{N}::`
- Excluded files matching `*test*` or `*facade*` in path/name
- Lines within 5 lines of `#[cfg(test)]` excluded
- Only direct cross-layer references (X importing Y, X≠Y)
