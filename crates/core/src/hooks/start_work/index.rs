use std::collections::HashMap;
use serde_json::Value;
use tokio;

use crate::hooks::start_work::types::{StartWorkInput, StartWorkOutput, MessagePart};
use crate::hooks::start_work::storage::{read_boulder_state, write_boulder_state, append_session_id, find_prometheus_plans, get_plan_progress, create_boulder_state, get_plan_name, clear_boulder_state};
use crate::hooks::start_work::constants::{HOOK_NAME, KEYWORD_PATTERN, CONTEXT_TAG, SYSTEM_REMINDER_OPEN, SYSTEM_REMINDER_CLOSE, PLAN_FILE_EXTENSION, BOULDER_STATE_FILE, PROMETHEUS_PLANS_DIR, SESSION_ID_PLACEHOLDER, TIMESTAMP_PLACEHOLDER};
use crate::hooks::claude_code_session_state::update_session_agent;

pub struct StartWorkHook {
    directory: String,
}

impl StartWorkHook {
    pub fn new(directory: String) -> Self {
        Self { directory }
    }

    pub async fn on_chat_message(
        &self,
        input: &StartWorkInput,
        output: &mut StartWorkOutput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let prompt_text = output.parts
            .iter()
            .filter(|p| p.part_type == "text" && p.text.is_some())
            .map(|p| p.text.as_ref().unwrap().as_str())
            .collect::<Vec<&str>>()
            .join("\n")
            .trim()
            .to_string();

        // Only trigger on actual command execution (contains <session-context> tag)
        // NOT on description text like "Start Sisyphus work session from Prometheus plan"
        let is_start_work_command = prompt_text.contains(CONTEXT_TAG);

        if !is_start_work_command {
            return Ok(());
        }

        log::info!("Processing start-work command. sessionID={}", &input.session_id);

        update_session_agent(&input.session_id, "atlas").await;

        let existing_state = read_boulder_state(&self.directory).await;
        let session_id = &input.session_id;
        let timestamp = chrono::Utc::now().to_rfc3339();

        let mut context_info = String::new();

        let explicit_plan_name = extract_user_request_plan_name(&prompt_text);

        if let Some(ref explicit_plan_name) = explicit_plan_name {
            log::info!("Explicit plan name requested. planName={}, sessionID={}", explicit_plan_name, session_id);

            let all_plans = find_prometheus_plans(&self.directory).await;
            let matched_plan = find_plan_by_name(&all_plans, explicit_plan_name);

            if let Some(ref matched_plan) = matched_plan {
                let progress = get_plan_progress(matched_plan).await;

                if progress.is_complete {
                    context_info = format!(
                        "## Plan Already Complete\n\n\
                         The requested plan \"{}\" has been completed.\n\
                         All {} tasks are done. Create a new plan with: /plan \"your task\"",
                        get_plan_name(matched_plan),
                        progress.total
                    );
                } else {
                    if existing_state.is_some() {
                        clear_boulder_state(&self.directory).await?;
                    }
                    let new_state = create_boulder_state(matched_plan, session_id).await;
                    write_boulder_state(&self.directory, &new_state).await?;

                    context_info = format!(
                        "## Auto-Selected Plan\n\n\
                         **Plan**: {}\n\
                         **Path**: {}\n\
                         **Progress**: {}/{} tasks\n\
                         **Session ID**: {}\n\
                         **Started**: {}\n\n\
                         {} has been created. Read the plan and begin execution.",
                        get_plan_name(matched_plan),
                        matched_plan,
                        progress.completed,
                        progress.total,
                        session_id,
                        timestamp,
                        BOULDER_STATE_FILE
                    );
                }
            } else {
                let incomplete_plans: Vec<String> = all_plans
                    .iter()
                    .filter(|p| {
                        let progress = get_plan_progress(p);
                        !progress.is_complete
                    })
                    .cloned()
                    .collect();

                if !incomplete_plans.is_empty() {
                    let plan_list = incomplete_plans
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let prog = get_plan_progress(p);
                            format!("{}. [{}] - Progress: {}/{}", 
                                   i + 1, 
                                   get_plan_name(p), 
                                   prog.completed, 
                                   prog.total)
                        })
                        .collect::<Vec<String>>()
                        .join("\n");

                    context_info = format!(
                        "## Plan Not Found\n\n\
                         Could not find a plan matching \"{}\".\n\n\
                         Available incomplete plans:\n{}",
                        explicit_plan_name,
                        plan_list
                    );
                } else {
                    context_info = format!(
                        "## Plan Not Found\n\n\
                         Could not find a plan matching \"{}\".\n\
                         No incomplete plans available. Create a new plan with: /plan \"your task\"",
                        explicit_plan_name
                    );
                }
            }
        } else if let Some(ref existing_state) = existing_state {
            let progress = get_plan_progress(&existing_state.active_plan).await;

            if !progress.is_complete {
                append_session_id(&self.directory, session_id).await?;
                context_info = format!(
                    "## Active Work Session Found\n\n\
                     **Status**: RESUMING existing work\n\
                     **Plan**: {}\n\
                     **Path**: {}\n\
                     **Progress**: {}/{} tasks completed\n\
                     **Sessions**: {} (current session appended)\n\
                     **Started**: {}\n\n\
                     The current session ({}) has been added to session_ids.\n\
                     Read the plan file and continue from the first unchecked task.",
                    existing_state.plan_name,
                    existing_state.active_plan,
                    progress.completed,
                    progress.total,
                    existing_state.session_ids.len() + 1,
                    existing_state.started_at,
                    session_id
                );
            } else {
                context_info = format!(
                    "## Previous Work Complete\n\n\
                     The previous plan ({}) has been completed.\n\
                     Looking for new plans...",
                    existing_state.plan_name
                );
            }
        } else {
            let plans = find_prometheus_plans(&self.directory).await;
            let incomplete_plans: Vec<String> = plans
                .iter()
                .filter(|p| {
                    let progress = get_plan_progress(p);
                    !progress.is_complete
                })
                .cloned()
                .collect();

            if plans.is_empty() {
                context_info.push_str("\n\n## No Plans Found\n\n\
                                      No Prometheus plan files found at .sisyphus/plans/\n\
                                      Use Prometheus to create a work plan first: /plan \"your task\"");
            } else if incomplete_plans.is_empty() {
                context_info.push_str(&format!("\n\n## All Plans Complete\n\n\
                                               All {} plan(s) are complete. Create a new plan with: /plan \"your task\"", 
                                               plans.len()));
            } else if incomplete_plans.len() == 1 {
                let plan_path = &incomplete_plans[0];
                let progress = get_plan_progress(plan_path).await;
                let new_state = create_boulder_state(plan_path, session_id).await;
                write_boulder_state(&self.directory, &new_state).await?;

                context_info.push_str(&format!("\n\n## Auto-Selected Plan\n\n\
                                               **Plan**: {}\n\
                                               **Path**: {}\n\
                                               **Progress**: {}/{} tasks\n\
                                               **Session ID**: {}\n\
                                               **Started**: {}\n\n\
                                               {} has been created. Read the plan and begin execution.",
                                               get_plan_name(plan_path),
                                               plan_path,
                                               progress.completed,
                                               progress.total,
                                               session_id,
                                               timestamp,
                                               BOULDER_STATE_FILE));
            } else {
                let plan_list = incomplete_plans
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let progress = get_plan_progress(p);
                        format!("{}. [{}] - Progress: {}/{}", 
                               i + 1, 
                               get_plan_name(p), 
                               progress.completed, 
                               progress.total)
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                context_info.push_str(&format!("\n\n{}{}Multiple Plans Found{}{}Current Time: {}{}Session ID: {}{}{}Ask the user which plan to work on. Present the options above and wait for their response.{}{}", 
                                               SYSTEM_REMINDER_OPEN,
                                               "\n\n## ", 
                                               "\n\n",
                                               "Current Time: ", 
                                               timestamp,
                                               "\n",
                                               session_id,
                                               "\n\n",
                                               plan_list,
                                               "\n\n",
                                               SYSTEM_REMINDER_CLOSE));
            }
        }

        // Find first text part and append context info
        for part in &mut output.parts {
            if part.part_type == "text" && part.text.is_some() {
                let mut text = part.text.as_ref().unwrap().clone();
                text = text.replace(SESSION_ID_PLACEHOLDER, session_id);
                text = text.replace(TIMESTAMP_PLACEHOLDER, &timestamp);
                text.push_str(&format!("\n\n---\n{}", context_info));
                part.text = Some(text);
                break;
            }
        }

        log::info!("Context injected. sessionID={}, hasExistingState={}", session_id, existing_state.is_some());

        Ok(())
    }
}

fn extract_user_request_plan_name(prompt_text: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?i)<user-request>\s*([\s\S]*?)\s*</user-request>").unwrap();
    if let Some(caps) = re.captures(prompt_text) {
        if let Some(raw_arg_match) = caps.get(1) {
            let raw_arg = raw_arg_match.as_str().trim();
            if !raw_arg.is_empty() {
                let cleaned_arg = regex::Regex::new(KEYWORD_PATTERN)
                    .unwrap()
                    .replace_all(raw_arg, "")
                    .trim()
                    .to_string();
                if !cleaned_arg.is_empty() {
                    return Some(cleaned_arg);
                }
            }
        }
    }
    None
}

fn find_plan_by_name(plans: &[String], requested_name: &str) -> Option<String> {
    let lower_name = requested_name.to_lowercase();

    // Exact match first
    for plan in plans {
        if get_plan_name(plan).await.to_lowercase() == lower_name {
            return Some(plan.clone());
        }
    }

    // Partial match
    for plan in plans {
        if get_plan_name(plan).await.to_lowercase().contains(&lower_name) {
            return Some(plan.clone());
        }
    }

    None
}

// get_plan_progress imported from storage

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio;

    #[tokio::test]
    async fn test_extract_user_request_plan_name() {
        let text = "<user-request>  my-plan-name  </user-request>";
        assert_eq!(extract_user_request_plan_name(text), Some("my-plan-name".to_string()));
        
        let text = "<user-request>  ultrawork my-plan  </user-request>";
        assert_eq!(extract_user_request_plan_name(text), Some("my-plan".to_string()));
    }

    #[tokio::test]
    async fn test_find_plan_by_name() {
        let plans = vec!["/path/to/plan1.md".to_string(), "/path/to/another-plan2.md".to_string()];
        assert_eq!(find_plan_by_name(&plans, "plan1").await, Some("/path/to/plan1.md".to_string()));
        assert_eq!(find_plan_by_name(&plans, "another").await, Some("/path/to/another-plan2.md".to_string()));
    }
}