use std::fs;
use std::path::{Path, PathBuf};
use crate::hooks::ralph_loop::types::RalphLoopState;
use crate::hooks::ralph_loop::constants::{DEFAULT_STATE_FILE, DEFAULT_MAX_ITERATIONS, DEFAULT_COMPLETION_PROMISE};

pub fn read_state(directory: &Path) -> Option<RalphLoopState> {
    let file_path = directory.join(DEFAULT_STATE_FILE);
    if !file_path.exists() { return None; }

    let content = fs::read_to_string(file_path).ok()?;
    // Simplified frontmatter parsing (You should use a proper parser in production)
    if !content.starts_with("---") { return None; }
    
    // Placeholder logic for extraction
    Some(RalphLoopState {
        active: true,
        iteration: 1,
        max_iterations: DEFAULT_MAX_ITERATIONS,
        completion_promise: DEFAULT_COMPLETION_PROMISE.to_string(),
        started_at: "".to_string(),
        prompt: "".to_string(),
        session_id: None,
        ultrawork: None,
    })
}

pub fn write_state(directory: &Path, state: &RalphLoopState) -> bool {
    let file_path = directory.join(DEFAULT_STATE_FILE);
    if let Some(parent) = file_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let content = format!(
        "---\nactive: {}\niteration: {}\nmax_iterations: {}\n---\n{}",
        state.active, state.iteration, state.max_iterations, state.prompt
    );
    fs::write(file_path, content).is_ok()
}

