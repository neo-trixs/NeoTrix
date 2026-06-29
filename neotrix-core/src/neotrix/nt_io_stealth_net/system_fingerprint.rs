//! 系统级指纹隐匿模块 v2 — 多浏览器 + 移动平台 + TLS 变体
//!
//! 对标开源项目:
//! - **BrowserForge**: 真实世界浏览器指纹分布
//! - **curl-impersonate**: TLS 指纹伪造 (JA3)
//! - **undetectable-fingerprint-nt_world_browse**: Canvas/WebGL/Platform 隐匿
//!
//! 覆盖维度:
//! - 浏览器品牌 (Chrome/Firefox/Safari/Edge) 独立 UA + Sec-CH-UA
//! - 平台/OS 一致性 (移动端 Android/iOS + 桌面端 Win/Mac/Linux/CrOS)
//! - TLS 指纹提示 (JA3 分片大小/顺序, 各浏览器不同)
//! - 多模态浏览器指纹 (Canvas/WebGL/Fonts/Audio/Screen)
//! - 时区/语言环境 (Accept-Language, Sec-CH-UA)
//! - 硬件指纹 (并发数/内存/屏幕)
//! - DNS 配置隐匿 (强制远端解析)

use rand::Rng;
use std::collections::HashMap;

use crate::neotrix::nt_io_http_factory::H2SettingsProfile;
use crate::neotrix::nt_io_http_factory::TlsVariant;

/// 浏览器品牌 — 独立影响 User-Agent + Sec-CH-UA + TLS 参数
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
    Edge,
}

impl Browser {
    pub fn all() -> &'static [Browser] {
        &[
            Browser::Chrome,
            Browser::Firefox,
            Browser::Safari,
            Browser::Edge,
        ]
    }

    pub fn compatible_platforms(&self) -> &'static [Platform] {
        match self {
            Browser::Chrome => &[
                Platform::Windows,
                Platform::MacOS,
                Platform::Linux,
                Platform::ChromeOS,
                Platform::Android,
            ],
            Browser::Firefox => &[
                Platform::Windows,
                Platform::MacOS,
                Platform::Linux,
                Platform::Android,
            ],
            Browser::Safari => &[Platform::MacOS, Platform::IOS],
            Browser::Edge => &[
                Platform::Windows,
                Platform::MacOS,
                Platform::Linux,
                Platform::Android,
            ],
        }
    }

    pub fn preferred_tls_variant(&self) -> TlsVariant {
        match self {
            Browser::Chrome | Browser::Edge => TlsVariant::ModernH2,
            Browser::Firefox => TlsVariant::LegacyHttp11,
            Browser::Safari => TlsVariant::LegacyStrict,
        }
    }

    pub fn preferred_h2_profile(&self) -> H2SettingsProfile {
        match self {
            Browser::Chrome => H2SettingsProfile::ChromeDefault,
            Browser::Firefox => H2SettingsProfile::FirefoxDefault,
            Browser::Safari => H2SettingsProfile::SafariDefault,
            Browser::Edge => H2SettingsProfile::EdgeDefault,
        }
    }

    pub fn sec_ch_ua_brand(&self, major_version: &str) -> String {
        match self {
            Browser::Chrome => format!(
                "\"Not)A;Brand\";v=\"99\", \"Google Chrome\";v=\"{}\", \"Chromium\";v=\"{}\"",
                major_version, major_version
            ),
            Browser::Edge => format!(
                "\"Not)A;Brand\";v=\"99\", \"Microsoft Edge\";v=\"{}\", \"Chromium\";v=\"{}\"",
                major_version, major_version
            ),
            Browser::Firefox | Browser::Safari => String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    ChromeOS,
    Android,
    IOS,
}

impl Platform {
    pub fn all() -> &'static [Platform] {
        &[
            Platform::Windows,
            Platform::MacOS,
            Platform::Linux,
            Platform::ChromeOS,
            Platform::Android,
            Platform::IOS,
        ]
    }

    pub fn is_mobile(&self) -> bool {
        matches!(self, Platform::Android | Platform::IOS)
    }

    pub fn user_agent_os(&self) -> &'static str {
        match self {
            Platform::Windows => "Windows NT 10.0; Win64; x64",
            Platform::MacOS => "Macintosh; Intel Mac OS X 10_15_7",
            Platform::Linux => "X11; Linux x86_64",
            Platform::ChromeOS => "X11; CrOS x86_64 14595.92.0",
            Platform::Android => "Linux; Android 14",
            Platform::IOS => "iPhone; CPU iPhone OS 17_2 like Mac OS X",
        }
    }

    pub fn sec_ch_ua_platform(&self) -> &'static str {
        match self {
            Platform::Windows => "\"Windows\"",
            Platform::MacOS => "\"macOS\"",
            Platform::Linux => "\"Linux\"",
            Platform::ChromeOS => "\"Chrome OS\"",
            Platform::Android => "\"Android\"",
            Platform::IOS => "\"iOS\"",
        }
    }

    pub fn navigator_platform(&self) -> &'static str {
        match self {
            Platform::Windows => "Win32",
            Platform::MacOS => "MacIntel",
            Platform::Linux => "Linux x86_64",
            Platform::ChromeOS => "Linux x86_64",
            Platform::Android => "Linux armv8l",
            Platform::IOS => "iPhone",
        }
    }

    pub fn default_timezone(&self) -> &'static str {
        match self {
            Platform::Windows => "America/New_York",
            Platform::MacOS => "America/New_York",
            Platform::Linux => "Etc/UTC",
            Platform::ChromeOS => "America/Los_Angeles",
            Platform::Android => "Asia/Shanghai",
            Platform::IOS => "America/New_York",
        }
    }

    pub fn default_locale(&self) -> &'static str {
        match self {
            Platform::Windows => "en-US",
            Platform::MacOS => "en-US",
            Platform::Linux => "en_US.UTF-8",
            Platform::ChromeOS => "en-US",
            Platform::Android => "en-US",
            Platform::IOS => "en-US",
        }
    }

    pub fn concurrency_range(&self) -> (u8, u8) {
        match self {
            Platform::Windows => (4, 16),
            Platform::MacOS => (4, 12),
            Platform::Linux => (2, 32),
            Platform::ChromeOS => (2, 8),
            Platform::Android => (4, 8),
            Platform::IOS => (4, 6),
        }
    }

    pub fn memory_range(&self) -> (u8, u8) {
        match self {
            Platform::Windows => (4, 64),
            Platform::MacOS => (8, 32),
            Platform::Linux => (2, 128),
            Platform::ChromeOS => (4, 16),
            Platform::Android => (4, 16),
            Platform::IOS => (3, 8),
        }
    }

    /// 桌面端常用屏幕分辨率列表（宽,高）
    pub fn desktop_screens() -> &'static [(u16, u16)] {
        &[
            (1920, 1080),
            (1366, 768),
            (2560, 1440),
            (1920, 1200),
            (1536, 864),
            (1440, 900),
            (1680, 1050),
            (1280, 720),
            (2560, 1600),
            (3440, 1440),
            (3840, 2160),
        ]
    }

    /// 移动端常用屏幕分辨率
    pub fn mobile_screens() -> &'static [(u16, u16)] {
        &[
            (390, 844),
            (393, 852),
            (430, 932),
            (414, 896),
            (375, 812),
            (412, 915),
            (360, 780),
            (1080, 2400),
            (1440, 3120),
            (1080, 2340),
        ]
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TlsFingerprintHint {
    pub alpn: Vec<String>,
    pub cipher_order: Vec<String>,
    pub tls_version: String,
    pub http_version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemFingerprint {
    pub platform: Platform,
    pub nt_world_browse: Browser,
    pub timezone: String,
    pub locale: String,
    pub accept_language: String,
    pub sec_ch_ua: String,
    pub sec_ch_ua_platform: String,
    pub sec_ch_ua_mobile: String,
    pub hardware_concurrency: u8,
    pub device_memory: u8,
    pub dns_leak_protection: bool,
    pub tls_fingerprint_hint: TlsFingerprintHint,
    pub tls_variant: TlsVariant,
    pub nt_world_browse_fp: BrowserFingerprintProfile,
}

impl Default for TlsFingerprintHint {
    fn default() -> Self {
        Self {
            alpn: vec!["h2".into(), "http/1.1".into()],
            cipher_order: vec![
                "TLS_AES_128_GCM_SHA256".into(),
                "TLS_AES_256_GCM_SHA384".into(),
                "TLS_CHACHA20_POLY1305_SHA256".into(),
            ],
            tls_version: "TLSv1.3".into(),
            http_version: "HTTP/2".into(),
        }
    }
}

/// 多模态浏览器指纹 — Canvas/WebGL/Fonts/Audio/Screen 参数组合
/// 从 (Platform, H2SettingsProfile) 派生，保证一致性
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserFingerprintProfile {
    pub canvas_noise_level: f64,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
    pub fonts: Vec<String>,
    pub audio_context_noise: f64,
    pub screen_width: u16,
    pub screen_height: u16,
    pub color_depth: u8,
    pub hardware_concurrency: u8,
    pub device_memory_gb: u8,
}

impl BrowserFingerprintProfile {
    /// 从平台 + H2 配置推导浏览器指纹
    pub fn from_platform_and_h2(platform: Platform, h2: H2SettingsProfile) -> Self {
        match (platform, h2) {
            (Platform::Windows, _) | (Platform::ChromeOS, _) => Self::chrome_windows(),
            (Platform::MacOS, H2SettingsProfile::SafariDefault) => Self::safari_macos(),
            (Platform::MacOS, _) => Self::chrome_macos(),
            (Platform::Linux, H2SettingsProfile::FirefoxDefault) => Self::firefox_linux(),
            (Platform::Linux, _) => Self::chrome_linux(),
            (Platform::Android, _) => Self::chrome_android(),
            (Platform::IOS, _) => Self::safari_macos(),
        }
    }

    fn chrome_windows() -> Self {
        Self {
            canvas_noise_level: 0.01,
            webgl_vendor: "Google Inc. (Intel)".into(),
            webgl_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 620 (0x00005917) Direct3D11 vs_5_0 ps_5_0, D3D11)".into(),
            fonts: vec!["Arial".into(), "Calibri".into(), "Cambria".into(), "Consolas".into(),
                        "Georgia".into(), "Microsoft Sans Serif".into(), "Segoe UI".into(),
                        "Tahoma".into(), "Times New Roman".into(), "Trebuchet MS".into(),
                        "Verdana".into(), "Webdings".into(), "Wingdings".into()],
            audio_context_noise: 0.0001,
            screen_width: 1920, screen_height: 1080, color_depth: 24,
            hardware_concurrency: 8, device_memory_gb: 8,
        }
    }

    fn chrome_macos() -> Self {
        Self {
            canvas_noise_level: 0.008,
            webgl_vendor: "Google Inc. (Apple)".into(),
            webgl_renderer: "ANGLE (Apple, Apple M2 Pro, OpenGL 4.1)".into(),
            fonts: vec![
                "Academy Engraved LET".into(),
                "American Typewriter".into(),
                "Apple Color Emoji".into(),
                "Apple SD Gothic Neo".into(),
                "Arial".into(),
                "Avenir".into(),
                "Baskerville".into(),
                "Big Caslon".into(),
                "Bodoni 72".into(),
                "Bradley Hand".into(),
                "Chalkboard SE".into(),
                "Cochin".into(),
                "Copperplate".into(),
                "Courier New".into(),
                "Didot".into(),
                "Futura".into(),
                "Geneva".into(),
                "Georgia".into(),
                "Gill Sans".into(),
                "Helvetica".into(),
                "Menlo".into(),
                "Monaco".into(),
                "Optima".into(),
                "Palatino".into(),
                "Papyrus".into(),
                "San Francisco".into(),
                "Savoye LET".into(),
                "Snell Roundhand".into(),
                "Times New Roman".into(),
                "Trebuchet MS".into(),
                "Verdana".into(),
            ],
            audio_context_noise: 0.0002,
            screen_width: 2560,
            screen_height: 1600,
            color_depth: 30,
            hardware_concurrency: 12,
            device_memory_gb: 16,
        }
    }

    fn safari_macos() -> Self {
        let mut p = Self::chrome_macos();
        p.webgl_vendor = "Apple Inc.".into();
        p.webgl_renderer = "Apple M2 Pro GPU".into();
        p.canvas_noise_level = 0.0;
        p.audio_context_noise = 0.0;
        p.screen_width = 1728;
        p.screen_height = 1117;
        p.color_depth = 30;
        p.hardware_concurrency = 10;
        p.device_memory_gb = 16;
        p
    }

    fn firefox_linux() -> Self {
        Self {
            canvas_noise_level: 0.02,
            webgl_vendor: "Mozilla Foundation".into(),
            webgl_renderer: "Mesa/X.org (Intel, Intel Graphics (RPL-P), llvmpipe)".into(),
            fonts: vec![
                "Bitstream Charter".into(),
                "Cantarell".into(),
                "Courier 10 Pitch".into(),
                "DejaVu Sans".into(),
                "DejaVu Sans Mono".into(),
                "DejaVu Serif".into(),
                "Droid Sans".into(),
                "Droid Sans Mono".into(),
                "Droid Serif".into(),
                "Fira Sans".into(),
                "FreeMono".into(),
                "FreeSans".into(),
                "FreeSerif".into(),
                "Liberation Mono".into(),
                "Liberation Sans".into(),
                "Liberation Serif".into(),
                "Noto Sans".into(),
                "Noto Sans Mono".into(),
                "Noto Serif".into(),
                "Open Sans".into(),
                "Roboto".into(),
                "Source Code Pro".into(),
                "Ubuntu".into(),
            ],
            audio_context_noise: 0.00005,
            screen_width: 1920,
            screen_height: 1080,
            color_depth: 24,
            hardware_concurrency: 4,
            device_memory_gb: 4,
        }
    }

    fn chrome_linux() -> Self {
        let mut p = Self::firefox_linux();
        p.webgl_vendor = "Google Inc. (Intel)".into();
        p.webgl_renderer = "ANGLE (Intel, Intel Graphics (RPL-P), Vulkan 1.3)".into();
        p.audio_context_noise = 0.00015;
        p.hardware_concurrency = 8;
        p.device_memory_gb = 8;
        p.fonts.push("Google Sans".into());
        p.fonts.push("Product Sans".into());
        p
    }

    fn chrome_android() -> Self {
        Self {
            canvas_noise_level: 0.005,
            webgl_vendor: "Google Inc. (Qualcomm)".into(),
            webgl_renderer: "ANGLE (Qualcomm, Adreno 730, Vulkan 1.1)".into(),
            fonts: vec![
                "Droid Sans Mono".into(),
                "Noto Sans".into(),
                "Roboto".into(),
                "Google Sans".into(),
                "Noto Color Emoji".into(),
            ],
            audio_context_noise: 0.0003,
            screen_width: 1080,
            screen_height: 2400,
            color_depth: 24,
            hardware_concurrency: 8,
            device_memory_gb: 8,
        }
    }

    /// 转为 CDP JavaScript 注入指令
    pub fn to_cdp_js(&self) -> String {
        format!(
            r#"
// Canvas noise
const _orig_getImageData = HTMLCanvasElement.prototype.getImageData;
HTMLCanvasElement.prototype.getImageData = function(x, y, w, h) {{
    const imageData = _orig_getImageData.call(this, x, y, w, h);
    const noise = {};
    for (let i = 0; i < imageData.data.length; i += 4) {{
        imageData.data[i] += (Math.random() - 0.5) * noise * 256;
        imageData.data[i+1] += (Math.random() - 0.5) * noise * 256;
        imageData.data[i+2] += (Math.random() - 0.5) * noise * 256;
    }}
    return imageData;
}};
// WebGL
const _orig_getParameter = WebGLRenderingContext.prototype.getParameter;
WebGLRenderingContext.prototype.getParameter = function(p) {{
    if (p === 37445) return "{}";
    if (p === 37446) return "{}";
    return _orig_getParameter.call(this, p);
}};
// Navigator overrides
Object.defineProperty(navigator, 'hardwareConcurrency', {{ get: () => {} }});
Object.defineProperty(navigator, 'deviceMemory', {{ get: () => {} }});
// Screen
Object.defineProperty(screen, 'width', {{ get: () => {} }});
Object.defineProperty(screen, 'height', {{ get: () => {} }});
Object.defineProperty(screen, 'colorDepth', {{ get: () => {} }});
"#,
            self.canvas_noise_level,
            self.webgl_vendor,
            self.webgl_renderer,
            self.hardware_concurrency,
            self.device_memory_gb,
            self.screen_width,
            self.screen_height,
            self.color_depth
        )
    }

    /// 转为 HTTP 头（用于非浏览器请求）
    pub fn to_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "X-Fingerprint-Canvas",
                format!("{:.4}", self.canvas_noise_level),
            ),
            ("X-Fingerprint-WebGL-Vendor", self.webgl_vendor.clone()),
            ("X-Fingerprint-WebGL-Renderer", self.webgl_renderer.clone()),
            (
                "X-Fingerprint-Fonts",
                self.fonts.join(", ").chars().take(200).collect(),
            ),
            (
                "X-Fingerprint-Screen",
                format!(
                    "{}x{}x{}",
                    self.screen_width, self.screen_height, self.color_depth
                ),
            ),
            (
                "X-Fingerprint-Hardware",
                format!(
                    "{} cores, {}GB RAM",
                    self.hardware_concurrency, self.device_memory_gb
                ),
            ),
        ]
    }
}

/// 系统指纹生成配置
#[derive(Debug, Clone)]
pub struct SystemFingerprintConfig {
    pub platform: Option<Platform>,
    pub nt_world_browse: Option<Browser>,
    pub h2_profile: Option<H2SettingsProfile>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub auto_consistent: bool,
}

impl Default for SystemFingerprintConfig {
    fn default() -> Self {
        Self {
            platform: None,
            nt_world_browse: None,
            h2_profile: None,
            timezone: None,
            locale: None,
            auto_consistent: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemFingerprintGenerator;

impl Default for SystemFingerprintGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemFingerprintGenerator {
    pub fn new() -> Self {
        Self
    }

    /// 生成一致的系统指纹（所有维度自动对齐平台 + 浏览器 + TLS）
    /// 对标 BrowserForge: 加权随机分布
    ///   桌面: Windows ~50%, macOS ~20%, Linux ~18%, ChromeOS ~5%
    ///   移动: Android ~5%, iOS ~2%
    ///   浏览器: Chrome ~55%, Firefox ~20%, Safari ~15%, Edge ~10%
    pub fn generate(&self, config: &SystemFingerprintConfig) -> SystemFingerprint {
        let mut rng = rand::thread_rng();

        let platform = config.platform.unwrap_or_else(|| {
            let roll: f64 = rng.gen();
            if roll < 0.50 {
                Platform::Windows
            } else if roll < 0.70 {
                Platform::MacOS
            } else if roll < 0.85 {
                Platform::Linux
            } else if roll < 0.93 {
                Platform::ChromeOS
            } else if roll < 0.97 {
                Platform::Android
            } else {
                Platform::IOS
            }
        });

        let nt_world_browse = config.nt_world_browse.unwrap_or_else(|| {
            let compat = Browser::all()
                .iter()
                .filter(|b| b.compatible_platforms().contains(&platform))
                .copied()
                .collect::<Vec<_>>();
            let roll: f64 = rng.gen();
            // 优先选 Chrome, 然后 Firefox/Safari/Edge
            for &b in &[
                Browser::Chrome,
                Browser::Firefox,
                Browser::Safari,
                Browser::Edge,
            ] {
                if compat.contains(&b) {
                    if roll < 0.55 && b == Browser::Chrome {
                        return b;
                    }
                    if roll < 0.75 && b == Browser::Firefox {
                        return b;
                    }
                    if roll < 0.90 && b == Browser::Safari {
                        return b;
                    }
                    if roll < 1.00 && b == Browser::Edge {
                        return b;
                    }
                }
            }
            compat[0]
        });

        let h2 = config
            .h2_profile
            .unwrap_or_else(|| nt_world_browse.preferred_h2_profile());
        let tls_variant = nt_world_browse.preferred_tls_variant();

        let timezone = config
            .timezone
            .clone()
            .unwrap_or_else(|| platform.default_timezone().to_string());

        let locale = config
            .locale
            .clone()
            .unwrap_or_else(|| platform.default_locale().to_string());

        let accept_language = Self::accept_language_from_locale(&locale);

        let (concurrency_min, concurrency_max) = platform.concurrency_range();
        let hardware_concurrency = rng.gen_range(concurrency_min..=concurrency_max);

        let (mem_min, mem_max) = platform.memory_range();
        let device_memory = {
            let mem = rng.gen_range(mem_min..=mem_max);
            mem.next_power_of_two()
        };

        let days_since_epoch = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400;

        let chrome_base = 120u32 + (days_since_epoch / 90) as u32;
        let _ff_base = 121u32 + (days_since_epoch / 90) as u32;
        let _safari_base = 17u32 + (days_since_epoch / 90) as u32;

        let (sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform) = match nt_world_browse {
            Browser::Chrome | Browser::Edge => {
                let major_version = chrome_base.to_string();
                let brand_str = nt_world_browse.sec_ch_ua_brand(&major_version);
                (
                    brand_str,
                    if platform.is_mobile() {
                        "?1".into()
                    } else {
                        "?0".into()
                    },
                    platform.sec_ch_ua_platform().to_string(),
                )
            }
            Browser::Firefox | Browser::Safari => (
                String::new(),
                "?0".into(),
                platform.sec_ch_ua_platform().to_string(),
            ),
        };

        let tls_fingerprint_hint = match nt_world_browse {
            Browser::Chrome | Browser::Edge => TlsFingerprintHint {
                alpn: vec!["h2".into(), "http/1.1".into()],
                cipher_order: vec![
                    "TLS_AES_128_GCM_SHA256".into(),
                    "TLS_AES_256_GCM_SHA384".into(),
                    "TLS_CHACHA20_POLY1305_SHA256".into(),
                ],
                tls_version: "TLSv1.3".into(),
                http_version: "HTTP/2".into(),
            },
            Browser::Firefox => TlsFingerprintHint {
                alpn: vec!["h2".into(), "http/1.1".into()],
                cipher_order: vec![
                    "TLS_AES_128_GCM_SHA256".into(),
                    "TLS_CHACHA20_POLY1305_SHA256".into(),
                    "TLS_AES_256_GCM_SHA384".into(),
                ],
                tls_version: "TLSv1.3".into(),
                http_version: "HTTP/2".into(),
            },
            Browser::Safari => TlsFingerprintHint {
                alpn: vec!["h2".into(), "http/1.1".into()],
                cipher_order: vec![
                    "TLS_AES_128_GCM_SHA256".into(),
                    "TLS_AES_256_GCM_SHA384".into(),
                    "TLS_CHACHA20_POLY1305_SHA256".into(),
                ],
                tls_version: "TLSv1.3".into(),
                http_version: "HTTP/2".into(),
            },
        };

        SystemFingerprint {
            platform,
            nt_world_browse,
            timezone,
            locale,
            accept_language,
            sec_ch_ua,
            sec_ch_ua_platform,
            sec_ch_ua_mobile,
            hardware_concurrency,
            device_memory,
            dns_leak_protection: true,
            tls_fingerprint_hint,
            tls_variant,
            nt_world_browse_fp: BrowserFingerprintProfile::from_platform_and_h2(platform, h2),
        }
    }

    /// 从 locale 推导 Accept-Language
    fn accept_language_from_locale(locale: &str) -> String {
        let lang = locale.split('.').next().unwrap_or(locale);
        let base = lang.replace('_', "-");
        match lang {
            "en-US" => "en-US,en;q=0.9".into(),
            "zh-CN" | "zh_CN" => "zh-CN,zh;q=0.9,en;q=0.8".into(),
            "ja-JP" | "ja_JP" => "ja-JP,ja;q=0.9,en;q=0.8".into(),
            "ko-KR" | "ko_KR" => "ko-KR,ko;q=0.9,en;q=0.8".into(),
            "fr-FR" | "fr_FR" => "fr-FR,fr;q=0.9,en;q=0.8".into(),
            "de-DE" | "de_DE" => "de-DE,de;q=0.9,en;q=0.8".into(),
            "es-ES" | "es_ES" => "es-ES,es;q=0.9,en;q=0.8".into(),
            "pt-BR" | "pt_BR" => "pt-BR,pt;q=0.9,en;q=0.8".into(),
            _ => format!("{},en;q=0.9", base),
        }
    }

    /// 转换为 HTTP 头映射（可注入任何 HTTP 客户端）
    /// 对标 curl-impersonate: 平台相关 Arch/Bitness 动态生成
    pub fn to_headers(fp: &SystemFingerprint) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Accept-Language".into(), fp.accept_language.clone());
        // Chrome/Edge 发 Sec-CH-UA, Firefox/Safari 不发
        if !fp.sec_ch_ua.is_empty() {
            headers.insert("Sec-CH-UA".into(), fp.sec_ch_ua.clone());
            headers.insert("Sec-CH-UA-Platform".into(), fp.sec_ch_ua_platform.clone());
            headers.insert("Sec-CH-UA-Mobile".into(), fp.sec_ch_ua_mobile.clone());
            let (arch, bitness) = match fp.platform {
                Platform::Windows | Platform::Linux => ("x86", "64"),
                Platform::MacOS => ("arm64", "64"),
                Platform::ChromeOS => ("x86", "64"),
                Platform::Android => ("arm64", "64"),
                Platform::IOS => ("arm64", "64"),
            };
            headers.insert("Sec-CH-UA-Arch".into(), arch.into());
            headers.insert("Sec-CH-UA-Bitness".into(), bitness.into());
            headers.insert("Sec-CH-UA-Model".into(), "".into());
        }
        headers.insert("DNT".into(), "1".into());
        for (k, v) in fp.nt_world_browse_fp.to_headers() {
            headers.insert(k.to_string(), v);
        }
        headers
    }

    /// 验证指纹各维度一致性
    pub fn validate_consistency(fp: &SystemFingerprint) -> Vec<String> {
        let mut issues = Vec::new();

        if !fp
            .nt_world_browse
            .compatible_platforms()
            .contains(&fp.platform)
        {
            issues.push(format!(
                "Browser {:?} not compatible with platform {:?}",
                fp.nt_world_browse, fp.platform
            ));
        }

        if !fp.sec_ch_ua.is_empty() {
            // 只有 Chrome/Edge 发 Sec-CH-UA, 必须与 platform 匹配
            if fp.sec_ch_ua_platform != fp.platform.sec_ch_ua_platform() {
                issues.push(format!(
                    "Sec-CH-UA-Platform mismatch: expected {}, got {}",
                    fp.platform.sec_ch_ua_platform(),
                    fp.sec_ch_ua_platform
                ));
            }
        }

        if fp.hardware_concurrency == 0 || fp.hardware_concurrency > 128 {
            issues.push(format!(
                "Unrealistic hardware_concurrency: {}",
                fp.hardware_concurrency
            ));
        }

        if fp.device_memory == 0 || fp.device_memory > 128 {
            issues.push(format!("Unrealistic device_memory: {}", fp.device_memory));
        }

        let expected_webgl_prefix = match fp.platform {
            Platform::Windows => "Google Inc. (Intel)",
            Platform::MacOS => "Google Inc. (Apple)",
            Platform::Linux => "Google Inc. (Intel)",
            Platform::ChromeOS => "Google Inc. (Intel)",
            Platform::Android | Platform::IOS => "",
        };
        if !fp
            .nt_world_browse_fp
            .webgl_vendor
            .starts_with(expected_webgl_prefix)
            && !fp.nt_world_browse_fp.webgl_vendor.contains("Apple Inc")
            && !fp.platform.is_mobile()
        {
            issues.push(format!(
                "WebGL vendor '{}' doesn't match platform {:?} (expected '{}')",
                fp.nt_world_browse_fp.webgl_vendor, fp.platform, expected_webgl_prefix
            ));
        }

        let expected_min_height = match fp.platform {
            Platform::MacOS => 900,
            Platform::Android | Platform::IOS => 700,
            _ => 720,
        };
        if fp.nt_world_browse_fp.screen_height < expected_min_height {
            issues.push(format!(
                "Screen height {} too low for {:?}",
                fp.nt_world_browse_fp.screen_height, fp.platform
            ));
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_consistency() {
        let gen = SystemFingerprintGenerator::new();
        let config = SystemFingerprintConfig {
            platform: Some(Platform::Windows),
            ..Default::default()
        };
        let fp = gen.generate(&config);
        assert_eq!(fp.platform, Platform::Windows);
        assert!(fp.hardware_concurrency >= 4);
        assert!(fp.hardware_concurrency <= 16);
        assert!(!fp.accept_language.is_empty());
    }

    #[test]
    fn test_random_platform_generation() {
        let gen = SystemFingerprintGenerator::new();
        let fp = gen.generate(&SystemFingerprintConfig::default());
        assert!(fp.timezone.len() > 0);
        assert!(fp.locale.len() > 0);
        assert!(fp.hardware_concurrency > 0);
        assert!(fp.device_memory > 0);
    }

    #[test]
    fn test_headers_injection() {
        let gen = SystemFingerprintGenerator::new();
        let fp = gen.generate(&SystemFingerprintConfig::default());
        let headers = SystemFingerprintGenerator::to_headers(&fp);
        assert!(headers.contains_key("Accept-Language"));
        assert!(headers.contains_key("DNT"));
        // Sec-CH-UA only present for Chrome/Edge
        if matches!(fp.nt_world_browse, Browser::Chrome | Browser::Edge) {
            assert!(headers.contains_key("Sec-CH-UA"));
            assert!(headers.contains_key("Sec-CH-UA-Platform"));
        }
    }

    #[test]
    fn test_validation_no_issues_for_consistent_fingerprint() {
        let gen = SystemFingerprintGenerator::new();
        let config = SystemFingerprintConfig {
            platform: Some(Platform::MacOS),
            ..Default::default()
        };
        let fp = gen.generate(&config);
        let issues = SystemFingerprintGenerator::validate_consistency(&fp);
        assert!(issues.is_empty(), "Issues found: {:?}", issues);
    }

    #[test]
    fn test_accept_language_derivation() {
        let lang = SystemFingerprintGenerator::accept_language_from_locale("zh-CN");
        assert!(lang.contains("zh-CN"));
        assert!(lang.contains("en"));
    }

    #[test]
    fn test_platform_timezone_default() {
        assert_eq!(Platform::Windows.default_timezone(), "America/New_York");
        assert_eq!(Platform::MacOS.default_timezone(), "America/New_York");
        assert!(Platform::Linux.default_timezone().len() > 0);
    }
}
