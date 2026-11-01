use crate::config::{CategoryConfig, Bl1nkConfig};
use crate::config::handler::{ConfigContext, ConfigHandler, ConfigHandlerDeps, ModelCacheState};
use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Mirrors the TS config-handler entrypoint.
/// This module adapts extension-driven config merging to the Rust handler.

const CORE_AGENT_ORDER: [&str; 4] = ["sisyphus", "hephaestus", "prometheus", "atlas"];

pub fn resolve_category_config<'a>(
    category_name: &str,
    user_categories: Option<&'a HashMap<String, CategoryConfig>>,
    default_categories: &'a HashMap<String, CategoryConfig>,
) -> Option<&'a CategoryConfig> {
    if let Some(user) = user_categories.and_then(|m| m.get(category_name)) {
        return Some(user);
    }
    default_categories.get(category_name)
}

pub fn reorder_agents_by_priority(
    agents: &serde_json::Map<String, Value>,
) -> serde_json::Map<String, Value> {
    let mut ordered = serde_json::Map::new();
    let mut seen = HashSet::new();

    for key in CORE_AGENT_ORDER {
        if let Some(value) = agents.get(key) {
            ordered.insert(key.to_string(), value.clone());
            seen.insert(key.to_string());
        }
    }

    for (key, value) in agents.iter() {
        if !seen.contains(key) {
            ordered.insert(key.clone(), value.clone());
        }
    }

    ordered
}

/// Entry point for extension-based config handling.
/// This delegates to the existing Rust ConfigHandler implementation.
pub async fn handle_config(
    ctx: ConfigContext,
    plugin_config: Bl1nkConfig,
    model_cache_state: ModelCacheState,
    config: &mut Value,
) -> Result<()> {
    let deps = ConfigHandlerDeps {
        ctx,
        plugin_config,
        model_cache_state,
    };

    let mut handler = ConfigHandler::new(deps);
    handler.handle_config(config).await
}
