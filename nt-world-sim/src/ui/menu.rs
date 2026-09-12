pub enum MenuTab {
    Inventory,
    Skills,
    Social,
    Map,
    Crafting,
    Collection,
    Options,
}

pub struct PauseMenu {
    pub open: bool,
    pub current_tab: MenuTab,
}

impl PauseMenu {
    pub fn new() -> Self {
        Self { open: false, current_tab: MenuTab::Inventory }
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn set_tab(&mut self, tab: MenuTab) {
        self.current_tab = tab;
    }

    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            MenuTab::Inventory => MenuTab::Skills,
            MenuTab::Skills => MenuTab::Social,
            MenuTab::Social => MenuTab::Map,
            MenuTab::Map => MenuTab::Crafting,
            MenuTab::Crafting => MenuTab::Collection,
            MenuTab::Collection => MenuTab::Options,
            MenuTab::Options => MenuTab::Inventory,
        };
    }
}

impl Default for PauseMenu {
    fn default() -> Self { Self::new() }
}
