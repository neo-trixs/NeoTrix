use super::api;
use super::session::Session;
use crate::neotrix::nt_act_gram::tl::{serialize_error, TlReader, TlWriter};

pub fn handle_rpc_call(session: &mut Session, data: &[u8]) -> Vec<u8> {
    let mut reader = TlReader::new(data.to_vec());
    let msg_id = match reader.read_int64() {
        Ok(id) => id,
        Err(_) => return serialize_error(400, "MSG_ID_INVALID"),
    };

    let _seq_no = match reader.read_uint32() {
        Ok(s) => s,
        Err(_) => return serialize_error(400, "SEQ_NO_INVALID"),
    };

    let content_len = match reader.read_uint32() {
        Ok(l) => l as usize,
        Err(_) => return serialize_error(400, "CONTENT_LEN_INVALID"),
    };

    let content_start = reader.position();
    if content_start + content_len > data.len() {
        return serialize_error(400, "CONTENT_TRUNCATED");
    }
    let content_bytes = &data[content_start..content_start + content_len];

    let mut content_reader = TlReader::new(content_bytes.to_vec());
    let constructor = match content_reader.read_uint32() {
        Ok(c) => c,
        Err(_) => return serialize_error(400, "CONSTRUCTOR_INVALID"),
    };

    let response = dispatch_method(session, constructor, &mut content_reader);

    let mut w = TlWriter::new();
    w.write_int64(msg_id);
    w.write_int64(session.next_msg_id());
    w.write_uint32(0x00000000);
    w.write_uint32(response.len() as u32);
    w.write_raw(&response);
    w.into_bytes()
}

fn dispatch_method(session: &mut Session, constructor: u32, reader: &mut TlReader) -> Vec<u8> {
    match constructor {
        0xc4f9186b => api::help_get_config(session, reader),
        0x5e592a1e => api::help_get_app_config(session, reader),
        0x89960931 => api::help_get_nearest_dc(session, reader),
        0xa2f05fba => api::help_get_promo_data(session, reader),
        0xb7e235fe => api::help_get_premium_promo(session, reader),
        0x60469778 => api::auth_send_code(session, reader),
        0xbcd51581 => api::auth_sign_in(session, reader),
        0x80eee427 => api::auth_check_password(session, reader),
        0xced3c06e => api::messages_send_message(session, reader),
        0xca30a5e1 => api::users_get_full_user(session, reader),
        0xfe458b07 => api::messages_get_dialogs(session, reader),
        0xefe350b1 => api::messages_get_chats(session, reader),
        0x76d9e7e1 => api::channels_get_channels(session, reader),
        _ => {
            log::warn!("unhandled RPC: 0x{:08x}", constructor);
            serialize_error(400, "METHOD_NOT_FOUND")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_method() {
        let mut s = Session::new();
        let mut w = TlWriter::new();
        w.write_int64(100);
        w.write_uint32(1);
        w.write_uint32(4);
        w.write_uint32(0xDEADBEEF);
        let resp = handle_rpc_call(&mut s, &w.into_bytes());
        let mut r = TlReader::new(resp);
        assert_eq!(r.read_int64().expect("value should be ok in test"), 100);
    }

    #[test]
    fn test_help_get_config_dispatch() {
        let mut s = Session::new();
        let mut w = TlWriter::new();
        w.write_int64(42);
        w.write_uint32(1);
        w.write_uint32(4);
        w.write_uint32(0xc4f9186b);
        let resp = handle_rpc_call(&mut s, &w.into_bytes());
        assert!(!resp.is_empty());
    }

    #[test]
    fn test_auth_send_code_dispatch() {
        let mut s = Session::new();
        let mut w = TlWriter::new();
        w.write_int64(1);
        w.write_uint32(1);
        w.write_uint32(4);
        w.write_uint32(0x60469778);
        let resp = handle_rpc_call(&mut s, &w.into_bytes());
        assert!(!resp.is_empty());
    }
}
