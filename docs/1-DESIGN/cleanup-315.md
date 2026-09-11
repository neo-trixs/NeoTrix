# Cross-Layer Reference Scan — cleanup-315

**Scan date**: 2026-09-11 20:00:57
**Source**: `/Users/neo/Downloads/neotrix/neotrix-core/src`
**Exclusions**: test files, facade files

## l1_action → other layers

### → l3_embodiment (3 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l1_action/nt_io/nt_io_provider/factory.rs:530:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow => true,
/Users/neo/Downloads/neotrix/neotrix-core/src/l1_action/nt_io/nt_io_provider/factory.rs:531:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::RequireConfirmation => {
/Users/neo/Downloads/neotrix/neotrix-core/src/l1_action/nt_io/nt_io_provider/factory.rs:535:            crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Deny => {
```

### → l5_cognition (2 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l1_action/nt_io/nt_io_neocodex/agent.rs:110:        Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>>,
/Users/neo/Downloads/neotrix/neotrix-core/src/l1_action/nt_io/nt_io_neocodex/agent.rs:233:            tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>,
```

## l2_perception → other layers

### → l1_action (62 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:157:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:165:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:236:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:358:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_usgs.rs:284:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_usgs.rs:293:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_usgs.rs:300:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_usgs.rs:409:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_bgpview.rs:124:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_bgpview.rs:228:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:137:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:148:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:158:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:346:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:359:        let hits = kb.search_permission_aware("healthcare", 10, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::Public).expect("search");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:379:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ucdp.rs:207:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ucdp.rs:216:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ucdp.rs:223:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ucdp.rs:352:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_aoi.rs:204:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_aoi.rs:213:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_aoi.rs:220:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_aoi.rs:374:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1094:        let (html, host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)?;
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1095:        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(&html);
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1109:        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(&html, url);
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1176:        let r = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::run_crawl_cycle(conn, max)?;
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1189:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::discover_from_seed(conn, topic)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1200:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::upsert_crawl_queue(conn, url, depth, domain, priority, ts)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1203:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes(conn)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1206:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes_by_type(conn, node_type)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1273:        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1286:        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html);
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1296:        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(html, "");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/mod.rs:1305:        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(html, "");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/crawl/unified.rs:284:                    crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Source,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/crawl/unified.rs:733:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Source,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdacs.rs:143:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdacs.rs:152:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdacs.rs:159:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdacs.rs:263:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ofac.rs:135:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ofac.rs:144:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ofac.rs:151:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ofac.rs:250:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_polymarket.rs:113:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_polymarket.rs:122:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_polymarket.rs:129:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_polymarket.rs:227:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_opencorporates.rs:125:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_opencorporates.rs:233:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_adsb.rs:129:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_adsb.rs:138:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_adsb.rs:145:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_adsb.rs:245:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:359:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:371:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:381:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:618:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:632:        let hits = kb.search_permission_aware("Apple", 10, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::Public).expect("search");
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:652:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
```

### → l3_embodiment (75 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:267:pub fn urlhaus_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:268:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(URLHAUS_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:270:pub fn urlhaus_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:271:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![urlhaus_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:273:pub fn cisa_kev_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:274:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(CISA_KEV_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:276:pub fn cisa_kev_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_urlhaus.rs:277:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![cisa_kev_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_usgs.rs:338:pub fn usgs_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_usgs.rs:339:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(USGS_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_usgs.rs:341:pub fn usgs_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_usgs.rs:342:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![usgs_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_bgpview.rs:151:pub fn bgpview_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_bgpview.rs:152:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(BGPVIEW_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_bgpview.rs:154:pub fn bgpview_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_bgpview.rs:155:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![bgpview_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:209:pub const GDELT_HOST: &str = crate::l3_embodiment::nt_shield::nt_shield_sandbox::INTEL_GDELT_HOST;
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:211:pub fn gdelt_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:212:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(GDELT_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:215:pub fn gdelt_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:216:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::intel_egress_policy()
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdelt.rs:328:        with_deny.rules.push(crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::deny(GDELT_HOST, "443"));
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ucdp.rs:274:pub fn ucdp_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ucdp.rs:275:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(UCDP_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ucdp.rs:277:pub fn ucdp_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ucdp.rs:278:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![ucdp_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_aoi.rs:269:pub fn aoi_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_aoi.rs:270:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(AOI_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_aoi.rs:272:pub fn aoi_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_aoi.rs:273:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![aoi_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/securitytrails.rs:225:pub fn securitytrails_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/securitytrails.rs:226:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(SECURITYTRAILS_API_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/securitytrails.rs:228:pub fn securitytrails_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/securitytrails.rs:229:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![securitytrails_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/fofa.rs:549:pub fn fofa_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/fofa.rs:550:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(FOFA_API_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/fofa.rs:553:pub fn fofa_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/fofa.rs:554:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/shodan.rs:173:pub fn shodan_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/shodan.rs:174:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(SHODAN_API_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/shodan.rs:176:pub fn shodan_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/shodan.rs:177:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![shodan_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/zoomeye.rs:143:pub fn zoomeye_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/zoomeye.rs:144:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(ZOOMEYE_API_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/zoomeye.rs:146:pub fn zoomeye_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/zoomeye.rs:147:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![zoomeye_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/censys.rs:141:pub fn censys_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/censys.rs:142:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(CENSYS_API_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/censys.rs:144:pub fn censys_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/osint/censys.rs:145:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![censys_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdacs.rs:194:pub fn gdacs_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdacs.rs:195:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(GDACS_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdacs.rs:197:pub fn gdacs_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_gdacs.rs:198:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![gdacs_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ofac.rs:180:pub fn ofac_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ofac.rs:181:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(OFAC_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ofac.rs:183:pub fn ofac_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_ofac.rs:184:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![ofac_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_polymarket.rs:158:pub fn polymarket_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_polymarket.rs:159:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(POLYMARKET_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_polymarket.rs:161:pub fn polymarket_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_polymarket.rs:162:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![polymarket_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_opencorporates.rs:156:pub fn opencorporates_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_opencorporates.rs:157:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(OPENCORPORATES_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_opencorporates.rs:159:pub fn opencorporates_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_opencorporates.rs:160:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![opencorporates_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_adsb.rs:175:pub fn adsb_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_adsb.rs:176:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(ADSB_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_adsb.rs:178:pub fn adsb_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_adsb.rs:179:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![adsb_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:434:pub fn edgar_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:435:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(EDGAR_HOST, "443")
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:438:pub fn edgar_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:439:    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![edgar_egress_rule()], false)
/Users/neo/Downloads/neotrix/neotrix-core/src/l2_perception/nt_world/nt_world_edgar.rs:599:        with_deny.rules.push(crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::deny(EDGAR_HOST, "443"));
```

## l3_embodiment → other layers

### → l1_action (3 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs:260:fn to_anthropic_response(resp: &crate::l1_action::nt_io::nt_io_provider::types::LlmResponse, model: &str) -> AnthropicResponse {
/Users/neo/Downloads/neotrix/neotrix-core/src/l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs:463:        let resp = crate::l1_action::nt_io::nt_io_provider::types::LlmResponse {
/Users/neo/Downloads/neotrix/neotrix-core/src/l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs:466:            usage: crate::l1_action::nt_io::nt_io_provider::types::Usage {
```

## l4_emotion → other layers

> No cross-layer imports found.

## l5_cognition → other layers

### → l1_action (115 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_core.rs:373:        let _ = kb.insert_or_get_node(&title, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Session, Some(summary), None, Some("neotrix"));
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs:438:            let _ = kb_ref.insert_or_get_node(&title, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Session, Some(&summary), None, Some("neotrix"));
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs:1074:            crate::l1_action::nt_io::nt_io_provider::gateway::run_periodic_re_evaluation();
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/mod.rs:134:        if let Ok(kb) = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:679:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch};
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:680:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:761:            if let Ok(clusters) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::list_clusters(&conn) {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:769:                crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::list_clusters(&conn)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_daily_intel.rs:54:                let _ = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::kv_set(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1476:                &crate::l1_action::nt_memory::nt_memory_kb::nt_memory_galaxy_hygiene::GalaxyHygieneConfig::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1594:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1600:            crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1603:            crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1606:            crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2081:            crate::l1_action::nt_act::nt_act_autonomy::cross_session_memory::CrossSessionMemorySelfTest
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/knowledge_pipeline.rs:123:            crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)?;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/knowledge_pipeline.rs:138:            crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_async(url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/web_miner.rs:203:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/web_miner.rs:238:        let (xml_text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/web_miner.rs:289:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_with_headers(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/web_miner.rs:363:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url_str)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/self_evolver.rs:215:        let (body, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/consciousness/hypercube_bridge.rs:97:    fn coord_from_kb_node(node: &crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::KnowledgeNode) -> HyperCoord {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:1959:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionPatternType::RecurringError
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:1961:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionPatternType::StrategyDiscovery
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:1968:                            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::EvolutionRecord {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2297:        let corpus = match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_resource_ingest::corpus_archive_path() {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2308:        let kb = match crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2325:        match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_cortex_sync::digest_sample(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2336:        match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_cortex_sync::prune_external(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3087:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3132:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_write_guard::WriteGuardAudit,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/loop_impl/seal_loop.rs:1000:            crate::l1_action::nt_io::nt_io_provider::gateway::register_gateway_for_re_evaluation(&gw);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs:516:        let mut ledger = crate::l1_action::nt_memory::evidence_ledger::SessionLedger::new(session_id);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs:518:            crate::l1_action::nt_memory::evidence_ledger::EvidenceType::Observation,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs:203:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs:238:        let (xml_text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&api_url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs:289:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_with_headers(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge/web_miner.rs:363:        let (text, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url_str)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs:32:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&url) {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs:78:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&url) {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/search.rs:110:        match crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(&search_url) {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/co_evolution.rs:586:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.into())).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/evolution/agent_capability/mod.rs:133:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/evolution/self_evolver.rs:215:        let (body, _host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/evolution/co_evolution.rs:586:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.into())).expect("open kb");
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/attention_router.rs:657:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Insight,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs:426:            // let verification_score = crate::l1_action::nt_io::nt_io_standalone::verify_answer(gold, response);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs:440:                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs:480:                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs:499:                    method: crate::l1_action::nt_io::nt_io_standalone::ReasoningMethod::Deductive,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/auto_crystallizer.rs:74:        let now = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_diversity::now_unix_secs() as u64;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:97:    inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:103:            inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:127:            inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:145:    inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:151:            inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:158:        crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::compute_entropy(prompt, context)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:171:            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Increasing => EntropyTrend::Increasing,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:172:            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Decreasing => EntropyTrend::Decreasing,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:173:            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Stable => EntropyTrend::Stable,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:179:            inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:197:    inner: crate::l1_action::nt_act::actions::sandbox::ActionSandbox,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:203:            inner: crate::l1_action::nt_act::actions::sandbox::ActionSandbox::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:211:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::Approved => SandboxVerdict::Approved,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:212:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::Denied => SandboxVerdict::Denied,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:213:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::RequiresApproval => SandboxVerdict::RequiresApproval,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:219:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::Approved => SandboxVerdict::Approved,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:220:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::Denied => SandboxVerdict::Denied,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:221:            crate::l1_action::nt_act::actions::sandbox::SandboxVerdict::RequiresApproval => SandboxVerdict::RequiresApproval,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/seal_pipeline.rs:235:            inner: crate::l1_action::nt_act::actions::sandbox::ActionSandbox::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/knowledge_store.rs:72:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/knowledge_store.rs:76:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::is_safe_fetch_url(url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/knowledge_store.rs:88:        let item = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::claim_next_crawl_url(&conn)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/knowledge_store.rs:106:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::mark_crawl_complete(&conn, id, success, error)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/knowledge_store.rs:112:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes_by_domain(&conn)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/knowledge_store.rs:118:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::enqueue_seed_urls(&conn, urls)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/knowledge_store.rs:123:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/knowledge_store.rs:127:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::is_safe_fetch_url(url)
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/l1_wrappers.rs:37:    inner: crate::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/l1_wrappers.rs:43:            inner: crate::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager::new(session_id),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/l1_wrappers.rs:102:    inner: std::sync::Mutex<crate::l1_action::nt_io::nt_io_user_avatar::DistillationEngine>,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/foundation/l1_wrappers.rs:108:            inner: std::sync::Mutex::new(crate::l1_action::nt_io::nt_io_user_avatar::DistillationEngine::new()),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:328:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.content);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:330:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(body);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:332:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.description);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:1652:                last_indexed_at: Some(crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now()),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:1653:                created_at: crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:1654:                updated_at: crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::now(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:10:use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:33:// pub use crate::l1_action::nt_act::nt_l1_shared_types::{
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:70:    fn get_snapshot(&self) -> crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:486:        let snapshot = crate::l1_action::nt_act::nt_act_types::ProjectSnapshot {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:21:// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:67:pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1580:impl crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::EvolutionLoopProvider for EvolutionLoop {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1581:    fn get_snapshot(&self) -> crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::ProjectSnapshotLite {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1583:        crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::ProjectSnapshotLite {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1600:    fn self_diagnose(&mut self) -> (Vec<String>, Vec<crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue>) {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1602:        let l1_issues: Vec<crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue> = pq.into_vec().into_iter().map(|di| {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1603:            crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1604:                issue: crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::Issue {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1611:                    ActionPlan::AddTestStub { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::AddTestStub { file: file.clone() },
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1612:                    ActionPlan::RunCargoFix => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::RunCargoFix,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1613:                    ActionPlan::RemoveTodo { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::RemoveTodo { file: file.clone() },
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1614:                    ActionPlan::SplitLargeFile { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::SplitLargeFile { file: file.clone() },
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1615:                    ActionPlan::ReviewUnsafe { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ReviewUnsafe { file: file.clone() },
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1616:                    ActionPlan::ReplaceUnwrap { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ReplaceUnwrap { file: file.clone() },
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1617:                    ActionPlan::HumanDecision { reason, options } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::HumanDecision { reason: reason.clone(), options: options.clone() },
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1618:                    ActionPlan::NoAction { reason } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::NoAction { reason: reason.clone() },
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1619:                    ActionPlan::AutoFix(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::AutoFix(s.clone()),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1620:                    ActionPlan::ManualReview(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ManualReview(s.clone()),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:1621:                    ActionPlan::Skip(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::Skip(s.clone()),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_core/nt_core_parallel/contract.rs:195:    kb: Option<crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase>,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_core/nt_core_parallel/contract.rs:210:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None).ok();
```

### → l2_perception (3 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2252:            crate::l2_perception::nt_world::nt_world_exploration_engine::ExplorationConfig::default(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:2255:            crate::l2_perception::nt_world::nt_world_exploration_engine::ExplorationEngine::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/mod.rs:16:/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。
```

### → l3_embodiment (12 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1451:            let auditor = crate::l3_embodiment::nt_shield::nt_shield_audit::create_reasoning_trace_auditor();
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1620:        self_tests.register(crate::l3_embodiment::nt_shield::nt_shield::browser_security::create_browser_security_self_test());
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1621:        self_tests.register(crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test());
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1626:            crate::l3_embodiment::nt_shield::nt_shield_audit::CohGuard::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1633:            crate::l3_embodiment::nt_shield::nt_shield_audit::ReasoningTraceGuard::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2025:            crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test()
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3104:        registry.register(crate::l3_embodiment::nt_shield::nt_shield::browser_security::create_browser_security_self_test());
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3105:        registry.register(crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test());
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:1558:                                crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::scan_skill_content(&skill.content);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:1559:                            let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::InspectionResult::Allow);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:1590:                            crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::scan_skill_content(&skill.content);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine.rs:1591:                        let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::nt_shield::tool_inspection_stack::InspectionResult::Allow);
```

### → l4_emotion (1 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3100:        //     crate::l4_emotion::nt_feel::nt_core_fep_iit::bridge::FEPIITBridge::new(),
```

### → l6_meta (21 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs:717:            self_improvement: crate::l6_meta::coordination::self_improvement::SelfImprovementLoop::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs:1028:    self_improvement: crate::l6_meta::coordination::self_improvement::SelfImprovementLoop,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:940:        use crate::l6_meta::coordination::self_improvement::SystemMetrics;
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:197:                    //         let input = crate::l6_meta::nt_meta::nt_core_intra_reflection::ReflectionInput {
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:205:                    //             crate::l6_meta::nt_meta::nt_core_intra_reflection::analyze(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:266:        let (infos, _problems): (Vec<_>, Vec<_>) = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::harness_infos_from_registry_export(&json);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:272:        let mut harness = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::new_harness();
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:276:        let actionable = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::harness_actionable_suggestions(&report, 0.7);
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:962:            let hexagram_states: Vec<crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::E8HexagramState> = Vec::new();
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1143:            let mut inspector = crate::l6_meta::nt_meta::auto_inspector::AutoInspector::new();
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1615:            crate::l6_meta::memory::evolution_harness::EvolutionHarness::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1616:                crate::l6_meta::memory::transcendent_loop::LoopConfig::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1656:        //     crate::l6_meta::nt_repair::nt_mind_causal_trace::CausalTraceSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1788:            let mut cm = crate::l6_meta::nt_repair::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:1815:            self_tests.register(Box::new(crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2042:        //     crate::l6_meta::nt_repair::nt_mind_causal_trace::CausalTraceSelfTest
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs:2054:            crate::l6_meta::memory::meta_observer::MetaObserverSelfTest
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3079:            let mut cm = crate::l6_meta::nt_repair::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:3102:        registry.register(Box::new(crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/mod.rs:22:/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。
/Users/neo/Downloads/neotrix/neotrix-core/src/l5_cognition/traits.rs:137:// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
```

## l6_meta → other layers

### → l1_action (1 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/memory/evolution_harness.rs:120:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
```

### → l5_cognition (15 references)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/memory/evolution_harness.rs:200:impl crate::l5_cognition::traits::EvolutionHarnessApi for EvolutionHarness {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/memory/evolution_harness.rs:205:    fn harness_infos_from_registry_export(json: &str) -> (Vec<crate::l5_cognition::traits::RegistryNodeInfo>, Vec<String>) {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/memory/evolution_harness.rs:207:        let infos = l6_infos.into_iter().map(|i| crate::l5_cognition::traits::RegistryNodeInfo {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/memory/evolution_harness.rs:220:        infos: &[crate::l5_cognition::traits::RegistryNodeInfo],
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/memory/evolution_harness.rs:253:    ) -> Vec<crate::l5_cognition::traits::RegistrySuggestion> {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/memory/evolution_harness.rs:260:            .map(|s| crate::l5_cognition::traits::RegistrySuggestion {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_consciousness_gold_standard.rs:229:impl crate::l5_cognition::traits::GoldStandardApi for ConsciousnessGoldStandard {}
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_eval_harness.rs:1267:impl crate::l5_cognition::traits::EvalHarnessApi for EvalHarness {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_eval_harness.rs:1268:    fn generate_regression_test(&self, candidate: &str) -> crate::l5_cognition::traits::RegressionCase {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_eval_harness.rs:1270:        crate::l5_cognition::traits::RegressionCase {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_eval_harness.rs:1278:    fn run_regression_test(&self, case: &crate::l5_cognition::traits::RegressionCase) -> crate::l5_cognition::traits::RegressionResult {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_eval_harness.rs:1286:        crate::l5_cognition::traits::RegressionResult {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_consciousness_monitor.rs:399:impl crate::l5_cognition::traits::ConsciousnessMonitorApi for ConsciousnessMonitor {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_consciousness_monitor.rs:408:    fn get_report(&self) -> crate::l5_cognition::traits::ConsciousnessAwarenessReport {
/Users/neo/Downloads/neotrix/neotrix-core/src/l6_meta/healing/nt_mind_consciousness_monitor.rs:410:        crate::l5_cognition::traits::ConsciousnessAwarenessReport {
```

## Non-layer modules importing from layers

### core → l1_action (55)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_core.rs:43:use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_pipeline::AbsorbEntry;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_core.rs:44:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_core.rs:1218:        let mut goal_lock = crate::l1_action::nt_act::goal_lock::GoalLock::new();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_core.rs:1993:        use crate::l1_action::nt_io::nt_io_neocodex::{SubagentDispatch, SubagentKind};
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_second_brain.rs:8:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_meta/knowledge_gap_detector.rs:8:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_forecast.rs:349:    use crate::l1_action::nt_io::nt_io_provider::factory;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_forecast.rs:350:    use crate::l1_action::nt_io::nt_io_provider::gateway::GatewayV2;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_e8/nt_core_community_ingester.rs:318:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_e8/nt_core_community_ingester.rs:2000:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.clone()))
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_e8/nt_core_community_ingester.rs:2013:            crate::l1_action::nt_memory::nt_memory_kb::types::NodeType::Concept
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_e8/nt_core_community_ingester.rs:2022:            crate::l1_action::nt_memory::nt_memory_kb::types::NodeType::Dataset
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_cad_consciousness.rs:327:    use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:118:        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:122:    //     crate::l1_action::nt_act::nt_act_orchestrator::arbiter_mediation::ArbiterMediator::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:125:    //     crate::l1_action::nt_act::nt_act_orchestrator::expert_team_diff::ExpertTeamWriter::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:128:        crate::l1_action::nt_act::nt_act_orchestrator::harness_scaffold::HarnessScaffold::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:131:        crate::l1_action::nt_act::nt_act_code::yagni_ladder::YagniLadder::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:148:        crate::l1_action::nt_io::nt_io_output_style::OutputGovernorSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:153:        crate::l1_action::nt_io::nt_io_multimodal_transform::VisionPreprocessor::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:156:        crate::l1_action::nt_io::nt_io_multimodal_transform::CpuTtsEngine::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:157:            crate::l1_action::nt_io::nt_io_multimodal_transform::VoiceLoader::empty(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:172:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_write_guard::WriteGuardAudit,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:191:        crate::l1_action::nt_act::nt_act_orchestrator::task_state_dag::TaskStateDagHealer,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:202:        crate::l1_action::nt_memory::nt_memory_kb::spill_storage::SpillStorageHealer,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:224:    //     crate::l1_action::nt_io::nt_io_provider::account_pool::AccountPoolHealer,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:287:        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:294:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:299:        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_write_guard::WriteGuardAudit,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:333:        crate::l1_action::nt_act::nt_act_autonomy::cross_session_memory::CrossSessionMemorySelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:338:        crate::l1_action::nt_act::nt_act_code::yagni_ladder::YagniLadder::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:341:        crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:359:        crate::l1_action::nt_io::nt_io_multimodal_transform::VisionPreprocessor::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:362:        crate::l1_action::nt_io::nt_io_multimodal_transform::CpuTtsEngine::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:363:            crate::l1_action::nt_io::nt_io_multimodal_transform::VoiceLoader::empty(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:451:        // use crate::l1_action::nt_io::nt_agent_agent_team::*;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:500:        use crate::l1_action::nt_io::nt_io_digital_human::*;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:545:        use crate::l1_action::nt_memory::nt_memory_leann_store::*;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:563:        use crate::l1_action::nt_memory::nt_memory_kb::bm25::{Bm25Document, Bm25Index};
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:633:        use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_reasoning.rs:6://! 受控边界: 反向引用 `crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase`
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_reasoning.rs:10:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_knowledge/cad_absorb.rs:10:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_tree/types.rs:5:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness/consciousness_runtime.rs:17:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness/consciousness_runtime.rs:150:            match crate::l1_action::nt_memory::nt_memory_kb::nt_field_ledger::field_journal_since(
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self/affective_interface.rs:905:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp))
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self/affective_interface.rs:936:        let mut pipe = crate::l1_action::nt_io::nt_io_digital_human::DigitalHumanPipeline::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self/affective_interface.rs:937:            crate::l1_action::nt_io::nt_io_digital_human::PersonaConfig::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self/self_model.rs:28:use crate::l1_action::nt_memory::nt_memory_kb::DualBrainWorkingMemory;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self/self_model.rs:311:        assert_eq!(wm.capacity(), crate::l1_action::nt_memory::nt_memory_kb::DEFAULT_WORKING_CAPACITY);
/Users/neo/Downloads/neotrix/neotrix-core/src/core/l7_capability/nt_core_orch_agent.rs:390:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/l7_capability/nt_core_orch_agent.rs:400:        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/l7_capability/nt_core_orch_agent.rs:1547:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.join("orch.db")))
/Users/neo/Downloads/neotrix/neotrix-core/src/core/l7_capability/l7_l1_bridge.rs:8:use crate::l1_action::traits::{
```

### core → l2_perception (34)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/cache.rs:276:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/factory.rs:16:            crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/factory.rs:20:            crate::l2_perception::nt_world::asset_map::asset_map_capability::create_asset_map_capability(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/hotreload.rs:329:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/hotreload.rs:340:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/hotreload.rs:356:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/orchestrator.rs:443:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/integration_tests.rs:12:        let nlp_cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/integration_tests.rs:13:        let asset_cap = crate::l2_perception::nt_world::asset_map::asset_map_capability::create_asset_map_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/integration_tests.rs:44:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/integration_tests.rs:97:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/integration_tests.rs:134:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/integration_tests.rs:226:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/integration_tests.rs:269:        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:4:// use crate::l2_perception::nt_world::cad_selftest;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:6:use crate::l2_perception::nt_world::osint::{UnifiedAbsorber, AbsorberConfig};
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:28:    // crate::l2_perception::nt_world::cad_crossmodal_selftest::register_cad_crossmodal_self_tests(
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:34:    // crate::l2_perception::nt_world::cad_generator::register_cad_generator_self_tests(&mut registry);
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:134:        crate::l2_perception::nt_world::nt_world_scrape::FitExtractor::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:137:        crate::l2_perception::nt_world::nt_world_crawl::resilient::ResilientCrawler::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:138:            crate::l2_perception::nt_world::nt_world_crawl::resilient::ThrottlePolicy::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:142:        crate::l2_perception::nt_world::crawl::agentic_browse::AgenticBrowseSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:145:        crate::l2_perception::nt_world::nt_world_osint::sweep::SweepDeltaSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:161:        crate::l2_perception::nt_world::osint::metadata::MetadataAggregator::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:164:        crate::l2_perception::nt_world::nt_world_video_pipeline::MediaSniffer::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:176:    crate::l2_perception::nt_world::nt_world_intel_selftest::register_intel_self_tests(registry);
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:209:    // crate::l2_perception::nt_world::cad_ch_selftest::CadCHSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:213:    //     crate::l2_perception::nt_world::cad_synthbal_selftest::CadSynthBalSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:217:    //     crate::l2_perception::nt_world::cad_brep_selftest::CadBRepTopologySelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:281:        crate::l2_perception::nt_world::nt_world_video_pipeline::MediaSniffer::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:598:        use crate::l2_perception::nt_world::nt_world_video_pipeline::*;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_knowledge/types.rs:55:impl From<crate::l2_perception::nt_world::nt_world_model::TaskType> for TaskType {
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_knowledge/types.rs:56:    fn from(tt: crate::l2_perception::nt_world::nt_world_model::TaskType) -> Self {
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_knowledge/types.rs:57:        use crate::l2_perception::nt_world::nt_world_model::TaskType as WT;
```

### core → l3_embodiment (16)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/factory.rs:18:            crate::l3_embodiment::nt_shield::nt_shield_ztnet::ztnet_capability::create_ztnet_capability(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_capability/integration_tests.rs:15:            crate::l3_embodiment::nt_shield::shield_capability::create_shield_capabilities();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:23:        crate::l3_embodiment::nt_shield::nt_shield_audit::ReasoningTraceGuard::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:50:    crate::l3_embodiment::nt_shield::nt_shield::nt_shield_skill_router::register_skill_router_self_tests(&mut registry);
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:58:        crate::l3_embodiment::nt_shield::nt_shield_traffic::FingerprintStore::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:61:        crate::l3_embodiment::nt_shield::nt_shield_sandbox::DeviceSandbox::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:62:            crate::l3_embodiment::nt_shield::nt_shield_sandbox::SandboxSpec::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:67:        crate::l3_embodiment::nt_shield::nt_shield_sandbox::stateful_bench::StatefulEgressBench,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:178:    crate::l3_embodiment::nt_shield::nt_shield_oversight::register_oversight_self_tests(registry);
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:188:        crate::l3_embodiment::nt_shield::nt_shield_sentry::SentryHealer,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:346:        crate::l3_embodiment::nt_shield::nt_shield_audit::ReasoningTraceGuard::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:349:        crate::l3_embodiment::nt_shield::nt_shield_traffic::FingerprintStore::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:352:        crate::l3_embodiment::nt_shield::nt_shield_sandbox::DeviceSandbox::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:353:            crate::l3_embodiment::nt_shield::nt_shield_sandbox::SandboxSpec::default(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:479:        use crate::l3_embodiment::nt_shield::nt_shield_agentic_scan::*;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/mod.rs:395:// pub use crate::l3_embodiment::nt_shield::nt_shield_impl::{
```

### core → l5_cognition (17)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_core.rs:1130:        let persona_router = crate::l5_cognition::nt_core::persona_routing::PersonaRouter::new();
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_core.rs:1143:            crate::l5_cognition::nt_core::persona_routing::PersonaType::Wedge => "wedge",
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_core.rs:1144:            crate::l5_cognition::nt_core::persona_routing::PersonaType::Prism => "prism",
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_context/mod.rs:11:pub use crate::l5_cognition::nt_core::context_assembly::{
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:52:    crate::l5_cognition::nt_mind::nt_mind::nt_mind_seal_ecc::register_seal_ecc_self_tests(&mut registry);
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:74:        crate::l5_cognition::nt_mind::evolution::evolution_loop::MetaHarnessOptimizer::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:77:        crate::l5_cognition::nt_mind::nt_mind_skill_engine::PromptLibrary::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:80:    crate::l5_cognition::nt_mind::nt_mind_skill_engine::register_skill_standard_self_tests(&mut registry);
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:83:        crate::l5_cognition::nt_mind::nt_mind::experience_tree::self_reflection::SelfReflectionEngine::new(8),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:87:        crate::l5_cognition::nt_mind::foundation::memory_bank::MemoryAdmissionGate::new(0.5, 100),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:195:        crate::l5_cognition::nt_mind::nt_mind_skill_engine::RevertibleEffectsHealer,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:198:        crate::l5_cognition::nt_mind::nt_mind_skill_engine::FiberLifecycleHealer,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:303:        crate::l5_cognition::nt_mind::evolution::evolution_loop::MetaHarnessOptimizer::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:306:        crate::l5_cognition::nt_mind::nt_mind_skill_engine::PromptLibrary::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:320:        crate::l5_cognition::nt_mind::nt_mind_skill_engine::AgentSkillsStandardSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_retrieval.rs:16:use crate::l5_cognition::nt_mind::infrastructure::code_graph::CodeGraph;
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_task_dispatcher.rs:24:use crate::l5_cognition::nt_mind::reason::reasoning_engine::engine_core::ReasoningEngine;
```

### core → l6_meta (13)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_consciousness_core.rs:2498:        use crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::{
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:91:        crate::l6_meta::nt_repair::nt_mind_eval_harness::SmallScaleMethod::new(1.0, 0.5, 32),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:102:        crate::l6_meta::nt_repair::nt_mind_eval_harness::HdaAttributionSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:106:        crate::l6_meta::nt_repair::nt_mind_eval_harness::SelfVerifiableRewardSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:180:    crate::l6_meta::coordination::nt_governance::register_human_oversight_self_tests(registry);
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:205:        crate::l6_meta::nt_repair::nt_mind_eval_harness::OracleLadderHealer,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:227:    //     crate::l6_meta::nt_repair::nt_mind_self_heal::SelfHealLoop::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:231:    //     crate::l6_meta::nt_repair::nt_mind_self_heal::SelfHealLoop::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:310:        crate::l6_meta::nt_repair::nt_mind_eval_harness::SmallScaleMethod::new(1.0, 0.5, 32),
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:313:        crate::l6_meta::nt_repair::nt_mind_eval_harness::HdaAttributionSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:316:        crate::l6_meta::nt_repair::nt_mind_eval_harness::SelfVerifiableRewardSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:324:    //     crate::l6_meta::nt_repair::nt_mind_causal_trace::CausalTraceSelfTest,
/Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_self_test_integration.rs:327:        crate::l6_meta::memory::meta_observer::MetaObserverSelfTest,
```

### cli → l1_action (76)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/tui/session_store.rs:42:    kb: crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/tui/session_store.rs:58:        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(base.join("knowledge.db")))
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:39:    pub action_sandbox: std::sync::Mutex<crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox>,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:65:            action_sandbox: std::sync::Mutex::new(crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox::new()),
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:186:            crate::l1_action::nt_act::nt_act_sandbox::SandboxVerdict::Denied => {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:192:            crate::l1_action::nt_act::nt_act_sandbox::SandboxVerdict::RequiresApproval => {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:200:            crate::l1_action::nt_act::nt_act_sandbox::SandboxVerdict::Approved => {}
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/cortex_cmds.rs:13:use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_resource_ingest::{
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/cortex_cmds.rs:16:use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_cortex_sync::{
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/cortex_cmds.rs:388:        match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_cortex_sync::prune_external(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/ui_cmds.rs:329:                    crate::l1_action::nt_io::nt_io_provider::clear_history();
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/ui_cmds.rs:332:                let msg = crate::l1_action::nt_io::nt_io_provider::failover_report();
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/ui_cmds.rs:335:                    let events = crate::l1_action::nt_io::nt_io_provider::failover_history();
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/ui_cmds.rs:337:                        "total": crate::l1_action::nt_io::nt_io_provider::total_failovers(),
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/types.rs:8:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:3:use crate::l1_action::nt_memory::nt_memory_kb::{
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:158:    // let _ = crate::l1_action::nt_memory::nt_memory_kb::setting_consistency::check_and_report_to_string(&conn, &mut out);
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:323:    let render_nodes = |items: &[crate::l1_action::nt_memory::nt_memory_kb::DiffNode], tag: &str| {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:369:    use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_embed::load_all_embeddings;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:370:    use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_distill::{
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:539:    let (mapped, report) = match crate::l1_action::nt_memory::nt_memory_kb::map_nodes(&conn, None, types.as_deref(), limit_opt)
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:574:    match crate::l1_action::nt_memory::nt_memory_kb::apply_mappings(&conn, &mapped) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:689:    let permission = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::Confidential;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:1488:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_schema::initialize(&conn).unwrap();
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:1505:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_schema::initialize(&conn).unwrap();
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:1527:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_schema::initialize(&conn).unwrap();
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:1559:            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_schema::initialize(&conn).unwrap();
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/skill_cmds.rs:9:use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::skill_list_all;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/skill_cmds.rs:10:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kanban_cmds.rs:28:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wiki_cmds.rs:8:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/agent_cmds.rs:7:// use crate::l1_action::nt_io::nt_agent_mcp_gateway::{ProgrammaticCall, ProgrammaticPlanner};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/agent_cmds.rs:40:    pub fn load_from_kb(&mut self, _kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase) -> Result<(), String> { Ok(()) }
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/agent_cmds.rs:41:    pub fn save_to_kb(&self, _kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase) -> Result<(), String> { Ok(()) }
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/acp_cmds.rs:13:use crate::l1_action::nt_io::nt_io_neocodex::{AcpServer, NeoCodexAgent};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/chain_cmds.rs:17:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/chain_cmds.rs:18:use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_visibility::{filter_visibility, Visibility, VisibilityConfig};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/consciousness_cmds.rs:15:use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/free_cmds.rs:13:use crate::l1_action::nt_io::nt_io_provider::factory::LlmProviderType;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:18:        fn try_open_kb() -> Option<crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase> {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:19:            crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None).ok()
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:210:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_cities(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:231:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_country_boundaries(&conn, url) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:258:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_peaks(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:287:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_airports(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:316:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_boundaries(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:345:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_boundaries(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:374:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::ingest_geo_volcanoes(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:395:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_vectors(&conn, url, "river") {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:414:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_vectors(&conn, url, "lake") {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:433:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_vectors(&conn, url, "coastline") {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:452:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::ingest_geo_vectors(&conn, url, "glacier") {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:471:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::geo_tag_nodes(&conn, limit) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:490:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::geo_tag_cities(&conn, limit) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:513:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::geo_linked_nodes(&conn, place, limit) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:544:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::geo_coverage_report(&conn, min) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:572:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::fetch_elevations(&conn, limit) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:591:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::fetch_weather_snapshot(&conn, limit) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:618:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::export_geo_ntpack(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:643:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::import_geo_ntpack_to_kb(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:674:                                match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::archive_geo_cold(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:694:                        match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::geo_cold_layers(&dir) {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:17:use crate::l1_action::nt_act::nt_act_crypto::CryptoAgent;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:280:fn chain_from_name(name: &str) -> crate::l1_action::nt_act::nt_act_crypto::ChainType {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:282:        "eth" | "ethereum" => crate::l1_action::nt_act::nt_act_crypto::ChainType::Ethereum,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:283:        "bsc" | "bnb" => crate::l1_action::nt_act::nt_act_crypto::ChainType::Bsc,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:284:        "polygon" | "matic" => crate::l1_action::nt_act::nt_act_crypto::ChainType::Polygon,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:285:        "arb" | "arbitrum" => crate::l1_action::nt_act::nt_act_crypto::ChainType::Arbitrum,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:286:        "opt" | "optimism" | "op" => crate::l1_action::nt_act::nt_act_crypto::ChainType::Optimism,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:287:        "base" => crate::l1_action::nt_act::nt_act_crypto::ChainType::Base,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:288:        "avax" | "avalanche" => crate::l1_action::nt_act::nt_act_crypto::ChainType::Avalanche,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:289:        _ => crate::l1_action::nt_act::nt_act_crypto::ChainType::Ethereum,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/swap_cmd.rs:2:use crate::l1_action::nt_act::nt_act_crypto::tx::TxBuilder;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/swap_cmd.rs:3:use crate::l1_action::nt_act::nt_act_crypto::CryptoAgent;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/swap_cmd.rs:4:use crate::l1_action::nt_act::nt_act_crypto::chain::ChainType;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/pool_health_cmds.rs:13:use crate::l1_action::nt_io::nt_io_provider::factory::create_gateway_async;
```

### cli → l2_perception (3)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:498:    use crate::l2_perception::nt_world::nt_world_crawl::asset_graph::AssetGraphWriter;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/goal_cmds.rs:9:use crate::l2_perception::nt_world::nt_world_model::TaskType;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:2:use crate::l2_perception::nt_world::nt_world_exploration_engine::{ExplorationEngine, ExplorationConfig};
```

### cli → l3_embodiment (24)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:7:use crate::l3_embodiment::nt_shield::nt_shield::guard::{GuardDecision, SecurityGuard};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:8:use crate::l3_embodiment::nt_shield::nt_shield::guardrails::{GuardrailConfig, GuardrailSystem};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:9:use crate::l3_embodiment::nt_shield::nt_shield::perm_chain::{PermissionChain, PermissionMode, PermissionResult};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:10:use crate::l3_embodiment::nt_shield::nt_shield::policy::{ActionPolicy, PolicyDecision};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:11:use crate::l3_embodiment::nt_shield::unified_defense::UnifiedDefenseLayer;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:17:    pub attack_results: Vec<crate::l3_embodiment::nt_shield::fullbreak::AttackResult>,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:18:    pub evasion_result: crate::l3_embodiment::nt_shield::cloud_evade::EvasionResult,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:43:    pub fullbreak: crate::l3_embodiment::nt_shield::fullbreak::FullbreakEngine,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:45:    pub cloud_evade: crate::l3_embodiment::nt_shield::cloud_evade::CloudEvadeEngine,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:67:            fullbreak: crate::l3_embodiment::nt_shield::fullbreak::FullbreakEngine::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:68:            cloud_evade: crate::l3_embodiment::nt_shield::cloud_evade::CloudEvadeEngine::new(),
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:271:    pub fn pending_guard_requests(&self) -> Vec<crate::l3_embodiment::nt_shield::nt_shield::guard::GuardRequest> {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:461:        s.policy.add_rule("file_read", crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow);
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:479:        s.policy.add_rule("file_read", crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow);
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:603:        let mut config = crate::l3_embodiment::nt_shield::guardrails::GuardrailConfig::default();
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:606:            guardrails: crate::l3_embodiment::nt_shield::guardrails::GuardrailSystem::new(config),
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:611:        s.policy.add_rule("file_read", crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow);
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:627:        s.policy.add_rule("file_read", crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow);
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:651:        s.policy.add_rule("file_write", crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow);
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:664:        s.policy.add_rule("file_write", crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow);
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/shield_enforcer.rs:666:        s.set_perm_chain_mode(crate::l3_embodiment::nt_shield::nt_shield::perm_chain::PermissionMode::BypassPermissions);
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/core_cmds.rs:8:use crate::l3_embodiment::nt_shield::nt_shield::key_encryption;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/permission_profiles.rs:378:use crate::l3_embodiment::nt_shield::nt_shield::perm_chain::PermissionMode;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/permission_profiles.rs:379:use crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision;
```

### cli → l5_cognition (53)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/tui/session_store.rs:209:    pub fn distill(&mut self) -> Result<crate::l5_cognition::nt_mind::foundation::distiller::DistillationReport, String> {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/tui/session_store.rs:210:        let mut d = crate::l5_cognition::nt_mind::foundation::distiller::SessionDistiller::with_paths(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/session_cmds.rs:7:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/doctor_cmds.rs:7:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/bench_cmds.rs:5:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/bench_cmds.rs:6:use crate::l5_cognition::nt_mind::nt_mind::benchmark::{bench_plan_reasoning, print_benchmark_table};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/cortex_cmds.rs:12:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/ui_cmds.rs:7:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/ui_cmds.rs:9:use crate::l5_cognition::nt_mind::nt_mind_background_loop::always_on::ALWAYS_ON_ENGINE;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/types.rs:6:pub(crate) use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kb_cmds.rs:88:            &std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/plugin_cmds.rs:6:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/cost_cmds.rs:7:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/goal_cmds.rs:7:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/goal_cmds.rs:8:use crate::l5_cognition::nt_mind::nt_mind::goal_loop::{GoalConfig, GoalTracker};
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/search_cmds.rs:5:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/git_cmds.rs:8:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/skill_cmds.rs:11:use crate::l5_cognition::nt_mind::nt_mind_skill_engine::SkillEngine;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/skill_cmds.rs:12:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kanban_cmds.rs:29:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kanban_cmds.rs:792://             item.efficiency_score = crate::l5_cognition::nt_core::nt_core_parallel::OptimalTaskAllocator::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kanban_cmds.rs:793://                 crate::l5_cognition::nt_core::nt_core_parallel::AllocationStrategy::Hybrid
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kanban_cmds.rs:982://                 let allocator = crate::l5_cognition::nt_core::nt_core_parallel::OptimalTaskAllocator::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kanban_cmds.rs:983://                     crate::l5_cognition::nt_core::nt_core_parallel::AllocationStrategy::Hybrid,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/kanban_cmds.rs:1001://                     let todo = crate::l5_cognition::nt_core::nt_core_parallel::TodoTask::new(
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wiki_cmds.rs:9:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/connector_cmds.rs:6:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/evidence_cmds.rs:6:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/plan_cmds.rs:68:    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/agent_cmds.rs:9:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/theme_cmd.rs:7:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/acp_cmds.rs:12:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/core_cmds.rs:7:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/chain_cmds.rs:19:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/consolidated_cmds.rs:13:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/hypothesis_cmds.rs:5:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/consciousness_cmds.rs:16:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/file_cmds.rs:12:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/brain_cmds.rs:9:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/free_cmds.rs:16:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/provider_cmds.rs:12:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:3:use crate::l5_cognition::nt_mind::nt_mind_background_loop::knowledge_pipeline::KnowledgeAbsorptionPipeline;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/explore_cmds.rs:14:    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/wallet_cmd.rs:16:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/profile_cmds.rs:9:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/swap_cmd.rs:61:        _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>>>,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/swap_cmd.rs:194:        _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>>>,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/swap_cmd.rs:326:        _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>>>,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/budget_cmds.rs:6:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/game_cmds.rs:13:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/pool_health_cmds.rs:15:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/session_recovery_cmds.rs:22:        _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>,
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/model_cmds.rs:6:use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
```

### cli → l6_meta (1)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/cli/commands/bench_cmds.rs:8:use crate::l6_meta::healing::nt_mind_eval_harness::{
```

### neotrix → l1_action (13)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_act/tools.rs:740:        if let Ok(kb) = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_shanhai_geo/geo_sync.rs:8:use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_geo::{upsert_geo, GeoRecord};
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:10:pub use crate::l1_action::{nt_act, nt_io, nt_memory};
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:51:pub use crate::l1_action::nt_io::nt_io_standalone::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:56:pub use crate::l1_action::nt_io::nt_io_provider::types::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:59:pub use crate::l1_action::nt_io::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:65:pub use crate::l1_action::nt_io::nt_io_telemetry;
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:68:pub use crate::l1_action::nt_act::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:72:pub use crate::l1_action::nt_act::nt_act_autonomy::nt_mind_automation::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:77:pub use crate::l1_action::nt_memory::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:80:pub use crate::l1_action::nt_memory_spatial;
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:128:pub use crate::l1_action::nt_io::nt_io_mention::{resolve_mentions, MentionResult};
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:129:pub use crate::l1_action::nt_io::nt_io_notify::{
```

### neotrix → l2_perception (3)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:11:pub use crate::l2_perception::{nt_world, nt_sense};
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:83:pub use crate::l2_perception::nt_world::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:136:pub use crate::l2_perception::nt_world::nt_world_scrape::{
```

### neotrix → l3_embodiment (9)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_core_event_bus.rs:111:                    let redacted = crate::l3_embodiment::nt_shield::nt_shield::redaction::redact_json_line(&line);
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:12:pub use crate::l3_embodiment::{nt_shield, nt_physical};
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:91:pub use crate::l3_embodiment::nt_shield::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:96:pub use crate::l3_embodiment::nt_shield::nt_shield_sandbox_entry;
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:98:pub use crate::l3_embodiment::nt_shield::nt_shield_stealth_net;
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:132:pub use crate::l3_embodiment::nt_shield::nt_shield_audit::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_harness/mod.rs:15:use crate::l3_embodiment::nt_shield::nt_shield::receipt::AgentReceipt;
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_harness/mod.rs:269:        let run_id = crate::l3_embodiment::nt_shield::nt_shield::receipt::hash_content(&req.instruction);
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_harness/mod.rs:481:        assert_eq!(receipt.output_hash, crate::l3_embodiment::nt_shield::nt_shield::receipt::hash_content(&resp.message));
```

### neotrix → l4_emotion (1)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:13:pub use crate::l4_emotion::nt_feel;
```

### neotrix → l5_cognition (7)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_core_event_bus.rs:336:                            crate::core::nt_core_event::CoreEvent::ConsciousnessCritique { quality, .. } if *quality < crate::l5_cognition::nt_mind::nt_mind_background_loop::CONSCIOUSNESS_THRESHOLDS.eventbus_critical => {
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_core_event_bus.rs:406:                        crate::core::nt_core_event::CoreEvent::ConsciousnessCritique { quality, .. } if *quality < crate::l5_cognition::nt_mind::nt_mind_background_loop::CONSCIOUSNESS_THRESHOLDS.eventbus_critical => {
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:14:pub use crate::l5_cognition::{nt_core, nt_mind};
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:34:pub use crate::l5_cognition::nt_core::nt_consciousness_core;
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:101:pub use crate::l5_cognition::nt_mind::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:107:pub use crate::l5_cognition::nt_mind::evolution;
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:108:pub use crate::l5_cognition::nt_mind::foundation;
```

### neotrix → l6_meta (4)
```
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:15:pub use crate::l6_meta::{nt_meta, nt_repair, nt_nexus};
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:111:pub use crate::l6_meta::coordination::nt_core_intra_reflection;
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:112:pub use crate::l6_meta::healing::{
/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/mod.rs:139:pub use crate::l6_meta::healing::nt_mind_consciousness_gold_standard::{
```

### config.rs → l3_embodiment (1)
```
1:use crate::l3_embodiment::nt_shield::nt_shield::key_encryption;
```

---

## Summary

| Direction | Count |
|-----------|-------|
| Layer→Layer cross-imports | 313 |
| Non-layer→Layer imports | 330 |
| **Total** | **643** |

643 cross-layer reference(s) found. Review above.
