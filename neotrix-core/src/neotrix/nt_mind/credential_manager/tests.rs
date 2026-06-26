use super::*;

#[test]
fn test_store_and_find_decrypted() {
    let mut cm = CredentialManager::new();
    cm.store("whatsapp.com", "user@test.com", "secret123", "my whatsapp");
    let found = cm.find("whatsapp.com");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].username, "user@test.com");
    assert_eq!(found[0].password, "secret123");
}

#[test]
fn test_password_is_encrypted_in_memory() {
    let mut cm = CredentialManager::new();
    cm.store("test.com", "u", "my-password", "");
    let stored = &cm.all()[0];
    assert!(!stored.password_nonce.is_empty());
    assert!(!stored.password_ct.is_empty());
    assert_ne!(String::from_utf8_lossy(&stored.password_ct), "my-password");
}

#[test]
fn test_different_entries_get_different_nonces() {
    let mut cm = CredentialManager::new();
    cm.store("a.com", "u1", "pass1", "");
    cm.store("b.com", "u2", "pass2", "");
    let a = &cm.all()[0];
    let b = &cm.all()[1];
    assert_ne!(a.password_nonce, b.password_nonce, "nonces must be unique");
}

#[test]
fn test_case_insensitive_domain() {
    let mut cm = CredentialManager::new();
    cm.store("WhatsApp.COM", "user", "pass", "");
    assert_eq!(cm.find("whatsapp.com").len(), 1);
    assert_eq!(cm.find("WHATSAPP.COM").len(), 1);
}

#[test]
fn test_store_and_find_multiple_per_domain() {
    let mut cm = CredentialManager::new();
    let e1 = cm.store("example.com", "alice", "pass1", "");
    let e2 = cm.store("example.com", "bob", "pass2", "");
    let found = cm.find("example.com");
    assert_eq!(found.len(), 2);
    let passwords: Vec<&str> = found.iter().map(|e| e.password.as_str()).collect();
    assert!(passwords.contains(&"pass1"));
    assert!(passwords.contains(&"pass2"));
    assert_eq!(e1.username, "alice");
    assert_eq!(e2.username, "bob");
}

#[test]
fn test_remove_rebuilds_index() {
    let mut cm = CredentialManager::new();
    let e1 = cm.store("a.com", "u1", "p1", "");
    let _e2 = cm.store("a.com", "u2", "p2", "");
    let _e3 = cm.store("b.com", "u3", "p3", "");
    assert!(cm.remove(&e1.id));
    let a_entries = cm.find("a.com");
    assert_eq!(a_entries.len(), 1);
    assert_eq!(a_entries[0].username, "u2");
    let b_entries = cm.find("b.com");
    assert_eq!(b_entries.len(), 1);
    assert_eq!(b_entries[0].username, "u3");
}

#[test]
fn test_id_monotonic() {
    let mut cm = CredentialManager::new();
    let e1 = cm.store("x.com", "u1", "p", "");
    cm.remove(&e1.id);
    let e2 = cm.store("x.com", "u2", "p", "");
    assert_ne!(e1.id, e2.id);
}

#[test]
fn test_remove_nonexistent() {
    let mut cm = CredentialManager::new();
    assert!(!cm.remove("nonexistent"));
}

#[test]
fn test_remove_clean_state() {
    let mut cm = CredentialManager::new();
    let e = cm.store("x.com", "u", "p", "");
    cm.remove(&e.id);
    assert!(cm.find("x.com").is_empty());
    assert!(cm.all().is_empty());
}

#[test]
fn test_auto_fill_script_generates() {
    let mut cm = CredentialManager::new();
    cm.store("test.com", "user@test.com", "mypass", "");
    let script = cm.auto_fill_script("test.com");
    assert!(script.is_some());
    assert!(script.as_ref().unwrap().contains("user@test.com"));
    assert!(script.as_ref().unwrap().contains("mypass"));
}

#[test]
fn test_auto_fill_script_escapes_backslash() {
    let mut cm = CredentialManager::new();
    cm.store("esc.com", "user", r"pass\word", "");
    let script = cm.auto_fill_script("esc.com").unwrap();
    assert!(
        script.contains("pass\\\\word"),
        "backslash should be escaped: {}",
        script
    );
}

#[test]
fn test_auto_fill_no_submit() {
    let mut cm = CredentialManager::new();
    cm.store("x.com", "u", "p", "");
    let script = cm.auto_fill_script("x.com").unwrap();
    assert!(!script.contains("submit"), "should not auto-submit");
}

#[test]
fn test_auto_fill_script_no_match() {
    let mut cm = CredentialManager::new();
    assert!(cm.auto_fill_script("unknown.com").is_none());
}

#[test]
fn test_wrong_key_fails_decryption() {
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    let mut wrong_key = key;
    wrong_key[0] ^= 0xFF;

    let mut cm = CredentialManager::with_key(key);
    cm.store("secret.com", "admin", "super-secret", "");

    let decrypted = cm.find("secret.com");
    assert_eq!(decrypted.len(), 1);
    assert_eq!(decrypted[0].password, "super-secret");

    let _cm_wrong = CredentialManager::with_key(wrong_key);
    let _loaded_cm = {
        let json = serde_json::to_string(&cm.all()).unwrap();
        let entries: Vec<CredentialEntry> = serde_json::from_str(&json).unwrap();
        let mut failures = 0;
        for e in &entries {
            let wrong_cipher = Aes256Gcm::new_from_slice(&wrong_key).unwrap();
            let nonce = Nonce::from_slice(&e.password_nonce);
            let result = wrong_cipher.decrypt(nonce, e.password_ct.as_slice());
            if result.is_err() {
                failures += 1;
            }
        }
        assert_eq!(
            failures,
            entries.len(),
            "all entries should fail with wrong key"
        );
    };
}

#[test]
fn test_save_and_load() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("creds.json");

    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);

    {
        let mut cm = CredentialManager::with_key(key);
        cm.store("a.com", "alice", "secret1", "note a");
        cm.store("b.com", "bob", "secret2", "note b");
        cm.save_to(&path).unwrap();
    }

    let mut cm = CredentialManager::load_from(&path, key).unwrap();
    let a = cm.find("a.com");
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].password, "secret1");
    let b = cm.find("b.com");
    assert_eq!(b[0].password, "secret2");
    assert_eq!(cm.all().len(), 2);
}

#[test]
fn test_load_empty_file() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("empty.json");
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    let result = CredentialManager::load_from(&path, key);
    assert!(result.is_err());
}

#[test]
fn test_derive_key_from_password_deterministic() {
    let salt = b"fixed-salt-12345678";
    let k1 = derive_key_from_password("mypassword", salt);
    let k2 = derive_key_from_password("mypassword", salt);
    assert_eq!(k1, k2, "same password + salt produces same key");

    let k3 = derive_key_from_password("different", salt);
    assert_ne!(k1, k3, "different password produces different key");
}

#[test]
fn test_key_from_env_var() {
    let key = load_or_generate_master_key();
    assert!(key.is_ok());
    assert_eq!(key.unwrap().len(), 32);
}

#[test]
fn test_store_returns_entry_with_encrypted_fields() {
    let mut cm = CredentialManager::new();
    let entry = cm.store("enc-test.com", "user123", "my-secret-password", "test note");
    assert_eq!(entry.domain, "enc-test.com");
    assert_eq!(entry.username, "user123");
    assert_eq!(entry.notes, "test note");
}

#[test]
fn test_roundtrip_unicode_password() {
    let mut cm = CredentialManager::new();
    cm.store("unicode.com", "用户", "密码🔑", "中文备注");
    let found = cm.find("unicode.com");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].username, "用户");
    assert_eq!(found[0].password, "密码🔑");
    assert_eq!(found[0].notes, "中文备注");
}

#[test]
fn test_audit_log_records_operation() {
    let mut cm = CredentialManager::new();
    cm.store("audit-test.com", "alice", "secret", "audit check");
    let entries: Vec<&AuditEntry> = cm.audit_log().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].operation, "store");
    assert_eq!(entries[0].source, "internal");
    assert!(entries[0].success);
    assert_eq!(entries[0].domain.as_deref(), Some("audit-test.com"));
}

#[test]
fn test_audit_log_trim() {
    let mut cm = CredentialManager::new();
    for i in 0..MAX_AUDIT_LOG_SIZE + 1 {
        cm.record_audit(
            Some(format!("d{}.com", i)),
            "store",
            "test",
            true,
            "trim test",
        );
    }
    let count = cm.audit_log().count();
    assert_eq!(count, MAX_AUDIT_LOG_SIZE);
}

#[test]
fn test_audit_log_filter_domain() {
    let mut cm = CredentialManager::new();
    cm.store("alpha.com", "u1", "p1", "");
    cm.store("beta.com", "u2", "p2", "");
    cm.store("alpha.com", "u3", "p3", "");
    let alpha_entries = cm.audit_log_by_domain("alpha.com");
    assert_eq!(alpha_entries.len(), 2);
    let beta_entries = cm.audit_log_by_domain("beta.com");
    assert_eq!(beta_entries.len(), 1);
}

#[test]
fn test_audit_log_since() {
    let mut cm = CredentialManager::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    cm.store("since-test.com", "u", "p", "");
    let entries = cm.audit_log_since(now);
    assert_eq!(entries.len(), 1);
    let entries_future = cm.audit_log_since(now + 999999);
    assert!(entries_future.is_empty());
}

#[test]
fn test_re_encrypt_preserves_data() {
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    let mut cm = CredentialManager::with_key(key);
    cm.store("a.com", "alice", "secret1", "note a");
    cm.store("b.com", "bob", "secret2", "note b");

    let mut new_key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut new_key);
    cm.re_encrypt(new_key).unwrap();

    let a = cm.find("a.com");
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].password, "secret1");
    let b = cm.find("b.com");
    assert_eq!(b[0].password, "secret2");
}

#[test]
fn test_re_encrypt_changes_ciphertext() {
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    let mut cm = CredentialManager::with_key(key);
    cm.store("a.com", "alice", "secret1", "");
    let old_ct = cm.all()[0].password_ct.clone();

    let mut new_key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut new_key);
    cm.re_encrypt(new_key).unwrap();

    let new_ct = &cm.all()[0].password_ct;
    assert_ne!(old_ct, *new_ct, "ciphertext should change after re_encrypt");
    let a = cm.find("a.com");
    assert_eq!(a[0].password, "secret1");
}

#[test]
fn test_health_check_weak_password() {
    let mut cm = CredentialManager::new();
    cm.store("weak.com", "user1", "ab", "short password");
    let report = cm.health_check();
    assert!(
        !report.weak_passwords.is_empty(),
        "expected weak password to be flagged"
    );
    assert!(report.weak_passwords.iter().any(|w| w.domain == "weak.com"));
}

#[test]
fn test_health_check_reuse() {
    let mut cm = CredentialManager::new();
    cm.store("site1.com", "alice", "sharedPass1!", "");
    cm.store("site2.com", "bob", "sharedPass1!", "");
    let report = cm.health_check();
    assert!(
        !report.reused_passwords.is_empty(),
        "expected reuse to be detected"
    );
    let total_reused: usize = report.reused_passwords.iter().map(|g| g.count).sum();
    assert_eq!(total_reused, 2, "expected 2 entries in reuse groups");
}

#[test]
fn test_health_check_expiry() {
    let mut cm = CredentialManager::new();
    cm.store("old.com", "user", "StrongPass1!", "");
    cm.entries[0].created_at = 1;
    let report = cm.health_check();
    assert!(
        !report.expired_passwords.is_empty(),
        "expected expired entry"
    );
    assert!(report.expired_passwords[0].age_days >= 180);
}
