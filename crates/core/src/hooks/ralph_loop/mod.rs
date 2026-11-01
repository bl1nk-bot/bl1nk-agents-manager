pub mod constants;
pub mod types;
pub mod storage;

use crate::hooks::ralph_loop::types::RalphLoopState;
use crate::hooks::ralph_loop::storage::{read_state, write_state};
use std::path::PathBuf;

pub struct RalphLoopHook {
    base_directory: PathBuf,
}

impl RalphLoopHook {
    pub fn new(base_directory: PathBuf) -> Self {
        Self { base_directory }
    }

    pub async fn on_session_idle(&self, session_id: &str) -> Option<String> {
        let state = read_state(&self.base_directory)?;
        if !state.active || state.iteration >= state.max_iterations { return None; }

        let mut next_state = state.clone();
        next_state.iteration += 1;
        write_state(&self.base_directory, &next_state);

        Some(format!("Looping... Iteration {}/{}", next_state.iteration, next_state.max_iterations))
    }
}
