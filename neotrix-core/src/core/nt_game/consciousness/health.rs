//! ConsciousnessTree health integration.
//!
//! Computes game-branch health scores from buffer stats and phi averages,
//! producing actionable recommendations for difficulty adjustment.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Types (self-contained)
// ═══════════════════════════════════════════════════════════════════

/// Buffer stats from the arena — aggregated episode statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferStats {
    /// Win rate across recent episodes [0, 1].
    pub win_rate: f64,
    /// Average trajectory reward.
    pub avg_reward: f64,
    /// Average reward variance (consistency proxy: low variance = consistent).
    pub reward_variance: f64,
    /// Number of episodes in buffer.
    pub episode_count: usize,
}

impl BufferStats {
    pub fn new(win_rate: f64, avg_reward: f64, reward_variance: f64, episode_count: usize) -> Self {
        Self {
            win_rate,
            avg_reward,
            reward_variance,
            episode_count,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Health Computation
// ═══════════════════════════════════════════════════════════════════

/// Compute game-branch health score [0.0, 1.0].
///
/// Factors:
/// - Win rate contribution (40%): sweet spot at 0.55-0.70
/// - Reward contribution (25%): higher reward = healthier
/// - Phi contribution (20%): higher phi_avg = more integration
/// - Consistency contribution (15%): lower variance = more consistent
pub fn compute_health(stats: &BufferStats, phi_avg: f64) -> f64 {
    // Win rate: sweet spot around 0.55-0.70 (not too easy, not too hard)
    let wr_score = if stats.win_rate >= 0.55 && stats.win_rate <= 0.70 {
        1.0
    } else if stats.win_rate < 0.55 {
        // Too hard: score drops linearly
        (stats.win_rate / 0.55).clamp(0.0, 1.0)
    } else {
        // Too easy: score drops linearly above 0.70
        ((1.0 - stats.win_rate) / 0.30).clamp(0.0, 1.0)
    };

    // Reward: normalize to [0, 1] assuming range [-1, 1]
    let reward_score = ((stats.avg_reward + 1.0) / 2.0).clamp(0.0, 1.0);

    // Phi: already [0, 1]
    let phi_score = phi_avg.clamp(0.0, 1.0);

    // Consistency: low variance = high consistency
    // Assume variance range [0, 1]
    let consistency_score = (1.0 - stats.reward_variance).clamp(0.0, 1.0);

    // Weighted combination
    let health = 0.40 * wr_score + 0.25 * reward_score + 0.20 * phi_score + 0.15 * consistency_score;

    health.clamp(0.0, 1.0)
}

// ═══════════════════════════════════════════════════════════════════
// Health Report
// ═══════════════════════════════════════════════════════════════════

/// Recommendation action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Recommendation {
    IncreaseDifficulty,
    Maintain,
    DecreaseDifficulty,
    IncreasePhi,
    ImproveConsistency,
}

impl Recommendation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IncreaseDifficulty => "increase_difficulty",
            Self::Maintain => "maintain",
            Self::DecreaseDifficulty => "decrease_difficulty",
            Self::IncreasePhi => "increase_phi",
            Self::ImproveConsistency => "improve_consistency",
        }
    }
}

/// Complete health report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub health_score: f64,
    pub recommendation: Recommendation,
    pub win_rate: f64,
    pub avg_reward: f64,
    pub phi_avg: f64,
}

/// Generate recommendation from health report.
pub fn recommend_action(report: &HealthReport) -> &'static str {
    if report.health_score >= 0.7 {
        // Doing well — increase difficulty
        Recommendation::IncreaseDifficulty.as_str()
    } else if report.health_score >= 0.4 {
        // Moderate — maintain current level
        Recommendation::Maintain.as_str()
    } else if report.phi_avg < 0.2 {
        // Low phi — focus on integration
        Recommendation::IncreasePhi.as_str()
    } else if report.avg_reward < -0.2 {
        // Struggling — decrease difficulty
        Recommendation::DecreaseDifficulty.as_str()
    } else {
        // Inconsistent — work on consistency
        Recommendation::ImproveConsistency.as_str()
    }
}

/// Generate health report from buffer stats and phi average.
pub fn generate_health_report(stats: &BufferStats, phi_avg: f64) -> HealthReport {
    let health_score = compute_health(stats, phi_avg);
    let report = HealthReport {
        health_score,
        recommendation: Recommendation::Maintain, // placeholder, recomputed below
        win_rate: stats.win_rate,
        avg_reward: stats.avg_reward,
        phi_avg,
    };

    // Determine recommendation
    let rec_str = recommend_action(&report);
    let recommendation = match rec_str {
        "increase_difficulty" => Recommendation::IncreaseDifficulty,
        "maintain" => Recommendation::Maintain,
        "decrease_difficulty" => Recommendation::DecreaseDifficulty,
        "increase_phi" => Recommendation::IncreasePhi,
        "improve_consistency" => Recommendation::ImproveConsistency,
        _ => Recommendation::Maintain,
    };

    HealthReport {
        health_score,
        recommendation,
        win_rate: stats.win_rate,
        avg_reward: stats.avg_reward,
        phi_avg,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_sweet_spot() {
        let stats = BufferStats::new(0.60, 0.5, 0.1, 50);
        let health = compute_health(&stats, 0.4);
        // Should be high: win rate in sweet spot, positive reward, decent phi
        assert!(health > 0.6);
    }

    #[test]
    fn test_health_low_win_rate() {
        let stats = BufferStats::new(0.20, -0.5, 0.3, 30);
        let health = compute_health(&stats, 0.1);
        assert!(health < 0.4);
    }

    #[test]
    fn test_health_high_win_rate() {
        // Too easy: win rate near 1.0
        let stats = BufferStats::new(0.95, 0.8, 0.05, 50);
        let health = compute_health(&stats, 0.5);
        assert!(health < 0.8);
    }

    #[test]
    fn test_health_zero_stats() {
        let stats = BufferStats::new(0.0, 0.0, 0.0, 0);
        let health = compute_health(&stats, 0.0);
        assert!(health >= 0.0 && health <= 1.0);
    }

    #[test]
    fn test_recommend_high_health() {
        let report = HealthReport {
            health_score: 0.85,
            recommendation: Recommendation::Maintain,
            win_rate: 0.65,
            avg_reward: 0.5,
            phi_avg: 0.4,
        };
        assert_eq!(recommend_action(&report), "increase_difficulty");
    }

    #[test]
    fn test_recommend_low_health_low_phi() {
        let report = HealthReport {
            health_score: 0.25,
            recommendation: Recommendation::Maintain,
            win_rate: 0.30,
            avg_reward: 0.1,
            phi_avg: 0.1,
        };
        assert_eq!(recommend_action(&report), "increase_phi");
    }

    #[test]
    fn test_recommend_low_health_low_reward() {
        let report = HealthReport {
            health_score: 0.30,
            recommendation: Recommendation::Maintain,
            win_rate: 0.25,
            avg_reward: -0.5,
            phi_avg: 0.3,
        };
        assert_eq!(recommend_action(&report), "decrease_difficulty");
    }

    #[test]
    fn test_recommend_moderate() {
        let report = HealthReport {
            health_score: 0.50,
            recommendation: Recommendation::Maintain,
            win_rate: 0.50,
            avg_reward: 0.0,
            phi_avg: 0.4,
        };
        assert_eq!(recommend_action(&report), "maintain");
    }

    #[test]
    fn test_generate_health_report() {
        let stats = BufferStats::new(0.60, 0.3, 0.15, 40);
        let report = generate_health_report(&stats, 0.35);
        assert!(report.health_score >= 0.0 && report.health_score <= 1.0);
        assert!(report.win_rate == 0.60);
    }

    #[test]
    fn test_recommendation_as_str() {
        assert_eq!(Recommendation::IncreaseDifficulty.as_str(), "increase_difficulty");
        assert_eq!(Recommendation::Maintain.as_str(), "maintain");
        assert_eq!(Recommendation::DecreaseDifficulty.as_str(), "decrease_difficulty");
        assert_eq!(Recommendation::IncreasePhi.as_str(), "increase_phi");
        assert_eq!(Recommendation::ImproveConsistency.as_str(), "improve_consistency");
    }
}
