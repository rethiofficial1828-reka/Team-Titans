//! Q-learning agent for adaptive decoy placement.
//! Based on Wang et al. (IEEE Access 2020) "Intelligent Deployment Policy for
//! Deception Resources via RL" and TechRxiv 2024 "Automated File Trap Selection".
//!
//! STATE  = discretized (dir_depth_bucket, file_density_bucket)
//! ACTION = decoy placement strategy (Sparse, DFS_First, Dense, EntropyHotspot)
//! REWARD = +100 if trip happened with <=1 file lost; scaled down as loss grows;
//!          -50 if attacker reached real files before any decoy.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action { Sparse, DfsFirst, Dense, EntropyHotspot }

const ACTIONS: [Action; 4] = [Action::Sparse, Action::DfsFirst, Action::Dense, Action::EntropyHotspot];

#[derive(Serialize, Deserialize, Clone)]
pub struct QLearner {
    q: HashMap<(u8, u8, u8), f64>,   // (depth_bucket, density_bucket, action) -> Q
    pub alpha: f64,                  // learning rate
    pub gamma: f64,                  // discount
    pub epsilon: f64,                // exploration
    pub total_episodes: u32,
}

impl Default for QLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl QLearner {
    pub fn new() -> Self {
        Self { 
            q: HashMap::new(), 
            alpha: 0.5, 
            gamma: 0.9, 
            epsilon: 0.2,
            total_episodes: 0 
        }
    }

    fn key(depth: u8, density: u8, a: Action) -> (u8, u8, u8) {
        (depth, density, a as u8)
    }

    /// ε-greedy action selection for a directory's (depth, density) state.
    pub fn choose(&self, depth: u8, density: u8) -> Action {
        if rand::random::<f64>() < self.epsilon {
            ACTIONS[rand::random::<usize>() % 4]        // explore
        } else {
            *ACTIONS.iter()                              // exploit
                .max_by(|a, b| {
                    let qa = self.q.get(&Self::key(depth, density, **a)).unwrap_or(&0.0);
                    let qb = self.q.get(&Self::key(depth, density, **b)).unwrap_or(&0.0);
                    qa.partial_cmp(qb).unwrap()
                }).unwrap()
        }
    }

    /// Reward shaping: caught early = high reward. Bellman update.
    pub fn learn(&mut self, depth: u8, density: u8, a: Action, files_lost: u32, tripped: bool) {
        let reward = if !tripped { -50.0 }
                     else { (100.0 - 20.0 * files_lost as f64).max(10.0) };

        let key = Self::key(depth, density, a);
        let old = *self.q.get(&key).unwrap_or(&0.0);
        // Terminal-ish update (single-step episode per incident)
        let new = old + self.alpha * (reward - old);
        self.q.insert(key, new);

        // Decay exploration as we learn (converge toward exploitation)
        self.epsilon = (self.epsilon * 0.995).max(0.02);
        self.total_episodes += 1;
    }
    
    pub fn get_q_table(&self) -> &HashMap<(u8, u8, u8), f64> {
        &self.q
    }
}
