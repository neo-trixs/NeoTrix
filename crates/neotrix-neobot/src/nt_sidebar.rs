//! `nt_sidebar` — 侧边栏页签与文件查看器的**注册表**。
//!
//! 吸收 `dsh-better-sidebar` 的服务化扩展面：那边把 `ctx.betterSidebar`
//! 开放给所有插件（`registerTab` / `registerFileViewer`），本模块是它在
//! neobot 里的对应物。两处都刻意做成**注册表**而不是写死的 `match`：
//!
//! 1. 决策只在**一处**：某文件开哪个查看器、某次 `sidebar_open` 跳哪页，
//!    都在这里定。前端不再各处 `if (ext === ".md")` 猜 —— 猜散了就必然
//!    有一处猜错，而且错了没人发现。
//! 2. 扩展有真接口：新增页签/查看器只往注册表里加一条，UI 侧按 `id` 认。
//!
//! 与源仓库的一处**刻意分歧**：那边由 JS 插件进程动态注册；neobot 是单进程
//! 本地 App，没有插件运行时，故注册表是进程内数据结构（`TabRegistry` 可以在
//! 启动期被扩展），但接口形状刻意保持一致（`id` + `order` + 数据源声明），
//! 将来真要接插件运行时，替换的是「谁在调 `register`」，不是这份契约。

use std::path::Path;

use crate::nt_error::NtBotError;

/// 一个侧边栏页签。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SidebarTab {
    /// 稳定 id（前端 `data-tab` / `sidebar_open` 的 topic 都用它）。
    pub id: String,
    pub title: String,
    /// 图标名（须与 `frontend/src/icons.ts` 的 `IconName` 对齐）。
    ///
    /// **缺图标不会降级**：前端是 `icon(tab.icon as IconName, 16)` —— 一个纯
    /// 类型断言，运行时无校验；`icon()` 直接插值 `PATHS[name]`，名字不在表里
    /// 就渲染出一个**空 svg**（连个点都没有）。今天四个内置图标都在表内，所以
    /// 没事；新增页签时写错名字只会得到一个看不见的按钮。
    /// 早先这里写「缺图标前端降级为点」，那是假的：照它办事的人会以为拼错名字
    /// 是安全的。真正的兜底得在前端加（`PATHS[name] ?? PATHS.file`），不在注释里。
    pub icon: String,
    /// 排序（小的在前）。
    pub order: u8,
    /// 该页需要哪些数据源。取值：`files` / `changes` / `tasks` / `chat`。
    ///
    /// **当前没有消费者**：`frontend/src/sidebar.ts` 只在类型里声明了
    /// `needs: string[]`，从不读它（数据是各页 render 时自己 invoke 的）。
    /// 早先这里写「前端据此决定要不要拉，省掉无谓的 IPC」，那是把契约当成了
    /// 现状 —— 照它做性能优化的人会以为少拉了一次，其实那次 IPC 照旧发生。
    pub needs: Vec<String>,
}

/// 一个文件查看器（认领一批扩展名）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ViewerSpec {
    /// 稳定 id（`open` 指令回给前端，前端据此选渲染器）。
    pub id: String,
    pub label: String,
    /// 认领的扩展名（小写、不带点）。
    pub extensions: Vec<String>,
    /// 可否编辑（`false` 的查看器前端不提供保存按钮）。
    pub editable: bool,
    /// 语法高亮语言 id（`plain` = 不高亮）。
    pub language: String,
}

/// 打开目标的解析结果（前端据此跳）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenTarget {
    /// 命中的页签 id。
    pub topic: String,
    /// 相对工作区根的路径（页签不需要路径时为空串）。
    pub path: String,
    /// 命中的查看器 id（无路径 / 无匹配时为 `None`）。
    pub viewer: Option<String>,
}

/// 内置四个页签（对应 better-sidebar 的五 tab 减去两个本机不适用的：
/// 终端与浏览器由宿主/系统提供，neobot 不自绘）。
pub fn builtin_tabs() -> Vec<SidebarTab> {
    vec![
        SidebarTab {
            id: "files".to_owned(),
            title: "文件".to_owned(),
            icon: "folder".to_owned(),
            order: 10,
            needs: vec!["files".to_owned()],
        },
        SidebarTab {
            id: "changes".to_owned(),
            title: "变动".to_owned(),
            icon: "branch".to_owned(),
            order: 20,
            needs: vec!["changes".to_owned()],
        },
        SidebarTab {
            id: "tasks".to_owned(),
            title: "任务".to_owned(),
            icon: "layers".to_owned(),
            order: 30,
            needs: vec!["tasks".to_owned()],
        },
        SidebarTab {
            id: "chat".to_owned(),
            title: "侧聊".to_owned(),
            icon: "chat".to_owned(),
            order: 40,
            needs: vec!["chat".to_owned()],
        },
    ]
}

/// 内置查看器：可编辑代码 / Markdown / 数据 / 图片 / 二进制兜底。
///
/// **顺序即优先级**：先命中先认（与 better-sidebar 的 viewer 认领顺序同律）。
/// 兜底 `editor` 不列扩展名，靠 `viewer_for` 的末尾兜底分支命中。
pub fn builtin_viewers() -> Vec<ViewerSpec> {
    vec![
        ViewerSpec {
            id: "code".to_owned(),
            label: "代码".to_owned(),
            extensions: [
                "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "go", "java", "kt", "swift",
                "c", "h", "cc", "cpp", "hpp", "cs", "rb", "php", "sh", "bash", "zsh", "sql", "toml",
                "yaml", "yml", "ini", "conf", "css", "scss", "less", "html", "vue", "svelte", "lua",
            ]
            .iter()
            .map(|ext| (*ext).to_owned())
            .collect(),
            editable: true,
            language: "auto".to_owned(),
        },
        ViewerSpec {
            id: "markdown".to_owned(),
            label: "文档".to_owned(),
            extensions: ["md", "markdown", "mdx"]
                .iter()
                .map(|ext| (*ext).to_owned())
                .collect(),
            editable: true,
            language: "markdown".to_owned(),
        },
        ViewerSpec {
            id: "data".to_owned(),
            label: "数据".to_owned(),
            extensions: ["json", "jsonc", "json5", "csv", "tsv", "log", "xml"]
                .iter()
                .map(|ext| (*ext).to_owned())
                .collect(),
            editable: true,
            language: "auto".to_owned(),
        },
        ViewerSpec {
            id: "image".to_owned(),
            label: "图片".to_owned(),
            extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico"]
                .iter()
                .map(|ext| (*ext).to_owned())
                .collect(),
            editable: false,
            language: "plain".to_owned(),
        },
        ViewerSpec {
            id: "editor".to_owned(),
            label: "文本".to_owned(),
            extensions: Vec::new(),
            editable: true,
            language: "plain".to_owned(),
        },
    ]
}

/// 页签注册表。
#[derive(Debug, Clone, Default)]
pub struct TabRegistry {
    tabs: Vec<SidebarTab>,
}

impl TabRegistry {
    /// 只含内置页签。
    pub fn with_builtins() -> Self {
        Self {
            tabs: builtin_tabs(),
        }
    }

    /// 注册一个页签。**重复 id 拒绝**（fail-closed：两个页签抢一个 id 时
    /// 静默后者覆盖前者，会让人在错误的页签里找东西）。
    pub fn register(&mut self, tab: SidebarTab) -> Result<(), NtBotError> {
        if tab.id.trim().is_empty() {
            return Err(NtBotError::Invalid("sidebar tab needs an id".to_owned()));
        }
        if self.tabs.iter().any(|existing| existing.id == tab.id) {
            return Err(NtBotError::Invalid(format!(
                "sidebar tab '{}' already registered",
                tab.id
            )));
        }
        self.tabs.push(tab);
        self.tabs
            .sort_by_key(|tab| (tab.order, tab.id.clone()));
        Ok(())
    }

    /// 页签（已按 order 排序）。
    pub fn tabs(&self) -> &[SidebarTab] {
        &self.tabs
    }

    /// 按 id 取页签。
    pub fn tab(&self, id: &str) -> Option<&SidebarTab> {
        self.tabs.iter().find(|tab| tab.id == id)
    }

    /// 是不是已注册页签（`sidebar_open` 的白名单）。
    pub fn is_known_topic(&self, id: &str) -> bool {
        self.tab(id).is_some()
    }
}

/// 扩展名（小写、不带点；无扩展名即空串）。
pub fn extension_of(rel: &str) -> String {
    Path::new(rel)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// 该路径开哪个查看器：先按扩展名认领，全不命中则兜底 `editor`。
///
/// 兜底而非返回 `None`：工作区里 `.gitignore`、`.env`、`Makefile` 这类
/// 无扩展名的文件很多，回落到「当纯文本打开」比「打不开」有用。
pub fn viewer_for<'a>(viewers: &'a [ViewerSpec], rel: &str) -> Option<&'a ViewerSpec> {
    let ext = extension_of(rel);
    if !ext.is_empty() {
        if let Some(hit) = viewers
            .iter()
            .find(|viewer| viewer.extensions.iter().any(|owned| owned == &ext))
        {
            return Some(hit);
        }
    }
    viewers.iter().find(|viewer| viewer.id == "editor")
}

/// 解析一次「打开」请求：topic 必须在册、target 必须过 jail 词法校验。
///
/// 路径型 topic（`files`）额外要求 target 非空；非路径型（`tasks`/`chat`）
/// 允许空 target（= 打开该页即可）。
pub fn resolve_open(
    registry: &TabRegistry,
    viewers: &[ViewerSpec],
    topic: &str,
    target: &str,
) -> Result<OpenTarget, NtBotError> {
    let topic = topic.trim();
    if !registry.is_known_topic(topic) {
        return Err(NtBotError::Invalid(format!(
            "sidebar topic '{topic}' is not registered"
        )));
    }
    let target = target.trim();
    crate::nt_workspace::check_rel(target)?;
    if target.is_empty() && topic == "files" {
        return Err(NtBotError::Invalid(
            "topic 'files' needs a target path".to_owned(),
        ));
    }
    let viewer = if target.is_empty() {
        None
    } else {
        viewer_for(viewers, target).map(|spec| spec.id.clone())
    };
    Ok(OpenTarget {
        topic: topic.to_owned(),
        path: target.to_owned(),
        viewer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_registry_is_sorted_and_unique() {
        let reg = TabRegistry::with_builtins();
        let ids: Vec<&str> = reg.tabs().iter().map(|tab| tab.id.as_str()).collect();
        assert_eq!(ids, vec!["files", "changes", "tasks", "chat"]);
        // 每个页签都要声明数据源，前端才不用猜。
        for tab in reg.tabs() {
            assert!(!tab.needs.is_empty(), "{} 缺数据源声明", tab.id);
        }
    }

    #[test]
    fn register_rejects_duplicate_and_empty_ids() {
        let mut reg = TabRegistry::with_builtins();
        let clash = SidebarTab {
            id: "files".to_owned(),
            title: "另一个文件页".to_owned(),
            icon: "folder".to_owned(),
            order: 99,
            needs: vec!["files".to_owned()],
        };
        let err = reg.register(clash).expect_err("dup id must fail");
        assert!(err.to_string().contains("already registered"), "{err}");
        // 失败后注册表没被污染。
        assert_eq!(reg.tabs().len(), 4);
        let nameless = SidebarTab {
            id: "  ".to_owned(),
            title: "无 id".to_owned(),
            icon: "chat".to_owned(),
            order: 5,
            needs: vec![],
        };
        assert!(reg.register(nameless).is_err());
        assert_eq!(reg.tabs().len(), 4);
    }

    #[test]
    fn register_inserts_and_resorts() {
        let mut reg = TabRegistry::with_builtins();
        reg.register(SidebarTab {
            id: "zzz-later".to_owned(),
            title: "后置页".to_owned(),
            icon: "info".to_owned(),
            order: 99,
            needs: vec!["tasks".to_owned()],
        })
        .expect("register");
        assert_eq!(reg.tabs().last().map(|t| t.id.as_str()), Some("zzz-later"));
        assert!(reg.is_known_topic("zzz-later"));
        assert!(!reg.is_known_topic("nope"));
    }

    #[test]
    fn viewer_claims_by_extension_in_priority_order() {
        let viewers = builtin_viewers();
        assert_eq!(viewer_for(&viewers, "a/b.rs").map(|v| v.id.as_str()), Some("code"));
        assert_eq!(viewer_for(&viewers, "README.md").map(|v| v.id.as_str()), Some("markdown"));
        assert_eq!(viewer_for(&viewers, "x.json").map(|v| v.id.as_str()), Some("data"));
        assert_eq!(viewer_for(&viewers, "logo.PNG").map(|v| v.id.as_str()), Some("image"));
        // 大小写不敏感。
        assert_eq!(viewer_for(&viewers, "A.TS").map(|v| v.id.as_str()), Some("code"));
    }

    #[test]
    fn viewer_falls_back_to_plain_editor() {
        let viewers = builtin_viewers();
        // 无扩展名（.gitignore / Makefile）也打得开。
        for name in [".gitignore", "Makefile", "LICENSE"] {
            let hit = viewer_for(&viewers, name).expect("fallback editor");
            assert_eq!(hit.id, "editor", "{name}");
            assert!(hit.editable);
        }
        // 没注册 editor 的自定义注册表不该硬塞兜底。
        assert!(viewer_for(&[], "x.rs").is_none());
    }

    #[test]
    fn image_viewer_is_read_only() {
        let viewers = builtin_viewers();
        let hit = viewer_for(&viewers, "a.png").expect("image");
        assert!(!hit.editable, "图片不该给保存按钮");
        assert!(viewer_for(&viewers, "a.rs").expect("code").editable);
    }

    #[test]
    fn resolve_open_gates_unknown_topics() {
        let reg = TabRegistry::with_builtins();
        let viewers = builtin_viewers();
        let err = resolve_open(&reg, &viewers, "evil", "a.txt").expect_err("unknown topic");
        assert!(err.to_string().contains("not registered"), "{err}");
    }

    #[test]
    fn resolve_open_jails_the_target() {
        let reg = TabRegistry::with_builtins();
        let viewers = builtin_viewers();
        for bad in ["../../etc/passwd", "/etc/passwd", "~/x", "a/../../b"] {
            let err = resolve_open(&reg, &viewers, "files", bad).expect_err(bad);
            assert!(err.to_string().contains("escapes workspace"), "{bad}: {err}");
        }
        // 越狱目标**连提议都不该发出去**（模型被注入也开不出工作区外的东西）。
    }

    #[test]
    fn resolve_open_needs_a_path_for_files_only() {
        let reg = TabRegistry::with_builtins();
        let viewers = builtin_viewers();
        // files 缺 target → 拒。
        assert!(resolve_open(&reg, &viewers, "files", "").is_err());
        // tasks / chat 缺 target 合法（= 打开该页即可）。
        for topic in ["tasks", "chat", "changes"] {
            let got = resolve_open(&reg, &viewers, topic, "").expect(topic);
            assert_eq!(got.topic, topic);
            assert_eq!(got.path, "");
            assert!(got.viewer.is_none());
        }
    }

    #[test]
    fn resolve_open_attaches_the_viewer() {
        let reg = TabRegistry::with_builtins();
        let viewers = builtin_viewers();
        let got = resolve_open(&reg, &viewers, "files", "src/main.rs").expect("open");
        assert_eq!(got.topic, "files");
        assert_eq!(got.path, "src/main.rs");
        assert_eq!(got.viewer.as_deref(), Some("code"));
    }
}
