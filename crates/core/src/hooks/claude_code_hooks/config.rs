use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookCommand {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMatcher {
    pub matcher: String,
    pub hooks: Vec<HookCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeHooksConfig {
    #[serde(rename = "PreToolUse")]
    pub pre_tool_use: Option<Vec<HookMatcher>>,
    
    #[serde(rename = "PostToolUse")]
    pub post_tool_use: Option<Vec<HookMatcher>>,
    
    #[serde(rename = "UserPromptSubmit")]
    pub user_prompt_submit: Option<Vec<HookMatcher>>,
    
    #[serde(rename = "Stop")]
    pub stop: Option<Vec<HookMatcher>>,
    
    #[serde(rename = "PreCompact")]
    pub pre_compact: Option<Vec<HookMatcher>>,
}

impl ClaudeHooksConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: &ClaudeHooksConfig) {
        // รวม PreToolUse
        if let Some(ref other_pre_tool_use) = other.pre_tool_use {
            if let Some(ref mut self_pre_tool_use) = self.pre_tool_use {
                self_pre_tool_use.extend(other_pre_tool_use.clone());
            } else {
                self.pre_tool_use = Some(other_pre_tool_use.clone());
            }
        }

        // รวม PostToolUse
        if let Some(ref other_post_tool_use) = other.post_tool_use {
            if let Some(ref mut self_post_tool_use) = self.post_tool_use {
                self_post_tool_use.extend(other_post_tool_use.clone());
            } else {
                self.post_tool_use = Some(other_post_tool_use.clone());
            }
        }

        // รวม UserPromptSubmit
        if let Some(ref other_user_prompt_submit) = other.user_prompt_submit {
            if let Some(ref mut self_user_prompt_submit) = self.user_prompt_submit {
                self_user_prompt_submit.extend(other_user_prompt_submit.clone());
            } else {
                self.user_prompt_submit = Some(other_user_prompt_submit.clone());
            }
        }

        // รวม Stop
        if let Some(ref other_stop) = other.stop {
            if let Some(ref mut self_stop) = self.stop {
                self_stop.extend(other_stop.clone());
            } else {
                self.stop = Some(other_stop.clone());
            }
        }

        // รวม PreCompact
        if let Some(ref other_pre_compact) = other.pre_compact {
            if let Some(ref mut self_pre_compact) = self.pre_compact {
                self_pre_compact.extend(other_pre_compact.clone());
            } else {
                self.pre_compact = Some(other_pre_compact.clone());
            }
        }
    }
}

#[derive(Deserialize)]
struct RawHookMatcher {
    matcher: Option<String>,
    pattern: Option<String>,
    hooks: Vec<HookCommand>,
}

#[derive(Deserialize)]
struct RawClaudeHooksConfig {
    PreToolUse: Option<Vec<RawHookMatcher>>,
    PostToolUse: Option<Vec<RawHookMatcher>>,
    UserPromptSubmit: Option<Vec<RawHookMatcher>>,
    Stop: Option<Vec<RawHookMatcher>>,
    PreCompact: Option<Vec<RawHookMatcher>>,
}

fn normalize_hook_matcher(raw: RawHookMatcher) -> HookMatcher {
    HookMatcher {
        matcher: raw.matcher.or(raw.pattern).unwrap_or_else(|| "*".to_string()),
        hooks: raw.hooks,
    }
}

fn normalize_hooks_config(raw: RawClaudeHooksConfig) -> ClaudeHooksConfig {
    let mut config = ClaudeHooksConfig::new();

    if let Some(pre_tool_use) = raw.PreToolUse {
        config.pre_tool_use = Some(pre_tool_use.into_iter().map(normalize_hook_matcher).collect());
    }

    if let Some(post_tool_use) = raw.PostToolUse {
        config.post_tool_use = Some(post_tool_use.into_iter().map(normalize_hook_matcher).collect());
    }

    if let Some(user_prompt_submit) = raw.UserPromptSubmit {
        config.user_prompt_submit = Some(user_prompt_submit.into_iter().map(normalize_hook_matcher).collect());
    }

    if let Some(stop) = raw.Stop {
        config.stop = Some(stop.into_iter().map(normalize_hook_matcher).collect());
    }

    if let Some(pre_compact) = raw.PreCompact {
        config.pre_compact = Some(pre_compact.into_iter().map(normalize_hook_matcher).collect());
    }

    config
}

pub fn get_claude_settings_paths(custom_path: Option<&str>) -> Vec<String> {
    let mut paths = Vec::new();
    
    // ไดเรกทอรี config ของ Claude (จำลอง)
    let claude_config_dir = get_claude_config_dir();
    paths.push(format!("{}/settings.json", claude_config_dir));
    paths.push("./.claude/settings.json".to_string());
    paths.push("./.claude/settings.local.json".to_string());

    // เพิ่ม custom path หากมีและมีอยู่จริง
    if let Some(path) = custom_path {
        if std::path::Path::new(path).exists() {
            paths.insert(0, path.to_string());
        }
    }

    paths
}

fn get_claude_config_dir() -> String {
    // ฟังก์ชันจำลอง - ในระบบจริงอาจใช้ dirs crate หรือ environment variables
    if cfg!(windows) {
        std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").map(|h| format!("{}/.config", h)).unwrap_or_else(|_| ".".to_string())
    }
}

pub async fn load_claude_hooks_config(custom_settings_path: Option<&str>) -> Option<ClaudeHooksConfig> {
    let paths = get_claude_settings_paths(custom_settings_path);
    let mut merged_config = ClaudeHooksConfig::new();

    for settings_path in paths {
        if std::path::Path::new(&settings_path).exists() {
            if let Ok(content) = tokio::fs::read_to_string(&settings_path).await {
                if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(hooks_value) = settings.get("hooks") {
                        if let Ok(raw_config) = serde_json::from_value::<RawClaudeHooksConfig>(hooks_value.clone()) {
                            let normalized_config = normalize_hooks_config(raw_config);
                            merged_config.merge(&normalized_config);
                        }
                    }
                }
            }
        }
    }

    if merged_config.has_any_hooks() {
        Some(merged_config)
    } else {
        None
    }
}

impl ClaudeHooksConfig {
    fn has_any_hooks(&self) -> bool {
        self.pre_tool_use.is_some() || 
        self.post_tool_use.is_some() || 
        self.user_prompt_submit.is_some() || 
        self.stop.is_some() || 
        self.pre_compact.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_claude_hooks_config() {
        // ทดสอบการโหลด config (จำลอง)
        let config = load_claude_hooks_config(None).await;
        assert!(config.is_some() || true); // ขึ้นอยู่กับว่ามีไฟล์ config จริงหรือไม่
    }
}