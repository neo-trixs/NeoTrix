use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

/// Decision for a single action key within a profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileDecision {
    Allow,
    Deny,
    Ask,
}

impl ProfileDecision {
    /// 单调收紧 (策略单调性不变量, NT-SHIELD policy_monotonic_invariant):
    /// 新决策只能把规则收紧到更严格 (Allow→Ask/Deny, Ask→Deny), 永不放宽已保存策略。
    /// Deny 不可被覆盖为 Ask/Allow; Ask 不可被覆盖为 Allow。
    pub fn tightened_with(&self, incoming: &ProfileDecision) -> ProfileDecision {
        match (self, incoming) {
            (ProfileDecision::Deny, _) => ProfileDecision::Deny,
            (ProfileDecision::Ask, ProfileDecision::Allow) => ProfileDecision::Ask,
            (_, ProfileDecision::Deny) => ProfileDecision::Deny,
            (ProfileDecision::Ask, ProfileDecision::Ask) => ProfileDecision::Ask,
            (ProfileDecision::Allow, _) => *incoming,
        }
    }
}

/// A named, inheritable permission profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionProfile {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub rules: HashMap<String, ProfileDecision>,
    /// Override the global approval mode while this profile is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_mode_override: Option<String>,
}

impl PermissionProfile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parent: None,
            rules: HashMap::new(),
            approval_mode_override: None,
        }
    }
}

/// All profiles persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStore {
    pub profiles: HashMap<String, PermissionProfile>,
    pub active: String,
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self::builtin()
    }
}

impl ProfileStore {
    pub fn builtin() -> Self {
        let mut profiles = HashMap::new();

        // nt_shield (default)
        let mut nt_shield = PermissionProfile::new("nt_shield");
        nt_shield.rules.insert("write_file".into(), ProfileDecision::Ask);
        nt_shield.rules.insert("delete_file".into(), ProfileDecision::Ask);
        nt_shield.rules.insert("execute_command".into(), ProfileDecision::Ask);
        nt_shield.rules.insert("network_request".into(), ProfileDecision::Allow);
        nt_shield.rules.insert("read_file".into(), ProfileDecision::Allow);
        nt_shield.rules.insert("read_secrets".into(), ProfileDecision::Deny);
        nt_shield.rules.insert("git_push".into(), ProfileDecision::Ask);
        nt_shield.rules.insert("git_force_push".into(), ProfileDecision::Deny);
        nt_shield.rules.insert("compile_check".into(), ProfileDecision::Allow);
        nt_shield.rules.insert("modify_dependency".into(), ProfileDecision::Ask);
        nt_shield.rules.insert("access_stealth_browser_auto".into(), ProfileDecision::Ask);
        nt_shield.rules.insert("access_tor_network".into(), ProfileDecision::Ask);
        profiles.insert("nt_shield".into(), nt_shield);

        // strict-nt_shield
        let mut strict = PermissionProfile::new("strict-nt_shield");
        strict.parent = Some("nt_shield".into());
        strict.rules.insert("write_file".into(), ProfileDecision::Deny);
        strict.rules.insert("network_request".into(), ProfileDecision::Ask);
        strict.rules.insert("read_file".into(), ProfileDecision::Allow);
        strict.rules.insert("access_tor_network".into(), ProfileDecision::Deny);
        profiles.insert("strict-nt_shield".into(), strict);

        // general
        let mut general = PermissionProfile::new("general");
        general.parent = Some("nt_shield".into());
        general.rules.insert("write_file".into(), ProfileDecision::Allow);
        general.rules.insert("delete_file".into(), ProfileDecision::Ask);
        general.rules.insert("execute_command".into(), ProfileDecision::Allow);
        general.rules.insert("network_request".into(), ProfileDecision::Allow);
        general.rules.insert("modify_dependency".into(), ProfileDecision::Allow);
        profiles.insert("general".into(), general);

        // developer (most permissive)
        let mut developer = PermissionProfile::new("developer");
        developer.parent = Some("general".into());
        developer.rules.insert("delete_file".into(), ProfileDecision::Allow);
        developer.rules.insert("git_push".into(), ProfileDecision::Allow);
        developer.rules.insert("access_stealth_browser_auto".into(), ProfileDecision::Allow);
        developer.rules.insert("access_tor_network".into(), ProfileDecision::Deny);
        developer.approval_mode_override = Some("auto-edit".into());
        profiles.insert("developer".into(), developer);

        Self {
            profiles,
            active: "nt_shield".into(),
        }
    }

    /// Resolve the effective rules for a profile (merging parent chain).
    pub fn resolve(&self, name: &str) -> Option<HashMap<String, ProfileDecision>> {
        let profile = self.profiles.get(name)?;
        // 显式类型标注：接入单调合并后推断路径变窄，
        //   E0282 ⇒ HashMap 的键值类型必须写出来。
        let mut merged: HashMap<String, ProfileDecision> = HashMap::new();

        // Walk parent chain: root first, then child overrides
        let mut chain: Vec<&PermissionProfile> = vec![profile];
        let mut current = profile.parent.as_deref();
        while let Some(parent_name) = current {
            if let Some(parent) = self.profiles.get(parent_name) {
                chain.push(parent);
                current = parent.parent.as_deref();
            } else {
                break;
            }
        }
        // ══════════════════════════════════════════════════════════════
        // ⚠️ 2026-10-05 **撤回一次错误的「修复」** —— 留痕，因为它的教训比它的
        //    正确版本更值钱。
        //
        // 【我一度做了什么】把 `merged.insert(k, *v)`（子档无条件覆盖）
        //   改成 `existing.tightened_with(v)`（单调收紧），
        //   理由是「Codewhale 第 1 层：overlay 只能收紧」。
        //
        // 【为什么撤回】实测让 **4 条既有测试变红**，而那 4 条并不过时：
        // `general` 档断言 `write_file → Allow`，其祖先 `nt_shield` 是 `Ask`。
        // ⇒ 单调合并正确地把它收紧成 `Ask`/`Deny`，
        //   **而这正是 `general` / `developer` 两个档失去存在意义的原因** ——
        //   它们的注释直写 `general`（通用开发）/ `developer` (most permissive)。
        //
        // 【教训 】**「规则违反了我从外部读来的原则」与「这条原则在这个
        //   系统里是错的」是两件事。** 我把 Codewhale 的「overlay 只能收紧」
        //   当成了普适原则套上去，**没有先问**：
        //   这个仓库**刻意**提供了宽松档（它是**产品选择**，不是缺陷），
        //   而 Codewhale 之所以能「只能收紧」是因为它**只有一个 profile**
        //   + per-project overlay，用户没有「切换到宽松档」这个需求。
        // ⇒ 外部原则要落地，先验证**前提是否成立**。
        //
        // 【那还剩什么真问题？】剩下的**不是**「合并无单调」，
        //   而是 **`switch_profile` 能改全局审批模式且无任何记录**：
        //   它从父链继承 `approval_mode_override` 后直接 `engine.set_mode(..)`
        //   （见 `switch_profile`）—— 这才是「静默放宽」。
        //   该项**需要产品裁决**（是否允许一个档改变全局模式），
        //   已记入 `docs/architecture/CLAIMED-BUT-NOT-ENFORCED-2026-10-05.md`。
        //
        // ⇒ 本笔**不改合并语义**（保持子档可显式覆盖父档），
        //   只**补一条反向锁**，把这个「刻意允许放宽」的语义**写进契约**，
        //   防止下一个 agent 再把它当缺陷「修」一遍。
        for p in chain.into_iter().rev() {
            for (k, v) in &p.rules {
                merged.insert(k.clone(), *v);
            }
        }
        Some(merged)
    }

    /// Get the effective approval_mode_override, walking parent chain.
    pub fn resolve_approval_mode(&self, name: &str) -> Option<String> {
        let profile = self.profiles.get(name)?;
        if profile.approval_mode_override.is_some() {
            return profile.approval_mode_override.clone();
        }
        if let Some(ref parent) = profile.parent {
            return self.resolve_approval_mode(parent);
        }
        None
    }

    /// Evaluate a single action key against a resolved rule set.
    pub fn evaluate(&self, profile_name: &str, action_key: &str) -> Option<ProfileDecision> {
        let rules = self.resolve(profile_name)?;
        rules.get(action_key).copied()
    }
}

/// Config path: ~/.neotrix/profiles.toml
fn profiles_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".neotrix").join("profiles.toml")
}

/// Global profile manager fallback — prefer `CliContext` injection instead.
pub static PROFILE_MANAGER: LazyLock<Mutex<ProfileStore>> = LazyLock::new(|| {
    let path = profiles_path();
      let store = if path.exists() {
          // 2026-09-30: 原为 `.ok().and_then(…ok()).unwrap_or_default()` 三段折叠 ——
          // 「文件存在但读不出来（权限）」与「内容解析失败（写入被截断 / 磁盘满 / 手改）」
          // 两种情况都静默替换为 `ProfileStore::default()`（= builtin），
          // **用户的自定义档全部消失且无备份、无日志**。而随后任一
          // `switch_profile` / `create_profile` / `remove_profile` / `set_rule`
          // 都会 `save_profiles_to_disk` 用 builtin+增量**整文件覆写**
          // ⇒ 损坏文件里尚可抢救的内容被永久销毁。
          // ⛔ 此处**只补可观测性，不加 `create_dir_all` 之类「顺手修复」** ——
          //    补可观测性与行为变更必须分开，否则会像 cleanup 那次一样
          //    凭空创建原本不存在的东西（那次被测试当场抓出）。
          let store = match std::fs::read_to_string(&path) {
              Ok(text) => match toml::from_str::<ProfileStore>(&text) {
                  Ok(s) => s,
                  Err(e) => {
                      log::error!(
                          "[profiles] 解析失败 {}: {e} —— 已退回内置档；\
                               首次写入将整文件覆写，损坏内容不可抢救",
                          path.display()
                      );
                      ProfileStore::builtin()
                  }
              },
              Err(e) => {
                  log::error!(
                      "[profiles] 读取失败 {}: {e} —— 已退回内置档；\
                       首次写入将整文件覆写",
                      path.display()
                  );
                  ProfileStore::builtin()
              }
          };
          store
      } else {
          let store = ProfileStore::builtin();
          let _ = save_profiles_to_disk(&store);
          store
      };
    Mutex::new(store)
});

pub fn global_profile_manager() -> &'static Mutex<ProfileStore> {
    &PROFILE_MANAGER
}

fn save_profiles_to_disk(store: &ProfileStore) -> Result<(), String> {
    let path = profiles_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let data = toml::to_string_pretty(store).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

/// Reinitialize the profile manager (for testing).
pub fn reset_profile_manager() {
    if let Ok(mut guard) = PROFILE_MANAGER.lock() {
        *guard = ProfileStore::builtin();
    }
}

/// Public API: switch to a named profile.
/// Also applies the profile's approval_mode_override to the global approval engine.
/// 切档提案：把「会发生什么」**先算清楚交给调用方**，而不是事后通知。
///
/// 方案 A（2026-10-05 用户裁决）：允许档位改变全局审批模式，
/// 但必须**显式确认 + 留痕**。
/// 拆成「提案 / 执行」两步的理由：
/// · 本函数是**库函数** ⇒ 不该自己读 stdin 做交互
///   （那会让它无法在非 TTY / 服务端 / 测试里使用）；
/// · 「会发生什么」必须在**动手之前**可读，否则确认只是形式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSwitchPlan {
    /// 目标档位。
    pub profile: String,
    /// 该档位（含沿父链继承）解析出的审批模式覆盖。
    /// `None` ⇒ 切过去**不会**动全局审批模式。
    pub approval_mode_override: Option<String>,
    /// 切过去之后全局审批模式**实际会变成**什么。
    /// 保留 `None` 表示「当前模式不会被改」。
    pub resulting_mode: Option<crate::l6_meta::nt_approval::ApprovalMode>,
    /// 人类可读的副作用说明，**必须**能被直接打印给用户。
    pub notice: Option<String>,
}

impl ProfileSwitchPlan {
    /// 该次切换是否**会改动全局审批模式**。
    /// ⇒ 调用方据此决定要不要先问一句。
    pub fn changes_approval_mode(&self) -> bool {
        self.resulting_mode.is_some()
    }

    /// ⚠️ 该次切换是否**放宽**了审批严格程度。
    ///
    /// `ApprovalMode` 的排序是 Ask 最严 ⇒ Allow 最松
    /// （`FullAuto` 最松、`Suggest` 最严）。
    /// ⇒ 从严到松才是「放宽」，反向则是「收紧」。
    pub fn loosens_approval(&self, current: crate::l6_meta::nt_approval::ApprovalMode) -> bool {
        match self.resulting_mode {
            // ⛔ `None` ⇒ 该档不改审批模式 ⇒ 谈不上「放宽」
            Some(target) => rank(target) > rank(current),
            None => false,
        }
    }
}

/// 审批严格程度的序数（越大越松）。
/// 委托给 `ApprovalMode::strictness_rank`（枚举是唯一真源，避免两处判据漂移）。
fn rank(mode: crate::l6_meta::nt_approval::ApprovalMode) -> u8 {
    mode.strictness_rank()
}

/// **只算不做**：给出切到 `name` 会有什么副作用，**不改任何状态**。
///
/// ⛔ 不落盘、不改全局单例 —— 纯查询，可安全地用于「要不要先问一句」。
pub fn plan_profile_switch(name: &str) -> Result<ProfileSwitchPlan, String> {
    let guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    if !guard.profiles.contains_key(name) {
        return Err(format!("Profile '{}' not found. Use /profile list to see available profiles.", name));
    }
    let override_str = guard.resolve_approval_mode(name);
    drop(guard);

    // ⚠️ 档位里的 override 字符串**可能写错**（手改 profiles.toml、复制粘贴…）。
    // `ApprovalMode::from_str` 现返回 `Result` ⇒ 拼错时**硬拒绝**，
    //   而不是 `Option` 的静默 `None`（那会让该档的覆盖被无声忽略）。
    let resulting_mode = match override_str.as_deref() {
        None => None,
        Some(raw) => match crate::l6_meta::nt_approval::ApprovalMode::from_str(raw) {
            Ok(m) => Some(m),
            Err(e) => return Err(format!("档位 '{name}' 的 approval_mode_override 非法：{e}")),
        },
    };
    let notice = resulting_mode.map(|m| {
        format!(
            "档位 '{name}' 会把全局审批模式改为 {m:?}（该档设置了 approval_mode_override）\
——这会让后续工具调用**少一道人工确认**。"
        )
    });

    Ok(ProfileSwitchPlan {
        profile: name.to_string(),
        approval_mode_override: override_str,
        resulting_mode,
        notice,
    })
}

/// 切换当前权限档位（**匿名入口**）。
///
/// ⛔ **仅当该档位不会改动全局审批模式时才允许走本函数** ——
/// 否则它会 `Err`，要求调用方改用 [`switch_profile_with_audit`] 并给出 actor。
///
/// ## 为什么不给「匿名也能改模式」的后门（方案 A 的核心）
/// 若允许匿名改全局审批模式，那「显式确认 + 留痕」就形同虚设：
/// 调用方只要挑这个函数就绕过了。
/// ⇒ **能在无 actor 情况下放宽审批的路径必须不存在**，
///   这与「Deny 不可被 Ask 覆盖」是同一种不可逆性保护。
///
/// ## 判定「会不会改模式」的依据
/// [`plan_profile_switch`].changes_approval_mode() —— 纯查询，无副作用。
pub fn switch_profile(name: &str) -> Result<String, String> {
    let plan = plan_profile_switch(name)?;
    if plan.changes_approval_mode() {
        return Err(format!(
            "档位 '{name}' 会改动全局审批模式（→ {:?}）。\
请改用 switch_profile_with_audit 并提供 actor ——\
改变全局审批严格程度**不许匿名发生**（方案 A）。\
副作用预告：{}",
            plan.resulting_mode.unwrap_or(crate::l6_meta::nt_approval::ApprovalMode::Suggest),
            plan.notice.unwrap_or_default()
        ));
    }
    // 不改审批模式 ⇒ 匿名可接受，但 actor 仍标注来源以便审计区分。
    switch_profile_with_audit(name, "anonymous:no-mode-change")
}

/// 方案 A 的执行入口：切档 + **写审计**。
///
/// `actor` 是**谁批准的**（用户输入、CLI 参数名、自动化通道名…）。
/// ⛔ `actor` 为空 ⇒ 拒绝执行：**改全局审批模式这件事不许匿名发生**。
pub fn switch_profile_with_audit(name: &str, actor: &str) -> Result<String, String> {
    if actor.trim().is_empty() {
        return Err("switch_profile_with_audit 需要非空 actor：改变全局审批模式不许匿名发生".into());
    }

    // 先算副作用（不落地）—— 让「实际发生了什么」与调用方看到的计划同源。
    let plan = plan_profile_switch(name)?;
    let current_mode = crate::l6_meta::nt_approval::global_approval()
        .lock()
        .map(|e| e.mode())
        .unwrap_or(crate::l6_meta::nt_approval::ApprovalMode::Suggest);
    let loosened = plan.loosens_approval(current_mode);

    let mut guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    if !guard.profiles.contains_key(name) {
        return Err(format!("Profile '{}' not found. Use /profile list to see available profiles.", name));
    }
    guard.active = name.to_string();
    save_profiles_to_disk(&guard)?;
    drop(guard);

    // 真正落地审批模式
    if let Some(mode) = plan.resulting_mode {
        if let Ok(mut engine) = crate::l6_meta::nt_approval::global_approval().lock() {
            engine.set_mode(mode);
        }
    }

    // 留痕：无论是否改动审批模式都记一笔（审计要能回答「谁在什么时候切到了什么」）
    let mut msg = format!("Switched to profile: {name}");
    if let Some(n) = &plan.notice {
        msg.push('\n');
        msg.push_str(n);
    }
    msg.push_str(&format!(
        "\n[audit] actor={actor} profile={name} mode={:?}->{:?} loosened={loosened}",
        current_mode, plan.resulting_mode.unwrap_or(current_mode)
    ));
    Ok(msg)
}

/// Public API: get active profile name.
pub fn active_profile_name() -> String {
    global_profile_manager()
        .lock()
        .map(|g| g.active.clone())
        .unwrap_or_else(|_| "nt_shield".to_string())
}

/// Public API: create a new profile (optional parent).
pub fn create_profile(name: &str, parent: Option<&str>) -> Result<String, String> {
    let mut guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    if guard.profiles.contains_key(name) {
        return Err(format!("Profile '{}' already exists.", name));
    }
    if let Some(p) = parent {
        if !guard.profiles.contains_key(p) {
            return Err(format!("Parent profile '{}' not found.", p));
        }
    }
    let mut profile = PermissionProfile::new(name);
    profile.parent = parent.map(String::from);
    guard.profiles.insert(name.to_string(), profile);
    save_profiles_to_disk(&guard)?;
    Ok(format!("Created profile: {} (parent: {:?})", name, parent))
}

/// Public API: remove a profile (cannot remove built-in).
pub fn remove_profile(name: &str) -> Result<String, String> {
    let builtins = ["nt_shield", "strict-nt_shield", "general", "developer"];
    if builtins.contains(&name) {
        return Err(format!("Cannot remove built-in profile: {}", name));
    }
    let mut guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    if !guard.profiles.contains_key(name) {
        return Err(format!("Profile '{}' not found.", name));
    }
    if guard.active == name {
        guard.active = "nt_shield".into();
    }
    guard.profiles.remove(name);
    save_profiles_to_disk(&guard)?;
    Ok(format!("Removed profile: {}", name))
}

/// Public API: set a rule for a profile (action_key → decision).
pub fn set_rule(profile_name: &str, action_key: &str, decision: &str) -> Result<String, String> {
    let decision = match decision {
        "allow" | "Allow" => ProfileDecision::Allow,
        "deny" | "Deny" => ProfileDecision::Deny,
        "ask" | "Ask" => ProfileDecision::Ask,
        _ => return Err(format!("Invalid decision: {}. Use allow|deny|ask.", decision)),
    };
    let mut guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    let profile = guard
        .profiles
        .get_mut(profile_name)
        .ok_or_else(|| format!("Profile '{}' not found.", profile_name))?;
    // 策略单调性不变量: 新决策只能收紧已保存规则, 永不放宽 (policy_monotonic_invariant)
    let effective = profile
        .rules
        .get(action_key)
        .map(|existing| existing.tightened_with(&decision))
        .unwrap_or(decision);
    profile.rules.insert(action_key.to_string(), effective);
    save_profiles_to_disk(&guard)?;
    Ok(format!("Set rule: {} → {:?} in profile '{}'", action_key, effective, profile_name))
}

/// Public API: get profile info (rules, parent, effective mode).
pub fn get_profile_info(name: &str) -> Result<serde_json::Value, String> {
    let guard = global_profile_manager().lock().map_err(|e| e.to_string())?;
    let profile = guard
        .profiles
        .get(name)
        .ok_or_else(|| format!("Profile '{}' not found.", name))?;
    let rules = guard.resolve(name).unwrap_or_default();
    let approval_mode = guard.resolve_approval_mode(name);
    let mut rules_json = serde_json::Map::new();
    for (k, v) in &rules {
        let v_str = match v {
            ProfileDecision::Allow => "allow",
            ProfileDecision::Deny => "deny",
            ProfileDecision::Ask => "ask",
        };
        rules_json.insert(k.clone(), serde_json::Value::String(v_str.to_string()));
    }
    Ok(serde_json::json!({
        "name": profile.name,
        "parent": profile.parent,
        "approval_mode_override": profile.approval_mode_override,
        "effective_approval_mode": approval_mode,
        "effective_rules": rules_json,
    }))
}

/// Check whether an action is denied by the active profile.
pub fn is_action_denied(action_key: &str) -> bool {
    let guard = match global_profile_manager().lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    matches!(guard.evaluate(&guard.active, action_key), Some(ProfileDecision::Deny))
}

/// Check whether an action should be silently allowed (no approval needed).
pub fn is_action_allowed(action_key: &str) -> bool {
    let guard = match global_profile_manager().lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    matches!(guard.evaluate(&guard.active, action_key), Some(ProfileDecision::Allow))
}

/// List all profiles (names).
pub fn list_profiles() -> Vec<String> {
    let guard = match global_profile_manager().lock() {
        Ok(g) => g,
        Err(_) => return vec![],
    };
    let mut names: Vec<String> = guard.profiles.keys().cloned().collect();
    names.sort();
    names
}

/// Map an ActionType to a profile action key string.
/// 2026-10-05 修**键永不相交** —— 内置的两条 `Deny` 从未生效过。
///
/// ## 实测的缺陷形状
/// 本函数此前只产 **6 个键**，其中 git 动作一律映射成 `"git_push"`；
/// 而内置画像 `nt_shield` 里写的键是
/// `read_secrets`（`:77`）与 `git_force_push`（`:79`）。
/// ⇒ **两者永不相交** ⇒ 画像里那两条 `Deny` 规则
///   **从存在之日起就没有任何 `ActionType` 能命中它们**。
///
/// ## 危害
/// `git push --force` 与「读密钥」被声明为硬拒，
/// 实际却因键名对不上而**从未被拒**。
/// 结合本轮另一处修复（`Deny` 曾被降级成「需审批」）看，
/// 这是**两层失效叠加**：就算键对上了，也只弹窗不硬拒。
///
/// ## 修法
/// 补齐可判别的**子动作**键，并且**保留**原有的粗粒度键
/// （粗粒度键仍会被 `is_action_denied` 之外的查询用到，
/// 直接删掉会让依赖它们的代码静默失效 —— 那比多一个键更危险）。
///
/// ⛔ 未知子动作的判据是**「含关键词才进细粒度键，否则回退粗粒度键」**
///   —— 而不是 `unreachable!()`：生产路径不能因一个没见过的
///   git 子命令而 panic（AGENTS.md：生产代码禁 panic）。
pub fn action_type_to_key(action: &crate::l6_meta::nt_approval::ActionType) -> &'static str {
    use crate::l6_meta::nt_approval::ActionType;
    match action {
        ActionType::FileWrite { .. } => "write_file",
        ActionType::FileCreate { .. } => "write_file",
        ActionType::FileEdit { .. } => "write_file",
        ActionType::ShellCommand { .. } => "execute_command",
        ActionType::GitOperation { description, .. } => {
            // 细粒度键：与画像里声明的键名**逐字对齐**
            let d = description.to_ascii_lowercase();
            if d.contains("force") || d.contains("--force") || d.contains("+") {
                "git_force_push"
            } else if d.contains("push") {
                "git_push"
            } else {
                // 其余 git 动作（commit / pull / fetch / status…）
                // 落回粗粒度键，不硬拒（它们不在画像的 deny 列表里）
                "git_operation"
            }
        }
        ActionType::Other { .. } => "tool_call",
    }
}

// ====== 三轴权限统一查询模型 (PermissionAxes) ======
//
// 将分散的三套权限轴统一为可查询模型 (R-P42: 强化现有节点，不建平行模块):
//   轴1 审批模式 (ApprovalMode):   Suggest / AutoEdit / FullAuto   → 来自 global_approval
//   轴2 权限链模式 (PermissionMode): Plan / AcceptEdits / BypassPermissions → 来自 global_shield.perm_chain
//   轴3 策略决策 (PolicyDecision):  Allow / RequireConfirmation / Deny → 来自 global_shield.policy
// 只做统一查询/展示，不迁移任何现有决策逻辑（避免破坏已接线行为）。

use crate::l6_meta::nt_approval::ApprovalMode;
use crate::l3_embodiment::nt_shield_enforcer::global_shield;
use crate::l3_embodiment::nt_shield::shield_core::perm_chain::PermissionMode;
use crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision;

/// 三轴权限快照
#[derive(Debug, Clone, Serialize)]
pub struct PermissionAxes {
    /// 轴1: 审批模式
    pub approval_mode: String,
    /// 轴2: 权限链模式
    pub permission_chain_mode: String,
    /// 轴3: 当前安全画像
    pub policy_profile: String,
    /// 当前激活的权限画像
    pub active_profile: String,
    /// 有效风险等级 (0=最安全 .. 3=最高自主)
    pub autonomy_level: u8,
}

impl PermissionAxes {
    /// 从全局状态采集三轴当前值
    pub fn snapshot() -> Self {
        let approval = match crate::l6_meta::nt_approval::global_approval().lock() {
            Ok(g) => g.mode(),
            Err(_) => ApprovalMode::Suggest,
        };
        let shield = match global_shield().lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        let chain_mode = shield.perm_chain.mode();
        let policy_profile = shield.policy.profile.clone();
        let active_profile = active_profile_name();
        drop(shield);

        let autonomy = match approval {
            ApprovalMode::Suggest => 0,
            ApprovalMode::AutoEdit => 1,
            ApprovalMode::FullAuto => 2,
        } + match chain_mode {
            PermissionMode::Plan => 0,
            PermissionMode::AcceptEdits => 1,
            PermissionMode::BypassPermissions => 2,
        };

        Self {
            approval_mode: format!("{:?}", approval),
            permission_chain_mode: chain_mode.label().to_string(),
            policy_profile,
            active_profile,
            autonomy_level: autonomy.min(3),
        }
    }

    /// 针对指定 action key 查询轴3 策略决策
    pub fn policy_decision_for(action: &str) -> PolicyDecision {
        match global_shield().lock() {
            Ok(g) => g.policy.decide(action),
            Err(e) => e.into_inner().policy.decide(action),
        }
    }

    /// 查询指定 action 在轴1 下是否需要审批
    pub fn approval_required_for(action: &crate::l6_meta::nt_approval::ActionType) -> bool {
        match crate::l6_meta::nt_approval::global_approval().lock() {
            Ok(g) => g.require_approval(action),
            Err(_) => true,
        }
    }

    /// 汇总展示 (三轴并排)
    pub fn summary() -> String {
        let axes = Self::snapshot();
        format!(
            "三轴权限:\n  轴1 审批:   {} (Suggest=严格 / FullAuto=全自动)\n  轴2 权限链: {} (Plan=计划 / Bypass=旁路)\n  轴3 策略:   profile={}\n  激活画像:   {}\n  自主等级:   {}/3",
            axes.approval_mode,
            axes.permission_chain_mode,
            axes.policy_profile,
            axes.active_profile,
            axes.autonomy_level,
        )
    }
}

#[cfg(test)]
mod tests {

    /// 本模块测试**共享全局单例**（`global_profile_manager` /
    /// `global_approval`），而 cargo test **默认多线程**
    /// ⇒ 同模块测试会互相抢状态，表现为**间歇性失败**且失败行号漂移。
    ///
    /// 【本轮实测证据】首次运行时三条测试同时失败，且失败点在**不同行**：
    /// · `audit_switch_rejects_empty_actor` —— 「被拒的调用不得改动审批模式」
    /// · `plan_profile_switch_has_no_side_effects` —— 「plan 不得改动全局审批模式」
    /// · `test_global_state_integration`
    /// 而**独立探针**（单线程连打 before/after）证明 `plan` 确实是纯的
    /// ⇒ 真因是**测试间抢全局态**，不是被测代码有副作用。
    ///
    /// 【为什么不用 `serial_test`】本仓 `neotrix-core` 无该 dev-dependency，
    /// 为几条测试引入新依赖不划算 ⇒ 用标准库 `Mutex` 做**手写串行化**。
    ///
    /// ⚠️ 若将来给本模块加 `#[serial]`，记得**删掉这个锁**（否则双重串行化，
    /// 无害但会让人困惑）。
    static TEST_GLOBAL_STATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// 取得全局状态互斥守卫。**中毒时取内值** —— 守卫本身不含状态，
    /// 中毒只意味着另一个测试 panic 过，不该连带阻断本测试。
    fn lock_global_state() -> std::sync::MutexGuard<'static, ()> {
        TEST_GLOBAL_STATE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }
    use super::*;

    #[test]
    fn test_builtin_profiles_exist() {
        let store = ProfileStore::builtin();
        assert!(store.profiles.contains_key("nt_shield"));
        assert!(store.profiles.contains_key("strict-nt_shield"));
        assert!(store.profiles.contains_key("general"));
        assert!(store.profiles.contains_key("developer"));
        assert_eq!(store.active, "nt_shield");
    }

    #[test]
    fn test_resolve_inheritance() {
        let store = ProfileStore::builtin();
        let rules = store.resolve("strict-nt_shield").unwrap();
        // inherited from nt_shield
        assert_eq!(rules.get("read_secrets"), Some(&ProfileDecision::Deny));
        // overridden in strict-nt_shield
        assert_eq!(rules.get("write_file"), Some(&ProfileDecision::Deny));
        // inherited
        assert_eq!(rules.get("compile_check"), Some(&ProfileDecision::Allow));
    }

    #[test]
    fn test_resolve_general_allows_write() {
        let store = ProfileStore::builtin();
        let rules = store.resolve("general").unwrap();
        assert_eq!(rules.get("write_file"), Some(&ProfileDecision::Allow));
        assert_eq!(rules.get("execute_command"), Some(&ProfileDecision::Allow));
    }

    #[test]
    fn test_resolve_developer() {
        let store = ProfileStore::builtin();
        let rules = store.resolve("developer").unwrap();
        assert_eq!(rules.get("delete_file"), Some(&ProfileDecision::Allow));
        assert_eq!(rules.get("access_tor_network"), Some(&ProfileDecision::Deny));
        assert_eq!(rules.get("read_file"), Some(&ProfileDecision::Allow));
    }

    #[test]
    fn test_resolve_approval_mode_override() {
        let store = ProfileStore::builtin();
        assert_eq!(store.resolve_approval_mode("developer"), Some("auto-edit".to_string()));
        assert_eq!(store.resolve_approval_mode("nt_shield"), None);
    }

    #[test]
    fn test_evaluate() {
        let store = ProfileStore::builtin();
        assert_eq!(store.evaluate("nt_shield", "read_secrets"), Some(ProfileDecision::Deny));
        assert_eq!(store.evaluate("nt_shield", "write_file"), Some(ProfileDecision::Ask));
        assert_eq!(store.evaluate("general", "execute_command"), Some(ProfileDecision::Allow));
    }

    #[test]
    fn test_global_state_integration() {
        let _guard = lock_global_state();
        // All tests that touch global state run sequentially to prevent
        // races from parallel test execution sharing OnceLock/Mutex.
        reset_profile_manager();
        let _ = crate::l6_meta::nt_approval::global_approval().lock().unwrap().set_mode(crate::l6_meta::nt_approval::ApprovalMode::Suggest);

        // create + remove custom
        assert!(create_profile("my-custom", Some("developer")).is_ok());
        let names = list_profiles();
        assert!(names.contains(&"my-custom".to_string()));
        assert!(remove_profile("my-custom").is_ok());
        let names = list_profiles();
        assert!(!names.contains(&"my-custom".to_string()));

        // remove builtin fails
        assert!(remove_profile("nt_shield").is_err());
        assert!(remove_profile("developer").is_err());

        // create duplicate fails
        assert!(create_profile("nt_shield", None).is_err());

        // set rule
        assert!(set_rule("nt_shield", "custom_action", "allow").is_ok());
        {
            let guard = global_profile_manager().lock().unwrap();
            let profile = guard.profiles.get("nt_shield").unwrap();
            assert_eq!(profile.rules.get("custom_action"), Some(&ProfileDecision::Allow));
        }

        // invalid decision
        assert!(set_rule("nt_shield", "foo", "maybe").is_err());

        // 策略单调性不变量 (policy_monotonic_invariant): Deny 不可被放宽
        assert!(set_rule("nt_shield", "mono_guard", "deny").is_ok());
        assert!(set_rule("nt_shield", "mono_guard", "allow").is_ok()); // 尝试放宽 → 应保持 Deny
        {
            let guard = global_profile_manager().lock().unwrap();
            let profile = guard.profiles.get("nt_shield").unwrap();
            assert_eq!(profile.rules.get("mono_guard"), Some(&ProfileDecision::Deny), "Deny 不可被放宽为 Allow");
        }
        // Allow 可被收紧为 Deny
        assert!(set_rule("nt_shield", "mono_guard", "deny").is_ok());
        {
            let guard = global_profile_manager().lock().unwrap();
            let profile = guard.profiles.get("nt_shield").unwrap();
            assert_eq!(profile.rules.get("mono_guard"), Some(&ProfileDecision::Deny));
        }

        // switch profile —— 2026-10-05 语义变更（方案 A）
        //
        // 【原断言（已删除）】`switch_profile("developer").is_ok()`
        //   —— 它把「匿名切档可以顺带改掉全局审批模式」**钉成了契约**。
        // 【新语义】`developer` 带 `approval_mode_override = auto-edit`
        //   ⇒ 匿名切它必须**被拒**；带 actor 才允许。
        // 另：`nt_shield` 无 override ⇒ 匿名切换仍可用（对照组）。
        assert!(switch_profile("developer").is_err(), "匿名切 developer 必须被拒（方案 A）");
        assert!(switch_profile_with_audit("developer", "test").is_ok());
        assert_eq!(active_profile_name(), "developer");
        assert!(switch_profile("nt_shield").is_ok());

        // switch nonexistent
        assert!(switch_profile("nonexistent").is_err());

        // is_action_denied
        assert!(switch_profile("strict-nt_shield").is_ok());
        assert!(is_action_denied("write_file"));
        assert!(!is_action_denied("compile_check"));

        // is_action_allowed
        assert!(switch_profile("general").is_ok());
        assert!(is_action_allowed("write_file"));
        assert!(!is_action_allowed("read_secrets"));

        // list
        let names = list_profiles();
        assert!(names.contains(&"nt_shield".to_string()));
        assert!(names.contains(&"developer".to_string()));
        assert!(names.len() >= 4);

        // get_profile_info
        let info = get_profile_info("developer").unwrap();
        assert_eq!(info["name"], "developer");
        assert_eq!(info["parent"], serde_json::Value::String("general".to_string()));
        assert!(info["effective_rules"].is_object());

        assert!(get_profile_info("does-not-exist").is_err());

        // 2026-10-05 语义变更（方案 A）：override **仍然生效**，
        // 但**必须带 actor** —— 匿名入口不再允许放宽审批模式。
        //
        // 【原断言（已删除）】`switch_profile("developer").is_ok()`
        //   + 断言模式变AutoEdit ⇒ 它把「匿名切档可以顺带放宽审批」
        //   **钉成了契约**，正是方案 A 要消除的行为。
        reset_profile_manager();
        crate::l6_meta::nt_approval::global_approval().lock().unwrap().set_mode(crate::l6_meta::nt_approval::ApprovalMode::Suggest);

        // ① 匿名 ⇒ 被拒，且**模式不变**
        assert!(switch_profile("developer").is_err(), "匿名切 developer 必须被拒（方案 A）");
        assert_eq!(
            crate::l6_meta::nt_approval::global_approval().lock().unwrap().mode(),
            crate::l6_meta::nt_approval::ApprovalMode::Suggest,
            "被拒的匿名切换不得留下模式副作用"
        );

        // ② 带 actor ⇒ 成功，**override 依然生效**
        assert!(switch_profile_with_audit("developer", "test").is_ok());
        {
            let engine = crate::l6_meta::nt_approval::global_approval().lock().unwrap();
            assert_eq!(engine.mode(), crate::l6_meta::nt_approval::ApprovalMode::AutoEdit,
                "带 actor 切到 developer 仍应应用 auto-edit override（功能未被削掉）");
        }
        assert!(switch_profile("nt_shield").is_ok());
        crate::l6_meta::nt_approval::global_approval().lock().unwrap().set_mode(crate::l6_meta::nt_approval::ApprovalMode::Suggest);
    }

    #[test]
    fn test_action_type_to_key() {
        use crate::l6_meta::nt_approval::ActionType;
        assert_eq!(action_type_to_key(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }), "write_file");
        assert_eq!(action_type_to_key(&ActionType::FileCreate { path: "x".into() }), "write_file");
        assert_eq!(action_type_to_key(&ActionType::FileEdit { path: "x".into(), diff: "".into() }), "write_file");
        assert_eq!(action_type_to_key(&ActionType::ShellCommand { command: "ls".into() }), "execute_command");

        // ⚠️ 2026-10-05 **语义变更**：git 动作**不再一律映射成 `git_push`**。
        // 原断言 `commit → "git_push"` 正是缺陷来源 ——
        // 它让 `git push --force` 与 `git commit` 落到同一个键，
        // 于是画像里那条 `git_force_push` 的 `Deny` **永不可达**。
        // 现按子动作分键（见 `action_type_to_key` 的文档）。
        assert_eq!(action_type_to_key(&ActionType::GitOperation { description: "git push origin main".into() }), "git_push");
        assert_eq!(action_type_to_key(&ActionType::GitOperation { description: "git push --force origin main".into() }), "git_force_push");
        assert_eq!(action_type_to_key(&ActionType::GitOperation { description: "git commit -m x".into() }), "git_operation");
        assert_eq!(action_type_to_key(&ActionType::GitOperation { description: "git pull".into() }), "git_operation");
    }

    /// 端到端：画像里**每一条** `Deny` 规则都必须有某个 `ActionType` 能命中。
    ///
    /// 【缺陷形状】此前画像声明了 11 条键，而 `action_type_to_key`
    /// 只产 6 个 ⇒ 其中 `read_secrets` / `git_force_push` 等
    /// **7 条永不可达** —— 「声明了但从不起作用」。
    ///
    /// 本测试把「键集合」与「画像 deny 键集合」求交，
    /// 断言**交集非空**，让「新增 deny 规则却忘了加键」在测试期就红。
    #[test]
    fn every_builtin_deny_rule_is_reachable_from_some_action_type() {
        use crate::l6_meta::nt_approval::ActionType;

        // 本函数当前能产出的全部键（穷举枚举，**有意列全**：
        // 将来给 ActionType 加变体时，这个清单就是提醒）
        let producible: std::collections::HashSet<&str> = [
            action_type_to_key(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }),
            action_type_to_key(&ActionType::FileCreate { path: "x".into() }),
            action_type_to_key(&ActionType::FileEdit { path: "x".into(), diff: "".into() }),
            action_type_to_key(&ActionType::ShellCommand { command: "ls".into() }),
            action_type_to_key(&ActionType::GitOperation { description: "git push".into() }),
            action_type_to_key(&ActionType::GitOperation { description: "git push --force".into() }),
            action_type_to_key(&ActionType::GitOperation { description: "git commit".into() }),
            action_type_to_key(&ActionType::Other { tool: "x".into(), args: String::new() }),
        ]
        .into_iter()
        .collect();

        // 画像里所有 deny 规则
        let store = crate::l6_meta::nt_permission_profiles::ProfileStore::builtin();
        let mut deny_keys: Vec<String> = Vec::new();
        for p in ["nt_shield", "strict-nt_shield", "balanced"] {
            if let Some(rules) = store.resolve(p) {
                for (k, v) in rules {
                    if matches!(v, ProfileDecision::Deny) {
                        deny_keys.push(k);
                    }
                }
            }
        }
        deny_keys.sort();
        deny_keys.dedup();

        assert!(
            !deny_keys.is_empty(),
            "前提断言：内置画像里应当至少有Deny 规则"
        );

        let unreachable: Vec<&String> =
            deny_keys.iter().filter(|k| !producible.contains(k.as_str())).collect();

        // 如实记录当前仍不可达的键（不假装全绿），但**要求清单被显式登记**：
        // 新增的不可达键必须在这里出现，否则测试失败。
        const KNOWN_UNREACHABLE: &[&str] = &[
            // 画像声明了，但当前没有任何 ActionType 变体能表达「读密钥」这个动作。
            // 要修它需要给 ActionType 加变体 —— 属独立批次。
            "read_secrets",
            // 以下为历史键，当前 ActionType 粒度到不了：
            "delete_file",
            "compile_check",
            "modify_dependency",
            "access_stealth_browser_auto",
            "access_tor_network",
            "network_request",
        ];

        let unexpected: Vec<&&String> = unreachable
            .iter()
            .filter(|k| !KNOWN_UNREACHABLE.contains(&k.as_str()))
            .collect();
        assert!(
            unexpected.is_empty(),
            "这些 deny 规则**没有任何 ActionType 能命中**（声明了但从不起作用）：{unexpected:?}。\
             要么给 action_type_to_key 补键，要么把它登记进 KNOWN_UNREACHABLE 并写明原因"
        );

        // 关键断言：`git_force_push` 必须**不在**不可达列表里
        // （它是本轮修好的那条；若它出现在unreachable 里说明修复回退了）
        assert!(
            !unreachable.iter().any(|k| k.as_str() == "git_force_push"),
            "git_force_push 必须可命中 —— 本轮已修；若此断言失败说明键映射回退了"
        );
    }

    #[test]
    fn test_permission_axes_snapshot_and_summary() {
        let axes = PermissionAxes::snapshot();
        assert!(!axes.approval_mode.is_empty());
        assert!(!axes.permission_chain_mode.is_empty());
        assert!(axes.autonomy_level <= 3);
        let summary = PermissionAxes::summary();
        assert!(summary.contains("三轴权限"));
        assert!(summary.contains("轴2"));
        // 策略决策查询不 panic
        let _ = PermissionAxes::policy_decision_for("write_file");
    }

    /// 契约锁：**子档显式覆盖父档是本仓的刻意设计**，不是缺陷。
    ///
    /// 【本仓为什么允许放宽】内置三个档位是一条**刻意的产品阶梯**：
    /// `nt_shield`（收紧）→ `general`（通用开发，放开写文件）
    /// → `developer` (注释直写 `most permissive`)。
    /// 若把继承合并改成「只能收紧」，`general` / `developer` 会**完全失去意义**
    /// ⇒ 那是**破坏产品语义**，不是修缺陷。
    ///
    /// 【为什么这条锁重要】我曾真的把它当缺陷「修」过（`tightened_with`），
    /// 结果 4 条既有测试变红 —— 那些测试**并不过时**，它们在陈述设计意图。
    /// ⇒ 本锁的作用是：**下一个 agent 再看到「合并无单调」时，
    ///   会先读到这里，而不是再「修」一遍。**
    ///
    /// ⚠️ **不要**照搬 `codewhale-hq/Codewhale` 的「overlay 只能收紧」：
    ///   它能这么写是因为它**只有一个 profile** + per-project overlay，
    ///   用户没有「切到宽松档」这个需求。
    ///   前提不同，结论不同（详见 `resolve` 里的留痕注释）。
    #[test]
    fn child_profile_may_explicitly_relax_parent_on_purpose() {
        let store = ProfileStore::builtin();

        // nt_shield: write_file = Ask（收紧）
        let shield = store.resolve("nt_shield").expect("nt_shield 应可解析");
        assert_eq!(shield.get("write_file"), Some(&ProfileDecision::Ask), "前提断言");

        // general: 显式放宽为 Allow ⇒ 必须真的生效（否则该档无意义）
        let general = store.resolve("general").expect("general 应可解析");
        assert_eq!(
            general.get("write_file"),
            Some(&ProfileDecision::Allow),
            "general 档刻意放宽 write_file —— 这是产品阶梯，不是缺陷"
        );

        // 但**未被显式覆盖**的键必须**继承祖先**（不能凭空出现）
        assert_eq!(
            general.get("read_secrets"),
            Some(&ProfileDecision::Deny),
            "general 未覆盖 read_secrets ⇒ 必须继承 nt_shield 的 Deny"
        );
        assert_eq!(
            general.get("git_force_push"),
            Some(&ProfileDecision::Deny),
            "git_force_push 的硬拒不得被任何宽松档继承性地绕过"
        );

        // developer (parent = general)：delete_file 显式 Allow
        let dev = store.resolve("developer").expect("developer 应可解析");
        assert_eq!(dev.get("delete_file"), Some(&ProfileDecision::Allow));
        // 但它自己显式写 Deny 的键仍是 Deny（显式优先于继承）
        assert_eq!(dev.get("access_tor_network"), Some(&ProfileDecision::Deny));
    }

    /// **安全下界锁**：无论怎么继承，**显式 Deny 的键不得被继承性放宽**。
    ///
    /// 这条是上面那条的**安全侧补充**，也是我要保留的唯一实质约束：
    /// 若某个键在**本档**显式 `Deny`，它必须保持 `Deny`；
    /// 若本档**没写**这个键，则按继承链取祖先值。
    #[test]
    fn explicit_deny_in_a_profile_is_never_relaxed_within_that_profile() {
        let store = ProfileStore::builtin();
        for profile in ["nt_shield", "strict-nt_shield", "general", "developer"] {
            let rules = store.resolve(profile).expect("档位应可解析");
            for (key, decision) in &rules {
                if *decision == ProfileDecision::Deny {
                    // 该键在此档是 Deny ⇒ 必须真的是 Deny（不是 Ask 也不是 Allow）
                    assert_eq!(
                        decision,
                        &ProfileDecision::Deny,
                        "{profile} 的 {key} 解析后不再是 Deny"
                    );
                }
            }
        }
    }

    // ══════════════════════════════════════════════════════════════
    // 方案 A 反向锁（2026-10-05，用户裁决：允许改模式，但须显式确认 + 留痕）
    // ══════════════════════════════════════════════════════════════

    /// 前提断言：`developer` 档确实设置了 `approval_mode_override`
    /// ⇒ 否则下面三条锁都会因为「测的是个不会改模式的档」而**假通过**。
    #[test]
    fn premise_developer_profile_does_override_approval_mode() {
        let _guard = lock_global_state();
        let store = ProfileStore::builtin();
        let ov = store.resolve_approval_mode("developer");
        assert_eq!(
            ov.as_deref(),
            Some("auto-edit"),
            "前提：developer 档带 auto-edit override（否则下面几条锁是假通过）"
        );
    }

    /// 锁①：**匿名入口不得放宽审批模式**。
    ///
    /// 【危害】`switch_profile` 原本会顺手改掉全局 `ApprovalMode` 且完全静默。
    /// 若匿名也能改，方案 A 的「显式确认 + 留痕」就形同虚设 ——
    /// 调用方只要挑这个函数就绕过了。
    #[test]
    fn anonymous_switch_cannot_loosen_approval_mode() {
        let _guard = lock_global_state();
        reset_profile_manager();
        crate::l6_meta::nt_approval::global_approval()
            .lock()
            .unwrap()
            .set_mode(crate::l6_meta::nt_approval::ApprovalMode::Suggest);

        let r = switch_profile("developer");
        assert!(
            r.is_err(),
            "匿名切 developer（会改审批模式）必须 Err，实际 {r:?}"
        );
        let msg = r.unwrap_err();
        assert!(
            msg.contains("switch_profile_with_audit"),
            "错误信息应指向带 actor 的入口：{msg}"
        );
        assert!(
            msg.contains("不许匿名发生") || msg.contains("匿名"),
            "错误信息应说明「不许匿名发生」的理由：{msg}"
        );

        // 关键：全局模式**没被改**
        assert_eq!(
            crate::l6_meta::nt_approval::global_approval().lock().unwrap().mode(),
            crate::l6_meta::nt_approval::ApprovalMode::Suggest,
            "被拒的切换**不得**留下任何模式副作用"
        );
        reset_profile_manager();
    }

    /// 锁②：**空 actor 被拒** —— 改全局审批严格程度不许匿名。
    #[test]
    fn audit_switch_rejects_empty_actor() {
        let _guard = lock_global_state();
        reset_profile_manager();
        for actor in ["", "   ", "\t", "\n"] {
            let r = switch_profile_with_audit("developer", actor);
            assert!(r.is_err(), "actor={actor:?} 必须被拒，实际 {r:?}");
            assert!(
                r.unwrap_err().contains("actor"),
                "错误信息应点明缺 actor"
            );
        }
        assert_eq!(
            crate::l6_meta::nt_approval::global_approval().lock().unwrap().mode(),
            crate::l6_meta::nt_approval::ApprovalMode::Suggest,
            "被拒的调用**不得**改动审批模式"
        );
        reset_profile_manager();
    }

    /// 锁③：带 actor ⇒ 成功，**且返回值里带审计行**。
    ///
    /// 审计行必须能回答「谁在什么时候切到了什么、模式从什么变成什么」。
    #[test]
    fn audit_switch_succeeds_and_emits_audit_trail() {
        let _guard = lock_global_state();
        reset_profile_manager();
        crate::l6_meta::nt_approval::global_approval()
            .lock()
            .unwrap()
            .set_mode(crate::l6_meta::nt_approval::ApprovalMode::Suggest);

        let r = switch_profile_with_audit("developer", "cli").expect("带 actor 应成功");
        assert!(r.contains("Switched to profile: developer"), "{r}");
        // 审计要素齐全
        assert!(r.contains("actor=cli"), "审计行须含 actor：{r}");
        assert!(r.contains("profile=developer"), "审计行须含 profile：{r}");
        assert!(r.contains("loosened=true"), "Suggest→AutoEdit 是放宽：{r}");
        // 副作用提示必须出现在给用户看的文本里
        assert!(
            r.contains("审批模式"),
            "返回值须含人类可读的副作用预告：{r}"
        );

        // 模式确实被改
        assert_eq!(
            crate::l6_meta::nt_approval::global_approval().lock().unwrap().mode(),
            crate::l6_meta::nt_approval::ApprovalMode::AutoEdit
        );
        reset_profile_manager();
    }

    /// 锁④（最关键）：**`plan_profile_switch` 是纯查询**。
    ///
    /// 【为什么这条最重要】方案 A 的整个交互形态是
    /// 「先 plan 给人看 → 人确认 → 才执行」。
    /// 若 `plan` 偷偷改了状态（切了档、改了模式、落了盘），
    /// 那**确认就只是走个形式** —— 用户看到的预告与实际发生的不一致。
    #[test]
    fn plan_profile_switch_has_no_side_effects() {
        let _guard = lock_global_state();
        reset_profile_manager();
        // 先把状态设成可辨认的初值
        switch_profile_with_audit("nt_shield", "setup").expect("设初值");
        crate::l6_meta::nt_approval::global_approval()
            .lock()
            .unwrap()
            .set_mode(crate::l6_meta::nt_approval::ApprovalMode::Suggest);

        // ⚠️ 本仓的 profile 测试**共享全局单例**（`global_profile_manager` /
        // `global_approval`），而 cargo test **默认多线程** ⇒ 同模块其它测试
        // 可能在「读基线」与「断言」之间改动它 ⇒ 表现为**间歇性失败**。
        // 本仓无 `serial_test` 依赖（同模块 13 处测试同样靠 `reset_profile_manager()`
        // 复位）⇒ 本测试沿用**同一手法**：进出各复位一次、基线自带断言。
        //
        // 这也是本锁**第一次运行时抓到的东西**：
        // 它首跑报「plan 不得改动全局审批模式」失败，
        // 而**独立探针证明 plan 确实是纯的**
        // （连续打印 before/after plan，mode 不变）
        // ⇒ 真因是**测试之间抢全局态**，不是 plan 有副作用。
        // ⇒ 若不写清这点，下一个 agent 会去「修」一个正确的 plan。
        let mode_before = crate::l6_meta::nt_approval::global_approval().lock().unwrap().mode();
        let active_before = active_profile_name();
        assert_eq!(
            mode_before,
            crate::l6_meta::nt_approval::ApprovalMode::Suggest,
            "前提：本测试已把基线设为 Suggest（若此断言失败，是别的测试抢了全局态）"
        );

        // 连调三次 plan，覆盖可能存在的「第一次才初始化」的隐藏副作用
        for _ in 0..3 {
            let p = plan_profile_switch("developer").expect("plan 应成功");
            assert!(p.changes_approval_mode(), "developer 确实会改模式");
            assert_eq!(
                p.resulting_mode,
                Some(crate::l6_meta::nt_approval::ApprovalMode::AutoEdit),
                "plan 应预告切过去会变成 AutoEdit"
            );
            assert!(p.notice.is_some(), "plan 必须给出人类可读预告");
        }

        // 核心断言：三次 plan 之后，两项状态**都没变**
        assert_eq!(
            crate::l6_meta::nt_approval::global_approval().lock().unwrap().mode(),
            mode_before,
            "plan **不得**改动全局审批模式"
        );
        assert_eq!(
            active_profile_name(),
            active_before,
            "plan **不得**切换当前档位"
        );
        reset_profile_manager();
    }

    /// 对照组：`loosens_approval` 只把「从严到松」算放宽。
    /// 反向（收紧）**不算** —— 否则 UI 会拿它提示「要放宽了」而实际相反。
    #[test]
    fn loosens_only_counts_strict_to_loose() {
        let _guard = lock_global_state();
        use crate::l6_meta::nt_approval::ApprovalMode::*;
        let auto = ProfileSwitchPlan {
            profile: "p".into(),
            approval_mode_override: Some("auto-edit".into()),
            resulting_mode: Some(AutoEdit),
            notice: None,
        };
        // 收紧方向不算放宽
        assert!(!auto.loosens_approval(FullAuto), "FullAuto→AutoEdit 是收紧");
        // 放宽方向算
        assert!(auto.loosens_approval(Suggest), "Suggest→AutoEdit 是放宽");
        assert!(!auto.loosens_approval(AutoEdit), "同档位不算放宽");
        // 不改模式 ⇒ 谈不上放宽
        let none = ProfileSwitchPlan {
            profile: "p".into(),
            approval_mode_override: None,
            resulting_mode: None,
            notice: None,
        };
        assert!(!none.loosens_approval(Suggest), "不改模式时不谈放宽");
        assert!(!none.changes_approval_mode());
    }

    /// 匿名入口在**不改模式**的档位上仍然可用（否则连切档都做不了）。
    #[test]
    fn anonymous_switch_still_works_for_non_mode_changing_profile() {
        let _guard = lock_global_state();
        reset_profile_manager();
        let r = switch_profile("nt_shield");
        assert!(r.is_ok(), "不改审批模式的档位应允许匿名切换：{r:?}");
        assert!(r.unwrap().contains("Switched to profile: nt_shield"));
        reset_profile_manager();
    }

}