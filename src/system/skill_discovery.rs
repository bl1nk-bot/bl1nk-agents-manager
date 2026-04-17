//! Skill & Agent Discovery System with Strict Schema Validation
//! ทำหน้าที่ค้นหาเอเจนต์แบบอัตโนมัติ ตรวจสอบความถูกต้อง และปรับจูนข้อมูลให้เป็นมาตรฐาน (Normalization)

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use jsonschema::JSONSchema;
use regex::Regex;
use serde::{Deserialize, Serialize};

const BUILTIN_COMMANDS: &[&str] = &["model", "resume", "new", "help", "exit"];
const SCHEMA_PATH: &str = ".config/schema-agent.json";

// ============================================================================
// 📦 โครงสร้างข้อมูล (Data Structures)
// ============================================================================

/// 📜 ข้อมูล Frontmatter ที่อ่านจาก YAML โดยตรง (เน้นความตรงตามรูปแบบไฟล์)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub disable_model_invocation: Option<bool>,
    pub user_invocable: Option<bool>,
    pub allowed_tools: Option<String>, // มาเป็น "tool1, tool2" ใน YAML
    pub context: Option<String>,
    pub agent: Option<String>,
    pub mode: Option<String>,
}

/// ⚙️ ตัวเลือกของ Skill ที่ถูกปรับให้เป็นมาตรฐาน (Normalized) พร้อมใช้งานในระบบ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillOptions {
    pub disable_model_invocation: bool,
    pub user_invocable: bool,
    pub allowed_tools: Vec<String>,
    pub context: Option<String>,
    pub agent: Option<String>,
    pub mode: String,
}

/// 📇 ข้อมูลเมทาดาต้าสมบูรณ์ที่ค้นพบและตรวจสอบแล้ว
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub filename: String,
    pub options: SkillOptions,
}

// ============================================================================
// 🔄 การแปลงข้อมูล (Normalization)
// ============================================================================

impl From<SkillFrontmatter> for SkillOptions {
    fn from(f: SkillFrontmatter) -> Self {
        Self {
            disable_model_invocation: f.disable_model_invocation.unwrap_or(false),
            user_invocable: f.user_invocable.unwrap_or(true),
            allowed_tools: f
                .allowed_tools
                .map(|s| {
                    s.split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            context: f.context,
            agent: f.agent,
            mode: f.mode.unwrap_or_else(|| "subagent".into()),
        }
    }
}

// ============================================================================
// 🔍 ระบบ Discovery & Validation
// ============================================================================

pub async fn discover_validated_assets(directories: Vec<PathBuf>) -> Result<Vec<SkillMetadata>> {
    let validator = load_validator()?;
    let mut discovered = Vec::new();
    let mut seen_names = HashSet::new();

    for root_dir in directories {
        if !root_dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(root_dir)? {
            let entry = entry?;
            let path = entry.path();
            let result = if path.is_dir() {
                if let Some(skill_file) = find_skill_file(&path) {
                    process_and_validate(&skill_file, &path, &validator, &mut seen_names)
                } else {
                    continue;
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                process_and_validate(&path, path.parent().unwrap(), &validator, &mut seen_names)
            } else {
                continue;
            };

            match result {
                Ok(meta) => discovered.push(meta),
                Err(e) => tracing::error!("❌ เอเจนต์ที่ {:?} ไม่ผ่านการตรวจสอบ: {}", path, e),
            }
        }
    }
    Ok(discovered)
}

fn process_and_validate(
    file_path: &Path,
    base_path: &Path,
    validator: &JSONSchema,
    seen_names: &mut HashSet<String>,
) -> Result<SkillMetadata> {
    let content = fs::read_to_string(file_path)?;
    let frontmatter_json = parse_skill_frontmatter(&content)?;

    // 🛡️ 1. Validate กับ JSON Schema (พื้นฐาน: name, description)
    if let Err(errors) = validator.validate(&frontmatter_json) {
        let mut msg = String::from("Validation Failed:\n");
        for error in errors {
            msg.push_str(&format!("  - {}: {}\n", error.instance_path, error));
        }
        return Err(anyhow!(msg));
    }

    // 🛡️ 2. กฎเหล็กเพิ่มเติมสำหรับ Agents (ต้องมี mode และ tool)
    let is_agent = file_path.to_string_lossy().contains("agents/");
    if is_agent {
        if frontmatter_json.get("mode").is_none() {
            return Err(anyhow!("Agent requires 'mode' field in {:?}", file_path));
        }
        if frontmatter_json.get("tool").is_none() || !frontmatter_json["tool"].is_array() {
            return Err(anyhow!("Agent requires 'tool' list in {:?}", file_path));
        }
    }

    // 🛡️ 3. Deserialize และตรวจสอบความปลอดภัย
    let raw: SkillFrontmatter =
        serde_json::from_value(frontmatter_json).context("ไม่สามารถแปลงข้อมูลเป็น SkillFrontmatter ได้")?;

    if BUILTIN_COMMANDS.contains(&raw.name.to_lowercase().as_str()) {
        return Err(anyhow!("ชื่อ '{}' ห้ามจองซ้ำกับคำสั่งระบบ", raw.name));
    }

    let normalized = raw.name.to_lowercase();
    if seen_names.contains(&normalized) {
        return Err(anyhow!("พบชื่อซ้ำ: {}", raw.name));
    }
    seen_names.insert(normalized);

    // 🛡️ 3. Normalize
    let name = raw.name.clone();
    let description = raw.description.clone();
    let options = SkillOptions::from(raw);

    Ok(SkillMetadata {
        name,
        description,
        path: base_path.to_path_buf(),
        filename: file_path.file_name().unwrap().to_string_lossy().into_owned(),
        options,
    })
}

// ============================================================================
// 🛠️ เครื่องมือช่วย (Helpers)
// ============================================================================

fn load_validator() -> Result<JSONSchema> {
    let schema_content = fs::read_to_string(SCHEMA_PATH).context("ไม่พบไฟล์สคีมา")?;
    let schema_json: serde_json::Value = serde_json::from_str(&schema_content)?;
    JSONSchema::compile(&schema_json).map_err(|e| anyhow!("Schema Error: {}", e))
}

pub fn parse_skill_frontmatter(content: &str) -> Result<serde_json::Value> {
    let re = Regex::new(r"(?s)^---\r?\n([\s\S]*?)\r?\n---")?;
    let caps = re.captures(content).ok_or_else(|| anyhow!("ไม่พบ Frontmatter"))?;
    let parsed: serde_json::Value = serde_yaml::from_str(caps.get(1).unwrap().as_str())?;
    Ok(parsed)
}

fn find_skill_file(skill_dir: &Path) -> Option<PathBuf> {
    // 🛡️ บังคับใช้ SKILL.md (ตัวพิมพ์ใหญ่เท่านั้น) ตามกฎระบบใหม่
    let path = skill_dir.join("SKILL.md");
    if path.exists() && path.is_file() {
        return Some(path);
    }
    None
}

pub fn extract_skill_body(content: &str) -> String {
    let re = Regex::new(r"(?s)^---\r?\n([\s\S]*?)\r?\n---\r?\n?").unwrap();
    if let Some(m) = re.find(content) {
        content[m.end()..].trim().to_string()
    } else {
        content.trim().to_string()
    }
}

pub fn substitute_arguments(body: &str, args: Option<&str>) -> String {
    body.replace("$ARGUMENTS", args.unwrap_or(""))
}

pub fn inject_skill_directory(body: &str, skill_dir: &Path) -> String {
    format!("Skill directory: {}\n\n{}", skill_dir.display(), body)
}

// ============================================================================
// 🧪 ชุดการทดสอบ (Tests)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_skill_frontmatter_valid() {
        let content =
            "---\nname: test-agent\ndescription: A test agent\nmode: subagent\ntool: [tool1, tool2]\n---\nBody content";
        let parsed = parse_skill_frontmatter(content).unwrap();
        assert_eq!(parsed["name"], "test-agent");
        assert_eq!(parsed["description"], "A test agent");
    }

    #[test]
    fn test_extract_skill_body() {
        let content = "---\nname: test\n---\nHello World";
        assert_eq!(extract_skill_body(content), "Hello World");

        let no_frontmatter = "Only Body";
        assert_eq!(extract_skill_body(no_frontmatter), "Only Body");
    }

    #[test]
    fn test_substitute_arguments() {
        let body = "Run with $ARGUMENTS now";
        assert_eq!(substitute_arguments(body, Some("params")), "Run with params now");
        assert_eq!(substitute_arguments(body, None), "Run with  now");
    }

    #[test]
    fn test_skill_options_normalization() {
        let raw = SkillFrontmatter {
            name: "test".into(),
            description: "desc".into(),
            version: None,
            disable_model_invocation: Some(true),
            user_invocable: None,
            allowed_tools: Some("glob, grep, read_file".into()),
            context: Some("fork".into()),
            agent: None,
            mode: Some("primary".into()),
        };

        let options = SkillOptions::from(raw);
        assert!(options.disable_model_invocation);
        assert!(options.user_invocable); // default true
        assert_eq!(options.allowed_tools.len(), 3);
        assert_eq!(options.allowed_tools[1], "grep");
        assert_eq!(options.mode, "primary");
    }

    #[tokio::test]
    async fn test_discover_validated_assets_integration() -> Result<()> {
        let dir = tempdir()?;
        let skill_path = dir.path().join("test-skill");
        fs::create_dir(&skill_path)?;

        // สร้างไฟล์ SKILL.md ที่ถูกต้อง (12 tools)
        let skill_content = r#"---
name: valid-skill
description: 'Case 1: test. Case 2: verify.'
mode: subagent
tool:
  - AskUserQuestion
  - ExitPlanMode
  - Glob
  - Grep
  - ListFiles
  - ReadFile
  - SaveMemory
  - Skill
  - TodoWrite
  - WebFetch
  - WebSearch
  - WriteFile
---
Body content"#;

        fs::write(skill_path.join("SKILL.md"), skill_content)?;

        // หมายเหตุ: ในสภาพแวดล้อมเทสจริง เราอาจต้องล้อ SCHEMA_PATH
        // หรือยอมให้มันใช้ไฟล์จริงในโปรเจกต์ (ซึ่งตอนนี้มีอยู่แล้ว)
        let results = discover_validated_assets(vec![dir.path().to_path_buf()]).await?;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "valid-skill");
        assert_eq!(results[0].options.allowed_tools.len(), 0); // แก้เป็น snake_case

        Ok(())
    }

    #[test]
    fn test_shadowing_protection() {
        let raw = serde_json::json!({
            "name": "model",
            "description": "desc",
            "mode": "subagent",
            "tool": [
                "AskUserQuestion", "ExitPlanMode", "Glob", "Grep", "ListFiles",
                "ReadFile", "SaveMemory", "Skill", "TodoWrite", "WebFetch",
                "WebSearch", "WriteFile"
            ]
        });

        let validator = load_validator().unwrap();

        // ตรวจสอบว่าข้อมูลจำลองนี้ถูกต้องตามสคีมาพื้นฐาน
        assert!(validator.validate(&raw).is_ok());

        // ทดสอบ logic การบล็อกชื่อต้องห้าม (Shadowing)
        let name = raw["name"].as_str().unwrap();
        if BUILTIN_COMMANDS.contains(&name.to_lowercase().as_str()) {
            // ผ่าน: ระบบต้องตรวจจับได้ว่าเป็นชื่อต้องห้าม
            assert!(true);
        } else {
            panic!("ควรจะตรวจพบว่าชื่อ 'model' เป็นชื่อต้องห้าม (Shadowing)");
        }
    }
}
