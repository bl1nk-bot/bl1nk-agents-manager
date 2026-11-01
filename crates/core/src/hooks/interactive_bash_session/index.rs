use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;

use crate::hooks::interactive_bash_session::storage::{load_interactive_bash_session_state, save_interactive_bash_session_state, clear_interactive_bash_session_state};
use crate::hooks::interactive_bash_session::constants::{OMO_SESSION_PREFIX, build_session_reminder_message};
use crate::hooks::interactive_bash_session::types::InteractiveBashSessionState;

#[derive(Debug, Clone)]
pub struct InteractiveBashSessionHook {
    session_states: Arc<RwLock<HashMap<String, InteractiveBashSessionState>>>,
}

impl InteractiveBashSessionHook {
    pub fn new() -> Self {
        Self {
            session_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_state(&self, session_id: &str) -> InteractiveBashSessionState {
        let mut states = self.session_states.write().await;
        
        if !states.contains_key(session_id) {
            let persisted = load_interactive_bash_session_state(session_id).await;
            let state = if let Some(persisted_state) = persisted {
                persisted_state
            } else {
                InteractiveBashSessionState {
                    session_id: session_id.to_string(),
                    tmux_sessions: HashSet::new(),
                    updated_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                }
            };
            states.insert(session_id.to_string(), state);
        }
        
        states.get(session_id).unwrap().clone()
    }

    async fn update_state(&self, session_id: &str, state: InteractiveBashSessionState) {
        let mut states = self.session_states.write().await;
        states.insert(session_id.to_string(), state.clone());
        
        // บันทึกไปยัง storage
        if let Err(e) = save_interactive_bash_session_state(&state).await {
            log::error!("Failed to save interactive bash session state: {}", e);
        }
    }

    fn is_omo_session(session_name: Option<&String>) -> bool {
        if let Some(name) = session_name {
            name.starts_with(OMO_SESSION_PREFIX)
        } else {
            false
        }
    }

    async fn kill_all_tracked_sessions(&self, state: &InteractiveBashSessionState) {
        for session_name in &state.tmux_sessions {
            // ในระบบจริง ควรเรียกคำสั่ง tmux kill-session
            // สำหรับตอนนี้ เราจะแค่ log
            log::info!("Attempting to kill tmux session: {}", session_name);
        }
    }

    pub async fn tool_execute_after(&self, tool: &str, session_id: &str, args: &Option<HashMap<String, Value>>, output: &mut String) {
        let tool_lower = tool.to_lowercase();

        if tool_lower != "interactive_bash" {
            return;
        }

        // ตรวจสอบว่า args มี tmux_command หรือไม่
        let tmux_command = if let Some(ref args_map) = args {
            if let Some(tmux_cmd_val) = args_map.get("tmux_command") {
                if let Some(tmux_cmd_str) = tmux_cmd_val.as_str() {
                    tmux_cmd_str
                } else {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        };

        let tokens = tokenize_command(tmux_command);
        let sub_command = find_subcommand(&tokens);
        let mut state = self.get_or_create_state(session_id).await;
        let mut state_changed = false;

        if output.starts_with("Error:") {
            return;
        }

        let is_new_session = sub_command == "new-session";
        let is_kill_session = sub_command == "kill-session";
        let is_kill_server = sub_command == "kill-server";

        let session_name = extract_session_name_from_tokens(&tokens, &sub_command);

        if is_new_session && Self::is_omo_session(session_name.as_ref()) {
            if let Some(name) = session_name {
                state.tmux_sessions.insert(name);
                state_changed = true;
            }
        } else if is_kill_session && Self::is_omo_session(session_name.as_ref()) {
            if let Some(name) = session_name {
                state.tmux_sessions.remove(&name);
                state_changed = true;
            }
        } else if is_kill_server {
            state.tmux_sessions.clear();
            state_changed = true;
        }

        if state_changed {
            state.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            self.update_state(session_id, state.clone()).await;
        }

        let is_session_operation = is_new_session || is_kill_session || is_kill_server;
        if is_session_operation {
            let tmux_sessions_vec: Vec<String> = state.tmux_sessions.iter().cloned().collect();
            let reminder = build_session_reminder_message(&tmux_sessions_vec);
            if !reminder.is_empty() {
                *output += &reminder;
            }
        }
    }

    pub async fn handle_event(&self, event_type: &str, properties: Option<&HashMap<String, Value>>) {
        if event_type == "session.deleted" {
            if let Some(props) = properties {
                if let Some(info_val) = props.get("info") {
                    if let Some(info_obj) = info_val.as_object() {
                        if let Some(session_id_val) = info_obj.get("id") {
                            if let Some(session_id) = session_id_val.as_str() {
                                let state = self.get_or_create_state(session_id).await;
                                self.kill_all_tracked_sessions(&state).await;
                                
                                let mut states = self.session_states.write().await;
                                states.remove(session_id);
                                
                                if let Err(e) = clear_interactive_bash_session_state(session_id).await {
                                    log::error!("Failed to clear interactive bash session state: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/**
 * Quote-aware command tokenizer with escape handling
 * Handles single/double quotes and backslash escapes
 */
fn tokenize_command(cmd: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '\0';
    let mut escaped = false;

    for ch in cmd.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if (ch == '\'' || ch == '"') && !in_quote {
            in_quote = true;
            quote_char = ch;
        } else if ch == quote_char && in_quote {
            in_quote = false;
            quote_char = '\0';
        } else if ch == ' ' && !in_quote {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/**
 * Normalize session name by stripping :window and .pane suffixes
 * e.g., "omo-x:1" -> "omo-x", "omo-x:1.2" -> "omo-x"
 */
fn normalize_session_name(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    let base = parts[0];
    let base_parts: Vec<&str> = base.split('.').collect();
    base_parts[0].to_string()
}

fn find_flag_value(tokens: &[String], flag: &str) -> Option<String> {
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i] == flag {
            return Some(tokens[i + 1].clone());
        }
    }
    None
}

/**
 * Extract session name from tokens, considering the subCommand
 * For new-session: prioritize -s over -t
 * For other commands: use -t
 */
fn extract_session_name_from_tokens(tokens: &[String], sub_command: &str) -> Option<String> {
    if sub_command == "new-session" {
        if let Some(s_flag) = find_flag_value(tokens, "-s") {
            return Some(normalize_session_name(&s_flag));
        }
        if let Some(t_flag) = find_flag_value(tokens, "-t") {
            return Some(normalize_session_name(&t_flag));
        }
    } else {
        if let Some(t_flag) = find_flag_value(tokens, "-t") {
            return Some(normalize_session_name(&t_flag));
        }
    }
    None
}

/**
 * Find the tmux subcommand from tokens, skipping global options.
 * tmux allows global options before the subcommand:
 * e.g., `tmux -L socket-name new-session -s omo-x`
 * Global options with args: -L, -S, -f, -c, -T
 * Standalone flags: -C, -v, -V, etc.
 * Special: -- (end of options marker)
 */
fn find_subcommand(tokens: &[String]) -> String {
    // Options that require an argument: -L, -S, -f, -c, -T
    let global_options_with_args = ["-L", "-S", "-f", "-c", "-T"];

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        // Handle end of options marker
        if token == "--" {
            // Next token is the subcommand
            if i + 1 < tokens.len() {
                return tokens[i + 1].clone();
            } else {
                return String::new();
            }
        }

        if global_options_with_args.contains(&token.as_str()) {
            // Skip the option and its argument
            i += 2;
            continue;
        }

        if token.starts_with('-') {
            // Skip standalone flags like -C, -v, -V
            i += 1;
            continue;
        }

        // Found the subcommand
        return token.clone();
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_command() {
        let tokens = tokenize_command("tmux new-session -s 'session name' -d");
        assert_eq!(tokens, vec!["tmux", "new-session", "-s", "session name", "-d"]);
        
        let tokens = tokenize_command("tmux attach-session -t \"session-2\"");
        assert_eq!(tokens, vec!["tmux", "attach-session", "-t", "session-2"]);
    }

    #[test]
    fn test_normalize_session_name() {
        assert_eq!(normalize_session_name("omo-x:1"), "omo-x");
        assert_eq!(normalize_session_name("omo-x:1.2"), "omo-x");
        assert_eq!(normalize_session_name("omo-y"), "omo-y");
    }

    #[test]
    fn test_find_subcommand() {
        let tokens = vec!["tmux".to_string(), "-L".to_string(), "socket".to_string(), "new-session".to_string(), "-s".to_string(), "test".to_string()];
        assert_eq!(find_subcommand(&tokens), "new-session");
        
        let tokens = vec!["tmux".to_string(), "kill-session".to_string(), "-t".to_string(), "test".to_string()];
        assert_eq!(find_subcommand(&tokens), "kill-session");
    }

    #[test]
    fn test_extract_session_name_from_tokens() {
        let tokens = vec!["tmux".to_string(), "new-session".to_string(), "-s".to_string(), "omo-test".to_string()];
        assert_eq!(extract_session_name_from_tokens(&tokens, "new-session"), Some("omo-test".to_string()));
        
        let tokens = vec!["tmux".to_string(), "attach-session".to_string(), "-t".to_string(), "omo-test".to_string()];
        assert_eq!(extract_session_name_from_tokens(&tokens, "attach-session"), Some("omo-test".to_string()));
    }

    #[test]
    fn test_is_omo_session() {
        let session_name = Some("omo-test".to_string());
        assert!(InteractiveBashSessionHook::is_omo_session(session_name.as_ref()));
        
        let session_name = Some("other-test".to_string());
        assert!(!InteractiveBashSessionHook::is_omo_session(session_name.as_ref()));
        
        assert!(!InteractiveBashSessionHook::is_omo_session(None));
    }
}