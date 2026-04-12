use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio;
use regex::Regex;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::hooks::claude_code_hooks::types::ClaudeHookEvent;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisabledHooksConfig {
    #[serde(rename = "Stop", skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    
    #[serde(rename = "PreToolUse", skip_serializing_if = "Option::is_none")]
    pub pre_tool_use: Option<Vec<String>>,
    
    #[serde(rename = "PostToolUse", skip_serializing_if = "Option::is_none")]
    pub post_tool_use: Option<Vec<String>>,
    
    #[serde(rename = "UserPromptSubmit", skip_serializing_if = "Option::is_none")]
    pub user_prompt_submit: Option<Vec<String>>,
    
    #[serde(rename = "PreCompact", skip_serializing_if = "Option::is_none")]
    pub pre_compact: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExtendedConfig {
    #[serde(rename = "disabledHooks", skip_serializing_if = "Option::is_none")]
    pub disabled_hooks: Option<DisabledHooksConfig>,
}

// แคช regex สำหรับประสิทธิภาพ
lazy_static::lazy_static! {
    static ref REGEX_CACHE: Arc<RwLock<HashMap<String, Regex>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

const USER_CONFIG_PATH: &str = ".bl1nk/opencode-cc-plugin.json";

fn get_project_config_path() -> String {
    // ใช้ current working directory ร่วมกับ path ของ config
    format!("{}/.bl1nk/opencode-cc-plugin.json", std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .to_string_lossy())
}

async fn load_config_from_path(path: &str) -> Option<PluginExtendedConfig> {
    if !std::path::Path::new(path).exists() {
        return None;
    }

    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            match serde_json::from_str::<PluginExtendedConfig>(&content) {
                Ok(config) => Some(config),
                Err(e) => {
                    log::error!("Failed to parse config: {}, error: {}", path, e);
                    None
                }
            }
        },
        Err(e) => {
            log::error!("Failed to read config file: {}, error: {}", path, e);
            None
        }
    }
}

fn merge_disabled_hooks(
    base: Option<&DisabledHooksConfig>,
    override_config: Option<&DisabledHooksConfig>,
) -> DisabledHooksConfig {
    match (base, override_config) {
        (None, None) => DisabledHooksConfig::default(),
        (Some(_), None) => base.cloned().unwrap_or_default(),
        (None, Some(override_cfg)) => override_cfg.clone(),
        (Some(base_cfg), Some(override_cfg)) => {
            DisabledHooksConfig {
                stop: override_cfg.stop.clone().or_else(|| base_cfg.stop.clone()),
                pre_tool_use: override_cfg.pre_tool_use.clone().or_else(|| base_cfg.pre_tool_use.clone()),
                post_tool_use: override_cfg.post_tool_use.clone().or_else(|| base_cfg.post_tool_use.clone()),
                user_prompt_submit: override_cfg.user_prompt_submit.clone().or_else(|| base_cfg.user_prompt_submit.clone()),
                pre_compact: override_cfg.pre_compact.clone().or_else(|| base_cfg.pre_compact.clone()),
            }
        }
    }
}

pub async fn load_plugin_extended_config() -> PluginExtendedConfig {
    let user_config = load_config_from_path(USER_CONFIG_PATH).await;
    let project_config_path = get_project_config_path();
    let project_config = load_config_from_path(&project_config_path).await;

    let merged = PluginExtendedConfig {
        disabled_hooks: Some(merge_disabled_hooks(
            user_config.as_ref().and_then(|c| c.disabled_hooks.as_ref()),
            project_config.as_ref().and_then(|c| c.disabled_hooks.as_ref()),
        )),
    };

    if user_config.is_some() || project_config.is_some() {
        log::info!("Plugin extended config loaded. userConfigExists={}, projectConfigExists={}, mergedDisabledHooks={:?}", 
                  user_config.is_some(), 
                  project_config.is_some(), 
                  merged.disabled_hooks);
    }

    merged
}

fn get_regex(pattern: &str) -> Result<Regex, regex::Error> {
    // ตรวจสอบแคชก่อน
    {
        let cache = REGEX_CACHE.blocking_read();
        if let Some(regex) = cache.get(pattern) {
            return Ok(regex.clone());
        }
    }

    // สร้าง regex ใหม่
    let regex = Regex::new(pattern);
    if let Ok(re) = regex {
        // เก็บในแคช
        {
            let mut cache = REGEX_CACHE.blocking_write();
            cache.insert(pattern.to_string(), re.clone());
        }
        Ok(re)
    } else {
        // ถ้าสร้าง regex ไม่ได้ ให้ escape pattern และลองใหม่
        let escaped_pattern = regex::escape(pattern);
        let regex = Regex::new(&escaped_pattern)?;
        // เก็บในแคช
        {
            let mut cache = REGEX_CACHE.blocking_write();
            cache.insert(pattern.to_string(), regex.clone());
        }
        Ok(regex)
    }
}

pub async fn is_hook_command_disabled(
    event_type: &ClaudeHookEvent,
    command: &str,
    config: Option<&PluginExtendedConfig>,
) -> bool {
    let config = match config {
        Some(cfg) => cfg,
        None => return false,
    };

    let patterns = match &config.disabled_hooks {
        Some(disabled_hooks) => match event_type {
            ClaudeHookEvent::Stop => &disabled_hooks.stop,
            ClaudeHookEvent::PreToolUse => &disabled_hooks.pre_tool_use,
            ClaudeHookEvent::PostToolUse => &disabled_hooks.post_tool_use,
            ClaudeHookEvent::UserPromptSubmit => &disabled_hooks.user_prompt_submit,
            ClaudeHookEvent::PreCompact => &disabled_hooks.pre_compact,
        },
        None => return false,
    };

    let patterns = match patterns {
        Some(patterns) => patterns,
        None => return false,
    };

    if patterns.is_empty() {
        return false;
    }

    for pattern in patterns {
        if let Ok(regex) = get_regex(pattern) {
            if regex.is_match(command) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_hook_command_disabled() {
        // สร้าง config ตัวอย่าง
        let config = PluginExtendedConfig {
            disabled_hooks: Some(DisabledHooksConfig {
                stop: Some(vec!["test-command".to_string()]),
                ..Default::default()
            }),
        };

        // ทดสอบว่า command ถูกปิดใช้งาน
        let result = is_hook_command_disabled(&ClaudeHookEvent::Stop, "test-command", Some(&config)).await;
        assert!(result);

        // ทดสอบว่า command ไม่ถูกปิดใช้งาน
        let result = is_hook_command_disabled(&ClaudeHookEvent::Stop, "other-command", Some(&config)).await;
        assert!(!result);
    }
}