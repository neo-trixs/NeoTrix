//! L2 → L0 error conversions.
//!
//! `From<L2Error> for NeoTrixError` impls live here (L2) rather than in
//! `l0_substrate::nt_core_error` to respect the L0 ← L2 dependency direction.

use crate::l0_substrate::nt_core_error::NeoTrixError;

impl From<crate::l2_perception::nt_world::asset_map::query::ParseError> for NeoTrixError {
    fn from(e: crate::l2_perception::nt_world::asset_map::query::ParseError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l2_perception::nt_world::social_access::traits::SocialAccessError> for NeoTrixError {
    fn from(e: crate::l2_perception::nt_world::social_access::traits::SocialAccessError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l2_perception::nt_world::source::offline_download::OfflineError> for NeoTrixError {
    fn from(e: crate::l2_perception::nt_world::source::offline_download::OfflineError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_perception::nt_world::asset_map::query::ParseError;
    use crate::l2_perception::nt_world::social_access::traits::SocialAccessError;
    use crate::l2_perception::nt_world::source::offline_download::OfflineError;

    /// 本文件此前**从未被编译**（`l2_perception/mod.rs` 漏了声明），
    /// 所以这些 impl 从未生效过，也**没有任何测试**证明它们可用。
    /// ⇒ 这里逐个断言 `?` 转换真的能编译并落到正确的错误变体。
    ///
    /// 这些测试住在本文件里 ⇒ 它们**本身**也只有在本文件被编译时才存在。
    ///   即「文件缺席」会让这 4 个测试一起消失，而不只是让断言失败。
    /// ⚠️ 局限（如实记录）：测试计数变化不是 CI 会主动失败的信号 ——
    ///   兜底靠 `self_audit` 的 mod-tree 扫描 + feature-gates 的 6 次 check。
    #[test]
    fn from_parse_error_maps_to_operation_failed() {
        let e: NeoTrixError = ParseError::InvalidField("bad".to_string()).into();
        match e {
            NeoTrixError::OperationFailed(msg) => {
                assert!(!msg.is_empty(), "错误消息不应为空")
            }
            other => panic!("期望 OperationFailed，实得 {:?}", other),
        }
    }

    #[test]
    fn from_social_access_error_maps_to_operation_failed() {
        let e: NeoTrixError = SocialAccessError::Network("boom".to_string()).into();
        match e {
            NeoTrixError::OperationFailed(msg) => assert!(
                msg.contains("boom"),
                "应保留原始错误信息，实得 {}",
                msg
            ),
            other => panic!("期望 OperationFailed，实得 {:?}", other),
        }
    }

    #[test]
    fn from_offline_error_maps_to_operation_failed() {
        // ⚠️ OfflineError::Io 的载荷是 `std::io::Error`（带 #[from]），
        //    不是 String —— 我第一版按 String 写，编译直接报错。
        let e: NeoTrixError = OfflineError::Io(std::io::Error::other("disk")).into();
        match e {
            NeoTrixError::OperationFailed(msg) => assert!(!msg.is_empty()),
            other => panic!("期望 OperationFailed，实得 {:?}", other),
        }
    }

    /// `?` 运算符路径：这才是这些 impl 存在的**唯一理由**。
    /// 若某天 impl 被删，这个函数会直接编译失败（而上面的测试仍可能绿）。
    fn _uses_question_mark() -> Result<(), NeoTrixError> {
        let parse = || -> Result<(), ParseError> {
            Err(ParseError::InvalidField("x".to_string()))
        };
        let social =
            || -> Result<(), SocialAccessError> { Err(SocialAccessError::Network("z".into())) };
        let offline =
            || -> Result<(), OfflineError> { Err(OfflineError::Io(std::io::Error::other("w"))) };
        parse()?;
        social()?;
        offline()?;
        Ok(())
    }

    #[test]
    fn question_mark_path_propagates_all_three() {
        assert!(_uses_question_mark().is_err());
    }
}
