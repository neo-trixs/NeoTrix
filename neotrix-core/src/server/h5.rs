//! H5 Web 界面（I-103 远程会话）
//! 提供手机浏览器可访问的聊天界面 + Token 认证 + CORS

use crate::server::http::AppState;
use axum::{
    http::{HeaderMap, HeaderValue},
    response::Html,
    routing::get,
    Router,
};

const H5_PAGE: &str = r##"<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>NeoTrix 远程会话</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, sans-serif; background: #0f0f1a; color: #e0e0e0; height: 100vh; display: flex; flex-direction: column; }
#header { padding: 12px 16px; background: #1a1a2e; border-bottom: 1px solid #2a2a4a; display: flex; justify-content: space-between; align-items: center; }
#header h1 { font-size: 16px; color: #00d4ff; }
#token-bar { padding: 8px 16px; background: #16213e; display: flex; gap: 8px; }
#token-bar input { flex:1; padding:6px 10px; border:1px solid #2a2a4a; border-radius:4px; background:#0f0f1a; color:#e0e0e0; }
#token-bar button { padding:6px 16px; border:none; border-radius:4px; background:#00d4ff; color:#000; cursor:pointer; }
#chat { flex:1; overflow-y:auto; padding:16px; }
.msg { margin-bottom:12px; max-width:85%; }
.msg.user { margin-left:auto; }
.msg .bubble { padding:10px 14px; border-radius:12px; line-height:1.5; font-size:14px; }
.msg.user .bubble { background:#00d4ff22; border:1px solid #00d4ff44; }
.msg.assistant .bubble { background:#2a2a4a; border:1px solid #3a3a5a; }
.msg .role { font-size:11px; color:#888; margin-bottom:4px; }
#input-bar { padding:12px 16px; background:#1a1a2e; border-top:1px solid #2a2a4a; display:flex; gap:8px; }
#input-bar input { flex:1; padding:10px 14px; border:1px solid #2a2a4a; border-radius:8px; background:#0f0f1a; color:#e0e0e0; font-size:14px; }
#input-bar button { padding:10px 20px; border:none; border-radius:8px; background:#00d4ff; color:#000; font-size:14px; cursor:pointer; }
#status { font-size:12px; color:#666; padding:4px 16px; text-align:center; }
</style></head><body>
<div id="header"><h1>🧠 NeoTrix</h1><span style="font-size:12px;color:#666;">远程会话</span></div>
<div id="token-bar">
  <input id="token-input" type="password" placeholder="输入访问 Token...">
  <button onclick="connect()">连接</button>
</div>
<div id="chat"></div>
<div id="status">未连接</div>
<div id="input-bar">
  <input id="msg-input" type="text" placeholder="输入消息..." disabled onkeydown="if(event.key==='Enter')send()">
  <button id="send-btn" disabled onclick="send()">发送</button>
</div>
<script>
let ws = null; let token = '';
function connect() { token = document.getElementById('token-input').value; if(!token) return;
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  ws = new WebSocket(proto+'//'+location.host+'/ws?token='+encodeURIComponent(token));
  ws.onopen = () => { document.getElementById('status').textContent = '已连接'; 
    document.getElementById('msg-input').disabled=false; document.getElementById('send-btn').disabled=false; };
  ws.onclose = () => { document.getElementById('status').textContent = '已断开'; 
    document.getElementById('msg-input').disabled=true; document.getElementById('send-btn').disabled=true; };
  ws.onmessage = (e) => addMsg('assistant', e.data); }
function send() { const inp = document.getElementById('msg-input'); const text = inp.value.trim(); if(!text) return;
  addMsg('user', text); ws.send(JSON.stringify({type:'text',token,content:text})); inp.value=''; }
function addMsg(role, content) {
  const div = document.createElement('div'); div.className = 'msg '+role;
  div.innerHTML = '<div class="role">'+(role==='user'?'你':'NeoTrix')+'</div><div class="bubble">'+esc(content)+'</div>';
  document.getElementById('chat').appendChild(div);
  div.scrollIntoView({behavior:'smooth'}); }
function esc(s) { const d=document.createElement('div'); d.textContent=s; return d.innerHTML; }
</script></body></html>"##;

/// Token 认证中间件
pub fn validate_token(headers: &HeaderMap, expected: &str) -> bool {
    if expected.is_empty() || expected == "dev" {
        return true;
    }
    headers
        .get("Authorization")
        .and_then(|v| match v.to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                log::warn!("[h5] non-utf8 auth header: {}", e);
                None
            }
        })
        .map(|v| v.strip_prefix("Bearer ").unwrap_or(v) == expected)
        .unwrap_or(false)
}

/// CORS 中间件
pub fn cors_headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    h.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET,POST,OPTIONS"),
    );
    h.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Authorization,Content-Type"),
    );
    h
}

/// H5 聊天页面
pub async fn h5_page() -> Html<&'static str> {
    Html(H5_PAGE)
}

pub fn h5_routes() -> Router<AppState> {
    Router::new().route("/chat", get(h5_page))
}
