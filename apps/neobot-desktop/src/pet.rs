//! 桌宠状态 —— **壳设置面**，不是 DSH 运行时。
//!
//! # 为什么单独一个模块
//!
//! 上游的桌宠分两层：窗口框架（透明置顶 `pet` 窗 + 穿透流，纯壳）与
//! 内容管理（宠物清单/资源包/会话气泡，DSH 运行时供给）。
//! 本仓只接前者能诚实做的部分：开关/当前选择/大小/ Wayland 能力位 ——
//! 它们只是**持久化几个键**，与运行时无关。
//! 清单与资源（`list_pets` / `import_pet` / `get_pet_asset` / `list_preset_pets` /
//! `push_pet_session` / `start_pet_mouse_stream`）仍是 Planned，见 `api.rs`。
//!
//! # 状态存在哪
//!
//! `<data_dir>/app-config.json`（与 `update_app_config` 同一份文件）：
//! `pet_enabled: bool`（缺省 false）· `active_pet: string`（缺省空串）·
//! `pet_size: number`（缺省缺席，按 100 处理）· `force_xwayland: bool`（缺省 false）。
//! 用同一个文件的理由：`update_app_config` 已是合并写入，
//! 两处各写各的文件 ⇒ 「设置页改的」与「桌宠读的」会分家。
//!
//! # 与上游的字段对齐
//!
//! `PetStatus{enabled, visible, active_pet, pet_size}` 与上游
//! `bridge/pet.rs PetStatus` + 前端 `PetStatus`（`use-pet-status.ts:8`）
//! **逐字段同名同形**（snake_case）。`visible` 恒等于 `enabled`
//!（上游亦如此：临时收起已移除，见其字段注释）。
//!
//! # 事件
//!
//! 改动成功后推 `pet://status`（前端 `usePetStatus` 正听这个）。
//! 推失败**不回滚**：状态已落盘，回滚会制造「设置页显示开、
//! 读出来是关」的分叉。推失败只报错，让调用方知道界面可能要手动刷新。

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::data_dir;

/// 桌宠完整状态。见模块头「字段对齐」。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetStatus {
    pub enabled: bool,
    pub visible: bool,
    pub active_pet: String,
    pub pet_size: Option<f64>,
}

/// 宠物大小合法区间（上游 `PET_SIZE_MIN..=PET_SIZE_MAX`，50–200，100 为基准）。
pub const PET_SIZE_MIN: f64 = 50.0;
pub const PET_SIZE_MAX: f64 = 200.0;

/// 桌宠状态变更事件名。前端 `usePetStatus` 听同一个名字。
pub const PET_STATUS_EVENT: &str = "pet://status";

fn read_cfg() -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let path = data_dir()?.join("app-config.json");
    match std::fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::Map::new()),
        Err(e) => Err(format!("读应用配置失败：{e}")),
        Ok(s) => {
            let v: serde_json::Value =
                serde_json::from_str(&s).map_err(|e| format!("应用配置不是合法 JSON：{e}"))?;
            v.as_object()
                .cloned()
                .ok_or_else(|| "应用配置不是 JSON 对象；已拒绝解析（请先人工检查）".to_owned())
        }
    }
}

fn write_cfg(map: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let path = data_dir()?.join("app-config.json");
    let body = serde_json::to_string_pretty(map).map_err(|e| format!("序列化失败：{e}"))?;
    std::fs::write(&path, body).map_err(|e| format!("写应用配置失败：{e}"))
}

fn status_of(cfg: &serde_json::Map<String, serde_json::Value>) -> PetStatus {
    let enabled = cfg.get("pet_enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let active_pet = cfg
        .get("active_pet")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let pet_size = cfg.get("pet_size").and_then(|v| v.as_f64());
    PetStatus { enabled, visible: enabled, active_pet, pet_size }
}

/// `get_pet_status() -> PetStatus`
///
/// ⛔ 读失败返回**默认关闭**而不是报错：pet 窗挂载即调，
/// 报错会让它 `catch` 后停在 `null`（加载中骨架），
/// 而「配置读失败」与「还没加载完」对用户是同一种白屏。
/// 调用方仍可经 `neobot_api_call` 看到契约是 Implemented ——
/// 这是「有默认值」不是「假装有宠物」：`active_pet` 为空串即未选择。
#[tauri::command]
pub fn get_pet_status() -> PetStatus {
    read_cfg().map(|c| status_of(&c)).unwrap_or(PetStatus {
        enabled: false,
        visible: false,
        active_pet: String::new(),
        pet_size: None,
    })
}

fn persist_and_emit(app: &AppHandle, cfg: serde_json::Map<String, serde_json::Value>) -> Result<PetStatus, String> {
    let st = status_of(&cfg);
    write_cfg(&cfg)?;
    Emitter::emit(app, PET_STATUS_EVENT, &st).map_err(|e| format!("推送桌宠状态到界面失败：{e}"))?;
    Ok(st)
}

/// `set_pet_enabled(enabled) -> PetStatus`
#[tauri::command]
pub fn set_pet_enabled(app: AppHandle, enabled: bool) -> Result<PetStatus, String> {
    // ⛔ 先开窗/藏窗，再落盘：窗建不起来就直接 Err，状态保持原样 ——
    // 反过来（先落盘再开窗）会留下「状态说开、窗没起来」的分叉。
    if enabled {
        let w = ensure_pet_window(&app)?;
        w.show().map_err(|e| format!("显示桌宠窗口失败：{e}"))?;
    } else if let Some(w) = app.get_webview_window(PET_WINDOW_LABEL) {
        w.hide().map_err(|e| format!("隐藏桌宠窗口失败：{e}"))?;
    }
    let mut cfg = read_cfg()?;
    cfg.insert("pet_enabled".to_owned(), serde_json::Value::Bool(enabled));
    persist_and_emit(&app, cfg)
}

/// `set_active_pet(id) -> PetStatus`
///
/// 空串 = 清除选择（与上游一致：空白持久值读回空串）。
/// ⛔ 只做 `trim`，不校验 id 是否存在 —— 清单命令（`list_pets`）
/// 还是 Planned，校验会把「先选后装」这条路堵死。
#[tauri::command]
pub fn set_active_pet(app: AppHandle, id: String) -> Result<PetStatus, String> {
    let mut cfg = read_cfg()?;
    cfg.insert(
        "active_pet".to_owned(),
        serde_json::Value::String(id.trim().to_owned()),
    );
    persist_and_emit(&app, cfg)
}

/// `set_pet_size(size) -> PetStatus`
///
/// ⛔ 越界**拒绝**而不是收敛：收敛会让设置页显示 100、
/// 实际想要 300 的用户以为自己设成了 300。
/// 前端另有 `normalizeSizePercent` 做显示收敛 —— 那是显示层的事。
#[tauri::command]
pub fn set_pet_size(app: AppHandle, size: f64) -> Result<PetStatus, String> {
    if !size.is_finite() || !(PET_SIZE_MIN..=PET_SIZE_MAX).contains(&size) {
        return Err(format!(
            "PET_SIZE_OUT_OF_RANGE: pet size percent must be within {PET_SIZE_MIN}..={PET_SIZE_MAX}"
        ));
    }
    let mut cfg = read_cfg()?;
    cfg.insert("pet_size".to_owned(), serde_json::json!(size));
    persist_and_emit(&app, cfg)
}

/// `get_pet_overlay_supported() -> boolean`
///
/// 桌宠置顶 + 绝对定位是否可用。macOS / Windows 恒 true；
/// Linux 原生 Wayland 下 GTK 的两项调用是 no-op（上游 `lib.rs pet_overlay_supported`）。
/// ⛔ 读的是 `WAYLAND_DISPLAY` / `GDK_BACKEND`，不是 `XDG_SESSION_TYPE` ——
/// 后者在 TTY 直起的合成器下为空而 GDK 照样走 Wayland（上游注释原话）。
#[tauri::command]
pub fn get_pet_overlay_supported() -> bool {
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
    #[cfg(target_os = "linux")]
    {
        let wayland = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
        let backend = std::env::var("GDK_BACKEND").unwrap_or_default();
        wayland.is_empty() || backend.split(',').next() == Some("x11")
    }
}

/// `get_force_xwayland() -> boolean` —— 下次启动是否强制 XWayland（缺省 false）。
#[tauri::command]
pub fn get_force_xwayland() -> bool {
    read_cfg().ok().and_then(|c| c.get("force_xwayland").and_then(|v| v.as_bool())).unwrap_or(false)
}

/// `set_force_xwayland(enabled) -> boolean`
///
/// ⛔ 返回**新值**（上游亦如此）：调用方据此渲染开关，
/// 返回 void 的话调用方只能「设完再读一次」，中间那次读可能读到旧值。
#[tauri::command]
pub fn set_force_xwayland(enabled: bool) -> Result<bool, String> {
    let mut cfg = read_cfg()?;
    cfg.insert("force_xwayland".to_owned(), serde_json::Value::Bool(enabled));
    write_cfg(&cfg)?;
    Ok(enabled)
}

/// 桌宠窗口 label（与前端 pet.html 窗 + capabilities 里的 `pet` 同名）。
pub const PET_WINDOW_LABEL: &str = "pet";

/// 桌宠基准宽 px（100% 档；与前端 `PET_BASE_WIDTH` 同源）。
pub const PET_BASE_WIDTH: f64 = 220.0;

/// 内置海豚精灵图（编译期嵌入 `icons/256x256.png`）。
///
/// ⛔ `include_bytes!` 而不是运行时读文件：运行时路径在 dev 与打包后不同
/// （`resources` 解析各平台有别），编译期嵌入两边完全一致。
const DOLPHIN_PNG: &[u8] = include_bytes!("../icons/256x256.png");

/// 桌宠窗口尺寸（正方形；长宽比缺省 1.0）。
///
/// ⛔ 上游按激活宠物的长宽比算，本仓初版按正方形 ——
/// 宠物页自己会调尺寸（`use-pet-window`），后端只给一个不离谱的初值。
/// 大小越界输入直接钳制（与 `set_pet_size` 的拒绝不同：这里是内部推导，
/// 没有「用户以为设上了」的误解面）。
pub fn pet_window_size(size_pct: f64) -> (f64, f64) {
    let pct = if size_pct.is_finite() {
        size_pct.clamp(PET_SIZE_MIN, PET_SIZE_MAX)
    } else {
        100.0
    };
    let w = PET_BASE_WIDTH * pct / 100.0;
    (w, w)
}

/// 确保桌宠窗口存在（不存在即按上游同形新建），返回窗口。
///
/// 窗形（抄上游 `desktop/pet.rs ensure_pet_window`）：透明 + 置顶 +
/// 无边框 + 不进任务栏 + 不可缩放 + 初始隐藏（调用方决定 show）。
/// 位置：app-config 里有 `pet_x/pet_y` 就恢复，没有就随系统摆。
pub fn ensure_pet_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    if let Some(w) = app.get_webview_window(PET_WINDOW_LABEL) {
        return Ok(w);
    }
    let status = get_pet_status();
    let (w, h) = pet_window_size(status.pet_size.unwrap_or(100.0));
    let mut builder =
        WebviewWindowBuilder::new(app, PET_WINDOW_LABEL, WebviewUrl::App("pet.html".into()))
            .title("NeoBot Pet")
            .inner_size(w, h)
            .resizable(false)
            .maximizable(false)
            .transparent(true)
            .always_on_top(true)
            .decorations(false)
            .skip_taskbar(true)
            .shadow(false)
            .accept_first_mouse(true)
            .visible(false);
    if let (Some(x), Some(y)) = (pet_pos("pet_x"), pet_pos("pet_y")) {
        builder = builder.position(x, y);
    }
    builder.build().map_err(|e| format!("创建桌宠窗口失败：{e}"))
}

fn pet_pos(key: &str) -> Option<f64> {
    read_cfg().ok()?.get(key)?.as_f64()
}

/// `set_pet_ignore_cursor_events(ignore) -> boolean`
///
/// 桌宠窗口的鼠标穿透开关（点透/接事件来回切）。返回**是否生效**。
///
/// ⛔ **只能由 pet 窗口调用**（`window.label() == "pet"`，与上游同）。
/// 其它窗口调直接拒 —— 穿透是个窗口级属性，调错窗口等于把主窗口点透。
///
/// ⛔ 隐藏窗口的穿透请求**吞掉并报 false**（上游 issue #437）：
/// tao 在未 realize 的窗口上设穿透会 panic，而 „没生效" 与 „崩了"
/// 对用户是两种处境，false 让调用方知道这次没设上。
#[tauri::command]
pub fn set_pet_ignore_cursor_events(window: tauri::WebviewWindow, ignore: bool) -> Result<bool, String> {
    if window.label() != PET_WINDOW_LABEL {
        return Err("PET_WINDOW_LABEL_MISMATCH: this command is restricted to the pet window".to_owned());
    }
    if ignore {
        let visible = window.is_visible().map_err(|e| format!("PET_WINDOW_STATE_FAILED: {e}"))?;
        if !visible {
            return Ok(false);
        }
    }
    window.set_ignore_cursor_events(ignore).map_err(|e| format!("设置鼠标穿透失败：{e}"))?;
    Ok(true)
}

/// `move_pet_window(delta_x, delta_y)`
///
/// 按物理像素**相对**移动桌宠窗口（与上游 `bridge/pet.rs` 同形）。
///
/// ⛔ 本命令第一版做的是绝对定位 `(x, y, always_on_top)` ——
/// 而 pet 窗自己的调用是 `{ deltaX, deltaY }`（`use-pet-window.ts:30`），
/// 两边对不上，调用静默失败（`.catch(()=>{})` 吞掉）。
/// 这与 `log_frontend` 2 参/3 参是同一类错：「前后端各自自洽、没人对着看」。
/// 参数名亦与上游逐字相同（`delta_x/delta_y`），线序行为与上游一致。
///
/// 移动后把位置记进 app-config（下次建窗恢复）。
/// ⛔ 记失败不报错：位置的真源是窗口本身（随时可读），
///
/// 盘上那份只是启动提示；为提示失败而报「移动失败」是谎报。
#[tauri::command]
pub fn move_pet_window(app: AppHandle, delta_x: i32, delta_y: i32) -> Result<(), String> {
    let w = app
        .get_webview_window(PET_WINDOW_LABEL)
        .ok_or_else(|| "桌宠窗口未创建".to_owned())?;
    let pos = w.outer_position().map_err(|e| format!("读窗口位置失败：{e}"))?;
    w.set_position(tauri::PhysicalPosition::new(pos.x + delta_x as i32, pos.y + delta_y as i32))
        .map_err(|e| format!("移动桌宠窗口失败：{e}"))?;
    if let Ok(mut cfg) = read_cfg() {
        cfg.insert("pet_x".to_owned(), serde_json::json!((pos.x + delta_x) as f64));
        cfg.insert("pet_y".to_owned(), serde_json::json!((pos.y + delta_y) as f64));
        let _ = write_cfg(&cfg);
    }
    Ok(())
}

/// 某来源的宠物根目录（`<data_dir>/pets/<source>`，不存在即建）。
fn pets_root(source: &str) -> Result<(std::path::PathBuf, neotrix_neobot::nt_pet::PetSource), String> {
    let src = neotrix_neobot::nt_pet::PetSource::parse(source.trim())
        .map_err(|e| e.to_string())?;
    let dir = data_dir()?.join("pets").join(src.as_str());
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建宠物目录失败：{e}"))?;
    Ok((dir, src))
}

/// `list_pets(source) -> PetListItem[]`
///
/// 扫 `<data_dir>/pets/<source>/`。codex 源首次列出时播种内置海豚 ——
///
/// ⛔ 播种失败不拦列表：种子是赠品，列表是正餐；
///
/// 为赠品失败而报「列表失败」是谎报。
#[tauri::command]
pub fn list_pets(source: String) -> Result<Vec<neotrix_neobot::nt_pet::PetListItem>, String> {
    let (root, src) = pets_root(&source)?;
    if src == neotrix_neobot::nt_pet::PetSource::Codex {
        let _ = neotrix_neobot::nt_pet::seed_dolphin(&root, DOLPHIN_PNG);
    }
    neotrix_neobot::nt_pet::list_pets(&root, src).map_err(|e| e.to_string())
}

/// `import_pet(name, data) -> PetListItem`
///
/// base64 zip 包（codex 宠物格式）：包上限/条目上限/解后上限/链接拒绝/
/// 单 pet.json/重名拒绝，全部走库侧同一条路（见 `nt_pet`）。
/// ⛔ `name` 是不可信展示数据（Unicode/空格），只用于报错文案，不参与落盘。
#[tauri::command]
pub fn import_pet(name: String, data: String) -> Result<neotrix_neobot::nt_pet::PetListItem, String> {
    let (root, _) = pets_root("codex")?;
    neotrix_neobot::nt_pet::import_pet(&root, &name, &data).map_err(|e| e.to_string())
}

/// `list_preset_pets() -> PresetPetItem[]`
///
/// ⛔ 恒返回空数组。本仓没有远端预设源（上游条目直连远端视频，无源可对），
/// 而本地宠物的位置是 `list_pets` + 内置海豚种子 —— 预设清单不是它们的别名，
/// 硬包装会让设置页出现两份 identical 列表。
/// 空不是缺省：调用方（`usePetSource`）对空清单走 `PET_NOT_FOUND` 可见诊断，
/// 与「命令不存在」的 catch 分支同诊断、少一次报错噪音。类型用
/// `serde_json::Value` 而不是空 struct —— 空 struct 暗示「有形状待填」，
/// 而这里的事实是「没有条目」。
#[tauri::command]
pub fn list_preset_pets() -> Vec<serde_json::Value> {
    Vec::new()
}

/// `get_pet_asset(id) -> PetAsset`
///
/// `id` 形如 `codex:xxx`（来源限定；与上游 `parse_qualified_id` 同）。
/// 内置海豚走静态单帧（1×1，整张即一帧；见 `nt_pet::seed_dolphin_grid`），
/// 其余走动画网格校验 —— 方图硬套 8×N 网格会切出碎帧，两条路必须分开.
#[tauri::command]
pub fn get_pet_asset(id: String) -> Result<neotrix_neobot::nt_pet::PetAsset, String> {
    use neotrix_neobot::nt_pet as pet_lib;
    let id = id.trim();
    let (source, manifest_id) = pet_lib::parse_qualified_id(id).map_err(|e| e.to_string())?;
    let (root, _) = pets_root(source.as_str())?;
    if source == pet_lib::PetSource::Codex && manifest_id == "neobot-dolphin" {
        let _ = pet_lib::seed_dolphin(&root, DOLPHIN_PNG);
        let dir = root.join("neobot-dolphin");
        let bytes = std::fs::read(dir.join("spritesheet.png"))
            .map_err(|e| format!("PET_ASSET_READ_FAILED: {e}"))?;
        let (version, columns, rows) = pet_lib::seed_dolphin_grid();
        return Ok(pet_lib::PetAsset {
            id: pet_lib::qualified_id(source, manifest_id),
            spritesheet: format!(
                "data:image/png;base64,{}",
                base64_light_encode(&bytes)
            ),
            sprite_version_number: version,
            columns,
            rows,
        });
    }
    pet_lib::pet_asset(&root, source, manifest_id).map_err(|e| e.to_string())
}

/// 最小 base64 编码（app 侧无 base64 依赖；资产 data URL 专用）。
///
/// ⛔ 手写编码只许用在这一个地方：通用需求（解码/校验）一律走库侧。
/// 字符集错一个，整张图就坏 —— 有测试锁 `TWFu`（"Man" 的标准向量）。
fn base64_light_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 缺省状态是关闭且未选择() {
        let st = status_of(&serde_json::Map::new());
        assert!(!st.enabled);
        assert!(!st.visible);
        assert!(st.active_pet.is_empty());
        assert!(st.pet_size.is_none());
    }

    #[test]
    fn 可见恒等于启用() {
        let mut cfg = serde_json::Map::new();
        cfg.insert("pet_enabled".to_owned(), serde_json::Value::Bool(true));
        let st = status_of(&cfg);
        assert!(st.visible, "visible 必须恒等于 enabled（上游同）");
    }

    #[test]
    fn 大小越界的判定与上游一致() {
        for bad in [f64::NAN, f64::INFINITY, 49.9, 200.1] {
            assert!(
                !bad.is_finite() || !(PET_SIZE_MIN..=PET_SIZE_MAX).contains(&bad),
                "{bad} 应被拒"
            );
        }
        for good in [50.0, 100.0, 200.0] {
            assert!((PET_SIZE_MIN..=PET_SIZE_MAX).contains(&good), "{good} 应放行");
        }
    }

    #[test]
    fn 状态序列化后是前端要的蛇形字段() {
        let v = serde_json::to_value(PetStatus {
            enabled: true,
            visible: true,
            active_pet: "dolphin".into(),
            pet_size: Some(100.0),
        })
        .expect("序列化");
        for k in ["enabled", "visible", "active_pet", "pet_size"] {
            assert!(v.get(k).is_some(), "缺字段 {k}");
        }
    }

    #[test]
    fn 事件名与前端监听一致() {
        assert_eq!(PET_STATUS_EVENT, "pet://status");
    }

    #[test]
    fn 窗label与capabilities及前端一致() {
        // 三处必须同名：capabilities 的 windows 项、pet.html 窗、前端 get_webview_window("pet")。
        assert_eq!(PET_WINDOW_LABEL, "pet");
    }

    #[test]
    fn 窗尺寸钳制() {
        assert_eq!(pet_window_size(100.0), (220.0, 220.0));
        assert_eq!(pet_window_size(200.0), (440.0, 440.0));
        // 越界按内部推导钳制（与 set_pet_size 的拒绝不同：这里没有用户误解面）。
        assert_eq!(pet_window_size(9999.0), pet_window_size(200.0));
        assert_eq!(pet_window_size(f64::NAN), (220.0, 220.0));
    }

    #[test]
    fn 自编码base64与标准向量一致() {
        // "Man" -> "TWFu"（RFC 4648 §10 标准向量）；错一个字符整张图就坏。
        assert_eq!(base64_light_encode(b"Man"), "TWFu");
        assert_eq!(base64_light_encode(b""), "");
        assert_eq!(base64_light_encode(b"f"), "Zg==");
        assert_eq!(base64_light_encode(b"fo"), "Zm8=");
    }

    #[test]
    fn 预设清单恒空且可序列化() {
        // 空即全部：调用方据此走 PET_NOT_FOUND 诊断，而不是 catch 分支。
        let v = serde_json::to_value(list_preset_pets()).expect("序列化");
        assert_eq!(v, serde_json::json!([]));
    }
}
