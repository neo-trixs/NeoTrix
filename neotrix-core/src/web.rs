use std::sync::Arc;
use std::time::Instant;

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::header,
    response::{sse::Event, IntoResponse, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};

use neotrix::neotrix::nt_core_kernel::EVOLUTION;
use neotrix::neotrix::nt_io_standalone::StandaloneEngine;

struct AppState {
    engine: Mutex<StandaloneEngine>,
    started_at: Instant,
    sse_tx: broadcast::Sender<String>,
}

#[derive(Deserialize)]
struct ReasonRequest {
    prompt: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime: f64,
}

#[derive(Deserialize)]
struct StageRequest {
    stage: usize,
}

#[derive(Serialize)]
struct StatsResponse {
    stage: usize,
    label: String,
    state_dim: usize,
    circuits: Vec<String>,
    energy: f64,
    conversation_length: usize,
    mode: String,
}

#[derive(Deserialize)]
struct WsIncoming {
    #[serde(rename = "type")]
    msg_type: String,
    prompt: Option<String>,
}

const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>NeoTrix</title>
<style>
  *{box-sizing:border-box;margin:0;padding:0}
  body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;
       background:#0d1117;color:#c9d1d9;display:flex;flex-direction:column;height:100vh}
  header{padding:1rem 2rem;border-bottom:1px solid #30363d;background:#161b22}
  h1{font-size:1.25rem;font-weight:600;color:#58a6ff}
  .container{display:flex;flex:1;overflow:hidden}
  .sidebar{width:280px;border-right:1px solid #30363d;padding:1rem;display:flex;flex-direction:column;gap:.5rem}
  .sidebar button{background:#21262d;border:1px solid #30363d;color:#c9d1d9;padding:.5rem;border-radius:6px;cursor:pointer}
  .sidebar button:hover{background:#30363d}
  .main{flex:1;display:flex;flex-direction:column}
  #output{flex:1;overflow-y:auto;padding:1rem;font-family:'JetBrains Mono','Fira Code',monospace;font-size:.875rem;white-space:pre-wrap;line-height:1.6}
  .input-row{display:flex;gap:.5rem;padding:.75rem 1rem;border-top:1px solid #30363d;background:#161b22}
  #input{flex:1;background:#0d1117;border:1px solid #30363d;border-radius:6px;padding:.5rem .75rem;color:#c9d1d9;font-size:.875rem;outline:none}
  #input:focus{border-color:#58a6ff}
  #send{background:#238636;border:none;color:#fff;padding:.5rem 1rem;border-radius:6px;cursor:pointer;font-weight:600}
  #send:hover{background:#2ea043}
  .stats{font-size:.75rem;color:#8b949e;padding:.5rem 1rem;border-top:1px solid #30363d}
  .token{color:#7ee787}.done{color:#58a6ff}.error{color:#f85149}.info{color:#8b949e}
</style>
</head>
<body>
<header><h1>NeoTrix · Reasoning Engine</h1></header>
<div class="container">
<div class="sidebar">
  <button onclick="fetchHealth()">Health Check</button>
  <button onclick="fetchStats()">Stats</button>
  <button onclick="connectWs()">WebSocket: Hello</button>
  <button onclick="clearOutput()">Clear</button>
</div>
<div class="main">
  <div id="output">Welcome to NeoTrix (standalone mode). Use the sidebar or type a prompt below.</div>
  <div class="input-row">
    <input id="input" placeholder="Ask something..." onkeydown="if(event.key==='Enter')sendPrompt()">
    <button id="send" onclick="sendPrompt()">Send</button>
  </div>
  <div class="stats" id="stats">connected via REST</div>
</div>
</div>
<script>
const out=document.getElementById('output');
const stats=document.getElementById('stats');
function log(cls,msg){const d=document.createElement('div');d.className=cls||'';d.textContent=msg;out.appendChild(d);out.scrollTop=out.scrollHeight}
async function api(method,path,body){const r=await fetch(path,{method,headers:{'Content-Type':'application/json'},body:body?JSON.stringify(body):undefined});return r.json()}
async function fetchHealth(){const h=await api('GET','/api/health');log('info',JSON.stringify(h,null,2))}
async function fetchStats(){const s=await api('GET','/api/stats');log('info',JSON.stringify(s,null,2))}
function clearOutput(){out.innerHTML=''}
async function sendPrompt(){const prompt=document.getElementById('input').value;if(!prompt)return;document.getElementById('input').value='';log('info','>>> '+prompt);const r=await api('POST','/api/reason',{prompt});log('token',r.response);stats.textContent='energy='+r.energy.toFixed(3)+' stage='+r.stage+' circuits='+r.circuits.join(', ')}
let ws=null;
function connectWs(){if(ws){ws.close();return}
  ws=new WebSocket('ws://'+location.host+'/ws/stream');log('info','[WS connecting...]');
  ws.onopen=()=>{log('info','[WS connected]');ws.send(JSON.stringify({type:'reason',prompt:'Hello, NeoTrix!'}))};
  ws.onmessage=(e)=>{const d=JSON.parse(e.data);if(d.type==='token')log('token',d.data);else if(d.type==='done'){log('done',d.response);stats.textContent='energy='+d.energy.toFixed(3)}else log('info',JSON.stringify(d))};
  ws.onclose=()=>{log('info','[WS closed]');ws=null}}
if(typeof EventSource!=='undefined'){const es=new EventSource('/api/events');
es.onmessage=(e)=>{try{const d=JSON.parse(e.data);stats.textContent='energy='+d.energy.toFixed(3)+' stage='+d.stage+' circuits='+d.circuits.length}catch(_){}}}
</script>
</body>
</html>
"##;

fn spawn_sse_broadcaster(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let engine = state.engine.lock().await;
            let s = engine.kernel.stats();
            let payload = serde_json::json!({
                "energy": s.energy,
                "stage": s.stage,
                "label": s.label,
                "circuits": s.active.iter().map(|m| format!("{:?}", m)).collect::<Vec<_>>(),
                "conversation_length": engine.conversation.len(),
                "mode": "standalone",
            });
            let _ = state.sse_tx.send(payload.to_string());
        }
    });
}

async fn handle_health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime: state.started_at.elapsed().as_secs_f64(),
    })
}

async fn handle_reason(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReasonRequest>,
) -> Json<serde_json::Value> {
    let prompt = req.prompt.trim().to_string();
    if prompt.is_empty() {
        return Json(serde_json::json!({
            "error": "empty prompt",
            "hint": "Send a non-empty prompt.",
        }));
    }
    let mut engine = state.engine.lock().await;
    let response = engine.reason(&prompt);
    let s = engine.kernel.stats();
    let circuits: Vec<String> = s.active.iter().map(|m| format!("{:?}", m)).collect();
    Json(serde_json::json!({
        "response": response,
        "energy": s.energy,
        "circuits": circuits,
        "stage": s.stage,
        "mode": "standalone",
    }))
}

async fn handle_stats(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    let engine = state.engine.lock().await;
    let s = engine.kernel.stats();
    let circuits: Vec<String> = s.active.iter().map(|m| format!("{:?}", m)).collect();
    Json(StatsResponse {
        stage: s.stage,
        label: s.label,
        state_dim: s.state_dim,
        circuits,
        energy: s.energy,
        conversation_length: engine.conversation.len(),
        mode: "standalone".into(),
    })
}

async fn handle_stage(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StageRequest>,
) -> Json<serde_json::Value> {
    let max_stage = EVOLUTION.len().saturating_sub(1);
    if req.stage > max_stage {
        return Json(serde_json::json!({
            "error": format!("stage {} out of range, max is {}", req.stage, max_stage),
        }));
    }
    let mut engine = state.engine.lock().await;
    engine.kernel.stage = req.stage;
    let label = EVOLUTION[req.stage].label;
    Json(serde_json::json!({ "stage": req.stage, "label": label }))
}

async fn handle_ws_upgrade(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(state, socket))
}

async fn handle_ws_socket(state: Arc<AppState>, mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        let text = match &msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };

        let incoming: WsIncoming = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({"type":"error","data":"invalid json"})
                            .to_string()
                            .into(),
                    ))
                    .await;
                continue;
            }
        };

        if incoming.msg_type != "reason" {
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"type":"error","data":"unknown message type"})
                        .to_string()
                        .into(),
                ))
                .await;
            continue;
        }

        let prompt = incoming.prompt.unwrap_or_default();
        let mut engine = state.engine.lock().await;
        let response = engine.reason(&prompt);
        let s = engine.kernel.stats();
        drop(engine);

        let token_chars: Vec<String> = if response.len() > 20 {
            let words: Vec<&str> = response.split(' ').collect();
            if words.len() > 3 {
                let mid = words.len() / 3;
                vec![
                    words[..mid].join(" "),
                    words[mid..2 * mid].join(" "),
                    words[2 * mid..].join(" "),
                ]
            } else {
                vec![response.clone()]
            }
        } else {
            vec![response.clone()]
        };

        for chunk in &token_chars {
            if socket
                .send(Message::Text(
                    serde_json::json!({"type":"token","data":chunk})
                        .to_string()
                        .into(),
                ))
                .await
                .is_err()
            {
                return;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        let circuits: Vec<String> = s.active.iter().map(|m| format!("{:?}", m)).collect();
        let _ = socket
            .send(Message::Text(
                serde_json::json!({
                    "type":"done",
                    "response":response,
                    "energy":s.energy,
                    "circuits":circuits,
                    "stage":s.stage,
                })
                .to_string()
                .into(),
            ))
            .await;
    }
}

async fn handle_sse(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = state.sse_tx.subscribe();
    let stream = futures::stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(data) => Some((Ok(Event::default().data(data)), rx)),
            Err(broadcast::error::RecvError::Closed) => None,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                Some((Ok(Event::default().data(format!("lagged:{}", n))), rx))
            }
        }
    });
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new().interval(tokio::time::Duration::from_secs(15)),
    )
}

const DEFAULT_DIST: &str = "neotrix-web-frontend/dist";

fn mime_for(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") || path.ends_with(".mjs") {
        "application/javascript"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else if path.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}

async fn try_serve_file(dist_dir: &str, rel_path: &str) -> Option<Response<Body>> {
    let cleaned = rel_path.trim_start_matches('/');
    if cleaned.contains("..") {
        return None;
    }
    let full = std::path::Path::new(dist_dir).join(cleaned);
    let target = if full.is_file() {
        full
    } else {
        let idx = std::path::Path::new(dist_dir).join("index.html");
        if idx.is_file() {
            idx
        } else {
            return None;
        }
    };
    match tokio::fs::read(&target).await {
        Ok(bytes) => {
            let mime = mime_for(target.to_str().unwrap_or(""));
            Some(
                Response::builder()
                    .header(header::CONTENT_TYPE, mime)
                    .body(Body::from(bytes))
                    .unwrap(),
            )
        }
        Err(_) => None,
    }
}

async fn handle_frontend(path: Option<String>) -> Response<Body> {
    let dist_dir = std::env::var("NEOTRIX_WEB_DIST").unwrap_or_else(|_| DEFAULT_DIST.to_string());
    let rel = path.as_deref().unwrap_or("index.html");
    if let Some(resp) = try_serve_file(&dist_dir, rel).await {
        return resp;
    }
    if rel != "index.html" {
        if let Some(resp) = try_serve_file(&dist_dir, "index.html").await {
            return resp;
        }
    }
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(INDEX_HTML))
        .unwrap()
}

async fn handle_frontend_root() -> Response<Body> {
    handle_frontend(None).await
}

async fn handle_frontend_assets(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    handle_frontend(Some(format!("assets/{}", path))).await
}

pub async fn run(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let engine = StandaloneEngine::new(9);
    let (sse_tx, _) = broadcast::channel::<String>(64);
    let state = Arc::new(AppState {
        engine: Mutex::new(engine),
        started_at: Instant::now(),
        sse_tx,
    });

    spawn_sse_broadcaster(state.clone());

    let app = Router::new()
        .route("/", get(handle_frontend_root))
        .route("/assets/*path", get(handle_frontend_assets))
        .route("/api/health", get(handle_health))
        .route("/api/reason", post(handle_reason))
        .route("/api/stats", get(handle_stats))
        .route("/api/stage", post(handle_stage))
        .route("/ws/stream", get(handle_ws_upgrade))
        .route("/api/events", get(handle_sse))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    log::info!("neotrix web listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
