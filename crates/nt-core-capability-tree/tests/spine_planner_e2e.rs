//! **脊柱 + 投影 + 组合 的端到端闭环验证**（2026-10-08）
//!
//! # 为什么需要这个文件
//!
//! `spine.rs` 与 `planner.rs` 各自有单测，但**合起来是否真能构成一条闭环**
//! 没有被任何测试证明过。这正是本仓反复出现的病：各件都测绿，
//! 「导出 ≠ 接入」「建成未用却看着健康」。
//!
//! 本文件按**宿主真实会走的顺序**跑一遍，每一步都断言**可观察的后果**：
//!
//! ```text
//! 登记执行器 → 派生态可执行性 → 按本轮 query 投影工具（随用随调）
//!            → 组合能力链路（自我组合）→ 真正执行 → 计数/拔除/换回
//! ```

use nt_core_capability_tree::planner::{compose_chain, project_tools, tool_name_index};
use nt_core_capability_tree::spine::{
    CapabilityDescriptor, CapabilityExecutor, CapabilityHealth, Spine, SpineExecutor,
};
use serde_json::{json, Value};

/// 一个**带状态**的执行器：证明脊柱承载的是 `Arc<dyn>`，
/// 而非无捕获 `fn` 指针（后者带不了插件所需的句柄）。
struct CountingExec {
    desc: CapabilityDescriptor,
    calls: std::sync::atomic::AtomicUsize,
    healthy: std::sync::atomic::AtomicBool,
}

impl CountingExec {
    fn new(id: &str, tags: &[&str], category: &str) -> Self {
        Self {
            desc: CapabilityDescriptor::new(id, format!("{id} 的计数执行器"))
                .with_tags(tags)
                .with_category(category)
                .with_version("1.0.0")
                .with_license("LicenseRef-Test")
                .with_input_schema(json!({
                    "type": "object",
                    "properties": { "path": { "type": "string", "description": "目标路径" } },
                    "required": ["path"],
                })),
            calls: std::sync::atomic::AtomicUsize::new(0),
            healthy: std::sync::atomic::AtomicBool::new(true),
        }
    }
}

impl CapabilityExecutor for CountingExec {
    fn descriptor(&self) -> CapabilityDescriptor {
        self.desc.clone()
    }
    fn execute(
        &self,
        input: Value,
        session: &str,
    ) -> nt_core_capability_tree::dispatch::BoxFuture<'static, Result<Value, String>> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let out = json!({
            "ok": true,
            "session": session,
            "echo": input,
            "calls": self.calls.load(std::sync::atomic::Ordering::SeqCst),
        });
        Box::pin(async move { Ok(out) })
    }
    fn health(&self) -> CapabilityHealth {
        if self.healthy.load(std::sync::atomic::Ordering::SeqCst) {
            CapabilityHealth::Healthy
        } else {
            CapabilityHealth::Unhealthy("注入的故障".to_owned())
        }
    }
}

fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime")
        .block_on(f)
}

/// 构造一条可信脊柱：一个清单内的真实 id（⇒ 派生态可执行）+ 一个纯插件。
fn demo_spine() -> Spine {
    let mut sp = Spine::new();
    sp.register(std::sync::Arc::new(CountingExec::new(
        "NT-ACT::nt_file_ability::genoffice",
        &["docx", "xlsx", "pptx", "pdf", "office", "stage:1"],
        "office/document-engine",
    )))
    .expect("登记");
    sp.register(std::sync::Arc::new(CountingExec::new(
        "NT-MIND::trade::trade_quote_negotiation",
        &["quote", "trade", "stage:2"],
        "trade/quote",
    )))
    .expect("登记");
    // 纯插件（不在清单里）—— 只要 license/version 齐备也能被派发
    sp.register(std::sync::Arc::new(CountingExec::new(
        "acme::plugin::render",
        &["render", "png", "stage:0"],
        "media/render",
    )))
    .expect("登记");
    // 脚手架：清单声明 Scaffold ⇒ 派生不得升格为可执行
    sp.register(std::sync::Arc::new(SpineExecutor::new(
        CapabilityDescriptor::new("NT-MIND::trade::foreign_trade_full_cycle", "外贸全链（脚手架）")
            .with_tags(&["trade", "stage:3"])
            .with_version("1.0.0")
            .with_license("LicenseRef-Test"),
        |_id: &str, _i: Value, _s: &str| Box::pin(async { Ok(Value::Null) }),
    )))
    .expect("登记");
    sp
}

/// **闭环**：登记 → 派生 → 投影（随用随调）→ 组合（自我组合）→ 执行。
#[test]
fn 脊柱到执行形成闭环() {
    let mut sp = demo_spine();

    // ── ① 派生态可执行性：清单里是 Scaffold 的那个不得被升格 ──
    assert_eq!(
        sp.executability("NT-MIND::trade::foreign_trade_full_cycle"),
        nt_core_capability_tree::market::Executability::Scaffold,
        "脚手架登记后仍须是 Scaffold"
    );
    assert_eq!(
        sp.executability("NT-ACT::nt_file_ability::genoffice"),
        nt_core_capability_tree::market::Executability::Executable,
        "登记且健康 ⇒ Executable"
    );
    // 清单里没有的纯插件：有 license/version ⇒ 照样可派发
    assert_eq!(
        sp.executability("acme::plugin::render"),
        nt_core_capability_tree::market::Executability::Executable,
        "纯插件凭自己的 license/version 上架，不依赖清单"
    );

    // ── ② 随用随调：按本轮 query 投影工具 ──
    let query = "把这个 docx 转成 pdf，再用 png 渲染一下";
    let tools = project_tools(&sp, query, 1.0, 10);
    let picked: Vec<&str> = tools.iter().map(|(_, m)| m.id.as_str()).collect();
    assert!(
        picked.contains(&"NT-ACT::nt_file_ability::genoffice"),
        "相关能力必须摆上桌：{picked:?}"
    );
    assert!(
        picked.contains(&"acme::plugin::render"),
        "纯插件也该按相关性被摆上桌：{picked:?}"
    );
    assert!(
        !picked.contains(&"NT-MIND::trade::foreign_trade_full_cycle"),
        "Scaffold 不得进模型面：{picked:?}"
    );
    // 无关 query 不得摆任何工具
    assert!(
        project_tools(&sp, "今天午饭吃什么", 1.0, 10).is_empty(),
        "无关 query 宁缺毋滥"
    );

    // ── ③ 工具名可精确回查到 id（路由正确性）──
    let ds = sp.descriptors();
    let idx = tool_name_index(&ds);
    let first = &tools[0].1.id;
    assert_eq!(
        idx.get(&nt_core_capability_tree::planner::tool_name(first))
            .map(String::as_str),
        Some(first.as_str()),
        "工具名必须精确回查"
    );

    // ── ④ 自我组合：阶段排序 + 确定性 ──
    let chain = compose_chain(&sp, query, 1.0, 10);
    assert!(chain.len() >= 2, "应组合出多步链路：{chain:?}");
    let stage_of = |id: &str| -> i64 {
        ds.iter()
            .find(|d| d.id == id)
            .and_then(|d| d.tags.iter().find_map(|t| t.strip_prefix("stage:").and_then(|v| v.parse().ok())))
            .unwrap_or(0)
    };
    for w in chain.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let (sa, sb) = (stage_of(&a.id), stage_of(&b.id));
        assert!(
            sa <= sb,
            "阶段必须非降序（这才是「链路」）：{a:?} 然后 {b:?}"
        );
    }
    // 确定性：同输入同输出
    assert_eq!(chain, compose_chain(&sp, query, 1.0, 10), "同输入必须同输出");

    // ── ⑤ 真正执行：走脊柱取到的执行器，状态确实被用上 ──
    let exec = sp.get("NT-ACT::nt_file_ability::genoffice").expect("取执行器");
    let out = block_on(exec.execute(
        json!({ "path": "/tmp/a.docx" }),
        "test:spine:e2e",
    ))
    .expect("执行应成功");
    assert_eq!(out["ok"], json!(true));
    assert_eq!(out["session"], json!("test:spine:e2e"), "会话键必须透传");
    assert_eq!(out["echo"]["path"], json!("/tmp/a.docx"), "入参必须透传");

    // ── ⑥ 热拔插：拔掉后立即不可派发，换回后恢复 ──
    let back = sp
        .unregister("acme::plugin::render", 0)
        .expect("应能拔掉");
    assert!(!sp.contains("acme::plugin::render"));
    assert!(
        !sp.dispatchable_ids().contains(&"acme::plugin::render".to_owned()),
        "拔掉后不得仍在可调度集合"
    );
    assert!(
        project_tools(&sp, "png 渲染", 1.0, 10)
            .iter()
            .all(|(_, m)| m.id != "acme::plugin::render"),
        "拔掉后不得再被投影到模型面"
    );
    sp.register(back).expect("换回应成功");
    assert!(
        sp.dispatchable_ids().contains(&"acme::plugin::render".to_owned()),
        "换回后应恢复可派发"
    );
}

/// **诚实性闭环**：不健康的执行器必须同时从「可派发」「模型面」「链路」三处消失。
///
/// 这三处若不同步，就会出现「模型看得见、点得着、却跑不了」或反之——
/// 正是 P1 修的那类假象在投影层的延续。
#[test]
fn 不健康者从三处同时消失() {
    let mut sp = demo_spine();
    let id = "acme::plugin::render";
    let query = "png 渲染";

    assert!(sp.dispatchable_ids().contains(&id.to_owned()));
    assert!(project_tools(&sp, query, 1.0, 10).iter().any(|(_, m)| m.id == id));
    assert!(compose_chain(&sp, query, 1.0, 10).iter().any(|s| s.id == id));

    // 注入不健康（执行器句柄在 spine 内部，用 replace 换入同 id 的坏实现）
    let mut broken = CountingExec::new(id, &["render", "png", "stage:0"], "media/render");
    broken.healthy.store(false, std::sync::atomic::Ordering::SeqCst);
    sp.replace(std::sync::Arc::new(broken)).expect("替换应成功");

    assert!(
        !sp.dispatchable_ids().contains(&id.to_owned()),
        "不健康者必须离开可调度集合"
    );
    assert!(
        project_tools(&sp, query, 1.0, 10).iter().all(|(_, m)| m.id != id),
        "不健康者不得进模型面"
    );
    assert!(
        compose_chain(&sp, query, 1.0, 10).iter().all(|s| s.id != id),
        "不健康者不得进链路"
    );
    assert_eq!(
        sp.health(id),
        Some(CapabilityHealth::Unhealthy("注入的故障".to_owned())),
        "健康态本身必须可读出"
    );
}

/// **插拔原子性**：`replace` 不得让脊柱出现「旧 id 在、新 id 不在」的中间态。
#[test]
fn replace后条目集合只增不减() {
    let mut sp = demo_spine();
    let before: Vec<String> = sp.ids();
    let old = sp
        .replace(std::sync::Arc::new(CountingExec::new(
            "new::plugin::hot",
            &["hot"],
            "misc",
        )))
        .expect("替换新 id 应成功");
    assert!(old.is_none(), "替换不存在的 id 没有旧值");
    let after: Vec<String> = sp.ids();
    assert_eq!(after.len(), before.len() + 1, "只应多出被替换/新增的那一条");
    for id in &before {
        assert!(after.contains(id), "既有 id 不得消失：{id}");
    }
}