// src/agents/creator.rs
use crate::registry::schema::{AgentJsonEntry, PolicyRuleJson, Registry};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// รายละเอียดข้อกำหนดของเอเจนต์สำหรับการสร้างใหม่ (v1.7.2 Standard)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub name: String,
    pub description: String,
    pub mode: String,
    pub tools: Vec<String>,
    pub task_type: String,
    pub tier: u8,
    pub priority: u16,
    pub policies: Vec<PolicyRuleJson>,
    pub capabilities: Vec<String>,
    pub color: Option<String>,
    pub system_prompt: String,
}

/// ตัวสร้างเอเจนต์ - รองรับมาตรฐาน Gemini CLI Policy Engine
pub struct AgentCreator {
    output_dir: String,
}

#[derive(Debug, Serialize)]
struct MdFrontmatter<'a> {
    name: &'a str,
    description: &'a str,
    mode: &'a str,
    tool: Vec<String>,
}

impl AgentCreator {
    pub fn new(_template_dir: String, output_dir: String) -> Self {
        Self { output_dir }
    }

    pub fn write_agent_split(&self, id: &str, spec: AgentSpec) -> Result<()> {
        let md_path = format!("{}/{}.md", self.output_dir, id);
        let json_path = format!("{}/agents.json", self.output_dir);

        // 1. เขียนไฟล์ .md (Capability Metadata)
        let md_frontmatter = MdFrontmatter {
            name: id,
            description: &spec.description,
            mode: &spec.mode,
            tool: spec.tools.clone(),
        };
        let yaml_str = serde_yaml::to_string(&md_frontmatter)?;
        let md_content = format!("---\n{}---\n\n{}", yaml_str, spec.system_prompt);
        fs::write(&md_path, md_content)?;

        // 2. อัปเดต agents.json (Policy Metadata)
        let mut registry: Registry = if Path::new(&json_path).exists() {
            let data = fs::read_to_string(&json_path)?;
            serde_json::from_str(&data).unwrap_or_else(|_| Registry {
                version: "1.7.2".into(),
                last_updated: None,
                agents: vec![],
            })
        } else {
            Registry {
                version: "1.7.2".into(),
                last_updated: None,
                agents: vec![],
            }
        };

        let new_entry = AgentJsonEntry {
            name: id.to_string(),
            file: format!("{}.md", id),
            agent_type: spec.task_type,
            tier: spec.tier,
            priority: spec.priority,
            policies: spec.policies,
            capabilities: spec.capabilities,
            color: spec.color,
        };

        if let Some(pos) = registry.agents.iter().position(|a| a.name == id) {
            registry.agents[pos] = new_entry;
        } else {
            registry.agents.push(new_entry);
        }

        let json_str = serde_json::to_string_pretty(&registry)?;
        fs::write(json_path, json_str)?;

        Ok(())
    }
}
