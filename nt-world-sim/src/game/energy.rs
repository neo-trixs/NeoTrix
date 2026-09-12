use crate::core::Resource;

#[derive(Debug, Clone)]
pub struct Energy {
    pub current: f32,
    pub max: f32,
    pub regen_rate: f32,
}

impl Energy {
    pub fn new(max: f32) -> Self {
        Self { current: max, max, regen_rate: 0.0 }
    }
    
    pub fn spend(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }
    
    pub fn restore(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn restore_full(&mut self) {
        self.current = self.max;
    }
    
    pub fn percentage(&self) -> f32 {
        self.current / self.max
    }
    
    pub fn is_empty(&self) -> bool {
        self.current <= 0.0
    }
}

impl Resource for Energy {}

impl Default for Energy {
    fn default() -> Self { Self::new(100.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_energy_spend() {
        let mut e = Energy::new(100.0);
        assert!(e.spend(50.0));
        assert!((e.current - 50.0).abs() < 0.01);
        assert!(!e.spend(60.0)); // Not enough
    }
    
    #[test]
    fn test_energy_restore() {
        let mut e = Energy::new(100.0);
        e.spend(80.0);
        e.restore(30.0);
        assert!((e.current - 50.0).abs() < 0.01);
        e.restore(60.0);
        assert!((e.current - 100.0).abs() < 0.01); // Capped at max
    }
}
