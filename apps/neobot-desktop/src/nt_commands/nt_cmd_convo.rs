//! `nt_cmd_convo` — 会话/附件/成员（IM 面）.

use super::{NeobotAttachItem, NeobotConvoItem, NeobotMemberItem, load_config, open_store};
/// 会话容器（DM/群组）

#[tauri::command]
pub fn neobot_convos() -> Result<Vec<NeobotConvoItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    Ok(store
        .list_conversations()
        .map_err(|err| err.to_string())?
        .into_iter()
        .map(convo_dto)
        .collect())
}
/// 会话容器（DM/群组）

#[tauri::command]
pub fn neobot_convo_group(title: String, members: Vec<String>) -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .create_conversation("group", &title, &members)
        .map_err(|err| err.to_string())
}
/// 会话容器（DM/群组）

#[tauri::command]
pub fn neobot_convo_dm(me: String, peer: String) -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.get_or_create_dm(&me, &peer).map_err(|err| err.to_string())
}
/// 会话容器（DM/群组）

#[tauri::command]
pub fn neobot_convo_ensure_default(me: String) -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.ensure_default_dm(&me).map_err(|err| err.to_string())
}
/// 会话容器（DM/群组）

#[tauri::command]
pub fn neobot_convo_rename(id: String, title: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.rename_conversation(&id, &title).map_err(|err| err.to_string())
}
/// 会话容器（DM/群组）

#[tauri::command]
pub fn neobot_convo_delete(id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.delete_conversation(&id).map_err(|err| err.to_string())
}
/// 免打扰开关 / 已读水位

#[tauri::command]
pub fn neobot_convo_mute(id: String, muted: bool) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .set_conversation_muted(&id, muted)
        .map_err(|err| err.to_string())
}
/// 免打扰开关 / 已读水位

#[tauri::command]
pub fn neobot_convo_mark_read(id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .mark_conversation_read(&id)
        .map_err(|err| err.to_string())
}
/// 会话容器（DM/群组）

#[tauri::command]
pub fn neobot_convo_add_member(id: String, member: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .add_conversation_member(&id, &member)
        .map_err(|err| err.to_string())
}
/// 会话容器（DM/群组）

#[tauri::command]
pub fn neobot_convo_rm_member(id: String, member: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .remove_conversation_member(&id, &member)
        .map_err(|err| err.to_string())
}

fn convo_dto(c: neotrix_neobot::Conversation) -> NeobotConvoItem {
    NeobotConvoItem {
        id: c.id,
        kind: c.kind,
        title: c.title,
        members: c.members,
        task_count: c.task_count,
        last_active: c.last_active,
        muted: c.muted,
        unread: c.unread,
    }
}
/// 附件落盘 + 行（50MB 上限）

#[tauri::command]
pub fn neobot_attach_add(convo_id: String, src_path: String) -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let src = std::path::Path::new(&src_path);
    let meta = std::fs::metadata(src).map_err(|err| format!("cannot read file: {err}"))?;
    if !meta.is_file() {
        return Err("not a file".to_owned());
    }
    if meta.len() > 50 * 1024 * 1024 {
        return Err(format!("file exceeds 50MB ({} bytes)", meta.len()));
    }
    let raw_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let safe: String = raw_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let safe = safe.trim().trim_start_matches('.').to_owned();
    let safe = if safe.is_empty() { "file".to_owned() } else { safe };
    let dest_name = format!(
        "{}-{safe}",
        uuid::Uuid::new_v4()
    );
    let dest_dir = config.data_dir.join("attachments");
    std::fs::create_dir_all(&dest_dir).map_err(|err| err.to_string())?;
    let dest = dest_dir.join(&dest_name);
    std::fs::copy(src, &dest).map_err(|err| err.to_string())?;
    let kind = neotrix_neobot::classify_attachment(&safe).to_owned();
    store
        .add_attachment(
            &convo_id,
            &kind,
            &safe,
            &dest.to_string_lossy(),
            meta.len() as i64,
        )
        .map_err(|err| err.to_string())
}
/// 附件落盘 + 行（50MB 上限）

#[tauri::command]
pub fn neobot_attachments(convo_id: String) -> Result<Vec<NeobotAttachItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    Ok(store
        .list_attachments(&convo_id)
        .map_err(|err| err.to_string())?
        .into_iter()
        .map(|a| NeobotAttachItem {
            id: a.id,
            kind: a.kind,
            name: a.name,
            path: a.path,
            size: a.size,
        })
        .collect())
}
/// 附件落盘 + 行（50MB 上限）

#[tauri::command]
pub fn neobot_attach_remove(id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.remove_attachment(&id).map_err(|err| err.to_string())
}
/// list/upsert/remove（首成员=owner）

#[tauri::command]
pub fn neobot_members() -> Result<Vec<NeobotMemberItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let owner = store.owner_name().map_err(|err| err.to_string())?;
    let members = store.list_members().map_err(|err| err.to_string())?;
    Ok(members
        .into_iter()
        .map(|(id, kind, _)| NeobotMemberItem {
            owner: owner.as_deref() == Some(&id),
            id,
            kind,
        })
        .collect())
}
/// list/upsert/remove（首成员=owner）

#[tauri::command]
pub fn neobot_member_add(id: String, kind: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.upsert_member(&id, &kind).map_err(|err| err.to_string())
}
/// list/upsert/remove（首成员=owner）

#[tauri::command]
pub fn neobot_member_remove(id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.remove_member(&id).map_err(|err| err.to_string())
}
