use crate::config::schema::Bl1nkConfig;
use crate::config::schema::{AgentOverrides, CategoriesConfig, ClaudeCodeConfig};
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ConfigLoadError {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigLoadContext {
    pub errors: Vec<ConfigLoadError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Jsonc,
    None,
}

#[derive(Debug, Clone)]
pub struct DetectedConfigFile {
    pub path: PathBuf,
    pub format: ConfigFormat,
}

pub fn get_opencode_config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OPENCODE_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".bl1nk");
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home).join(".bl1nk");
    }
    PathBuf::from(".bl1nk")
}

pub fn detect_config_file(base_path: &Path) -> DetectedConfigFile {
    let jsonc_path = base_path.with_extension("jsonc");
    if jsonc_path.exists() {
        return DetectedConfigFile {
            path: jsonc_path,
            format: ConfigFormat::Jsonc,
        };
    }
    let json_path = base_path.with_extension("json");
    if json_path.exists() {
        return DetectedConfigFile {
            path: json_path,
            format: ConfigFormat::Json,
        };
    }
    DetectedConfigFile {
        path: base_path.with_extension("json"),
        format: ConfigFormat::None,
    }
}

fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            continue;
        }

        if c == '/' {
            if let Some('/') = chars.peek().copied() {
                chars.next();
                while let Some(next) = chars.next() {
                    if next == '\n' {
                        out.push('\n');
                        break;
                    }
                }
                continue;
            }
            if let Some('*') = chars.peek().copied() {
                chars.next();
                let mut prev = '\0';
                while let Some(next) = chars.next() {
                    if prev == '*' && next == '/' {
                        break;
                    }
                    prev = next;
                }
                continue;
            }
        }

        out.push(c);
    }

    out
}

pub fn parse_jsonc<T: DeserializeOwned>(input: &str) -> Result<T, serde_json::Error> {
    let stripped = strip_jsonc_comments(input);
    serde_json::from_str(&stripped)
}

pub fn migrate_config_file(_config_path: &Path, _raw_config: &serde_json::Value) {
    // Placeholder for future migrations.
}

pub fn load_config_from_path(
    config_path: &Path,
    ctx: &mut ConfigLoadContext,
) -> Option<Bl1nkConfig> {
    if !config_path.exists() {
        return None;
    }

    match fs::read_to_string(config_path) {
        Ok(content) => {
            let raw_config: serde_json::Value = match parse_jsonc(&content) {
                Ok(value) => value,
                Err(err) => {
                    ctx.errors.push(ConfigLoadError {
                        path: config_path.display().to_string(),
                        error: format!("Validation error: {err}"),
                    });
                    return None;
                }
            };

            migrate_config_file(config_path, &raw_config);

            match serde_json::from_value::<Bl1nkConfig>(raw_config) {
                Ok(config) => Some(config),
                Err(err) => {
                    ctx.errors.push(ConfigLoadError {
                        path: config_path.display().to_string(),
                        error: format!("Validation error: {err}"),
                    });
                    None
                }
            }
        }
        Err(err) => {
            ctx.errors.push(ConfigLoadError {
                path: config_path.display().to_string(),
                error: err.to_string(),
            });
            None
        }
    }
}

fn merge_vec_unique<T: Clone + std::cmp::Eq + std::hash::Hash>(
    base: Option<Vec<T>>,
    override_vec: Option<Vec<T>>,
) -> Option<Vec<T>> {
    let mut set = HashSet::new();
    if let Some(base_vec) = base {
        for item in base_vec {
            set.insert(item);
        }
    }
    if let Some(override_vec) = override_vec {
        for item in override_vec {
            set.insert(item);
        }
    }
    if set.is_empty() {
        None
    } else {
        Some(set.into_iter().collect())
    }
}

fn merge_agent_overrides(base: AgentOverrides, override_cfg: AgentOverrides) -> AgentOverrides {
    let mut merged = AgentOverrides::default();
    merged.overrides.extend(base.overrides);
    merged.overrides.extend(override_cfg.overrides);
    merged
}

fn merge_categories(
    base: Option<CategoriesConfig>,
    override_cfg: Option<CategoriesConfig>,
) -> Option<CategoriesConfig> {
    let mut merged = CategoriesConfig::new();
    if let Some(base_cfg) = base {
        merged.extend(base_cfg);
    }
    if let Some(override_cfg) = override_cfg {
        merged.extend(override_cfg);
    }
    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

fn merge_claude_code(
    base: Option<ClaudeCodeConfig>,
    override_cfg: Option<ClaudeCodeConfig>,
) -> Option<ClaudeCodeConfig> {
    match (base, override_cfg) {
        (None, None) => None,
        (Some(base_cfg), None) => Some(base_cfg),
        (None, Some(override_cfg)) => Some(override_cfg),
        (Some(base_cfg), Some(override_cfg)) => Some(ClaudeCodeConfig {
            mcp: override_cfg.mcp.or(base_cfg.mcp),
            commands: override_cfg.commands.or(base_cfg.commands),
            skills: override_cfg.skills.or(base_cfg.skills),
            agents: override_cfg.agents.or(base_cfg.agents),
            hooks: override_cfg.hooks.or(base_cfg.hooks),
            plugins: override_cfg.plugins.or(base_cfg.plugins),
            plugins_override: override_cfg.plugins_override.or(base_cfg.plugins_override),
        }),
    }
}

pub fn merge_configs(
    base: Bl1nkConfig,
    override_cfg: Bl1nkConfig,
) -> Bl1nkConfig {
    Bl1nkConfig {
        schema: override_cfg.schema.or(base.schema),
        new_task_system_enabled: override_cfg
            .new_task_system_enabled
            .or(base.new_task_system_enabled),
        default_run_agent: override_cfg.default_run_agent.or(base.default_run_agent),
        disabled_mcps: merge_vec_unique(base.disabled_mcps, override_cfg.disabled_mcps),
        disabled_agents: merge_vec_unique(base.disabled_agents, override_cfg.disabled_agents),
        disabled_skills: merge_vec_unique(base.disabled_skills, override_cfg.disabled_skills),
        disabled_hooks: merge_vec_unique(base.disabled_hooks, override_cfg.disabled_hooks),
        disabled_commands: merge_vec_unique(base.disabled_commands, override_cfg.disabled_commands),
        disabled_tools: merge_vec_unique(base.disabled_tools, override_cfg.disabled_tools),
        agents: merge_agent_overrides(base.agents, override_cfg.agents),
        categories: merge_categories(base.categories, override_cfg.categories),
        claude_code: merge_claude_code(base.claude_code, override_cfg.claude_code),
        sisyphus_agent: override_cfg.sisyphus_agent.or(base.sisyphus_agent),
        comment_checker: override_cfg.comment_checker.or(base.comment_checker),
        experimental: override_cfg.experimental.or(base.experimental),
        auto_update: override_cfg.auto_update.or(base.auto_update),
        skills: override_cfg.skills.or(base.skills),
        ralph_loop: override_cfg.ralph_loop.or(base.ralph_loop),
        background_task: override_cfg.background_task.or(base.background_task),
        notification: override_cfg.notification.or(base.notification),
        babysitting: override_cfg.babysitting.or(base.babysitting),
        git_master: override_cfg.git_master.or(base.git_master),
        browser_automation_engine: override_cfg
            .browser_automation_engine
            .or(base.browser_automation_engine),
        websearch: override_cfg.websearch.or(base.websearch),
        tmux: override_cfg.tmux.or(base.tmux),
        sisyphus: override_cfg.sisyphus.or(base.sisyphus),
        migrations: merge_vec_unique(base.migrations, override_cfg.migrations),
    }
}

pub fn load_plugin_config(directory: &Path) -> (Bl1nkConfig, ConfigLoadContext) {
    let config_dir = get_opencode_config_dir();
    let user_base_path = config_dir.join("bl1nk");
    let user_detected = detect_config_file(&user_base_path);
    let user_config_path = if user_detected.format != ConfigFormat::None {
        user_detected.path
    } else {
        user_base_path.with_extension("json")
    };

    let project_base_path = directory.join(".bl1nk").join("bl1nk");
    let project_detected = detect_config_file(&project_base_path);
    let project_config_path = if project_detected.format != ConfigFormat::None {
        project_detected.path
    } else {
        project_base_path.with_extension("json")
    };

    let mut ctx = ConfigLoadContext::default();

    let mut config = load_config_from_path(&user_config_path, &mut ctx)
        .unwrap_or_else(|| Bl1nkConfig::default());

    if let Some(project_config) = load_config_from_path(&project_config_path, &mut ctx) {
        config = merge_configs(config, project_config);
    }

    (config, ctx)
}
