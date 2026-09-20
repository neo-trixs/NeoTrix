//! L1 → L0 error conversions.
//!
//! `From<LError> for NeoTrixError` impls live here (L1) rather than in
//! `l0_substrate::nt_core_error` to respect the L0 ← L1 dependency direction.

use crate::l0_substrate::nt_core_error::NeoTrixError;

impl From<crate::l1_action::nt_act::nt_act_trade::error::TradeError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::nt_act_trade::error::TradeError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::agent_protocol::AgentProtocolError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::agent_protocol::AgentProtocolError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::nt_act_voice::VoiceError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::nt_act_voice::VoiceError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::nt_act_trade::capability_registry::TradeCapabilityError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::nt_act_trade::capability_registry::TradeCapabilityError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::nt_act_trade::knowledge_base::KnowledgeBaseError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::nt_act_trade::knowledge_base::KnowledgeBaseError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::nt_act_trade::extractors::chrome_decrypt::ChromeDecryptError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::nt_act_trade::extractors::chrome_decrypt::ChromeDecryptError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::nt_act_trade::extractors::joinf::JoinfError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::nt_act_trade::extractors::joinf::JoinfError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::nt_act_code::recipe_refactor::RecipeError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::nt_act_code::recipe_refactor::RecipeError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::deferred_loader::DeferredError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::deferred_loader::DeferredError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_act::acp_protocol::ProtocolError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_act::acp_protocol::ProtocolError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_action_facade::FacadeError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_action_facade::FacadeError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::traits::CapabilityError> for NeoTrixError {
    fn from(e: crate::l1_action::traits::CapabilityError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_io::nt_l1_error::L1Error> for NeoTrixError {
    fn from(e: crate::l1_action::nt_io::nt_l1_error::L1Error) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

#[cfg(feature = "desktop")]
impl From<crate::l1_action::nt_io::nt_io_desktop::updater_signing::SigningError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_io::nt_io_desktop::updater_signing::SigningError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_io::nt_io_multimodal_transform::TtsError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_io::nt_io_multimodal_transform::TtsError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_io::nt_io_provider::gateway::execution::InferenceError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_io::nt_io_provider::gateway::execution::InferenceError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_io::nt_io_provider::pool::account_pool::AccountPoolError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_io::nt_io_provider::pool::account_pool::AccountPoolError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_io::universal_model::traits::ModelError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_io::universal_model::traits::ModelError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_media::audio_decode::AudioError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_media::audio_decode::AudioError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_media::hls::HlsError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_media::hls::HlsError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_media::streaming::PipelineError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_media::streaming::PipelineError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_media::thumbnail::ThumbError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_media::thumbnail::ThumbError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_media::yt_extract::YtError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_media::yt_extract::YtError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_core_task_dispatcher::TaskDispatchError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_core_task_dispatcher::TaskDispatchError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_io::nt_io_provider::gateway::observability::PluginError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_io::nt_io_provider::gateway::observability::PluginError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l1_action::nt_io::nt_io_provider::gateway::observability::MiddlewareError> for NeoTrixError {
    fn from(e: crate::l1_action::nt_io::nt_io_provider::gateway::observability::MiddlewareError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}
