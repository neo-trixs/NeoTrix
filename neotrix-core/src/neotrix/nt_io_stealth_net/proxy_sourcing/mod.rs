pub mod core;
pub mod types;

pub use types::*;

pub use core::{
    batch_validate, dedup_proxies, fetch_and_validate, parse_json_array, parse_plain_text,
    validate_proxy, validate_proxy_multi_target, ProxySourcing,
};
