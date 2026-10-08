//! types — 从 `nt_io_browser_engine.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。



/// Browser action
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BrowserAction {
    /// Navigate to URL
    Navigate { url: String },
    /// Click element (v1: 仅支持有 href 的 <a>；submit 控件请用 SubmitForm)
    Click { selector: String },
    /// Type text into input (按 name 键记录待提交值)
    Type { selector: String, text: String },
    /// Take screenshot（仅 ChromeHeadless 后端）
    Screenshot,
    /// Get page content（标题 + 正文 + 链接表 + 表单表）
    GetContent,
    /// Scroll page（静态抓取下仅记录意图）
    Scroll { direction: ScrollDirection, amount: Option<f64> },
    /// Fill form field (按 name 键记录待提交值)
    FillField { selector: String, value: String },
    /// Select dropdown option(s)（Http：按 name 记录；Cdp：真设置 + change 事件）
    SelectOption {
        selector: String,
        values: Vec<String>,
    },
    /// Check checkbox（Http：按 name 记录选中值；Cdp：未选中才点）
    Check { selector: String },
    /// Uncheck checkbox（Http：删除待填值；Cdp：已选中才点）
    Uncheck { selector: String },
    /// Press key on focused element（Cdp 真按键；Http 仅记录）
    Press { selector: String, key: String },
    /// Get text of element（快照子集/innerText，比整页 GetContent 省 token）
    GetText { selector: String },
    /// Print page to PDF（ChromeHeadless/Cdp 真打印；Http 不支持）
    PrintPdf,
    /// Save session to JSON file（token 默认脱敏：Literal 置空，File 源加载时自动热加载）
    SaveState { path: String },
    /// Load session from JSON file（新 ID，老会话不覆盖；需同版本格式）
    LoadState { path: String },
    /// Download URL to file（Http 真下载，50MB 上限；Chrome/Cdp 不支持）
    Download { url: String, path: String },
    /// Record file(s) for upload input（提交时 multipart 真上传；路径即时校验存在性）
    Upload {
        selector: String,
        files: Vec<String>,
    },
    /// Submit form (`None` = 第一个表单；`#<n>` = 按序号；CSS = 在快照 HTML 中匹配)
    SubmitForm { selector: Option<String> },
    /// Wait for element（轮询重抓直到匹配或超时）
    WaitForElement { selector: String, timeout_ms: u64 },
    /// Sleep（等待异步渲染/执行完成；钳制 ≤60s，超长请链多个 Sleep）
    Sleep { ms: u64 },
    /// Execute JavaScript（dump-dom 管道不支持，如实报错，CDP 在途）
    ExecuteJs { script: String },
    /// Go back
    GoBack,
    /// Go forward
    GoForward,
    /// Reload page
    Reload,
    /// Close tab
    CloseTab,
    /// Open new tab
    NewTab { url: Option<String> },
    /// Switch tab
    SwitchTab { index: usize },
}

/// 后端种类
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum BackendKind {
    /// 纯内存占位（默认）：零网络，单测 hermetic
    #[default]
    Mock,
    /// 真抓取：HTTP + 自研 CookieJar + scraper DOM
    Http,
    /// 真渲染：系统 Chrome headless（无 profile）
    ChromeHeadless,
    /// 真操控：chromiumoxide CDP（点击/输入/求值/截图/Cookie 双向同步）
    Cdp,
}


/// 后端能力声明（诚实面：不支持的动作直接报错，不装成功）
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct BackendCaps {
    pub javascript: bool,
    pub screenshot: bool,
    pub form_submit: bool,
    pub cookie_persist: bool,
}

impl BackendKind {
    pub fn caps(self) -> BackendCaps {
        match self {
            BackendKind::Mock => BackendCaps {
                javascript: false,
                screenshot: false,
                form_submit: false,
                cookie_persist: false,
            },
            BackendKind::Http => BackendCaps {
                javascript: false,
                screenshot: false,
                form_submit: true,
                cookie_persist: true,
            },
            BackendKind::ChromeHeadless => BackendCaps {
                javascript: true,
                screenshot: true,
                form_submit: true,
                cookie_persist: false,
            },
            BackendKind::Cdp => BackendCaps {
                javascript: true,
                screenshot: true,
                form_submit: true,
                cookie_persist: true,
            },
        }
    }
}

/// Scroll direction
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Browser action result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserResult {
    /// Whether action succeeded
    pub success: bool,
    /// Output (text content, screenshot path, etc.)
    pub output: String,
    /// Current URL
    pub current_url: String,
    /// Page title
    pub title: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// 动作后验证证据（tsaagan 式 verify-first：URL 变化/状态码/内容规模；console/network 仅 CDP 在途）
    pub verify: Option<VerifyBlock>,
}

/// 动作后验证证据（与输出同行返回，免二次快照）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifyBlock {
    /// 请求 URL 与最终 URL 是否不同（跳转/提交生效信号）
    pub url_changed: bool,
    /// 最终 URL
    pub final_url: String,
    /// HTTP 状态码（Http/Cdp；Mock/Chrome 为 None）
    pub http_status: Option<u16>,
    /// 正文字符数
    pub text_len: usize,
    /// 链接数/表单数（可交互规模）
    pub links: usize,
    pub forms: usize,
    /// 控制台错误数（Http 未知 = None；CDP 事件订阅在途）
    pub console_errors: Option<u64>,
    /// 失败请求数（同上）
    pub failed_requests: Option<u64>,
}

impl BrowserResult {
    pub(crate) fn ok(output: String, current_url: String, title: Option<String>, took_ms: u64) -> Self {
        Self {
            success: true,
            output,
            current_url,
            title,
            error: None,
            duration_ms: took_ms,
            verify: None,
        }
    }

    pub(crate) fn ok_verified(
        output: String,
        current_url: String,
        title: Option<String>,
        took_ms: u64,
        verify: VerifyBlock,
    ) -> Self {
        Self {
            success: true,
            output,
            current_url,
            title,
            error: None,
            duration_ms: took_ms,
            verify: Some(verify),
        }
    }

    pub(crate) fn fail(
        error: String,
        current_url: String,
        title: Option<String>,
        took_ms: u64,
    ) -> Self {
        Self {
            success: false,
            output: String::new(),
            current_url,
            title,
            error: Some(error),
            duration_ms: took_ms,
            verify: None,
        }
    }
}

/// 链接引用（带序号，供 Agent 下一步 Click/引用）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkRef {
    pub index: usize,
    pub text: String,
    pub href: String,
}

/// 表单字段
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormField {
    pub name: String,
    pub kind: String,
    pub value: String,
}

/// 表单规格
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormSpec {
    pub index: usize,
    pub action: String,
    pub method: String,
    pub fields: Vec<FormField>,
}

/// 页面快照（一次抓取的结构化产物）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PageSnapshot {
    pub url: String,
    pub title: Option<String>,
    pub text: String,
    pub links: Vec<LinkRef>,
    pub forms: Vec<FormSpec>,
    /// 截断后的原始 HTML（128k 上限），供选择器二次解析
    pub raw_html: String,
    /// HTTP 状态码（Mock/Chrome 无渲染状态时为 None）
    pub http_status: Option<u16>,
}

/// 由快照组装验证证据
pub(crate) fn verify_for(requested_url: &str, snap: &PageSnapshot) -> VerifyBlock {
    VerifyBlock {
        url_changed: snap.url != requested_url,
        final_url: snap.url.clone(),
        http_status: snap.http_status,
        text_len: snap.text.chars().count(),
        links: snap.links.len(),
        forms: snap.forms.len(),
        console_errors: None,
        failed_requests: None,
    }
}
