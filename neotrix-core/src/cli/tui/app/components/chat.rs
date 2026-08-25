//! Chat component

use async_trait::async_trait;
#[allow(unused_imports)]
use crate::cli::tui::app::components::component_trait::Component;
#[allow(unused_imports)]
use crate::cli::tui::app::state::AppState;
#[allow(unused_imports)]
use crate::cli::tui::app::actions::Action;
#[allow(unused_imports)]
use crate::cli::tui::app::effects::Effect;

#[derive(Debug, Default)]
pub struct ChatComponent;

#[async_trait]
impl Component for ChatComponent {
    fn id(&self) -> String { "chat".into() }
    fn name(&self) -> &'static str { "Chat" }
    fn view(&self, _state: &AppState, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    async fn update(&mut self, _state: &mut AppState, _action: Action) -> Vec<Effect> { Vec::new() }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
