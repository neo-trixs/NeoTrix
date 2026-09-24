//! js — 从 `nt_io_browser_engine.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use scraper::{Html, Selector};
use super::error::BrowserError;
use super::types::{FormSpec, PageSnapshot};

/// CDP 用 JS 表达式构造器（纯函数，可单测；选择器经 JSON 转义防注入）
pub(crate) fn js_select_option(selector: &str, values: &[String]) -> String {
    let sel = serde_json::to_string(selector).unwrap_or_else(|_| "\"\"".to_string());
    let vals = serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string());
    format!(
        "(()=>{{const el=document.querySelector({sel});if(!el)return 'no-match';\
        const vs={vals};[...el.options].forEach(o=>{{o.selected=vs.includes(o.value||o.text)}});\
        el.dispatchEvent(new Event('input',{{bubbles:true}}));\
        el.dispatchEvent(new Event('change',{{bubbles:true}}));return 'ok:'+el.value;}})()"
    )
}

/// CDP checkbox 设置表达式（纯函数）
pub(crate) fn js_set_checked(selector: &str, checked: bool) -> String {
    let sel = serde_json::to_string(selector).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "(()=>{{const el=document.querySelector({sel});if(!el)return 'no-match';\
        if(!!el.checked==={checked})return 'already';el.click();return 'toggled';}})()"
    )
}

// ============================================================================
// 表单选择（纯函数）
// ============================================================================

pub(crate) fn pick_form(
    snap: &PageSnapshot,
    selector: Option<&str>,
) -> Result<FormSpec, BrowserError> {
    match selector {
        None => snap.forms.first().cloned().ok_or_else(|| {
            BrowserError::ActionFailed("page has no forms".to_string())
        }),
        Some(s) if s.starts_with('#') => {
            let idx: usize = s.strip_prefix('#').unwrap_or("").parse().map_err(|_| {
                BrowserError::ActionFailed(format!("bad form selector '{s}'"))
            })?;
            snap.forms.get(idx).cloned().ok_or_else(|| {
                BrowserError::ActionFailed(format!("no form #{idx}"))
            })
        }
        Some(s) => {
            // CSS：v1 简化语义——命中任一 <form> 即用第一个表单（序号请用 #n 精确指定）
            let doc = Html::parse_document(&snap.raw_html);
            let sel = Selector::parse(s).map_err(|_| {
                BrowserError::ActionFailed(format!("bad form selector '{s}'"))
            })?;
            let hit_form = doc
                .select(&sel)
                .any(|el| el.value().name() == "form");
            if !hit_form {
                return Err(BrowserError::ActionFailed(format!(
                    "no form matches '{s}'（可用 #n 按序号）"
                )));
            }
            snap.forms.first().cloned().ok_or_else(|| {
                BrowserError::ActionFailed("page has no forms".to_string())
            })
        }
    }
}
