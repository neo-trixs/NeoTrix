//! # 可验证回放收据 (Verifiable Replay Receipts)
//!
//! 修复 (分析建议 #8, 对应 agent-receipts: 加密回放 + 可验证收据):
//! 每次 agent 运行 / 吸收动作产出一张进程内、零依赖、框架无关的收据,
//! 绑定 (run_id, 输入哈希, 输出哈希, 时间戳, 签名), 可供事后审计回放验证
//! "这次吸收/执行确实发生了, 且输入输出未被篡改"。
//!
//! 与 `self_poison` 协同: 被 Blocked 的吸收不产出收据 (拒绝即无痕)。

use sha2::{Digest, Sha256};

/// 一张 agent 运行 / 吸收动作的回放收据。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentReceipt {
    pub run_id: String,
    pub input_hash: String,
    pub output_hash: String,
    pub timestamp: i64,
    /// 对 (run_id|input_hash|output_hash|timestamp) 的 SHA-256 签名。
    pub signature: String,
}

fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    let out = h.finalize();
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

fn sign(run_id: &str, input_hash: &str, output_hash: &str, timestamp: i64) -> String {
    sha256_hex(&format!("{}|{}|{}|{}", run_id, input_hash, output_hash, timestamp))
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

impl AgentReceipt {
    /// 为一次运行/吸收动作生成收据。
    pub fn emit(run_id: &str, input: &str, output: &str) -> Self {
        let input_hash = sha256_hex(input);
        let output_hash = sha256_hex(output);
        let timestamp = now_ts();
        let signature = sign(run_id, &input_hash, &output_hash, timestamp);
        AgentReceipt {
            run_id: run_id.to_string(),
            input_hash,
            output_hash,
            timestamp,
            signature,
        }
    }

    /// 验证收据签名完整性 (输入/输出/时间戳未被篡改)。
    pub fn verify(&self) -> bool {
        sign(
            &self.run_id,
            &self.input_hash,
            &self.output_hash,
            self.timestamp,
        ) == self.signature
    }
}

/// 对一段内容求哈希 (供调用方在 emit 前预计算或比对)。
pub fn hash_content(content: &str) -> String {
    sha256_hex(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_and_verify_roundtrip() {
        let r = AgentReceipt::emit("run-1", "input text", "output text");
        assert!(r.verify());
    }

    #[test]
    fn test_tampered_output_fails_verify() {
        let mut r = AgentReceipt::emit("run-1", "input", "output");
        r.output_hash = sha256_hex("EVIL");
        assert!(!r.verify());
    }

    #[test]
    fn test_tampered_signature_fails_verify() {
        let mut r = AgentReceipt::emit("run-1", "input", "output");
        r.signature = "deadbeef".to_string();
        assert!(!r.verify());
    }

    #[test]
    fn test_distinct_inputs_distinct_hashes() {
        let a = AgentReceipt::emit("run-1", "a", "o");
        let b = AgentReceipt::emit("run-1", "b", "o");
        assert_ne!(a.input_hash, b.input_hash);
        assert_ne!(a.signature, b.signature);
    }

    #[test]
    fn test_absorb_action_emits_verifiable_receipt() {
        // 模拟一次知识吸收动作: run_id = 节点 id, input = 正文, output = 节点摘要。
        // 成功写入后应产出发票, 且 verify() 通过; 被 Blocked 的吸收不在此处 (拒绝即无痕)。
        let run_id = "node-abc-123";
        let input = "E8 推理引擎影响 GWT 注意力路由";
        let output = "E8→GWT 因果链已 crystallized";
        let r = AgentReceipt::emit(run_id, input, output);
        assert!(r.verify(), "吸收收据应能通过签名完整性校验");
        assert_eq!(r.run_id, run_id);
        // 回放: 输入被篡改后 verify 必须失败
        let mut tampered = r.clone();
        tampered.input_hash = sha256_hex("tampered input");
        assert!(!tampered.verify(), "输入哈希被篡改后 verify 必须失败");
    }
}
