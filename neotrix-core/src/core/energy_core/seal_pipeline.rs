pub struct SEALPipelineImpl;

impl SEALPipelineImpl {
    pub fn new() -> Self { Self }
    pub async fn receive_wisdom(&mut self, _wisdom: crate::core::l7_capability::types::Wisdom) {}
}
