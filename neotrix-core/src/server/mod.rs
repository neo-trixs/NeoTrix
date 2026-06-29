pub mod h5;
pub mod http;
pub mod ios;
pub mod session;
pub mod ws;

pub use http::start_server;
pub use ios::{ios_routes, IosAppState};
pub use ws::{ImAdapter, TelegramAdapter, WhatsAppAdapter, WsBridge, WsBridgeConfig};
pub mod compressor;
pub mod gateway;
pub mod openai_compat;
pub mod server_interface;
pub mod voice_agent;
pub mod webrtc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_bridge_config_default() {
        let cfg = WsBridgeConfig::default();
        assert_eq!(cfg.heartbeat_interval, std::time::Duration::from_secs(30));
    }

    #[test]
    fn test_ws_bridge_config_clone() {
        let cfg = WsBridgeConfig::default();
        let cloned = WsBridgeConfig { ..cfg };
        assert_eq!(cloned.max_reconnect_attempts, cfg.max_reconnect_attempts);
    }
}
