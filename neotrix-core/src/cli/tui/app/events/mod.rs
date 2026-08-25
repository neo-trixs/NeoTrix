//! Event Bus - Decoupled event system

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use serde::{Deserialize, Serialize};

/// 事件类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    KeyPress,
    MouseClick,
    MouseMove,
    Resize,
    FocusChange,
    SessionChange,
    MessageReceived,
    ToolCall,
    StreamChunk,
    StreamEnd,
    Error,
    ConfigChange,
    ThemeChange,
    Custom(String),
}

/// 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

/// 订阅者
type Subscriber = Box<dyn Fn(Event) + Send + Sync>;

/// 事件总线
pub struct EventBus {
    tx: broadcast::Sender<Event>,
    subscribers: Arc<Mutex<HashMap<EventType, Vec<Subscriber>>>>,
    subscription_id: Arc<std::sync::atomic::AtomicU64>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1000);
        Self {
            tx,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            subscription_id: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// 发布事件
    pub fn publish(&self, event: Event) {
        let _ = self.tx.send(event);
    }

    /// 订阅事件类型
    pub fn subscribe(&self, event_type: EventType, handler: Box<dyn Fn(Event) + Send + Sync>) -> u64 {
        let mut subscribers = futures::executor::block_on(self.subscribers.lock());
        let id = self.subscription_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        subscribers.entry(event_type).or_default().push(handler);
        id
    }

    /// 取消订阅
    pub fn unsubscribe(&self, event_type: EventType, id: u64) {
        let mut subscribers = futures::executor::block_on(self.subscribers.lock());
        if let Some(handlers) = subscribers.get_mut(&event_type) {
            handlers.retain(|_| true); // 简化：实际需要存储 ID
        }
    }

    /// 获取接收器
    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}

/// 事件发射器 trait
pub trait EventEmitter {
    fn emit(&self, event: Event);
}

/// 默认事件总线实例
impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// 键位绑定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyBindings {
    pub bindings: std::collections::HashMap<String, String>, // key -> action
}

impl KeyBindings {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        // 默认绑定
        bindings.insert("ctrl+c".to_string(), "Quit".to_string());
        bindings.insert("ctrl+d".to_string(), "Quit".to_string());
        bindings.insert("enter".to_string(), "InputSubmit".to_string());
        bindings.insert("ctrl+c".to_string(), "InputCancel".to_string());
        bindings.insert("up".to_string(), "InputHistoryPrev".to_string());
        bindings.insert("down".to_string(), "InputHistoryNext".to_string());
        bindings.insert("left".to_string(), "InputMoveCursorLeft".to_string());
        bindings.insert("right".to_string(), "InputMoveCursorRight".to_string());
        bindings.insert("ctrl+left".to_string(), "InputMoveWordBackward".to_string());
        bindings.insert("ctrl+right".to_string(), "InputMoveWordForward".to_string());
        bindings.insert("home".to_string(), "InputMoveCursorHome".to_string());
        bindings.insert("end".to_string(), "InputMoveCursorEnd".to_string());
        bindings.insert("backspace".to_string(), "InputDeleteBackward".to_string());
        bindings.insert("delete".to_string(), "InputDeleteForward".to_string());
        bindings.insert("ctrl+r".to_string(), "HistorySearchOpen".to_string());
        bindings.insert("ctrl+s".to_string(), "LayoutToggleSessions".to_string());
        bindings.insert("ctrl+t".to_string(), "LayoutToggleTools".to_string());
        bindings.insert("ctrl+l".to_string(), "LayoutToggleLineNumbers".to_string());
        bindings.insert("ctrl+w".to_string(), "LayoutToggleWordWrap".to_string());
        bindings.insert("ctrl+a".to_string(), "LayoutToggleAutoScroll".to_string());
        bindings.insert("ctrl+v".to_string(), "VimModeEnter".to_string());
        bindings.insert("escape".to_string(), "VimModeExit".to_string());
        bindings.insert("ctrl+r".to_string(), "HistorySearchOpen".to_string());
        bindings.insert("ctrl+f".to_string(), "SearchOpen".to_string());
        bindings.insert("f3".to_string(), "SearchNext".to_string());
        bindings.insert("shift+f3".to_string(), "SearchPrev".to_string());
        bindings.insert("ctrl+shift+c".to_string(), "ChatCopyMessage".to_string());
        bindings.insert("ctrl+shift+v".to_string(), "ChatCopyCodeBlock".to_string());
        bindings.insert("ctrl+shift+x".to_string(), "ChatCopyToolCall".to_string());
        bindings.insert("ctrl+shift+r".to_string(), "ChatRetryLast".to_string());
        bindings.insert("ctrl+shift+g".to_string(), "ChatRegenerateLast".to_string());
        bindings.insert("ctrl+shift+d".to_string(), "ChatDeleteMessage".to_string());
        bindings.insert("ctrl+shift+e".to_string(), "ChatEditMessage".to_string());
        bindings.insert("ctrl+t".to_string(), "ChatToggleThinking".to_string());
        bindings.insert("ctrl+shift+t".to_string(), "ChatToggleToolCall".to_string());
        bindings.insert("f1".to_string(), "HelpOpen".to_string());
        bindings.insert("ctrl+shift+p".to_string(), "CommandPaletteOpen".to_string());
        bindings.insert("ctrl+shift+n".to_string(), "SessionNewTab".to_string());
        bindings.insert("ctrl+w".to_string(), "SessionCloseCurrent".to_string());
        bindings.insert("ctrl+shift+w".to_string(), "SessionCloseAll".to_string());
        bindings.insert("ctrl+pageup".to_string(), "SessionPrev".to_string());
        bindings.insert("ctrl+pagedown".to_string(), "SessionNext".to_string());
        bindings.insert("alt+left".to_string(), "SessionMoveLeft".to_string());
        bindings.insert("alt+right".to_string(), "SessionMoveRight".to_string());
        Self { bindings }
    }

    pub fn get_action(&self, key: &str) -> Option<String> {
        self.bindings.get(key).cloned()
    }

    pub fn bind(&mut self, key: String, action: String) {
        self.bindings.insert(key, action);
    }

    pub fn unbind(&mut self, key: &str) {
        self.bindings.remove(key);
    }
}

/// 斜杠命令注册表
pub struct CommandRegistry {
    commands: HashMap<String, CommandDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDef {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub aliases: Vec<String>,
    pub handler: String, // handler function name
    pub args: Vec<CommandArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self { commands: HashMap::new() }
    }

    pub fn register(&mut self, cmd: CommandDef) {
        self.commands.insert(cmd.name.clone(), cmd.clone());
        for _alias in &cmd.aliases {
            // 别名也注册
        }
    }

    pub fn get(&self, name: &str) -> Option<&CommandDef> {
        self.commands.get(name)
    }

    pub fn get_by_alias(&self, alias: &str) -> Option<&CommandDef> {
        self.commands.values().find(|cmd| cmd.aliases.contains(&alias.to_string()))
    }

    pub fn list(&self) -> Vec<&CommandDef> {
        self.commands.values().collect()
    }

    pub fn completions(&self, prefix: &str) -> Vec<String> {
        self.commands.keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        // 注册内置命令
        registry.register(CommandDef {
            name: "help".to_string(),
            description: "显示帮助信息".to_string(),
            usage: "/help [command]".to_string(),
            aliases: vec!["h".to_string(), "?".to_string()],
            handler: "help".to_string(),
            args: vec![],
        });
        registry.register(CommandDef {
            name: "quit".to_string(),
            description: "退出程序".to_string(),
            usage: "/quit".to_string(),
            aliases: vec!["q".to_string(), "exit".to_string()],
            handler: "quit".to_string(),
            args: vec![],
        });
        registry.register(CommandDef {
            name: "clear".to_string(),
            description: "清空聊天历史".to_string(),
            usage: "/clear".to_string(),
            aliases: vec!["c".to_string()],
            handler: "chat_clear".to_string(),
            args: vec![],
        });
        registry.register(CommandDef {
            name: "theme".to_string(),
            description: "切换主题".to_string(),
            usage: "/theme [name]".to_string(),
            aliases: vec!["t".to_string()],
            handler: "theme".to_string(),
            args: vec![CommandArg {
                name: "name".to_string(),
                description: "主题名称".to_string(),
                required: false,
                default: None,
            }],
        });
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_bindings() {
        let kb = KeyBindings::new();
        assert_eq!(kb.get_action("enter"), Some("InputSubmit".to_string()));
        assert_eq!(kb.get_action("ctrl+c"), Some("Quit".to_string()));
    }

    #[test]
    fn test_command_registry() {
        let registry = CommandRegistry::default();
        assert!(registry.get("help").is_some());
        assert_eq!(registry.completions("h"), vec!["help"]);
    }
}
