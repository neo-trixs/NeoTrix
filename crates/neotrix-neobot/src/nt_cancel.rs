//! `nt_cancel` — 协作取消的**信号载体**（`Arc<AtomicBool>` 的具名封装）。
//!
//! 本 crate 刻意**没有 async 运行时**（`Cargo.toml` 里声明的 `tokio` 在核心
//! 路径上一行未用）。因此所有取消点都是**同步轮询点**，不是等待点：
//! 跑轮在检查点问一次「停了吗」，而不是 `await` 一次通知。
//!
//! **为什么是 `AtomicBool` 而不是 `Notify`**：为一个「不存在等待方」的
//! 通知语义引入 `Notify`，会诱使后人在检查点写 `.notified().await` ——
//! 那是把地基推翻。唯一能到亚秒的地方是 `execute_bash` 的 50ms 轮询，
//! 而那 50ms 本来就是既有粒度，不值得为它引入一个 runtime。
//!
//! **为什么 `Relaxed` 够**：置位与检查之间没有需要保护的其他数据。
//! 唯一需要跨执行流一致性的产物在 SQLite 里，由 store 自己的事务保证。
//! `SeqCst` 在这里是白付屏障。
//!
//! **它不承载任何面向用户的文案**：「已停 / 停在哪」由 dispatch 侧产出
//! （它持有令牌、知道自己置过位）。载体一旦开始编故事，诚实边界就没了。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 停止令牌：克隆共享同一面旗。
///
/// 一轮专属一枚（由调度层按 `conversation_id` 登记）。**「哪一跳被取消」
/// 不装在这里** —— 那是 `run_loop` 的返回值（`(TurnStatus, Option<usize>)`）
/// 的事。给一个跨执行流共享的载体加内部可变性，会逼出 `Mutex` 或 `unsafe`，
/// 两个都不要。
#[derive(Debug, Clone, Default)]
pub struct StopToken {
    flag: Arc<AtomicBool>,
}

impl StopToken {
    /// 新令牌：`flag = false`。
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 置位（`Relaxed`：只是一面旗，没有配套数据）。
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    /// 检查点问的就是这个。
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    /// 清旗 —— **只在入轮用一次**（`nt_agent` 的 C0 检查点）。
    ///
    /// 语义是「进入这一轮时的值」就是「这一轮专属的停止意图」：
    /// 上一轮遗留的置位（或者一次打在已结束轮次上的 `cancel()`）不许
    /// 误伤下一轮。故名是「入口清」而不是「用完清」。
    pub fn reset(&self) {
        self.flag.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::StopToken;

    #[test]
    fn stop_token_starts_false_and_flips_once() {
        let token = StopToken::new();
        assert!(!token.is_cancelled(), "新令牌必须是干净的");
        token.cancel();
        assert!(token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled(), "重复置位必须幂等");
        token.reset();
        assert!(!token.is_cancelled(), "reset 必须真的清掉");
    }

    /// `Arc` 语义：克隆体与本体是**同一面旗**。
    ///
    /// 这条不是「测一下 `Clone` 能不能编过」，而是钉住**将来**调度层按会话
    /// 登记令牌的用法：登记表里那份克隆，与跑轮手上那份，必须能互相看见。
    /// 若哪天有人「顺手」改成深拷贝（`AtomicBool` 不 `Clone`，只能退回
    /// `Arc::new(*self.flag)` 那种独立副本），本测试立刻红。
    #[test]
    fn stop_token_clones_share_one_flag() {
        let token = StopToken::new();
        let clone = token.clone();
        assert!(!clone.is_cancelled());
        clone.cancel();
        assert!(token.is_cancelled(), "克隆置位，本体必须看得见");
        token.reset();
        assert!(!clone.is_cancelled(), "本体清旗，克隆必须看得见");
    }

    /// 令牌要能被放进按会话登记的注册表（切片 C2），故必须 `Send + Sync`。
    #[test]
    fn stop_token_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StopToken>();
    }
}
