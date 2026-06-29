use base64::Engine;

pub fn base64_decode(text: &str) -> Result<String, String> {
    let text = text.trim().replace('\n', "");
    let text = if text.contains('-') || text.contains('_') {
        text.replace('-', "+").replace('_', "/")
    } else {
        text
    };
    let padded = match text.len() % 4 {
        0 => text,
        r => {
            let pad = 4 - r;
            format!("{}{}", text, "=".repeat(pad))
        }
    };
    let bytes = Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        padded.as_bytes(),
    )
    .map_err(|e| format!("base64 decode error: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("utf8 error: {}", e))
}

pub const FREE_PROXY_SCRAPERS: &[(&str, &str)] = &[
    ("geonode", "https://proxylist.geonode.com/api/proxy-list?limit=100&page=1&sort_by=lastChecked&sort_type=desc"),
    ("proxy-list", "https://www.proxy-list.download/api/v1/get?type=http"),
    ("proxyscrape", "https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all"),
];

pub const DEFAULT_SUBSCRIPTIONS: &[&str] = &[
    "https://raw.githubusercontent.com/freefq/free/master/v2",
    "https://raw.githubusercontent.com/mahdibland/ShadowsocksAggregator/master/Eternity.txt",
    "https://raw.githubusercontent.com/ssrsub/ssr/master/ss-sub",
];
