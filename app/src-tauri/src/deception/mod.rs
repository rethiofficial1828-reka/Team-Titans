pub mod containment;
pub mod policy_engine;
pub mod entropy_monitor;

use std::sync::Mutex;
use tauri::State;
use serde::{Serialize, Deserialize};

use policy_engine::{QLearner, Action};

pub struct DeceptionState {
    pub learner: Mutex<QLearner>,
}

#[derive(Serialize)]
pub struct DeceptionStatus {
    pub epsilon: f64,
    pub total_episodes: u32,
    pub q_table_size: usize,
    pub latest_action: Option<String>,
}

#[tauri::command]
pub async fn get_deception_status(state: State<'_, DeceptionState>) -> Result<DeceptionStatus, String> {
    let learner = state.learner.lock().unwrap();
    Ok(DeceptionStatus {
        epsilon: learner.epsilon,
        total_episodes: learner.total_episodes,
        q_table_size: learner.get_q_table().len(),
        latest_action: None,
    })
}

#[derive(Deserialize)]
pub struct SimulateTripArgs {
    pub depth: u8,
    pub density: u8,
    pub files_lost: u32,
    pub tripped: bool,
}

#[tauri::command]
pub async fn simulate_deception_trip(
    state: State<'_, DeceptionState>,
    args: SimulateTripArgs,
) -> Result<String, String> {
    let mut learner = state.learner.lock().unwrap();
    let action = learner.choose(args.depth, args.density);
    
    // Simulate Bellman update
    learner.learn(args.depth, args.density, action, args.files_lost, args.tripped);
    
    let strategy_name = match action {
        Action::Sparse => "Sparse",
        Action::DfsFirst => "DFS_First (0_, aa_)",
        Action::Dense => "Dense",
        Action::EntropyHotspot => "EntropyHotspot",
    };
    
    // In a real scenario we would call containment::FrozenAttacker::contain(pid)
    // but here we just simulate the telemetry update.
    
    Ok(format!("Simulated trap! Strategy: {}. Reward applied via Bellman update.", strategy_name))
}
