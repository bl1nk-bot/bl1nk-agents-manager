// src/rule_parser.rs
//! พาร์เซอร์สำหรับกฎสิทธิ์ (Permission Rules) ตามรูปแบบ Claude Code
//! ใช้สำหรับตรวจสอบและสร้างกฎจากบริบทการเรียกใช้เครื่องมือของเอเจนต์

use regex::{Regex, Escape as _};
use glob::Pattern;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Types & Constants
// ─────────────────────────────────────────────────────────────────────────────

/// ประเภทของตัวระบุ (specifier) สำหรับกฎ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecifierKind {
    Command,  // shell command
    Path,     // file path
    Domain,   // web domain
    Literal,  // exact match (Skill, Agent)
}

/// กฎสิทธิ์ที่ผ่านการ parse แล้ว
#[derive(Debug, Clone)]
pub struct PermissionRule {
    pub raw: String,
    pub tool_name: String,               // canonical name
    pub specifier: Option<String>,
    pub specifier_kind: Option<SpecifierKind>,
}

/// บริบทสำหรับการตรวจสอบสิทธิ์
#[derive(Debug, Clone, Default)]
pub struct PermissionCheckContext {
    pub tool_name: String,
    pub command: Option<String>,
    pub file_path: Option<String>,
    pub domain: Option<String>,
    pub specifier: Option<String>,       // สำหรับ literal tools (Skill, Agent)
}

/// บริบทสำหรับ path matching (project root, cwd)
#[derive(Debug, Clone)]
pub struct PathMatchContext {
    pub project_root: PathBuf,
    pub cwd: PathBuf,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool aliases & categories (เหมือน TypeScript)
// ─────────────────────────────────────────────────────────────────────────────

lazy_static::lazy_static! {
    static ref TOOL_NAME_ALIASES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Shell
        m.insert("run_shell_command", "run_shell_command");
        m.insert("Shell", "run_shell_command");
        m.insert("ShellTool", "run_shell_command");
        m.insert("Bash", "run_shell_command");
        // Edit
        m.insert("edit", "edit");
        m.insert("Edit", "edit");
        m.insert("EditTool", "edit");
        // Write
        m.insert("write_file", "write_file");
        m.insert("WriteFile", "write_file");
        m.insert("WriteFileTool", "write_file");
        m.insert("Write", "write_file");
        // Read
        m.insert("read_file", "read_file");
        m.insert("ReadFile", "read_file");
        m.insert("ReadFileTool", "read_file");
        m.insert("Read", "read_file");
        // Grep
        m.insert("grep_search", "grep_search");
        m.insert("Grep", "grep_search");
        m.insert("GrepTool", "grep_search");
        m.insert("search_file_content", "grep_search");
        m.insert("SearchFiles", "grep_search");
        // Glob
        m.insert("glob", "glob");
        m.insert("Glob", "glob");
        m.insert("GlobTool", "glob");
        m.insert("FindFiles", "glob");
        // List directory
        m.insert("list_directory", "list_directory");
        m.insert("ListFiles", "list_directory");
        m.insert("ListFilesTool", "list_directory");
        m.insert("ReadFolder", "list_directory");
        // Memory
        m.insert("save_memory", "save_memory");
        m.insert("SaveMemory", "save_memory");
        m.insert("SaveMemoryTool", "save_memory");
        // Todo
        m.insert("todo_write", "todo_write");
        m.insert("TodoWrite", "todo_write");
        m.insert("TodoWriteTool", "todo_write");
        // Web
        m.insert("web_fetch", "web_fetch");
        m.insert("WebFetch", "web_fetch");
        m.insert("WebFetchTool", "web_fetch");
        m.insert("web_search", "web_search");
        m.insert("WebSearch", "web_search");
        m.insert("WebSearchTool", "web_search");
        // Agent
        m.insert("agent", "agent");
        m.insert("Agent", "agent");
        m.insert("AgentTool", "agent");
        m.insert("task", "agent");
        m.insert("Task", "agent");
        m.insert("TaskTool", "agent");
        // Skill
        m.insert("skill", "skill");
        m.insert("Skill", "skill");
        m.insert("SkillTool", "skill");
        // LSP
        m.insert("lsp", "lsp");
        m.insert("Lsp", "lsp");
        m.insert("LspTool", "lsp");
        // ExitPlanMode
        m.insert("exit_plan_mode", "exit_plan_mode");
        m.insert("ExitPlanMode", "exit_plan_mode");
        m.insert("ExitPlanModeTool", "exit_plan_mode");
        // Legacy
        m.insert("replace", "edit");
        m
    };

    static ref SHELL_TOOLS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("run_shell_command");
        s
    };

    static ref READ_TOOLS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("read_file");
        s.insert("grep_search");
        s.insert("glob");
        s.insert("list_directory");
        s
    };

    static ref EDIT_TOOLS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("edit");
        s.insert("write_file");
        s
    };

    static ref WEBFETCH_TOOLS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("web_fetch");
        s
    };

    static ref FILE_TARGETED_TOOLS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("read_file");
        s.insert("edit");
        s.insert("write_file");
        s
    };

    static ref CANONICAL_TO_RULE_DISPLAY: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("read_file", "Read");
        m.insert("grep_search", "Read");
        m.insert("glob", "Read");
        m.insert("list_directory", "Read");
        m.insert("edit", "Edit");
        m.insert("write_file", "Edit");
        m.insert("run_shell_command", "Bash");
        m.insert("web_fetch", "WebFetch");
        m.insert("web_search", "WebSearch");
        m.insert("agent", "Agent");
        m.insert("skill", "Skill");
        m.insert("save_memory", "SaveMemory");
        m.insert("todo_write", "TodoWrite");
        m.insert("lsp", "Lsp");
        m.insert("exit_plan_mode", "ExitPlanMode");
        m
    };

    static ref DISPLAY_NAME_TO_VERB: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Read", "read files");
        m.insert("Edit", "edit files");
        m.insert("Bash", "run commands");
        m.insert("WebFetch", "fetch from");
        m.insert("WebSearch", "search the web");
        m.insert("Agent", "use agent");
        m.insert("Skill", "use skill");
        m.insert("SaveMemory", "save memory");
        m.insert("TodoWrite", "write todos");
        m.insert("Lsp", "use LSP");
        m.insert("ExitPlanMode", "exit plan mode");
        m
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool name resolution & categorization
// ─────────────────────────────────────────────────────────────────────────────

pub fn resolve_tool_name(raw_name: &str) -> String {
    TOOL_NAME_ALIASES.get(raw_name).map(|&s| s.to_string()).unwrap_or_else(|| raw_name.to_string())
}

pub fn get_specifier_kind(canonical_tool_name: &str) -> SpecifierKind {
    if SHELL_TOOLS.contains(canonical_tool_name) {
        SpecifierKind::Command
    } else if READ_TOOLS.contains(canonical_tool_name) || EDIT_TOOLS.contains(canonical_tool_name) {
        SpecifierKind::Path
    } else if WEBFETCH_TOOLS.contains(canonical_tool_name) {
        SpecifierKind::Domain
    } else {
        SpecifierKind::Literal
    }
}

pub fn tool_matches_rule_tool_name(rule_tool_name: &str, context_tool_name: &str) -> bool {
    if rule_tool_name == context_tool_name {
        return true;
    }
    if rule_tool_name == "read_file" && READ_TOOLS.contains(context_tool_name) {
        return true;
    }
    if rule_tool_name == "edit" && EDIT_TOOLS.contains(context_tool_name) {
        return true;
    }
    false
}

pub fn get_rule_display_name(canonical_tool_name: &str) -> String {
    CANONICAL_TO_RULE_DISPLAY.get(canonical_tool_name).map(|&s| s.to_string()).unwrap_or_else(|| canonical_tool_name.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Rule parsing
// ─────────────────────────────────────────────────────────────────────────────

pub fn parse_rule(raw: &str) -> PermissionRule {
    let trimmed = raw.trim();
    // Handle legacy :* suffix
    let normalized = trimmed.replace(":*", " *");
    let open_paren = normalized.find('(');

    if open_paren.is_none() {
        let canonical = resolve_tool_name(&normalized);
        return PermissionRule {
            raw: trimmed.to_string(),
            tool_name: canonical,
            specifier: None,
            specifier_kind: None,
        };
    }

    let open = open_paren.unwrap();
    let tool_part = normalized[..open].trim();
    let specifier = if normalized.ends_with(')') {
        Some(normalized[open + 1..normalized.len() - 1].to_string())
    } else {
        None
    };

    let canonical = resolve_tool_name(tool_part);
    let kind = specifier.as_ref().map(|_| get_specifier_kind(&canonical));

    PermissionRule {
        raw: trimmed.to_string(),
        tool_name: canonical,
        specifier,
        specifier_kind: kind,
    }
}

pub fn parse_rules(raws: &[String]) -> Vec<PermissionRule> {
    raws.iter().filter(|r| !r.trim().is_empty()).map(|r| parse_rule(r)).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Shell command matching
// ─────────────────────────────────────────────────────────────────────────────

pub fn split_compound_command(command: &str) -> Vec<String> {
    let tokens = match shell_quote::parse::parse(command) {
        Ok(tokens) => tokens,
        Err(_) => return vec![command.to_string()],
    };

    let operators = ["&&", "||", ";;", "|&", "|", ";"];
    let mut commands = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        if operators.contains(&token.as_str()) {
            if !current.trim().is_empty() {
                commands.push(current.trim().to_string());
                current.clear();
            }
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(token);
        }
        i += 1;
    }
    if !current.trim().is_empty() {
        commands.push(current.trim().to_string());
    }
    if commands.is_empty() {
        vec![command.to_string()]
    } else {
        commands
    }
}

fn strip_leading_variable_assignments(cmd: &str) -> String {
    let re = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*=").unwrap();
    let mut stripped = cmd.to_string();
    while re.is_match(&stripped) {
        if let Some(idx) = stripped.find(' ') {
            stripped = stripped[idx + 1..].to_string();
        } else {
            break;
        }
    }
    stripped
}

pub fn matches_command_pattern(pattern: &str, command: &str) -> bool {
    let normalized = strip_leading_variable_assignments(command);
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return normalized == pattern || normalized.starts_with(&format!("{} ", pattern));
    }

    let mut regex_str = String::from("^");
    let mut pos = 0;
    while pos < pattern.len() {
        if let Some(star_idx) = pattern[pos..].find('*') {
            let star_idx = pos + star_idx;
            let literal_before = &pattern[pos..star_idx];
            if star_idx > 0 && pattern.chars().nth(star_idx - 1) == Some(' ') {
                let without_trailing_space = &literal_before[..literal_before.len() - 1];
                regex_str.push_str(&regex::escape(without_trailing_space));
                regex_str.push_str("( .*)?");
            } else {
                regex_str.push_str(&regex::escape(literal_before));
                regex_str.push_str(".*");
            }
            pos = star_idx + 1;
        } else {
            regex_str.push_str(&regex::escape(&pattern[pos..]));
            break;
        }
    }
    regex_str.push('$');
    if let Ok(re) = Regex::new(&regex_str) {
        re.is_match(&normalized)
    } else {
        normalized == pattern
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// File path matching (gitignore style)
// ─────────────────────────────────────────────────────────────────────────────

fn to_posix_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

pub fn resolve_path_pattern(specifier: &str, project_root: &Path, cwd: &Path) -> String {
    if specifier.starts_with("//") {
        return specifier[1..].to_string();
    }
    if let Some(rest) = specifier.strip_prefix("~/") {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        return to_posix_path(&home.join(rest));
    }
    if let Some(rest) = specifier.strip_prefix('/') {
        return to_posix_path(&project_root.join(rest));
    }
    if let Some(rest) = specifier.strip_prefix("./") {
        return to_posix_path(&cwd.join(rest));
    }
    to_posix_path(&cwd.join(specifier))
}

pub fn matches_path_pattern(specifier: &str, file_path: &str, ctx: &PathMatchContext) -> bool {
    let resolved_pattern = resolve_path_pattern(specifier, &ctx.project_root, &ctx.cwd);
    if let Ok(pattern) = Pattern::new(&resolved_pattern) {
        pattern.matches(&to_posix_path(Path::new(file_path)))
    } else {
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Domain matching
// ─────────────────────────────────────────────────────────────────────────────

pub fn matches_domain_pattern(specifier: &str, domain: &str) -> bool {
    let pattern = if specifier.starts_with("domain:") {
        specifier[7..].trim()
    } else {
        specifier.trim()
    };
    if pattern.is_empty() || domain.is_empty() {
        return false;
    }
    let domain_lower = domain.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    if domain_lower == pattern_lower {
        return true;
    }
    if domain_lower.ends_with(&format!(".{}", pattern_lower)) {
        return true;
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP tool matching
// ─────────────────────────────────────────────────────────────────────────────

fn matches_mcp_pattern(pattern: &str, tool_name: &str) -> bool {
    if pattern == tool_name {
        return true;
    }
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return tool_name.starts_with(prefix);
    }
    let pattern_parts: Vec<&str> = pattern.split("__").collect();
    let tool_parts: Vec<&str> = tool_name.split("__").collect();
    if pattern_parts.len() == 2 && tool_parts.len() >= 3
        && pattern_parts[0] == tool_parts[0] && pattern_parts[1] == tool_parts[1] {
        return true;
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// Unified rule matching
// ─────────────────────────────────────────────────────────────────────────────

pub fn matches_rule(
    rule: &PermissionRule,
    tool_name: &str,
    command: Option<&str>,
    file_path: Option<&str>,
    domain: Option<&str>,
    path_context: Option<&PathMatchContext>,
    specifier: Option<&str>,
) -> bool {
    let canonical_ctx_tool = resolve_tool_name(tool_name);

    // MCP
    if rule.tool_name.starts_with("mcp__") || canonical_ctx_tool.starts_with("mcp__") {
        return matches_mcp_pattern(&rule.tool_name, &canonical_ctx_tool);
    }

    if !tool_matches_rule_tool_name(&rule.tool_name, &canonical_ctx_tool) {
        return false;
    }

    let spec = rule.specifier.as_deref();
    if spec.is_none() {
        return true;
    }
    let spec = spec.unwrap();

    let kind = rule.specifier_kind.unwrap_or_else(|| get_specifier_kind(&rule.tool_name));
    match kind {
        SpecifierKind::Command => {
            if let Some(cmd) = command {
                matches_command_pattern(spec, cmd)
            } else {
                false
            }
        }
        SpecifierKind::Path => {
            if let Some(fp) = file_path {
                let ctx = path_context.unwrap_or(&PathMatchContext {
                    project_root: PathBuf::from("."),
                    cwd: PathBuf::from("."),
                });
                matches_path_pattern(spec, fp, ctx)
            } else {
                false
            }
        }
        SpecifierKind::Domain => {
            if let Some(dom) = domain {
                matches_domain_pattern(spec, dom)
            } else {
                false
            }
        }
        SpecifierKind::Literal => {
            let value = command.or(specifier);
            if let Some(val) = value {
                val == spec
            } else {
                false
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Build minimum-scope permission rules
// ─────────────────────────────────────────────────────────────────────────────

pub fn build_permission_rules(ctx: &PermissionCheckContext) -> Vec<String> {
    let canonical = resolve_tool_name(&ctx.tool_name);
    let display = get_rule_display_name(&canonical);
    let kind = get_specifier_kind(&canonical);

    match kind {
        SpecifierKind::Command => {
            if let Some(cmd) = &ctx.command {
                vec![format!("{}({})", display, cmd)]
            } else {
                vec![display]
            }
        }
        SpecifierKind::Path => {
            if let Some(fp) = &ctx.file_path {
                let dir_path = if FILE_TARGETED_TOOLS.contains(canonical.as_str()) {
                    Path::new(fp).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "/".to_string())
                } else {
                    fp.clone()
                };
                let specifier = if dir_path.starts_with('/') {
                    format!("/{}/**", dir_path.trim_start_matches('/'))
                } else {
                    format!("{}/**", dir_path)
                };
                vec![format!("{}({})", display, specifier)]
            } else {
                vec![display]
            }
        }
        SpecifierKind::Domain => {
            if let Some(dom) = &ctx.domain {
                vec![format!("{}({})", display, dom)]
            } else {
                vec![display]
            }
        }
        SpecifierKind::Literal => {
            if let Some(spec) = &ctx.specifier {
                vec![format!("{}({})", display, spec)]
            } else {
                vec![display]
            }
        }
    }
}

pub fn build_human_readable_rule_label(rules: &[String]) -> String {
    let mut parts = Vec::new();
    for rule in rules {
        if let Some(paren_idx) = rule.find('(') {
            let display = &rule[..paren_idx];
            let specifier = &rule[paren_idx + 1..rule.len() - 1];
            let verb = DISPLAY_NAME_TO_VERB.get(display).map(|&v| v).unwrap_or(display);

            let canonical = CANONICAL_TO_RULE_DISPLAY.iter()
                .find_map(|(k, &v)| if v == display { Some(*k) } else { None })
                .unwrap_or(display);
            let kind = get_specifier_kind(canonical);

            match kind {
                SpecifierKind::Path => {
                    let mut cleaned = specifier.replace("/**", "/").replace("/*", "/");
                    if cleaned.starts_with("//") {
                        cleaned = cleaned[1..].to_string();
                    }
                    if !cleaned.ends_with('/') {
                        cleaned.push('/');
                    }
                    parts.push(format!("{} in {}", verb, cleaned));
                }
                SpecifierKind::Command => {
                    parts.push(format!("run '{}' commands", specifier));
                }
                SpecifierKind::Domain => {
                    parts.push(format!("{} {}", verb, specifier));
                }
                _ => {
                    parts.push(format!("{} \"{}\"", verb, specifier));
                }
            }
        } else {
            let verb = DISPLAY_NAME_TO_VERB.get(rule.as_str()).map(|&v| v).unwrap_or(rule);
            parts.push(verb.to_string());
        }
    }
    parts.join(", ")
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_tool_name() {
        assert_eq!(resolve_tool_name("Bash"), "run_shell_command");
        assert_eq!(resolve_tool_name("Read"), "read_file");
        assert_eq!(resolve_tool_name("Write"), "write_file");
        assert_eq!(resolve_tool_name("Edit"), "edit");
    }

    #[test]
    fn test_parse_rule() {
        let rule = parse_rule("Bash(git *)");
        assert_eq!(rule.tool_name, "run_shell_command");
        assert_eq!(rule.specifier, Some("git *".to_string()));

        let rule2 = parse_rule("Read");
        assert_eq!(rule2.tool_name, "read_file");
        assert!(rule2.specifier.is_none());
    }

    #[test]
    fn test_command_matching() {
        assert!(matches_command_pattern("git *", "git status"));
        assert!(matches_command_pattern("ls*", "lsof"));
        assert!(!matches_command_pattern("ls *", "lsof"));
    }

    #[test]
    fn test_path_matching() {
        let ctx = PathMatchContext {
            project_root: PathBuf::from("/project"),
            cwd: PathBuf::from("/project"),
        };
        let pattern = "/src/**/*.rs";
        assert!(matches_path_pattern(pattern, "/project/src/main.rs", &ctx));
        assert!(!matches_path_pattern(pattern, "/project/README.md", &ctx));
    }

    #[test]
    fn test_build_permission_rules() {
        let ctx = PermissionCheckContext {
            tool_name: "Read".to_string(),
            file_path: Some("/home/user/secrets/.env".to_string()),
            ..Default::default()
        };
        let rules = build_permission_rules(&ctx);
        assert_eq!(rules, vec!["Read(//home/user/secrets/**)"]);
    }
}