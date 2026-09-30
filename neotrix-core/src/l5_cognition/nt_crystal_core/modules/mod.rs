pub mod memory_module;
pub mod perception_module;
pub mod action_module;
pub mod emotion_module;
pub mod safety_module;
pub mod meta_module;

pub use memory_module::NtMemoryModule;
pub use perception_module::NtPerceptionModule;
pub use action_module::NtActionModule;
pub use emotion_module::NtEmotionModule;
pub use safety_module::NtSafetyModule;
pub use meta_module::NtMetaModule;

use super::ctm::CTMModule;
use super::consciousness::CrystalConsciousness;
use std::sync::{Arc, Mutex};

/// 创建所有 CTM 模块
pub fn create_all_modules(consciousness: Arc<Mutex<CrystalConsciousness>>) -> Vec<Box<dyn CTMModule>> {
    vec![
        Box::new(NtMemoryModule::new(consciousness)),
        Box::new(NtPerceptionModule::new()),
        Box::new(NtActionModule::new()),
        Box::new(NtEmotionModule::new()),
        Box::new(NtSafetyModule::new()),
        Box::new(NtMetaModule::new()),
    ]
}
