use super::session::Session;
use crate::neotrix::nt_io_gram::tl::{TlReader, TlWriter};

pub fn rpc_response_body(constructor: u32, body: &[u8]) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(constructor);
    w.write_raw(body);
    w.into_bytes()
}

pub fn write_json_value(w: &mut TlWriter, val: &serde_json::Value) {
    match val {
        serde_json::Value::Null => {
            w.write_uint32(0x7e5ef2cb);
            w.write_string("null");
        }
        serde_json::Value::Bool(b) => {
            w.write_bool(*b);
        }
        serde_json::Value::Number(n) => {
            w.write_uint32(0x29ea9c29);
            w.write_double(n.as_f64().unwrap_or(0.0));
        }
        serde_json::Value::String(s) => {
            w.write_uint32(0xb3f0efb5);
            w.write_string(s);
        }
        serde_json::Value::Array(arr) => {
            w.write_uint32(0x707b2f73);
            w.write_uint32(arr.len() as u32);
            for item in arr {
                write_json_value(w, item);
            }
        }
        serde_json::Value::Object(map) => {
            w.write_uint32(0x99c2d5c1);
            w.write_uint32(map.len() as u32);
            for (k, v) in map {
                w.write_string(k);
                write_json_value(w, v);
            }
        }
    }
}

pub fn help_get_config(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    rpc_response_body(
        0xc4f9186b,
        &[
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x1c, 0xcb, 0x5c, 0x15, 0x01, 0x00, 0x00, 0x00, 0x1f, 0x2c, 0x3d, 0x4e,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00,
        ],
    )
}

pub fn help_get_app_config(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    let json = build_premium_config_json();
    let mut w = TlWriter::new();
    w.write_uint32(0x9e5d5c09);
    w.write_uint32(0);
    write_json_value(&mut w, &json);
    w.into_bytes()
}

pub fn help_get_nearest_dc(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x8e1a1775);
    w.write_string("neotrix");
    w.write_int32(1);
    w.write_int32(1);
    w.into_bytes()
}

pub fn help_get_promo_data(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x8e1a1775);
    w.write_int32(0);
    w.write_int32(0);
    w.write_string("");
    w.write_uint32(0x1cb5c415);
    w.write_uint32(0);
    w.into_bytes()
}

pub fn help_get_premium_promo(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x3159fcd7);
    w.write_string("");
    w.write_string("");
    w.write_uint32(0x1cb5c415);
    w.write_uint32(0);
    w.write_uint32(0x1cb5c415);
    w.write_uint32(0);
    w.into_bytes()
}

pub fn auth_send_code(_session: &mut Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x768d6f4d);
    w.write_string("+9999999999");
    w.write_uint32(0x00000000);
    w.into_bytes()
}

pub fn auth_sign_in(_session: &mut Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x786a70f2);
    w.write_int32(0);
    let user = User {
        id: 1000001,
        first_name: "NeoTrix".into(),
        last_name: "User".into(),
        username: "neotrix_user".into(),
        premium: true,
        ..Default::default()
    };
    write_user(&mut w, &user);
    w.into_bytes()
}

pub fn auth_check_password(session: &mut Session, reader: &mut TlReader) -> Vec<u8> {
    auth_sign_in(session, reader)
}

#[derive(Default)]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub phone: String,
    pub premium: bool,
}

pub fn write_user(w: &mut TlWriter, user: &User) {
    let flags = if user.premium {
        0x10000001u32
    } else {
        0x00000001u32
    };
    w.write_uint32(flags);
    w.write_int64(user.id);
    w.write_int64(0);
    w.write_string(&user.first_name);
    w.write_string(&user.last_name);
    w.write_string(&user.username);
    w.write_string(&user.phone);
    w.write_string("default");
    w.write_int64(0);
    w.write_string("");
    w.write_string("en");
}

pub fn messages_get_dialogs(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x15ba6c40);
    w.write_uint32(0x1cb5c415);
    w.write_uint32(0);
    w.write_uint32(0x1cb5c415);
    w.write_uint32(0);
    w.write_uint32(0x1cb5c415);
    w.write_uint32(0);
    w.write_uint32(0x5cb6ab2d);
    w.into_bytes()
}

pub fn messages_send_message(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x62e5f6f2);
    w.write_bool(true);
    w.write_int32(0);
    w.write_int32(0);
    w.write_int32(0);
    w.into_bytes()
}

pub fn users_get_full_user(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0xb72b2cb4);
    let user = User {
        id: 1000001,
        first_name: "NeoTrix".into(),
        last_name: "User".into(),
        username: "neotrix_user".into(),
        premium: true,
        ..Default::default()
    };
    write_user(&mut w, &user);
    w.write_string("About NeoTrix Self-Hosted Telegram");
    w.write_int32(100);
    w.write_int32(100);
    w.write_int32(0);
    w.write_int32(0);
    w.into_bytes()
}

pub fn messages_get_chats(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    let mut w = TlWriter::new();
    w.write_uint32(0x90a6d4f7);
    w.write_uint32(0x1cb5c415);
    w.write_uint32(0);
    w.into_bytes()
}

pub fn channels_get_channels(_session: &Session, _reader: &mut TlReader) -> Vec<u8> {
    messages_get_chats(_session, _reader)
}

fn build_premium_config_json() -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("channels_limit_default".into(), 1000.into());
    map.insert("channels_limit_premium".into(), 1000.into());
    map.insert("upload_max_fileparts_default".into(), 8000.into());
    map.insert("upload_max_fileparts_premium".into(), 8000.into());
    map.insert("dialog_filters_limit_default".into(), 30.into());
    map.insert("dialog_filters_limit_premium".into(), 30.into());
    map.insert("dialog_filters_chats_limit_default".into(), 200.into());
    map.insert("dialog_filters_chats_limit_premium".into(), 200.into());
    map.insert("dialogs_pinned_limit_default".into(), 10.into());
    map.insert("dialogs_pinned_limit_premium".into(), 10.into());
    map.insert("dialogs_folder_pinned_limit_default".into(), 200.into());
    map.insert("dialogs_folder_pinned_limit_premium".into(), 200.into());
    map.insert("channels_public_limit_default".into(), 20.into());
    map.insert("channels_public_limit_premium".into(), 20.into());
    map.insert("caption_length_limit_default".into(), 4096.into());
    map.insert("caption_length_limit_premium".into(), 4096.into());
    map.insert("about_length_limit_default".into(), 140.into());
    map.insert("about_length_limit_premium".into(), 140.into());
    map.insert("saved_gifs_limit_default".into(), 400.into());
    map.insert("saved_gifs_limit_premium".into(), 400.into());
    map.insert("stickers_faved_limit_default".into(), 10.into());
    map.insert("stickers_faved_limit_premium".into(), 10.into());
    map.insert("chatlist_invites_limit_default".into(), 100.into());
    map.insert("chatlist_invites_limit_premium".into(), 100.into());
    map.insert("chatlists_joined_limit_default".into(), 20.into());
    map.insert("chatlists_joined_limit_premium".into(), 20.into());
    map.insert("recommended_channels_limit_default".into(), 100.into());
    map.insert("recommended_channels_limit_premium".into(), 100.into());
    map.insert("saved_dialogs_pinned_limit_default".into(), 100.into());
    map.insert("saved_dialogs_pinned_limit_premium".into(), 100.into());
    map.insert("story_expiring_limit_default".into(), 100.into());
    map.insert("story_expiring_limit_premium".into(), 100.into());
    map.insert("story_caption_length_limit_default".into(), 2048.into());
    map.insert("story_caption_length_limit_premium".into(), 2048.into());
    map.insert("stories_sent_weekly_limit_default".into(), 700.into());
    map.insert("stories_sent_weekly_limit_premium".into(), 700.into());
    map.insert("stories_sent_monthly_limit_default".into(), 3000.into());
    map.insert("stories_sent_monthly_limit_premium".into(), 3000.into());
    map.insert("stories_suggested_reactions_limit_default".into(), 5.into());
    map.insert("stories_suggested_reactions_limit_premium".into(), 5.into());
    map.insert("stories_entities".into(), "enabled".into());
    map.insert("transcribe_audio_trial_weekly_number".into(), 999999.into());
    map.insert("transcribe_audio_trial_duration_max".into(), 999999.into());
    map.insert("reactions_user_max_default".into(), 3.into());
    map.insert("reactions_user_max_premium".into(), 3.into());
    map.insert("premium_purchase_blocked".into(), true.into());
    map.insert("premium_show_promotion".into(), false.into());
    serde_json::Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_get_config_constructor() {
        let s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = help_get_config(&s, &mut r);
        assert_eq!(&result[..4], &[0x6b, 0x18, 0xf9, 0xc4]);
    }

    #[test]
    fn test_help_get_app_config_constructor() {
        let s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = help_get_app_config(&s, &mut r);
        assert_eq!(&result[..4], &[0x09, 0x5c, 0x5d, 0x9e]);
    }

    #[test]
    fn test_premium_config_no_promotion() {
        let json = build_premium_config_json();
        assert_eq!(json["premium_show_promotion"], false);
        assert_eq!(json["premium_purchase_blocked"], true);
        assert_eq!(json["channels_limit_default"], 1000);
        assert_eq!(json["channels_limit_premium"], 1000);
        assert_eq!(json["stories_entities"], "enabled");
    }

    #[test]
    fn test_user_premium_flag() {
        let user = User {
            id: 1,
            first_name: "T".into(),
            premium: true,
            ..Default::default()
        };
        let mut w = TlWriter::new();
        write_user(&mut w, &user);
        let data = w.into_bytes();
        let flags = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        assert!(flags & 0x10000000 != 0);
    }

    #[test]
    fn test_auth_sign_in_returns_user() {
        let mut s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = auth_sign_in(&mut s, &mut r);
        assert_eq!(&result[..4], &[0xf2, 0x70, 0x6a, 0x78]);
    }

    #[test]
    fn test_messages_get_dialogs_not_empty() {
        let s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = messages_get_dialogs(&s, &mut r);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_json_object_serialization() {
        let json = serde_json::json!({"key": "val", "num": 42});
        let mut w = TlWriter::new();
        write_json_value(&mut w, &json);
        let data = w.into_bytes();
        assert_eq!(&data[..4], &[0xc1, 0xd5, 0xc2, 0x99]);
    }

    #[test]
    fn test_get_nearest_dc() {
        let s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = help_get_nearest_dc(&s, &mut r);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_help_get_promo_data() {
        let s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = help_get_promo_data(&s, &mut r);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_channels_eq_messages_get_chats() {
        let s = Session::new();
        let a = channels_get_channels(&s, &mut TlReader::new(vec![]));
        let b = messages_get_chats(&s, &mut TlReader::new(vec![]));
        assert_eq!(a, b);
    }

    #[test]
    fn test_help_get_premium_promo() {
        let s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = help_get_premium_promo(&s, &mut r);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_users_get_full_user() {
        let s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = users_get_full_user(&s, &mut r);
        assert_eq!(&result[..4], &[0xb4, 0x2c, 0x2b, 0xb7]);
    }

    #[test]
    fn test_messages_send_message() {
        let s = Session::new();
        let mut r = TlReader::new(vec![]);
        let result = messages_send_message(&s, &mut r);
        assert_eq!(&result[..4], &[0xf2, 0xf6, 0xe5, 0x62]);
    }
}
