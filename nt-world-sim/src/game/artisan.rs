use super::inventory::Item;
use super::economy::Economy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineType {
    PreserveJar,
    Keg,
    CheesePress,
    Loom,
    SeedMaker,
}

impl MachineType {
    pub fn process_time(&self) -> u32 {
        match self {
            MachineType::PreserveJar => 24,
            MachineType::Keg => 48,
            MachineType::CheesePress => 8,
            MachineType::Loom => 12,
            MachineType::SeedMaker => 1,
        }
    }

    pub fn quality_multiplier(&self) -> f32 {
        match self {
            MachineType::PreserveJar => 2.0,
            MachineType::Keg => 3.0,
            MachineType::CheesePress => 2.5,
            MachineType::Loom => 2.0,
            MachineType::SeedMaker => 1.0,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            MachineType::PreserveJar => "Preserve Jar",
            MachineType::Keg => "Keg",
            MachineType::CheesePress => "Cheese Press",
            MachineType::Loom => "Loom",
            MachineType::SeedMaker => "Seed Maker",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtisanMachine {
    pub machine_type: MachineType,
    pub input: Option<Item>,
    pub output: Option<Item>,
    pub process_time: u32,
    pub hours_remaining: u32,
    pub quality_multiplier: f32,
}

impl ArtisanMachine {
    pub fn new(machine_type: MachineType) -> Self {
        let process_time = machine_type.process_time();
        let quality_multiplier = machine_type.quality_multiplier();
        Self {
            machine_type,
            input: None,
            output: None,
            process_time,
            hours_remaining: 0,
            quality_multiplier,
        }
    }

    pub fn insert_input(&mut self, item: Item) -> bool {
        if self.input.is_none() && self.output.is_none() {
            self.input = Some(item);
            self.hours_remaining = self.process_time;
            true
        } else {
            false
        }
    }

    pub fn process_hour(&mut self) -> bool {
        if self.hours_remaining > 0 {
            self.hours_remaining -= 1;
            if self.hours_remaining == 0 {
                self.convert_input();
                return true;
            }
        }
        false
    }

    fn convert_input(&mut self) {
        if let Some(input) = self.input.take() {
            let mut output = input;
            output.base_value = (output.base_value as f32 * self.quality_multiplier) as u32;
            output.name = format!("{} Product", self.machine_type.name());
            output.stack = 1;
            self.output = Some(output);
        }
    }

    pub fn collect_output(&mut self) -> Option<Item> {
        self.output.take()
    }

    pub fn collect_output_for_sale(&mut self, economy: &mut Economy, day: u32) -> bool {
        if let Some(item) = self.output.take() {
            let value = item.base_value as i32;
            economy.earn(value, &item.name, day);
            true
        } else {
            false
        }
    }

    pub fn is_processing(&self) -> bool {
        self.hours_remaining > 0
    }

    pub fn has_output(&self) -> bool {
        self.output.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_none() && self.output.is_none()
    }

    pub fn progress_percent(&self) -> f32 {
        if self.process_time == 0 {
            return 0.0;
        }
        let elapsed = self.process_time - self.hours_remaining;
        (elapsed as f32 / self.process_time as f32) * 100.0
    }
}

impl Default for ArtisanMachine {
    fn default() -> Self {
        Self::new(MachineType::PreserveJar)
    }
}

pub struct ArtisanWorkshop {
    pub machines: Vec<ArtisanMachine>,
    pub capacity: usize,
}

impl ArtisanWorkshop {
    pub fn new(capacity: usize) -> Self {
        Self {
            machines: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add_machine(&mut self, machine_type: MachineType) -> bool {
        if self.machines.len() < self.capacity {
            self.machines.push(ArtisanMachine::new(machine_type));
            true
        } else {
            false
        }
    }

    pub fn tick_all(&mut self) -> Vec<(usize, MachineType)> {
        let mut completed = Vec::new();
        for (i, machine) in self.machines.iter_mut().enumerate() {
            if machine.process_hour() {
                completed.push((i, machine.machine_type));
            }
        }
        completed
    }

    pub fn empty_count(&self) -> usize {
        self.machines.iter().filter(|m| m.is_empty()).count()
    }

    pub fn processing_count(&self) -> usize {
        self.machines.iter().filter(|m| m.is_processing()).count()
    }

    pub fn ready_count(&self) -> usize {
        self.machines.iter().filter(|m| m.has_output()).count()
    }
}

impl Default for ArtisanWorkshop {
    fn default() -> Self {
        Self::new(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item(id: u32, name: &str, value: u32) -> Item {
        Item {
            id,
            name: name.to_string(),
            description: String::new(),
            stack: 1,
            max_stack: 99,
            base_value: value,
            category: super::super::inventory::ItemCategory::Crop,
            quality: super::super::item::ItemQuality::Normal,
        }
    }

    #[test]
    fn test_machine_insert_and_process() {
        let mut m = ArtisanMachine::new(MachineType::PreserveJar);
        assert!(m.is_empty());
        assert!(m.insert_input(test_item(1, "Tomato", 60)));
        assert!(!m.is_empty());
        assert!(m.is_processing());
        // Process all hours
        for _ in 0..23 {
            assert!(!m.process_hour());
        }
        assert!(m.process_hour());
        assert!(m.has_output());
        let out = m.collect_output().unwrap();
        assert_eq!(out.base_value, 120); // 60 * 2.0
    }

    #[test]
    fn test_machine_rejects_when_busy() {
        let mut m = ArtisanMachine::new(MachineType::Keg);
        m.insert_input(test_item(1, "A", 10));
        assert!(!m.insert_input(test_item(2, "B", 20)));
    }

    #[test]
    fn test_machine_collect_output_for_sale() {
        let mut m = ArtisanMachine::new(MachineType::CheesePress);
        m.insert_input(test_item(1, "Milk", 40));
        for _ in 0..7 {
            m.process_hour();
        }
        m.process_hour();
        let mut eco = Economy::new(0);
        assert!(m.collect_output_for_sale(&mut eco, 1));
        assert_eq!(eco.gold, 100); // 40 * 2.5
    }

    #[test]
    fn test_machine_progress() {
        let mut m = ArtisanMachine::new(MachineType::SeedMaker);
        m.insert_input(test_item(1, "Seed", 10));
        assert_eq!(m.progress_percent(), 0.0);
        m.process_hour();
        assert_eq!(m.progress_percent(), 100.0);
    }

    #[test]
    fn test_workshop() {
        let mut ws = ArtisanWorkshop::new(2);
        assert!(ws.add_machine(MachineType::Keg));
        assert!(ws.add_machine(MachineType::Loom));
        assert!(!ws.add_machine(MachineType::PreserveJar));
        assert_eq!(ws.empty_count(), 2);
        ws.machines[0].insert_input(test_item(1, "Juice", 80));
        assert_eq!(ws.processing_count(), 1);
    }

    #[test]
    fn test_workshop_tick() {
        let mut ws = ArtisanWorkshop::new(2);
        ws.add_machine(MachineType::SeedMaker);
        ws.machines[0].insert_input(test_item(1, "Seed", 10));
        let completed = ws.tick_all();
        assert_eq!(completed.len(), 1);
        assert_eq!(ws.ready_count(), 1);
    }

    #[test]
    fn test_machine_type_names() {
        assert_eq!(MachineType::Keg.name(), "Keg");
        assert_eq!(MachineType::Loom.process_time(), 12);
        assert!((MachineType::CheesePress.quality_multiplier() - 2.5).abs() < f32::EPSILON);
    }
}
