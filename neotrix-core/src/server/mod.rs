pub mod http;
pub mod ws;
pub mod h5;
pub mod session;
pub mod ios;

pub use http::start_server;
pub use ios::{IosAppState, ios_routes};
pub use ws::{WsBridge, WsBridgeConfig, ImAdapter, TelegramAdapter, WhatsAppAdapter};
pub mod server_interface;

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
