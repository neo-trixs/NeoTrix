use crate::ui::renderer::Renderer;

pub struct AgentListPanel {
    pub position: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub visible: bool,
    pub selected_index: Option<usize>,
    pub scroll_offset: usize,
    pub max_visible: usize,
    pub sort_mode: SortMode,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortMode {
    Distance,
    Faction,
    Fitness,
}

pub struct AgentListEntry {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub energy: f32,
    pub health: f32,
    pub faction: u32,
    pub fitness: f32,
    pub distance: f32,
}

impl AgentListPanel {
    pub fn new(position: (f32, f32)) -> Self {
        Self {
            position,
            width: 180.0,
            height: 250.0,
            visible: true,
            selected_index: None,
            scroll_offset: 0,
            max_visible: 15,
            sort_mode: SortMode::Distance,
        }
    }

    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
        self.scroll_offset = 0;
    }

    pub fn select_at(&mut self, y: f32) -> Option<usize> {
        if !self.visible {
            return None;
        }

        let py = self.position.1;
        let line_height = 16.0;
        let start_y = py + 24.0;

        if y < start_y || y > start_y + self.max_visible as f32 * line_height {
            return None;
        }

        let index = ((y - start_y) / line_height) as usize + self.scroll_offset;
        self.selected_index = Some(index);
        Some(index)
    }

    pub fn deselect(&mut self) {
        self.selected_index = None;
    }

    pub fn render(&self, renderer: &mut Renderer, entries: &[AgentListEntry]) {
        if !self.visible {
            return;
        }

        let px = self.position.0;
        let py = self.position.1;

        renderer.draw_rect_filled(px, py, self.width, self.height, 0xAA000000);
        renderer.draw_rect_outline(px, py, self.width, self.height, 0xFF4A90D9, 1);

        let sort_label = match self.sort_mode {
            SortMode::Distance => "Dist",
            SortMode::Faction => "Fact",
            SortMode::Fitness => "Fit",
        };
        renderer.draw_text_screen(
            &format!("Agents ({})", sort_label),
            px + 8.0, py + 6.0, 0xFFFFFFFF, 12,
        );

        let line_height = 16.0;
        let start_y = py + 24.0;
        let visible_start = self.scroll_offset;
        let visible_end = (visible_start + self.max_visible).min(entries.len());

        for (i, idx) in (visible_start..visible_end).enumerate() {
            let entry = &entries[idx];
            let y = start_y + i as f32 * line_height;

            let bg = if self.selected_index == Some(idx) {
                0xFF4A90D9
            } else {
                0x00000000
            };
            if bg != 0x00000000 {
                renderer.draw_rect_filled(px + 2.0, y, self.width - 4.0, line_height, bg);
            }

            renderer.draw_text_screen(
                &format!("#{} E:{:.0} H:{:.0}", entry.id, entry.energy, entry.health),
                px + 6.0, y + 2.0,
                entry.faction,
                9,
            );
        }

        if entries.len() > self.max_visible {
            let indicator_h = (self.max_visible as f32 / entries.len() as f32) * (self.height - 30.0);
            let indicator_y = py + 24.0 + (self.scroll_offset as f32 / entries.len() as f32) * (self.height - 30.0);
            renderer.draw_rect_filled(
                px + self.width - 6.0,
                indicator_y,
                4.0,
                indicator_h,
                0xFF4A90D9,
            );
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl Default for AgentListPanel {
    fn default() -> Self {
        Self::new((10.0, 360.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entries() -> Vec<AgentListEntry> {
        vec![
            AgentListEntry {
                id: "0".to_string(),
                x: 10.0,
                y: 20.0,
                energy: 80.0,
                health: 90.0,
                faction: 0xFF00FF00,
                fitness: 0.85,
                distance: 5.0,
            },
            AgentListEntry {
                id: "1".to_string(),
                x: 50.0,
                y: 60.0,
                energy: 50.0,
                health: 70.0,
                faction: 0xFF0000FF,
                fitness: 0.6,
                distance: 20.0,
            },
        ]
    }

    #[test]
    fn test_agent_list_creation() {
        let al = AgentListPanel::new((10.0, 10.0));
        assert!(al.visible);
        assert!(al.selected_index.is_none());
    }

    #[test]
    fn test_agent_list_select() {
        let mut al = AgentListPanel::new((10.0, 10.0));
        let result = al.select_at(30.0);
        assert!(result.is_some());
    }

    #[test]
    fn test_agent_list_deselect() {
        let mut al = AgentListPanel::new((10.0, 10.0));
        al.selected_index = Some(0);
        al.deselect();
        assert!(al.selected_index.is_none());
    }

    #[test]
    fn test_agent_list_sort_mode() {
        let mut al = AgentListPanel::new((10.0, 10.0));
        al.set_sort_mode(SortMode::Fitness);
        assert_eq!(al.sort_mode, SortMode::Fitness);
    }

    #[test]
    fn test_agent_list_render() {
        let mut r = Renderer::new(800, 600);
        let al = AgentListPanel::default();
        al.render(&mut r, &make_entries());
    }

    #[test]
    fn test_agent_list_toggle() {
        let mut al = AgentListPanel::default();
        al.toggle();
        assert!(!al.visible);
    }
}
