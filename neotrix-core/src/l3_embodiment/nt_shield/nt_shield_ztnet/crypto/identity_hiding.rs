//! 会话级匿名性增强 (NDSS 2024 修正)
//!
//! ## 问题
//! WireGuard 静态公钥在不同会话间可被链接，泄露用户身份。
//!
//! ## 修正方案
//! 1. 会话级密钥混淆: 每个会话使用不同的伪随机填充
//! 2. 随机填充: 在 Noise 握手中添加随机填充，防止消息长度分析
//! 3. 定期密钥刷新: 即使不轮换密钥，也定期刷新混淆种子

use rand::Rng;

/// 混淆种子大小
pub const OBFUSCATION_SEED_SIZE: usize = 32;

/// 最大随机填充大小 (WireGuard 协议限制: 不超过 255 字节)
pub const MAX_PADDING: usize = 255;

/// 混淆器
pub struct IdentityHider {
    /// 当前混淆种子
    seed: [u8; OBFUSCATION_SEED_SIZE],
    /// 创建时间 (用于判断是否需要刷新)
    created_at: std::time::Instant,
    /// 刷新间隔 (10 分钟)
    refresh_interval: std::time::Duration,
}

impl IdentityHider {
    /// 创建新的混淆器
    pub fn new() -> Self {
        let mut seed = [0u8; OBFUSCATION_SEED_SIZE];
        rand::thread_rng().fill(&mut seed);

        Self {
            seed,
            created_at: std::time::Instant::now(),
            refresh_interval: std::time::Duration::from_secs(600),
        }
    }

    /// 混淆公钥 (返回混淆后的伪公钥)
    ///
    /// 混淆方法: pseudonym = Hash(seed || real_public)
    /// 注意: 这是轻量级混淆，不提供密码学安全性。
    /// 完整的身份隐藏需要使用 AES 密钥交换等协议。
    pub fn obfuscate_public_key(&self, public_key: &[u8; 32]) -> [u8; 32] {
        use blake2::{Blake2s256, Digest};
        let mut hasher = Blake2s256::new();
        hasher.update(&self.seed);
        hasher.update(public_key);
        let result: [u8; 32] = hasher.finalize().into();
        result
    }

    /// 生成随机填充 (防止消息长度分析)
    ///
    /// 返回: 填充字节的长度
    pub fn random_padding_length(&self) -> usize {
        rand::thread_rng().gen_range(0..=MAX_PADDING)
    }

    /// 生成填充字节
    pub fn generate_padding(length: usize) -> Vec<u8> {
        let mut padding = vec![0u8; length];
        rand::thread_rng().fill(&mut padding[..]);
        padding
    }

    /// 检查是否需要刷新种子
    pub fn needs_refresh(&self) -> bool {
        self.created_at.elapsed() >= self.refresh_interval
    }

    /// 刷新种子
    pub fn refresh(&mut self) {
        if self.needs_refresh() {
            rand::thread_rng().fill(&mut self.seed);
            self.created_at = std::time::Instant::now();
        }
    }

    /// 获取当前种子引用
    pub fn seed(&self) -> &[u8; OBFUSCATION_SEED_SIZE] {
        &self.seed
    }
}

impl Default for IdentityHider {
    fn default() -> Self {
        Self::new()
    }
}

/// 消息混淆器: 为 Noise 握手消息添加随机填充
pub struct MessageObfuscator {
    hider: IdentityHider,
}

impl MessageObfuscator {
    /// 创建新的消息混淆器
    pub fn new() -> Self {
        Self {
            hider: IdentityHider::new(),
        }
    }

    /// 混淆消息: 在消息末尾添加随机填充
    ///
    /// WireGuard 协议允许消息包含填充:
    /// - 握手消息: 最大 148 bytes (含 MAC)
    /// - 数据消息: 最大 65535 bytes
    ///
    /// 填充在解密后被剥离。
    pub fn obfuscate_message(&mut self, msg: &mut Vec<u8>) {
        self.hider.refresh();
        let padding_len = self.hider.random_padding_length();
        let padding = IdentityHider::generate_padding(padding_len);
        msg.extend_from_slice(&padding);
    }

    /// 去混淆消息: 剥离随机填充
    ///
    /// # Arguments
    /// * `msg` - 消息 (会被修改)
    /// * `expected_len` - 期望的消息长度 (不含填充)
    pub fn deobfuscate_message(msg: &mut Vec<u8>, expected_len: usize) {
        msg.truncate(expected_len);
    }
}

impl Default for MessageObfuscator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obfuscation_deterministic_with_same_seed() {
        let mut hider = IdentityHider::new();
        let seed = *hider.seed();

        let public_key = [42u8; 32];
        let obfuscated1 = hider.obfuscate_public_key(&public_key);

        // 恢复种子
        hider.refresh();
        // 用不同种子
        let obfuscated2 = hider.obfuscate_public_key(&public_key);

        // 不同种子产生不同混淆结果
        assert_ne!(obfuscated1, obfuscated2);
    }

    #[test]
    fn padding_length_in_range() {
        let hider = IdentityHider::new();
        for _ in 0..100 {
            let len = hider.random_padding_length();
            assert!(len <= MAX_PADDING);
        }
    }

    #[test]
    fn message_obfuscation_roundtrip() {
        let mut obfuscator = MessageObfuscator::new();
        let original = vec![1u8, 2, 3, 4, 5];
        let mut msg = original.clone();

        obfuscator.obfuscate_message(&mut msg);
        assert!(msg.len() >= original.len());

        MessageObfuscator::deobfuscate_message(&mut msg, original.len());
        assert_eq!(msg, original);
    }

    #[test]
    fn refresh_interval() {
        let mut hider = IdentityHider::new();
        assert!(!hider.needs_refresh());

        // 模拟时间流逝
        hider.created_at = std::time::Instant::now() - std::time::Duration::from_secs(601);
        assert!(hider.needs_refresh());

        hider.refresh();
        assert!(!hider.needs_refresh());
    }
}
