//! IntelSource 桥接层 — 将现有 12 个情报源适配到统一架构
//!
//! 将 `data_source/` 下的 12 个情报源包装为统一的 `IntelSource` trait，
//! 无需修改现有情报源代码。

use super::unified::*;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// GDELT 桥接
// ============================================================================

pub struct GdeltBridge;

impl DataSource for GdeltBridge {
    fn id(&self) -> &str {
        "gdelt"
    }
    fn name(&self) -> &str {
        "GDELT DOC 2.0"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for GdeltBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!(
                "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=ArtList&format=json",
                urlencoding::encode(&q)
            );

            let resp = reqwest::get(&url)
                .await
                .map_err(|e| format!("GDELT fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("GDELT read failed: {}", e))?;

            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("GDELT parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(articles) = json["articles"].as_array() {
                for (i, article) in articles.iter().enumerate() {
                    let mut metadata = HashMap::new();
                    if let Some(lang) = article["language"].as_str() {
                        metadata.insert("language".into(), lang.into());
                    }
                    if let Some(date) = article["seendate"].as_str() {
                        metadata.insert("date".into(), date.into());
                    }
                    if let Some(domain) = article["domain"].as_str() {
                        metadata.insert("domain".into(), domain.into());
                    }

                    items.push(IntelItem {
                        id: format!("gdelt:{}", i),
                        title: article["title"].as_str().unwrap_or("").to_string(),
                        content: article["domain"].as_str().unwrap_or("").to_string(),
                        url: article["url"].as_str().map(|s| s.to_string()),
                        timestamp: article["seendate"].as_str().map(|s| s.to_string()),
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "gdelt".into(),
            })
        })
    }
}

// ============================================================================
// SEC EDGAR 桥接
// ============================================================================

pub struct EdgarBridge;

impl DataSource for EdgarBridge {
    fn id(&self) -> &str {
        "edgar"
    }
    fn name(&self) -> &str {
        "SEC EDGAR"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for EdgarBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!(
                "https://efts.sec.gov/LATEST/search-index?q={}&dateRange=custom&startdt=2024-01-01&enddt=2024-12-31&forms=10-K,10-Q",
                urlencoding::encode(&q)
            );

            let resp = reqwest::get(&url)
                .await
                .map_err(|e| format!("EDGAR fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("EDGAR read failed: {}", e))?;

            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("EDGAR parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(hits) = json["hits"]["hits"].as_array() {
                for (i, hit) in hits.iter().enumerate() {
                    let src = &hit["_source"];
                    let mut metadata = HashMap::new();
                    if let Some(form) = src["form_type"].as_str() {
                        metadata.insert("form_type".into(), form.into());
                    }
                    if let Some(date) = src["file_date"].as_str() {
                        metadata.insert("date".into(), date.into());
                    }

                    let file_url = src["file_url"].as_str().map(|s| {
                        format!("https://www.sec.gov/{}", s)
                    });

                    items.push(IntelItem {
                        id: format!("edgar:{}", i),
                        title: src["file_description"].as_str().unwrap_or("").to_string(),
                        content: src["display_names"].as_str().unwrap_or("").to_string(),
                        url: file_url,
                        timestamp: src["file_date"].as_str().map(|s| s.to_string()),
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "edgar".into(),
            })
        })
    }
}

// ============================================================================
// USGS 桥接
// ============================================================================

pub struct UsgsBridge;

impl DataSource for UsgsBridge {
    fn id(&self) -> &str {
        "usgs"
    }
    fn name(&self) -> &str {
        "USGS Earthquakes"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for UsgsBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            let feed_url = if q.contains("week") || q.contains("7d") {
                "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_week.geojson"
            } else if q.contains("month") || q.contains("30d") {
                "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_month.geojson"
            } else {
                "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_day.geojson"
            };

            let resp = reqwest::get(feed_url)
                .await
                .map_err(|e| format!("USGS fetch failed: {}", e))?;

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("USGS parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(features) = json["features"].as_array() {
                for feature in features.iter() {
                    let props = &feature["properties"];
                    let geometry = &feature["geometry"];

                    let mut metadata = HashMap::new();
                    if let Some(mag) = props["mag"].as_f64() {
                        metadata.insert("magnitude".into(), format!("{:.1}", mag));
                    }
                    if let Some(coords) = geometry["coordinates"].as_array() {
                        if coords.len() >= 2 {
                            metadata.insert(
                                "coordinates".into(),
                                format!("{}, {}", coords[0], coords[1]),
                            );
                        }
                    }
                    if let Some(place) = props["place"].as_str() {
                        metadata.insert("place".into(), place.into());
                    }

                    items.push(IntelItem {
                        id: feature["id"].as_str().unwrap_or("").to_string(),
                        title: props["title"].as_str().unwrap_or("").to_string(),
                        content: props["place"].as_str().unwrap_or("").to_string(),
                        url: props["url"].as_str().map(|s| s.to_string()),
                        timestamp: props["time"]
                            .as_i64()
                            .map(|t| {
                                chrono::DateTime::from_timestamp_millis(t)
                                    .map(|dt| dt.to_rfc3339())
                                    .unwrap_or_default()
                            }),
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "usgs".into(),
            })
        })
    }
}

// ============================================================================
// GDACS 桥接
// ============================================================================

pub struct GdacsBridge;

impl DataSource for GdacsBridge {
    fn id(&self) -> &str {
        "gdacs"
    }
    fn name(&self) -> &str {
        "GDACS Disasters"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for GdacsBridge {
    fn fetch(
        &self,
        _query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        Box::pin(async move {
            let resp = reqwest::get("https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH")
                .await
                .map_err(|e| format!("GDACS fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("GDACS read failed: {}", e))?;

            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("GDACS parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(events) = json["result"].as_array() {
                for (i, evt) in events.iter().enumerate() {
                    let mut metadata = HashMap::new();
                    if let Some(event_type) = evt["eventtype"].as_str() {
                        metadata.insert("event_type".into(), event_type.into());
                    }
                    if let Some(alert) = evt["alertlevel"].as_str() {
                        metadata.insert("alert_level".into(), alert.into());
                    }
                    if let Some(sev) = evt["severitydata"]["severity"].as_str() {
                        metadata.insert("severity".into(), sev.into());
                    }

                    items.push(IntelItem {
                        id: format!("gdacs:{}", evt["eventid"].as_str().unwrap_or(&i.to_string())),
                        title: evt["name"].as_str().unwrap_or("").to_string(),
                        content: format!(
                            "{} @ ({}, {})",
                            evt["eventtype"].as_str().unwrap_or(""),
                            evt["lat"].as_str().unwrap_or(""),
                            evt["lon"].as_str().unwrap_or("")
                        ),
                        url: Some(format!(
                            "https://www.gdacs.org/report.aspx?eventid={}",
                            evt["eventid"].as_str().unwrap_or("")
                        )),
                        timestamp: evt["fromdate"].as_str().map(|s| s.to_string()),
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "gdacs".into(),
            })
        })
    }
}

// ============================================================================
// UCDP 桥接
// ============================================================================

pub struct UcdpBridge;

impl DataSource for UcdpBridge {
    fn id(&self) -> &str {
        "ucdp"
    }
    fn name(&self) -> &str {
        "UCDP Conflicts"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for UcdpBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            let endpoint = if q.contains("state") || q.contains("interstate") {
                "https://ucdp.uu.se/api/state_based"
            } else if q.contains("ucdp") {
                "https://ucdp.uu.se/api/ucdp_events"
            } else {
                "https://ucdp.uu.se/api/ged_events"
            };

            let resp = reqwest::get(endpoint)
                .await
                .map_err(|e| format!("UCDP fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("UCDP read failed: {}", e))?;

            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("UCDP parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(events) = json["events"].as_array() {
                for evt in events {
                    let mut metadata = HashMap::new();
                    if let Some(country) = evt["country"].as_str() {
                        metadata.insert("country".into(), country.into());
                    }
                    if let Some(side_a) = evt["side_a"].as_str() {
                        metadata.insert("side_a".into(), side_a.into());
                    }
                    if let Some(side_b) = evt["side_b"].as_str() {
                        metadata.insert("side_b".into(), side_b.into());
                    }
                    if let Some(deaths) = evt["best"].as_str() {
                        metadata.insert("deaths_best".into(), deaths.into());
                    }
                    if let Some(violence) = evt["type_of_violence"].as_str() {
                        metadata.insert("violence_type".into(), violence.into());
                    }

                    let id = evt["id"].as_str().unwrap_or("");
                    items.push(IntelItem {
                        id: format!("ucdp:{}", id),
                        title: format!(
                            "UCDP {} - {}",
                            id,
                            evt["country"].as_str().unwrap_or("")
                        ),
                        content: format!(
                            "{} vs {}",
                            evt["side_a"].as_str().unwrap_or(""),
                            evt["side_b"].as_str().unwrap_or("")
                        ),
                        url: Some(format!(
                            "https://ucdp.uu.se/#/event/{}/{}",
                            evt["year"].as_str().unwrap_or(""),
                            id
                        )),
                        timestamp: evt["start_date"].as_str().map(|s| s.to_string()),
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "ucdp".into(),
            })
        })
    }
}

// ============================================================================
// URLhaus 桥接
// ============================================================================

pub struct UrlhausBridge;

impl DataSource for UrlhausBridge {
    fn id(&self) -> &str {
        "urlhaus"
    }
    fn name(&self) -> &str {
        "URLhaus Malicious URLs"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for UrlhausBridge {
    fn fetch(
        &self,
        _query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        Box::pin(async move {
            let resp = reqwest::Client::new()
                .post("https://urlhaus-api.abuse.ch/v1/url/recent/")
                .form(&[("limit", "50")])
                .send()
                .await
                .map_err(|e| format!("URLhaus fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("URLhaus read failed: {}", e))?;

            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("URLhaus parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(urls) = json["urls"].as_array() {
                for (i, entry) in urls.iter().enumerate() {
                    let mut metadata = HashMap::new();
                    if let Some(threat) = entry["threat"].as_str() {
                        metadata.insert("threat".into(), threat.into());
                    }
                    if let Some(status) = entry["url_status"].as_str() {
                        metadata.insert("url_status".into(), status.into());
                    }
                    if let Some(host) = entry["host"].as_str() {
                        metadata.insert("host".into(), host.into());
                    }
                    if let Some(reporter) = entry["reporter"].as_str() {
                        metadata.insert("reporter".into(), reporter.into());
                    }
                    if let Some(md5) = entry["md5_hash"].as_str() {
                        metadata.insert("md5".into(), md5.into());
                    }
                    if let Some(sha256) = entry["sha256_hash"].as_str() {
                        metadata.insert("sha256".into(), sha256.into());
                    }

                    let url_str = entry["url"].as_str().unwrap_or("");
                    items.push(IntelItem {
                        id: format!("urlhaus:{}", i),
                        title: url_str.to_string(),
                        content: format!(
                            "{} | {} | host={}",
                            entry["threat"].as_str().unwrap_or(""),
                            entry["url_status"].as_str().unwrap_or(""),
                            entry["host"].as_str().unwrap_or("")
                        ),
                        url: Some(url_str.to_string()),
                        timestamp: entry["date_added"].as_str().map(|s| s.to_string()),
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "urlhaus".into(),
            })
        })
    }
}

// ============================================================================
// OFAC 桥接
// ============================================================================

pub struct OfacBridge;

impl DataSource for OfacBridge {
    fn id(&self) -> &str {
        "ofac"
    }
    fn name(&self) -> &str {
        "OFAC Sanctions SDN"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for OfacBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string().to_lowercase();
        Box::pin(async move {
            let resp = reqwest::get("https://www.treasury.gov/ofac/downloads/sdn.xml")
                .await
                .map_err(|e| format!("OFAC fetch failed: {}", e))?;

            let xml = resp
                .text()
                .await
                .map_err(|e| format!("OFAC read failed: {}", e))?;

            let items = parse_ofac_xml(&xml, &q);

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "ofac".into(),
            })
        })
    }
}

fn parse_ofac_xml(xml: &str, query: &str) -> Vec<IntelItem> {
    let mut items = Vec::new();
    let mut cursor = 0;
    while let Some(entry_start) = xml[cursor..].find("<sdnEntry>") {
        let abs_start = cursor + entry_start;
        let entry_end = match xml[abs_start..].find("</sdnEntry>") {
            Some(i) => abs_start + i,
            None => break,
        };
        let block = &xml[abs_start..entry_end];

        let uid = extract_tag(block, "uid", 0).unwrap_or("").to_string();
        let first = extract_tag(block, "firstName", 0).unwrap_or("").to_string();
        let last = extract_tag(block, "lastName", 0).unwrap_or("").to_string();
        let sdn_type = extract_tag(block, "sdnType", 0).unwrap_or("").to_string();

        let mut programs = Vec::new();
        let mut p = 0;
        while let Some(ps) = block[p..].find("<program>") {
            let pa = p + ps;
            if let Some(pe) = block[pa..].find("</program>") {
                programs.push(block[pa + 9..pa + pe].to_string());
                p = pa + pe;
            } else {
                break;
            }
        }

        let name = if first.is_empty() {
            last.clone()
        } else {
            format!("{} {}", first, last)
        };

        if !uid.is_empty() && !name.is_empty() {
            let name_lower = name.to_lowercase();
            let programs_str = programs.join(",");
            let programs_lower = programs_str.to_lowercase();

            if query.is_empty()
                || name_lower.contains(query)
                || programs_lower.contains(query)
            {
                let mut metadata = HashMap::new();
                metadata.insert("sdn_type".into(), sdn_type.clone());
                metadata.insert("programs".into(), programs_str.clone());

                items.push(IntelItem {
                    id: format!("ofac:{}", uid),
                    title: name.clone(),
                    content: format!("type={} | programs={}", sdn_type, programs_str),
                    url: Some(format!(
                        "https://sanctionssearch.ofac.treas.gov/SanctionsDSL/id/{}",
                        uid
                    )),
                    timestamp: None,
                    metadata,
                });
            }
        }
        cursor = entry_end + 11;
    }
    items
}

fn extract_tag<'a>(xml: &'a str, tag: &str, from: usize) -> Option<&'a str> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml[from..].find(&open).map(|i| from + i + open.len())?;
    let end = xml[start..].find(&close).map(|i| start + i)?;
    Some(&xml[start..end])
}

// ============================================================================
// Polymarket 桥接
// ============================================================================

pub struct PolymarketBridge;

impl DataSource for PolymarketBridge {
    fn id(&self) -> &str {
        "polymarket"
    }
    fn name(&self) -> &str {
        "Polymarket Prediction Markets"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for PolymarketBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string().to_lowercase();
        Box::pin(async move {
            let resp = reqwest::get("https://gamma-api.polymarket.com/markets?limit=50&active=true")
                .await
                .map_err(|e| format!("Polymarket fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("Polymarket read failed: {}", e))?;

            let markets: Vec<serde_json::Value> =
                serde_json::from_str(&body).map_err(|e| format!("Polymarket parse failed: {}", e))?;

            let mut items = Vec::new();
            for market in &markets {
                let question = market["question"].as_str().unwrap_or("").to_string();
                let question_lower = question.to_lowercase();
                let desc = market["description"].as_str().unwrap_or("").to_lowercase();

                if !q.is_empty() && !question_lower.contains(&q) && !desc.contains(&q) {
                    continue;
                }

                let slug = market["slug"].as_str().unwrap_or("");
                let mut metadata = HashMap::new();
                if let Some(volume) = market["volume"].as_str() {
                    metadata.insert("volume".into(), volume.into());
                }
                if let Some(end_date) = market["end_date"].as_str() {
                    metadata.insert("end_date".into(), end_date.into());
                }
                if let Some(outcomes) = market["outcomes"].as_str() {
                    metadata.insert("outcomes".into(), outcomes.into());
                }
                if let Some(prices) = market["outcome_prices"].as_str() {
                    metadata.insert("outcome_prices".into(), prices.into());
                }
                metadata.insert(
                    "active".into(),
                    market["active"].as_bool().unwrap_or(false).to_string(),
                );

                items.push(IntelItem {
                    id: format!("polymarket:{}", market["id"].as_str().unwrap_or("")),
                    title: question,
                    content: market["description"].as_str().unwrap_or("").to_string(),
                    url: Some(format!("https://polymarket.com/event/{}", slug)),
                    timestamp: market["end_date"].as_str().map(|s| s.to_string()),
                    metadata,
                });
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "polymarket".into(),
            })
        })
    }
}

// ============================================================================
// AOI 桥接 (Area of Interest / Geofence)
// ============================================================================

pub struct AoiBridge;

impl DataSource for AoiBridge {
    fn id(&self) -> &str {
        "aoi"
    }
    fn name(&self) -> &str {
        "AOI Geofence Monitor"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for AoiBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            let (min_lat, max_lat, min_lon, max_lon) = parse_geofence_query(&q);

            let url = format!(
                "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&minlatitude={}&maxlatitude={}&minlongitude={}&maxlongitude={}",
                min_lat, max_lat, min_lon, max_lon
            );

            let resp = reqwest::get(&url)
                .await
                .map_err(|e| format!("AOI fetch failed: {}", e))?;

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("AOI parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(features) = json["features"].as_array() {
                for feature in features {
                    let props = &feature["properties"];
                    let geometry = &feature["geometry"];
                    let coords = geometry["coordinates"].as_array();

                    let (lat, lon) = coords
                        .map(|c| {
                            let lon = c.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let lat = c.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                            (lat, lon)
                        })
                        .unwrap_or((0.0, 0.0));

                    let mut metadata = HashMap::new();
                    if let Some(mag) = props["mag"].as_f64() {
                        metadata.insert("magnitude".into(), format!("{:.1}", mag));
                    }
                    metadata.insert(
                        "coordinates".into(),
                        format!("{}, {}", lat, lon),
                    );
                    if let Some(place) = props["place"].as_str() {
                        metadata.insert("place".into(), place.into());
                    }
                    if let Some(depth) = coords.and_then(|c| c.get(2)).and_then(|v| v.as_f64()) {
                        metadata.insert("depth_km".into(), format!("{:.1}", depth));
                    }

                    items.push(IntelItem {
                        id: feature["id"].as_str().unwrap_or("").to_string(),
                        title: props["title"].as_str().unwrap_or("").to_string(),
                        content: props["place"].as_str().unwrap_or("").to_string(),
                        url: props["url"].as_str().map(|s| s.to_string()),
                        timestamp: props["time"]
                            .as_i64()
                            .map(|t| {
                                chrono::DateTime::from_timestamp_millis(t)
                                    .map(|dt| dt.to_rfc3339())
                                    .unwrap_or_default()
                            }),
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "aoi".into(),
            })
        })
    }
}

fn parse_geofence_query(query: &str) -> (f64, f64, f64, f64) {
    let q = query.to_lowercase();
    if q.contains("tokyo") || q.contains("japan") {
        (34.0, 36.5, 138.0, 141.0)
    } else if q.contains("us") || q.contains("usa") || q.contains("united states") {
        (24.0, 50.0, -125.0, -66.0)
    } else if q.contains("europe") {
        (35.0, 70.0, -10.0, 40.0)
    } else if q.contains("china") {
        (18.0, 54.0, 73.0, 135.0)
    } else if q.contains("mideast") || q.contains("middle east") {
        (12.0, 42.0, 25.0, 63.0)
    } else {
        (35.0, 36.0, 139.0, 140.0)
    }
}

// ============================================================================
// ADSB 桥接
// ============================================================================

pub struct AdsbBridge;

impl DataSource for AdsbBridge {
    fn id(&self) -> &str {
        "adsb"
    }
    fn name(&self) -> &str {
        "ADS-B Flight Tracker"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for AdsbBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            let (lat, lon) = parse_adsb_query(&q);

            let url = format!("https://api.adsb.lol/v2/point/{}/{}", lat, lon);
            let resp = reqwest::get(&url)
                .await
                .map_err(|e| format!("ADS-B fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("ADS-B read failed: {}", e))?;

            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("ADS-B parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(ac_list) = json["ac"].as_array() {
                for aircraft in ac_list {
                    let icao = aircraft["icao"].as_str().unwrap_or("");
                    let callsign = aircraft["callsign"].as_str().unwrap_or("").trim();
                    let mil = aircraft["mil"].as_bool().unwrap_or(false);

                    let mut metadata = HashMap::new();
                    metadata.insert("icao".into(), icao.into());
                    metadata.insert("callsign".into(), callsign.into());
                    metadata.insert("military".into(), mil.to_string());
                    if let Some(alt) = aircraft["alt_baro"].as_i64() {
                        metadata.insert("altitude_ft".into(), alt.to_string());
                    }
                    if let Some(gs) = aircraft["gs"].as_i64() {
                        metadata.insert("ground_speed_kts".into(), gs.to_string());
                    }
                    if let Some(track) = aircraft["track"].as_i64() {
                        metadata.insert("track_deg".into(), track.to_string());
                    }
                    if let Some(ac_type) = aircraft["type"].as_str() {
                        metadata.insert("aircraft_type".into(), ac_type.into());
                    }
                    if let Some(reg) = aircraft["reg"].as_str() {
                        metadata.insert("registration".into(), reg.into());
                    }
                    metadata.insert(
                        "lat".into(),
                        aircraft["lat"].as_f64().unwrap_or(0.0).to_string(),
                    );
                    metadata.insert(
                        "lon".into(),
                        aircraft["lon"].as_f64().unwrap_or(0.0).to_string(),
                    );

                    let title = if callsign.is_empty() {
                        format!("ICAO {}", icao)
                    } else {
                        callsign.to_string()
                    };

                    items.push(IntelItem {
                        id: format!("adsb:{}", icao),
                        title,
                        content: format!(
                            "type={} mil={} alt={} gs={}",
                            aircraft["type"].as_str().unwrap_or(""),
                            mil,
                            aircraft["alt_baro"].as_i64().unwrap_or(0),
                            aircraft["gs"].as_i64().unwrap_or(0)
                        ),
                        url: Some(format!("https://globe.adsb.lol/?icao={}", icao)),
                        timestamp: None,
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "adsb".into(),
            })
        })
    }
}

fn parse_adsb_query(query: &str) -> (f64, f64) {
    let q = query.to_lowercase();
    if q.contains("tokyo") || q.contains("japan") {
        (35.6, 139.7)
    } else if q.contains("new york") || q.contains("nyc") {
        (40.7, -74.0)
    } else if q.contains("london") {
        (51.5, -0.1)
    } else if q.contains("beijing") {
        (39.9, 116.4)
    } else if q.contains("moscow") {
        (55.7, 37.6)
    } else {
        (35.6, 139.7)
    }
}

// ============================================================================
// BGPView 桥接
// ============================================================================

pub struct BgpviewBridge;

impl DataSource for BgpviewBridge {
    fn id(&self) -> &str {
        "bgpview"
    }
    fn name(&self) -> &str {
        "BGPView Network Intel"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for BgpviewBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!(
                "https://api.bgpview.io/search?query={}",
                urlencoding::encode(&q)
            );

            let resp = reqwest::get(&url)
                .await
                .map_err(|e| format!("BGPView fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("BGPView read failed: {}", e))?;

            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("BGPView parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(results) = json["data"]["results"].as_array() {
                for (i, result) in results.iter().enumerate() {
                    let r#type = result["type"].as_str().unwrap_or("");
                    let name = result["name"].as_str().unwrap_or("");
                    let description = result["description"].as_str().unwrap_or("");

                    let mut metadata = HashMap::new();
                    metadata.insert("result_type".into(), r#type.into());
                    if let Some(asn) = result["asn"].as_u64() {
                        metadata.insert("asn".into(), format!("AS{}", asn));
                    }
                    if let Some(ip) = result["ip"].as_str() {
                        metadata.insert("ip".into(), ip.into());
                    }
                    if let Some(cc) = result["country_code"].as_str() {
                        metadata.insert("country_code".into(), cc.into());
                    }

                    let title = if !name.is_empty() {
                        format!("[{}] {}", r#type, name)
                    } else {
                        format!("[{}] {}", r#type, result["ip"].as_str().unwrap_or(""))
                    };

                    items.push(IntelItem {
                        id: format!("bgpview:{}", i),
                        title,
                        content: description.to_string(),
                        url: Some(format!(
                            "https://bgpview.io/search?query={}",
                            urlencoding::encode(if !name.is_empty() { name } else { result["ip"].as_str().unwrap_or("") })
                        )),
                        timestamp: None,
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "bgpview".into(),
            })
        })
    }
}

// ============================================================================
// OpenCorporates 桥接
// ============================================================================

pub struct OpencorporatesBridge;

impl DataSource for OpencorporatesBridge {
    fn id(&self) -> &str {
        "opencorporates"
    }
    fn name(&self) -> &str {
        "OpenCorporates"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Intel]
    }
}

impl IntelSource for OpencorporatesBridge {
    fn fetch(
        &self,
        query: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IntelResult, String>> + Send>>
    {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!(
                "https://api.opencorporates.com/v0.9/companies/search?q={}&per_page=10",
                urlencoding::encode(&q)
            );

            let resp = reqwest::get(&url)
                .await
                .map_err(|e| format!("OpenCorporates fetch failed: {}", e))?;

            let body = resp
                .text()
                .await
                .map_err(|e| format!("OpenCorporates read failed: {}", e))?;

            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("OpenCorporates parse failed: {}", e))?;

            let mut items = Vec::new();
            if let Some(companies) = json["results"]["companies"].as_array() {
                for company in companies {
                    let detail = &company["company"];
                    let name = detail["name"].as_str().unwrap_or("");
                    let jurisdiction = detail["jurisdiction_code"].as_str().unwrap_or("");
                    let number = detail["company_number"].as_str().unwrap_or("");

                    let mut metadata = HashMap::new();
                    metadata.insert("jurisdiction".into(), jurisdiction.into());
                    metadata.insert("company_number".into(), number.into());
                    if let Some(inc_date) = detail["incorporation_date"].as_str() {
                        metadata.insert("incorporation_date".into(), inc_date.into());
                    }
                    if let Some(addr) = detail["registered_address_in_full"].as_str() {
                        metadata.insert("registered_address".into(), addr.into());
                    }

                    let oc_url = format!(
                        "https://opencorporates.com/companies/{}/{}",
                        jurisdiction, number
                    );

                    items.push(IntelItem {
                        id: format!("oc:{}/{}", jurisdiction, number),
                        title: name.to_string(),
                        content: format!(
                            "jurisdiction={} | incorporated={}",
                            jurisdiction,
                            detail["incorporation_date"].as_str().unwrap_or("")
                        ),
                        url: Some(oc_url),
                        timestamp: detail["incorporation_date"].as_str().map(|s| s.to_string()),
                        metadata,
                    });
                }
            }

            Ok(IntelResult {
                total: items.len(),
                items,
                source_id: "opencorporates".into(),
            })
        })
    }
}

// ============================================================================
// 通用桥接工厂
// ============================================================================

/// 创建所有情报源桥接器 (12 个)
pub fn create_intel_bridges() -> Vec<Arc<dyn IntelSource>> {
    vec![
        Arc::new(GdeltBridge),
        Arc::new(EdgarBridge),
        Arc::new(UsgsBridge),
        Arc::new(GdacsBridge),
        Arc::new(UcdpBridge),
        Arc::new(UrlhausBridge),
        Arc::new(OfacBridge),
        Arc::new(PolymarketBridge),
        Arc::new(AoiBridge),
        Arc::new(AdsbBridge),
        Arc::new(BgpviewBridge),
        Arc::new(OpencorporatesBridge),
    ]
}

/// 创建所有情报源桥接器 — 别名，供外部模块调用
pub fn bridge_all_intel_sources() -> Vec<Arc<dyn IntelSource>> {
    create_intel_bridges()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdelt_bridge() {
        let bridge = GdeltBridge;
        assert_eq!(bridge.id(), "gdelt");
        assert_eq!(bridge.domains(), vec![SourceDomain::Intel]);
    }

    #[test]
    fn test_edgar_bridge() {
        let bridge = EdgarBridge;
        assert_eq!(bridge.id(), "edgar");
        assert!(!bridge.requires_key());
    }

    #[test]
    fn test_create_all_bridges() {
        let bridges = create_intel_bridges();
        assert_eq!(bridges.len(), 12);
    }

    #[test]
    fn test_all_bridge_ids() {
        let bridges = create_intel_bridges();
        let ids: Vec<&str> = bridges.iter().map(|b| b.id()).collect();
        assert!(ids.contains(&"gdelt"));
        assert!(ids.contains(&"edgar"));
        assert!(ids.contains(&"usgs"));
        assert!(ids.contains(&"gdacs"));
        assert!(ids.contains(&"ucdp"));
        assert!(ids.contains(&"urlhaus"));
        assert!(ids.contains(&"ofac"));
        assert!(ids.contains(&"polymarket"));
        assert!(ids.contains(&"aoi"));
        assert!(ids.contains(&"adsb"));
        assert!(ids.contains(&"bgpview"));
        assert!(ids.contains(&"opencorporates"));
    }

    #[test]
    fn test_ofac_xml_parse() {
        let xml = r#"<sdnList><sdnEntry><uid>1</uid><firstName>John</firstName><lastName>Doe</lastName><sdnType>Individual</sdnType><programList><program>SDGT</program></programList></sdnEntry></sdnList>"#;
        let items = parse_ofac_xml(xml, "john");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "John Doe");
    }

    #[test]
    fn test_ofac_xml_parse_all() {
        let xml = r#"<sdnList><sdnEntry><uid>1</uid><firstName>John</firstName><lastName>Doe</lastName><sdnType>Individual</sdnType><programList><program>SDGT</program></programList></sdnEntry></sdnList>"#;
        let items = parse_ofac_xml(xml, "");
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_ofac_xml_parse_no_match() {
        let xml = r#"<sdnList><sdnEntry><uid>1</uid><firstName>John</firstName><lastName>Doe</lastName><sdnType>Individual</sdnType><programList><program>SDGT</program></programList></sdnEntry></sdnList>"#;
        let items = parse_ofac_xml(xml, "alice");
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_geofence_query_parse() {
        assert_eq!(parse_geofence_query("tokyo earthquakes"), (34.0, 36.5, 138.0, 141.0));
        assert_eq!(parse_geofence_query("us earthquakes"), (24.0, 50.0, -125.0, -66.0));
        assert_eq!(parse_geofence_query("random"), (35.0, 36.0, 139.0, 140.0));
    }

    #[test]
    fn test_adsb_query_parse() {
        assert_eq!(parse_adsb_query("tokyo flights"), (35.6, 139.7));
        assert_eq!(parse_adsb_query("new york flights"), (40.7, -74.0));
        assert_eq!(parse_adsb_query("random"), (35.6, 139.7));
    }
}
