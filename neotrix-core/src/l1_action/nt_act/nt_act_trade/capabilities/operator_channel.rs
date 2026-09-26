//! Operator channel — 操作员切换与管理员视图（WSD001 实证 2026-09-23）.
//!
//! `POST /a/operator/switch {cid: 目标操作员id, lid: 当前操作员id}`：
//! - 管理员（WSD001, operator_id=136154）4/4 成功；业务员（WSD006）
//!   四组合全拒——切换是管理员专属能力。
//! - 切换写**服务端 session**，后续请求视图即变（同 `/b/emails`
//!   端点 total 从 0→2）；用完必须 `switch_back` 防污染。
//! - 管理员自己的邮箱可为 0（未配个人邮箱），不代表权限墙；
//!   判断权限看切换后他人 total>0。
//! - 操作员 id 提取：登录后 localStorage `user_info.user` JSON 中
//!   `userName` 匹配 + `id` 字段；兜底探 `/a/operator/login`。
//!
//! 与 `follow_logs` 的分工：`/d/customer/logs` 不吃 switch/X-User；
//! 本模块服务按操作员分邮箱/跟进 dashboard 的端点。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 切换端点（base = "/rapi/a"）
pub const SWITCH_PATH: &str = "/operator/switch";

/// 切换请求体（前端语义：cid=目标 data-id, lid=当前 user.id）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwitchRequest {
    pub cid: u64,
    pub lid: u64,
}

/// 构造切换请求。
pub fn switch_request(target_operator_id: u64, current_operator_id: u64) -> SwitchRequest {
    SwitchRequest {
        cid: target_operator_id,
        lid: current_operator_id,
    }
}

/// 管理员/当前登录档案（密码永不入库）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorProfile {
    pub username: String,
    pub operator_id: u64,
    /// 公司 cid（x-cid 头，WSD = 65491）
    pub cid: u64,
}

impl OperatorProfile {
    /// 自己 → 自己的切换请求（switch_back 用）
    pub fn identity_switch(&self) -> SwitchRequest {
        switch_request(self.operator_id, self.operator_id)
    }
}

/// 切换到目标操作员。
pub fn switch_to(profile: &OperatorProfile, target: u64) -> SwitchRequest {
    switch_request(target, profile.operator_id)
}

/// 从 localStorage dump 提取操作员档案。
///
/// 接受 `{"localStorage.user_info": "<json string>", ...}` 形状
/// （键含 `.user_info` 或值内嵌 `userName`）；在嵌套对象/列表里找
/// `userName == username`（大小写不敏感）的条目，取 `id`/`operatorId`。
/// 找不到返回 `None`（调用方走 managerUsers 兜底，不猜 id）。
pub fn parse_operator_profile(
    storage: &serde_json::Value,
    username: &str,
    default_cid: u64,
) -> Option<OperatorProfile> {
    let want = username.to_ascii_uppercase();
    let obj = storage.as_object()?;

    // 1) 直接命中的值是 JSON 字符串
    for (_key, val) in obj {
        if let Some(s) = val.as_str() {
            if !s.contains(&want) {
                continue;
            }
            if let Some(found) = find_user_in_json(s, &want) {
                return Some(OperatorProfile {
                    username: username.to_string(),
                    operator_id: found,
                    cid: default_cid,
                });
            }
        }
    }
    // 2) 值本身已是对象/数组
    for (_key, val) in obj {
        if val.is_string() {
            continue;
        }
        if let Some(found) = find_user_in_value(val, &want) {
            return Some(OperatorProfile {
                username: username.to_string(),
                operator_id: found,
                cid: default_cid,
            });
        }
    }
    None
}

fn find_user_in_json(s: &str, want_upper: &str) -> Option<u64> {
    let v: serde_json::Value = serde_json::from_str(s).ok()?;
    find_user_in_value(&v, want_upper)
}

fn find_user_in_value(v: &serde_json::Value, want_upper: &str) -> Option<u64> {
    match v {
        serde_json::Value::Object(map) => {
            let uname = map
                .get("userName")
                .or_else(|| map.get("username"))
                .and_then(|x| x.as_str())
                .map(|x| x.to_ascii_uppercase());
            if uname.as_deref() == Some(want_upper) {
                for k in ["id", "operatorId", "userId"] {
                    if let Some(n) = map.get(k).and_then(|x| x.as_u64()) {
                        return Some(n);
                    }
                }
            }
            // 常见包裹键
            for k in ["user", "operator", "info", "current", "loginUser", "data"] {
                if let Some(inner) = map.get(k) {
                    if let Some(found) = find_user_in_value(inner, want_upper) {
                        return Some(found);
                    }
                }
            }
            // 深一层扫（localStorage 可能是任意嵌套）
            for inner in map.values() {
                if inner.is_object() || inner.is_array() {
                    if let Some(found) = find_user_in_value(inner, want_upper) {
                        return Some(found);
                    }
                }
            }
            None
        }
        serde_json::Value::Array(arr) => arr.iter().find_map(|x| find_user_in_value(x, want_upper)),
        _ => None,
    }
}

/// 视图守卫：跟踪当前视图操作员，保证用完切回。
///
/// 状态机：`Self` → `Target(n)` → 必须回 `Self` 才允许下一次 switch_to。
/// 防的是「切到别人忘切回，后续抓取污染进他人视图」。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewGuard {
    home_operator_id: u64,
    current_operator_id: u64,
    switched: bool,
}

impl ViewGuard {
    pub fn new(home_operator_id: u64) -> Self {
        Self {
            home_operator_id,
            current_operator_id: home_operator_id,
            switched: false,
        }
    }

    pub fn home(&self) -> u64 {
        self.home_operator_id
    }

    pub fn current(&self) -> u64 {
        self.current_operator_id
    }

    pub fn is_switched(&self) -> bool {
        self.switched
    }

    /// 请求切到 target；若已在他人视图且 target≠home，先要求回巢
    /// （返回 Err 防嵌套切换）。成功后更新状态，返回应发送的请求体。
    pub fn switch_to(&mut self, target: u64) -> Result<SwitchRequest, String> {
        if target == self.current_operator_id {
            // 幂等：已在目标视图，无需请求（优先于嵌套守卫，
            // 否则重复切同一目标会被误判为嵌套切换）
            return Ok(switch_request(target, self.home_operator_id));
        }
        if self.switched && target != self.home_operator_id {
            return Err(format!(
                "view already switched to {}; switch_back to {} first",
                self.current_operator_id, self.home_operator_id
            ));
        }
        let req = switch_request(target, self.home_operator_id);
        self.current_operator_id = target;
        self.switched = target != self.home_operator_id;
        Ok(req)
    }

    /// 请求切回 home；已在 home 则幂等返回。
    pub fn switch_back(&mut self) -> SwitchRequest {
        let req = switch_request(self.home_operator_id, self.home_operator_id);
        self.current_operator_id = self.home_operator_id;
        self.switched = false;
        req
    }
}

/// 切换响应是否成功（`success=true`；业务员被拒时 false + errMsg）。
pub fn switch_succeeded(resp: &serde_json::Value) -> bool {
    resp.get("success").and_then(|v| v.as_bool()) == Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn switch_request_shape_matches_frontend() {
        let r = switch_request(136130, 136154);
        assert_eq!(r.cid, 136130);
        assert_eq!(r.lid, 136154);
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["cid"], 136130);
        assert_eq!(v["lid"], 136154);
    }

    #[test]
    fn identity_switch_uses_self_both_fields() {
        let p = OperatorProfile {
            username: "WSD001".into(),
            operator_id: 136154,
            cid: 65491,
        };
        let r = p.identity_switch();
        assert_eq!(r.cid, 136154);
        assert_eq!(r.lid, 136154);
        assert_eq!(switch_to(&p, 187820).lid, 136154);
    }

    #[test]
    fn parse_profile_from_user_info_string() {
        let storage = json!({
            "localStorage.user_info": "{\"user\":{\"id\":136154,\"userName\":\"WSD001\"}}"
        });
        let p = parse_operator_profile(&storage, "WSD001", 65491).expect("profile");
        assert_eq!(p.operator_id, 136154);
        assert_eq!(p.cid, 65491);
    }

    #[test]
    fn parse_profile_nested_and_case_insensitive() {
        let storage = json!({
            "session": {"user": [{"userName": "wsd001", "operatorId": 136154}]}
        });
        let p = parse_operator_profile(&storage, "WSD001", 65491).expect("profile");
        assert_eq!(p.operator_id, 136154);
        // 不匹配返回 None，不猜
        assert!(parse_operator_profile(&storage, "WSD999", 65491).is_none());
    }

    #[test]
    fn view_guard_round_trip_and_no_nested_switch() {
        let mut g = ViewGuard::new(136154);
        assert!(!g.is_switched());
        let r = g.switch_to(136130).unwrap();
        assert_eq!((r.cid, r.lid), (136130, 136154));
        assert!(g.is_switched());
        // 未回巢禁止再切第三人
        assert!(g.switch_to(189651).is_err());
        // 回巢
        let back = g.switch_back();
        assert_eq!((back.cid, back.lid), (136154, 136154));
        assert!(!g.is_switched());
        assert_eq!(g.current(), 136154);
        // 回巢后再切允许
        assert!(g.switch_to(189651).is_ok());
        // 幂等：重复切同一目标
        assert!(g.switch_to(189651).is_ok());
    }

    #[test]
    fn switch_succeeded_reads_success_flag() {
        assert!(switch_succeeded(&json!({"success": true})));
        assert!(!switch_succeeded(
            &json!({"success": false, "errMsg": "401 authorize"})
        ));
        assert!(!switch_succeeded(&json!({"errMsg": "系统繁忙"})));
    }
}
