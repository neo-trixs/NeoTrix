use crate::core::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DailyEarning {
    pub day: u32,
    pub amount: i32,
    pub source: String,
}

pub struct Economy {
    pub gold: i32,
    pub total_earned: i32,
    pub total_spent: i32,
    pub daily_earnings: Vec<DailyEarning>,
}

impl Economy {
    pub fn new(starting_gold: i32) -> Self {
        Self {
            gold: starting_gold,
            total_earned: 0,
            total_spent: 0,
            daily_earnings: Vec::new(),
        }
    }

    pub fn earn(&mut self, amount: i32, source: &str, day: u32) -> bool {
        if amount > 0 {
            self.gold += amount;
            self.total_earned += amount;
            self.daily_earnings.push(DailyEarning {
                day,
                amount,
                source: source.to_string(),
            });
            true
        } else {
            false
        }
    }

    pub fn spend(&mut self, amount: i32) -> bool {
        if amount > 0 && self.gold >= amount {
            self.gold -= amount;
            self.total_spent += amount;
            true
        } else {
            false
        }
    }

    pub fn can_afford(&self, amount: i32) -> bool {
        self.gold >= amount
    }

    pub fn daily_summary(&self, day: u32) -> i32 {
        self.daily_earnings
            .iter()
            .filter(|e| e.day == day)
            .map(|e| e.amount)
            .sum()
    }

    pub fn lifetime_summary(&self) -> (i32, i32) {
        (self.total_earned, self.total_spent)
    }

    pub fn clear_daily(&mut self, day: u32) {
        self.daily_earnings.retain(|e| e.day != day);
    }
}

impl Default for Economy {
    fn default() -> Self {
        Self::new(500)
    }
}

impl Resource for Economy {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_economy_earn_spend() {
        let mut eco = Economy::new(100);
        assert!(eco.earn(50, "crop", 1));
        assert_eq!(eco.gold, 150);
        assert_eq!(eco.total_earned, 50);
        assert!(eco.spend(30));
        assert_eq!(eco.gold, 120);
        assert_eq!(eco.total_spent, 30);
    }

    #[test]
    fn test_economy_cannot_overdraft() {
        let mut eco = Economy::new(10);
        assert!(!eco.spend(20));
        assert_eq!(eco.gold, 10);
    }

    #[test]
    fn test_economy_negative_earn() {
        let mut eco = Economy::new(100);
        assert!(!eco.earn(-10, "bad", 1));
        assert_eq!(eco.gold, 100);
    }

    #[test]
    fn test_daily_summary() {
        let mut eco = Economy::new(0);
        eco.earn(10, "a", 1);
        eco.earn(20, "b", 1);
        eco.earn(5, "c", 2);
        assert_eq!(eco.daily_summary(1), 30);
        assert_eq!(eco.daily_summary(2), 5);
        assert_eq!(eco.daily_summary(3), 0);
    }

    #[test]
    fn test_can_afford() {
        let eco = Economy::new(100);
        assert!(eco.can_afford(100));
        assert!(!eco.can_afford(101));
    }

    #[test]
    fn test_lifetime_summary() {
        let mut eco = Economy::new(0);
        eco.earn(100, "a", 1);
        eco.earn(50, "b", 1);
        eco.spend(30);
        let (earned, spent) = eco.lifetime_summary();
        assert_eq!(earned, 150);
        assert_eq!(spent, 30);
    }
}
