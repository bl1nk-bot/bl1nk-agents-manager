use crate::hooks::rules_injector::types::RuleMetadata;
use regex::Regex;
use std::sync::OnceLock;

pub struct RuleFrontmatterResult {
    pub metadata: RuleMetadata,
    pub body: String,
}

pub fn parse_rule_frontmatter(content: &str) -> RuleFrontmatterResult {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(?s)^---\r?\n(.*?)\r?\n---\r?\n?(.*)$").unwrap());

    if let Some(caps) = re.captures(content) {
        let yaml_content = caps.get(1).map_or("", |m| m.as_str());
        let body = caps.get(2).map_or("", |m| m.as_str());

        let metadata = parse_yaml_simple(yaml_content);
        RuleFrontmatterResult {
            metadata,
            body: body.to_string(),
        }
    } else {
        RuleFrontmatterResult {
            metadata: RuleMetadata::default(),
            body: content.to_string(),
        }
    }
}

fn parse_yaml_simple(yaml: &str) -> RuleMetadata {
    let mut metadata = RuleMetadata::default();
    let lines: Vec<&str> = yaml.lines().collect();
    
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || !line.contains(':') {
            i += 1;
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ':').collect();
        let key = parts[0].trim();
        let value = parts[1].trim();

        match key {
            "description" => metadata.description = Some(clean_value(value)),
            "alwaysApply" => metadata.always_apply = Some(value == "true"),
            "globs" | "paths" | "applyTo" => {
                // Simplified array/string parsing logic
                let mut globs = Vec::new();
                if value.starts_with('[') && value.ends_with(']') {
                    // Inline array: ["a", "b"]
                    let inner = &value[1..value.len()-1];
                    for item in inner.split(',') {
                        globs.push(clean_value(item.trim()));
                    }
                } else if value.is_empty() {
                    // Multi-line array:
                    //   - item
                    let mut j = i + 1;
                    while j < lines.len() {
                        let next_line = lines[j];
                        if next_line.trim().starts_with("- ") {
                            globs.push(clean_value(&next_line.trim()[2..]));
                            j += 1;
                        } else if next_line.trim().is_empty() {
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    i = j - 1;
                } else {
                    // Comma separated or single string
                    for item in value.split(',') {
                        globs.push(clean_value(item.trim()));
                    }
                }
                
                let mut existing = metadata.globs.unwrap_or_default();
                existing.extend(globs);
                metadata.globs = Some(existing);
            }
            _ => {} // Ignore unknown keys
        }
        i += 1;
    }
    metadata
}

fn clean_value(v: &str) -> String {
    v.trim_matches(|c| c == '"' || c == '\'').to_string()
}
