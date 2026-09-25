//! nt_tests — pipeline 验收测试 (兄弟模块, 显式导入, 行为零变更).
//! 原 `pipeline.rs` 末尾 inline `mod tests` 整体搬移至此.

use super::nt_assemble::seal_pipeline;
use super::nt_types::BrainStage;

#[test]
fn test_external_brain_digest_registered() {
    // C4 接线: 外置大脑消化闭环 stage 必须注册进 SEAL 调度管线 (Dark Forest: 接线或删除)
    let pipe = seal_pipeline();
    assert!(
        pipe.stages
            .iter()
            .any(|s| s.name() == "external_brain_digest"),
        "external_brain_digest stage 未注册进 seal_pipeline"
    );
}
