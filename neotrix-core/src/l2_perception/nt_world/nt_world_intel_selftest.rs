#![forbid(unsafe_code)]

//! 情报工具 SelfTest 注册 — 激活 ConsciousnessTree NT-WORLD 分支 (T1 存在 + T2 注册).
//!
//! 依据 ConsciousnessTree 研究: 情报工具因未实现 `SelfTest` 且未注册, 对 NT-WORLD 分支
//! 不可见 (无 module_count/health 贡献). 本模块以 fixture 解析作为离线 SelfTest, 验证各
//! 工具解析管线健康 (Dark Forest: 接入即验证), 并通过 `register_intel_self_tests` 接入
//! `SelfTestRegistry` (R-P42 强化现有注册点, 非平行适配器). 覆盖 12 个工具
//! (gdelt/edgar/usgs/gdacs/ucdp/urlhaus/ofac/polymarket/aoi/adsb + 外部吸收批次
//! bgpview/opencorporates).

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
use crate::l2_perception::{
    nt_world_adsb, nt_world_aoi, nt_world_bgpview, nt_world_edgar, nt_world_gdacs,
    nt_world_gdelt, nt_world_ofac, nt_world_opencorporates, nt_world_polymarket,
    nt_world_ucdp, nt_world_usgs, nt_world_urlhaus,
};

pub struct GdeltIntelSelfTest;
impl SelfTest for GdeltIntelSelfTest {
    fn name(&self) -> &str { "intel:gdelt" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_gdelt::GdeltFetcher::parse_articles(nt_world_gdelt::GDELT_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.is_empty() { return Err(vec!["gdelt fixture parsed empty".into()]); }
        Ok(())
    }
}

pub struct EdgarIntelSelfTest;
impl SelfTest for EdgarIntelSelfTest {
    fn name(&self) -> &str { "intel:edgar" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        nt_world_edgar::EdgarFetcher::parse_submissions(nt_world_edgar::EDGAR_FIXTURE_JSON)
            .map(|_| ())
            .map_err(|e| vec![e])
    }
}

pub struct UsgsIntelSelfTest;
impl SelfTest for UsgsIntelSelfTest {
    fn name(&self) -> &str { "intel:usgs" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_usgs::UsgsFetcher::parse_geojson(nt_world_usgs::USGS_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.len() != 2 { return Err(vec![format!("usgs fixture expected 2, got {}", v.len())]); }
        Ok(())
    }
}

pub struct GdacsIntelSelfTest;
impl SelfTest for GdacsIntelSelfTest {
    fn name(&self) -> &str { "intel:gdacs" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_gdacs::GdacsFetcher::parse_json(nt_world_gdacs::GDACS_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.len() != 2 { return Err(vec![format!("gdacs fixture expected 2, got {}", v.len())]); }
        Ok(())
    }
}

pub struct UcdpIntelSelfTest;
impl SelfTest for UcdpIntelSelfTest {
    fn name(&self) -> &str { "intel:ucdp" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_ucdp::UcdpFetcher::parse_json(nt_world_ucdp::UCDP_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.len() != 2 { return Err(vec![format!("ucdp fixture expected 2, got {}", v.len())]); }
        Ok(())
    }
}

pub struct UrlhausIntelSelfTest;
impl SelfTest for UrlhausIntelSelfTest {
    fn name(&self) -> &str { "intel:urlhaus" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_urlhaus::UrlhausFetcher::parse_json(nt_world_urlhaus::URLHAUS_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.is_empty() { return Err(vec!["urlhaus fixture parsed empty".into()]); }
        Ok(())
    }
}

pub struct CisaKevIntelSelfTest;
impl SelfTest for CisaKevIntelSelfTest {
    fn name(&self) -> &str { "intel:cisa-kev" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_urlhaus::CisaKevFetcher::parse_json(nt_world_urlhaus::CISA_KEV_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.is_empty() { return Err(vec!["cisa-kev fixture parsed empty".into()]); }
        Ok(())
    }
}

pub struct OfacIntelSelfTest;
impl SelfTest for OfacIntelSelfTest {
    fn name(&self) -> &str { "intel:ofac" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_ofac::OfacFetcher::parse_xml(nt_world_ofac::OFAC_FIXTURE_XML)
            .map_err(|e| vec![e])?;
        if v.len() != 2 { return Err(vec![format!("ofac fixture expected 2, got {}", v.len())]); }
        Ok(())
    }
}

pub struct PolymarketIntelSelfTest;
impl SelfTest for PolymarketIntelSelfTest {
    fn name(&self) -> &str { "intel:polymarket" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_polymarket::PolymarketFetcher::parse_json(nt_world_polymarket::POLYMARKET_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.is_empty() { return Err(vec!["polymarket fixture parsed empty".into()]); }
        Ok(())
    }
}

pub struct AoiIntelSelfTest;
impl SelfTest for AoiIntelSelfTest {
    fn name(&self) -> &str { "intel:aoi" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_aoi::AoiMonitor::parse_geojson(nt_world_aoi::AOI_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.len() != 2 { return Err(vec![format!("aoi fixture expected 2, got {}", v.len())]); }
        Ok(())
    }
}

pub struct AdsbIntelSelfTest;
impl SelfTest for AdsbIntelSelfTest {
    fn name(&self) -> &str { "intel:adsb" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_adsb::AdsbFetcher::parse_json(nt_world_adsb::ADSB_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.len() != 2 { return Err(vec![format!("adsb fixture expected 2, got {}", v.len())]); }
        Ok(())
    }
}

pub struct BgpviewIntelSelfTest;
impl SelfTest for BgpviewIntelSelfTest {
    fn name(&self) -> &str { "intel:bgpview" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_bgpview::BgpviewFetcher::parse_json(nt_world_bgpview::BGPVIEW_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.len() != 2 { return Err(vec![format!("bgpview fixture expected 2, got {}", v.len())]); }
        Ok(())
    }
}

pub struct OpencorporatesIntelSelfTest;
impl SelfTest for OpencorporatesIntelSelfTest {
    fn name(&self) -> &str { "intel:opencorporates" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let v = nt_world_opencorporates::OpencorporatesFetcher::parse_json(nt_world_opencorporates::OC_FIXTURE_JSON)
            .map_err(|e| vec![e])?;
        if v.len() != 2 { return Err(vec![format!("opencorporates fixture expected 2, got {}", v.len())]); }
        Ok(())
    }
}

/// 将情报工具注册进 SelfTestRegistry (T2 注册) — 由 `register_absorbed_modules` 调用。
pub fn register_intel_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(GdeltIntelSelfTest));
    registry.register(Box::new(EdgarIntelSelfTest));
    registry.register(Box::new(UsgsIntelSelfTest));
    registry.register(Box::new(GdacsIntelSelfTest));
    registry.register(Box::new(UcdpIntelSelfTest));
    registry.register(Box::new(UrlhausIntelSelfTest));
    registry.register(Box::new(CisaKevIntelSelfTest));
    registry.register(Box::new(OfacIntelSelfTest));
    registry.register(Box::new(PolymarketIntelSelfTest));
    registry.register(Box::new(AoiIntelSelfTest));
    registry.register(Box::new(AdsbIntelSelfTest));
    registry.register(Box::new(BgpviewIntelSelfTest));
    registry.register(Box::new(OpencorporatesIntelSelfTest));
}
