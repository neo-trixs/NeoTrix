//! NT-SHIELD 输入输出守卫子模块
//!
//! 负责 I/O 端点的安全拦截与审计:
//! - InputGatekeeper: 入口请求校验
//! - OutputSentinel: 出口内容审查
//! - PromptGuardian: Prompt 注入防护

pub mod input_gatekeeper;
pub mod output_sentinel;
pub mod prompt_guardian;

// 2026-10-05 接线：此前本目录 1,492 行**从未被编译**（父级只声明了上面 3 个子模块）。
//   它是全仓**唯一**的破坏性命令（`rm -rf`/`sudo`/`curl | sh`）与凭据泄露
//   检测实现，而 `input_gatekeeper` 的其余检查只有 4 个词的 黑名单
//   （`malware`/`exploit`/`backdoor`/`ransomware`，见 input_gatekeeper.rs:65-70）。
pub mod agent_guardrails;

pub use input_gatekeeper::InputGatekeeper;
pub use output_sentinel::OutputSentinel;
pub use prompt_guardian::PromptGuardian;
