//! nt_memory_unify: 统一存储原语 (config/secrets/session/cookies/assets/skills/rkyv/migration).
//! 由单文件拆为目录模块, 外部路径保持不变.
pub mod nt_unify_config;
pub mod nt_unify_session;
pub mod nt_unify_skills;

pub use nt_unify_config::{
    config_all_sections, config_delete, config_get, config_list_section, config_set,
    is_compressed_value, kv_delete, kv_exists, kv_get, kv_list, kv_list_namespaces,
    kv_purge_namespace, kv_set, now, secret_delete, secret_get, secret_list, secret_set,
    NONCE_LEN, VALUE_COMPRESSED_MAGIC,
};
pub use nt_unify_session::{
    asset_delete, asset_list, asset_load, asset_store, cookie_delete, cookie_get,
    cookie_list_domain, cookie_purge_expired, cookie_set, rkyv_delete, rkyv_list, rkyv_load,
    rkyv_store, session_log_append, session_log_get, session_log_list_sessions, store_stats,
};
pub use nt_unify_skills::{
    domain_ns, domain_skills, migrate_from_files, skill_content_hash, skill_delete,
    skill_list_all, skill_search, skill_upsert, unify_domain_mapping, MigrationReport,
    SkillRecord, DOMAIN_SKILL_MAPPING,
};
