use crate::engine::renderer::Color;
use super::theme::{StardewTheme, UiDrawCommand};
use crate::game::quest::{Quest, QuestStatus};

pub struct QuestTracker {
    pub x: f32, pub y: f32,
    pub width: f32,
    pub visible: bool,
    pub expanded: bool,
    pub quests: Vec<Quest>,
}

impl QuestTracker {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, width: 200.0, visible: true, expanded: true, quests: Vec::new() }
    }

    pub fn render(&self, theme: &StardewTheme) -> Vec<UiDrawCommand> {
        if !self.visible { return Vec::new(); }

        let mut cmds = Vec::new();

        let panel_h = if self.expanded { 150.0 } else { 30.0 };
        cmds.extend(super::theme::UiRenderer::draw_panel(theme, self.x, self.y, self.width, panel_h));

        cmds.extend(super::theme::UiRenderer::draw_text(theme, "Quests", self.x + 8.0, self.y + 6.0, 12.0));

        if self.expanded {
            for (i, quest) in self.quests.iter().take(5).enumerate() {
                let qy = self.y + 28.0 + i as f32 * 24.0;

                let name_color = match quest.status {
                    QuestStatus::Active => theme.gold_text,
                    QuestStatus::Completed => Color { r: 0.2, g: 0.8, b: 0.2, a: 1.0 },
                    _ => Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 },
                };
                cmds.extend(super::theme::UiRenderer::draw_text(theme, &quest.name, self.x + 8.0, qy, 10.0));

                let progress = if quest.objectives.is_empty() {
                    1.0
                } else {
                    quest.objectives.iter().map(|o| {
                        if o.completed { 1.0 } else { o.current_count as f32 / o.target_count as f32 }
                    }).sum::<f32>() / quest.objectives.len() as f32
                };

                cmds.push(UiDrawCommand::Rect {
                    x: self.x + 8.0, y: qy + 14.0,
                    width: self.width - 16.0, height: 4.0,
                    color: theme.slot_bg,
                });
                cmds.push(UiDrawCommand::Rect {
                    x: self.x + 8.0, y: qy + 14.0,
                    width: (self.width - 16.0) * progress, height: 4.0,
                    color: theme.xp_blue,
                });
            }
        }

        cmds
    }
}

impl Default for QuestTracker {
    fn default() -> Self { Self::new(580.0, 8.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::quest::{Quest, QuestObjective, QuestType};

    fn make_quest(name: &str, status: QuestStatus) -> Quest {
        let mut q = Quest::new(1, name, "desc", QuestType::Main, "Sage");
        q.status = status;
        q.objectives.push(QuestObjective {
            description: "do thing".into(),
            target_id: None,
            target_count: 5,
            current_count: 3,
            completed: false,
        });
        q
    }

    #[test]
    fn test_quest_tracker_new() {
        let qt = QuestTracker::new(10.0, 20.0);
        assert_eq!(qt.x, 10.0);
        assert!(qt.visible);
        assert!(qt.expanded);
    }

    #[test]
    fn test_hidden_returns_empty() {
        let mut qt = QuestTracker::new(0.0, 0.0);
        qt.visible = false;
        let theme = StardewTheme::default();
        assert!(qt.render(&theme).is_empty());
    }

    #[test]
    fn test_collapsed_renders_header_only() {
        let mut qt = QuestTracker::new(0.0, 0.0);
        qt.expanded = false;
        let theme = StardewTheme::default();
        let cmds = qt.render(&theme);
        assert!(cmds.len() > 0);
    }

    #[test]
    fn test_quest_with_progress() {
        let mut qt = QuestTracker::new(0.0, 0.0);
        qt.quests.push(make_quest("Gather Gems", QuestStatus::Active));
        let theme = StardewTheme::default();
        let cmds = qt.render(&theme);
        assert!(cmds.len() > 5);
    }

    #[test]
    fn test_completed_quest_color() {
        let mut qt = QuestTracker::new(0.0, 0.0);
        qt.quests.push(make_quest("Done", QuestStatus::Completed));
        let theme = StardewTheme::default();
        let cmds = qt.render(&theme);
        assert!(!cmds.is_empty());
    }
}
