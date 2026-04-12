pub mod types;
pub mod detector;
pub mod switcher;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::hooks::think_mode::types::ThinkModeState;
use crate::hooks::think_mode::detector::detect_think_keyword;
use crate::hooks::think_mode::switcher::get_high_variant;

pub struct ThinkModeHook {
    states: Arc<RwLock<HashMap<String, ThinkModeState>>>,
}

impl ThinkModeHook {
    pub fn new() -> Self {
        Self { states: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn on_chat_params(
        &self,
        session_id: &str,
        prompt_text: &str,
        current_model_id: &mut String,
    ) -> anyhow::Result<()> {
        let mut state = ThinkModeState {
            requested: false,
            model_switched: false,
            thinking_config_injected: false,
            provider_id: None,
            model_id: Some(current_model_id.clone()),
        };

        if !detect_think_keyword(prompt_text) {
            self.states.write().await.insert(session_id.to_string(), state);
            return Ok(());
        }

        state.requested = true;

        if let Some(high_variant) = get_high_variant(current_model_id) {
            *current_model_id = high_variant;
            state.model_switched = true;
        }

        self.states.write().await.insert(session_id.to_string(), state);
        Ok(())
    }
}
