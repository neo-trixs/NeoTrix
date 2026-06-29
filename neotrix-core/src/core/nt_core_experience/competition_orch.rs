// G397: Multi-role competition orchestrator — Agent-Loop-Skills tournament pattern
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompetitorRole {
    Proposer,
    Critic,
    Judge,
    Reviewer,
    Synthesizer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competitor {
    pub id: u64,
    pub name: String,
    pub role: CompetitorRole,
    pub score: f64,
    pub total_matches: u64,
    pub wins: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionProposal {
    pub id: u64,
    pub competitor_id: u64,
    pub description: String,
    pub predicted_gain: f64,
    pub realized_gain: Option<f64>,
    pub vsa_fingerprint: Vec<u8>,
    pub step: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeVerdict {
    pub proposal_id: u64,
    pub score: f64,
    pub critique: String,
    pub calibrated_score: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionRound {
    pub round: u64,
    pub proposals: Vec<CompetitionProposal>,
    pub verdicts: BTreeMap<u64, JudgeVerdict>,
    pub winner_id: Option<u64>,
    pub runner_up_id: Option<u64>,
    pub kept: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionOrchestrator {
    pub competitors: Vec<Competitor>,
    pub rounds: VecDeque<CompetitionRound>,
    pub max_rounds: usize,
    pub round_counter: u64,
    pub best_score: f64,
    pub best_proposal: Option<CompetitionProposal>,
    pub alpha: f64,
    pub epsilon: f64,
}

impl CompetitionOrchestrator {
    pub fn new() -> Self {
        Self {
            competitors: Vec::new(),
            rounds: VecDeque::with_capacity(100),
            max_rounds: 100,
            round_counter: 0,
            best_score: 0.0,
            best_proposal: None,
            alpha: 0.05,
            epsilon: 0.1,
        }
    }

    pub fn register_competitor(&mut self, name: &str, role: CompetitorRole) -> u64 {
        let id = self.competitors.len() as u64 + 1;
        self.competitors.push(Competitor {
            id,
            name: name.to_string(),
            role,
            score: 0.5,
            total_matches: 0,
            wins: 0,
        });
        id
    }

    pub fn submit_proposal(
        &mut self,
        competitor_id: u64,
        description: &str,
        predicted_gain: f64,
    ) -> Option<CompetitionProposal> {
        let _competitor = self.competitors.iter().find(|c| c.id == competitor_id)?;
        let id = self.round_counter * 100 + competitor_id;
        Some(CompetitionProposal {
            id,
            competitor_id,
            description: description.to_string(),
            predicted_gain,
            realized_gain: None,
            vsa_fingerprint: Vec::new(),
            step: self.round_counter,
        })
    }

    pub fn run_round(&mut self, proposals: Vec<CompetitionProposal>) -> CompetitionRound {
        self.round_counter += 1;
        let mut verdicts = BTreeMap::new();

        // Judge evaluates each proposal
        for proposal in &proposals {
            let raw_score = proposal.predicted_gain.max(0.0).min(1.0);
            let calibrated_score = self.calibrate(raw_score, proposal.competitor_id);
            let confidence = self.compute_confidence(proposal.competitor_id);
            verdicts.insert(
                proposal.id,
                JudgeVerdict {
                    proposal_id: proposal.id,
                    score: raw_score,
                    critique: self.generate_critique(proposal, calibrated_score),
                    calibrated_score,
                    confidence,
                },
            );
        }

        // Find winner (highest calibrated score)
        let winner_id = verdicts
            .iter()
            .max_by(|a, b| {
                a.1.calibrated_score
                    .partial_cmp(&b.1.calibrated_score)
                    .unwrap_or(Ordering::Equal)
            })
            .map(|(id, _)| *id);

        let runner_up_id = verdicts
            .iter()
            .filter(|(id, _)| Some(**id) != winner_id)
            .max_by(|a, b| {
                a.1.calibrated_score
                    .partial_cmp(&b.1.calibrated_score)
                    .unwrap_or(Ordering::Equal)
            })
            .map(|(id, _)| *id);

        // Update competitor scores
        if let Some(wid) = winner_id {
            if let Some(proposal) = proposals.iter().find(|p| p.id == wid) {
                if let Some(comp) = self
                    .competitors
                    .iter_mut()
                    .find(|c| c.id == proposal.competitor_id)
                {
                    comp.wins += 1;
                    comp.score = (1.0 - self.alpha) * comp.score + self.alpha * 1.0;
                }
            }
        }
        for proposal in &proposals {
            if let Some(comp) = self
                .competitors
                .iter_mut()
                .find(|c| c.id == proposal.competitor_id)
            {
                comp.total_matches += 1;
            }
        }

        let kept = winner_id.is_some();
        if kept {
            if let Some(wid) = winner_id {
                if let Some(proposal) = proposals.iter().find(|p| p.id == wid) {
                    if proposal.predicted_gain > self.best_score {
                        self.best_score = proposal.predicted_gain;
                        self.best_proposal = Some(proposal.clone());
                    }
                }
            }
        }

        let round = CompetitionRound {
            round: self.round_counter,
            proposals: proposals.clone(),
            verdicts,
            winner_id,
            runner_up_id,
            kept,
        };
        if self.rounds.len() >= self.max_rounds {
            self.rounds.pop_front();
        }
        self.rounds.push_back(round.clone());
        round
    }

    fn calibrate(&self, raw: f64, competitor_id: u64) -> f64 {
        let trust = self
            .competitors
            .iter()
            .find(|c| c.id == competitor_id)
            .map_or(0.5, |c| c.score);
        raw * (0.5 + trust * 0.5)
    }

    fn compute_confidence(&self, competitor_id: u64) -> f64 {
        self.competitors
            .iter()
            .find(|c| c.id == competitor_id)
            .map_or(0.5, |c| {
                if c.total_matches == 0 {
                    0.5
                } else {
                    c.wins as f64 / c.total_matches as f64
                }
            })
    }

    fn generate_critique(&self, proposal: &CompetitionProposal, calibrated: f64) -> String {
        format!(
            "Proposal[{}]: '{}' predicted_gain={:.3}, calibrated={:.3} — {}",
            proposal.id,
            &proposal.description[..proposal.description.len().min(40)],
            proposal.predicted_gain,
            calibrated,
            if calibrated > 0.6 {
                "Strong candidate"
            } else if calibrated > 0.3 {
                "Moderate potential"
            } else {
                "Weak signal, needs revision"
            }
        )
    }

    pub fn finalize_round(&mut self, round_idx: usize, realized_gains: &[(u64, f64)]) {
        if round_idx >= self.rounds.len() {
            return;
        }
        if let Some(round) = self.rounds.get_mut(round_idx) {
            for (pid, gain) in realized_gains {
                if let Some(proposal) = round.proposals.iter_mut().find(|p| p.id == *pid) {
                    proposal.realized_gain = Some(*gain);
                }
            }
            // Re-evaluate winner based on realized gains
            let best_realized = realized_gains
                .iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
            if let Some((pid, _)) = best_realized {
                round.winner_id = Some(*pid);
            }
        }
    }

    pub fn leaderboard(&self) -> Vec<(String, f64, u64, u64)> {
        let mut board: Vec<_> = self
            .competitors
            .iter()
            .map(|c| (c.name.clone(), c.score, c.wins, c.total_matches))
            .collect();
        board.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        board
    }
}
