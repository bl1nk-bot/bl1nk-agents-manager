// src/permission_manager.rs
//! ตัวจัดการสิทธิ์ (PermissionManager) สำหรับควบคุมการเข้าถึงเครื่องมือต่าง ๆ
//! โดยประเมินกฎ allow, ask, deny ตามลำดับความสำคัญ
//!
//! กฎจะถูกตรวจสอบตามลำดับ:
//!   1. deny  → ปฏิเสธทันที
//!   2. ask   → ถามผู้ใช้
//!   3. allow → อนุญาต
//!   4. (ไม่มีกฎตรงกัน) → default (ขึ้นกับโหมดการอนุมัติ)
//!
//! กฎมีที่มาจากสองแหล่ง:
//!   - Session rules: กฎชั่วคราวสำหรับ session ปัจจุบัน (เช่น ผู้ใช้กด "Always allow for this session")
//!   - Persistent rules: กฎถาวรจากไฟล์ settings
//!
//! สำหรับคำสั่ง Shell ที่ซับซ้อน (compound commands) จะแยกเป็นคำสั่งย่อย
//! และประเมินทีละคำสั่ง แล้วเลือกผลลัพธ์ที่เข้มงวดที่สุด

use crate::rule_parser::{
    parse_rule, parse_rules, matches_rule, resolve_tool_name, split_compound_command,
    PathMatchContext, PermissionCheckContext, PermissionRule,
};
use crate::shell_semantics::{extract_shell_operations, detect_command_substitution};
use crate::shell_ast::is_shell_command_read_only_ast;
use std::collections::HashSet;

/// ประเภทของกฎ (allow, ask, deny)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleType {
    Allow,
    Ask,
    Deny,
}

/// ขอบเขตของกฎ (session = ชั่วคราว, user = ถาวรจาก settings)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleScope {
    Session,
    User,
}

/// กฎพร้อมข้อมูลแหล่งที่มา ใช้สำหรับแสดงผลใน UI (/permissions)
#[derive(Debug, Clone)]
pub struct RuleWithSource {
    pub rule: PermissionRule,
    pub rule_type: RuleType,
    pub scope: RuleScope,
}

/// ผลการตัดสินใจสิทธิ์ (PermissionDecision)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionDecision {
    Allow = 0,
    Default = 1,
    Ask = 2,
    Deny = 3,
}

/// ชุดของกฎที่แบ่งตามประเภท
#[derive(Debug, Clone, Default)]
struct PermissionRuleSet {
    allow: Vec<PermissionRule>,
    ask: Vec<PermissionRule>,
    deny: Vec<PermissionRule>,
}

/// Trait สำหรับเข้าถึงค่าคอนฟิกจาก Config
/// แยกออกมาเพื่อหลีกเลี่ยงการอ้างอิงวงกลม (circular dependency)
pub trait PermissionManagerConfig: Send + Sync {
    /// กฎ allow ที่รวมจาก settings และ SDK params แล้ว
    fn get_permissions_allow(&self) -> Option<Vec<String>>;
    /// กฎ ask (จาก settings เท่านั้น)
    fn get_permissions_ask(&self) -> Option<Vec<String>>;
    /// กฎ deny (จาก settings + excludeTools)
    fn get_permissions_deny(&self) -> Option<Vec<String>>;
    /// ไดเรกทอรีรากของโปรเจกต์ (สำหรับ resolve path pattern)
    fn get_project_root(&self) -> Option<String>;
    /// ไดเรกทอรีทำงานปัจจุบัน
    fn get_cwd(&self) -> Option<String>;
    /// โหมดการอนุมัติปัจจุบัน (plan/default/auto-edit/yolo)
    fn get_approval_mode(&self) -> Option<String>;
    /// รายการ coreTools แบบ legacy (whitelist)
    /// ถ้ามีค่าและไม่ว่าง จะจำกัดเฉพาะเครื่องมือที่อยู่ในรายการนี้เท่านั้น
    fn get_core_tools(&self) -> Option<Vec<String>>;
}

/// ตัวจัดการสิทธิ์หลัก
pub struct PermissionManager<C: PermissionManagerConfig> {
    config: C,
    /// กฎถาวรจาก settings
    persistent_rules: PermissionRuleSet,
    /// กฎชั่วคราวสำหรับ session ปัจจุบัน
    session_rules: PermissionRuleSet,
    /// ชื่อเครื่องมือมาตรฐานที่อนุญาตจาก coreTools เดิม (whitelist)
    core_tools_allow_list: Option<HashSet<String>>,
}

impl<C: PermissionManagerConfig> PermissionManager<C> {
    pub fn new(config: C) -> Self {
        Self {
            config,
            persistent_rules: PermissionRuleSet::default(),
            session_rules: PermissionRuleSet::default(),
            core_tools_allow_list: None,
        }
    }

    /// เริ่มต้นตัวจัดการ โดยโหลดกฎจาก config และสร้าง coreTools allowlist
    pub fn initialize(&mut self) {
        // แปลงกฎจาก config strings เป็น PermissionRule
        self.persistent_rules = PermissionRuleSet {
            allow: parse_rules(&self.config.get_permissions_allow().unwrap_or_default()),
            ask: parse_rules(&self.config.get_permissions_ask().unwrap_or_default()),
            deny: parse_rules(&self.config.get_permissions_deny().unwrap_or_default()),
        };

        // สร้าง coreTools allowlist จาก legacy coreTools
        if let Some(raw_core_tools) = self.config.get_core_tools() {
            if !raw_core_tools.is_empty() {
                let set: HashSet<String> = raw_core_tools
                    .iter()
                    .map(|t| parse_rule(t).tool_name)
                    .collect();
                self.core_tools_allow_list = Some(set);
            }
        }
    }

    // ---------------------------------------------------------------------------
    // การประเมินผลหลัก
    // ---------------------------------------------------------------------------

    /// ประเมินผลการตัดสินใจสำหรับบริบทการเรียกใช้เครื่องมือที่กำหนด
    /// สำหรับคำสั่ง Shell ที่มีหลายคำสั่ง (compound) จะแยกและประเมินทีละคำสั่ง
    /// แล้วเลือกผลลัพธ์ที่เข้มงวดที่สุด (deny > ask > allow)
    pub async fn evaluate(&self, ctx: &PermissionCheckContext) -> PermissionDecision {
        let command = ctx.command.as_deref();

        // ถ้าเป็นคำสั่ง Shell และมีหลายคำสั่งย่อย
        if let Some(cmd) = command {
            let sub_commands = split_compound_command(cmd);
            if sub_commands.len() > 1 {
                return self.evaluate_compound_command(ctx, &sub_commands).await;
            }
        }

        let decision = self.evaluate_single(ctx).await;

        // สำหรับ Shell command ถ้าไม่ตรงกฎใด (default) ให้ใช้การวิเคราะห์ AST เพื่อตัดสินจริง
        if decision == PermissionDecision::Default
            && ctx.tool_name == "run_shell_command"
            && command.is_some()
        {
            return self.resolve_default_permission(command.unwrap()).await;
        }

        decision
    }

    /// ประเมินคำสั่งเดี่ยว (หรือคำสั่งย่อย) เทียบกับกฎทั้งหมด
    /// สำหรับ Shell command จะพิจารณาทั้งกฎ Bash โดยตรง และกฎที่ได้จากการจำลอง
    /// การดำเนินการกับไฟล์/เครือข่าย (virtual operations) แล้วเลือกผลลัพธ์ที่เข้มงวดที่สุด
    async fn evaluate_single(&self, ctx: &PermissionCheckContext) -> PermissionDecision {
        let tool_name = &ctx.tool_name;
        let command = ctx.command.as_deref();
        let file_path = ctx.file_path.as_deref();
        let domain = ctx.domain.as_deref();
        let specifier = ctx.specifier.as_deref();

        // สร้าง context สำหรับ path matching
        let path_ctx = if let (Some(proj), Some(cwd)) = (
            self.config.get_project_root(),
            self.config.get_cwd(),
        ) {
            Some(PathMatchContext {
                project_root: proj.into(),
                cwd: cwd.into(),
            })
        } else {
            None
        };

        // ฟังก์ชันช่วยในการตรวจสอบกฎในชุดที่กำหนด
        let check_rules = |rules: &[PermissionRule]| -> Option<PermissionDecision> {
            for rule in rules {
                if matches_rule(
                    rule,
                    tool_name,
                    command,
                    file_path,
                    domain,
                    path_ctx.as_ref(),
                    specifier,
                ) {
                    // คืนค่าผลลัพธ์ตามประเภทของกฎที่ตรง
                    // เรารู้ประเภทจากภายนอก (caller จะส่ง rules ของ allow/ask/deny)
                    return Some(PermissionDecision::Allow); // ค่าชั่วคราว
                }
            }
            None
        };

        // ตรวจสอบกฎ deny (session ก่อน แล้ว persistent)
        if check_rules(&self.session_rules.deny).is_some() {
            return PermissionDecision::Deny;
        }
        if check_rules(&self.persistent_rules.deny).is_some() {
            return PermissionDecision::Deny;
        }

        // ตรวจสอบกฎ ask
        if check_rules(&self.session_rules.ask).is_some() {
            return PermissionDecision::Ask;
        }
        if check_rules(&self.persistent_rules.ask).is_some() {
            return PermissionDecision::Ask;
        }

        // ตรวจสอบกฎ allow
        if check_rules(&self.session_rules.allow).is_some() {
            return PermissionDecision::Allow;
        }
        if check_rules(&self.persistent_rules.allow).is_some() {
            return PermissionDecision::Allow;
        }

        // ถ้าเป็น Shell command ให้ประเมิน virtual operations ด้วย
        if tool_name == "run_shell_command" && command.is_some() {
            let cwd = path_ctx
                .as_ref()
                .map(|p| p.cwd.clone())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let ops = extract_shell_operations(command.unwrap(), &cwd);
            if !ops.is_empty() {
                let virtual_decision = self.evaluate_shell_virtual_ops(&ops, path_ctx.as_ref()).await;
                // เลือกผลลัพธ์ที่เข้มงวดกว่า
                return virtual_decision.max(PermissionDecision::Default);
            }
        }

        PermissionDecision::Default
    }

    /// ประเมินผลของการดำเนินการเสมือนที่สกัดจากคำสั่ง Shell
    /// แต่ละ operation จะถูกประเมินเสมือนเรียกเครื่องมือโดยตรง (เช่น read_file, web_fetch)
    /// แล้วเลือกผลลัพธ์ที่เข้มงวดที่สุด
    async fn evaluate_shell_virtual_ops(
        &self,
        ops: &[crate::shell_semantics::ShellOperation],
        path_ctx: Option<&PathMatchContext>,
    ) -> PermissionDecision {
        let mut worst = PermissionDecision::Default;
        for op in ops {
            let op_ctx = PermissionCheckContext {
                tool_name: op.virtual_tool.clone(),
                file_path: op.file_path.clone(),
                domain: op.domain.clone(),
                ..Default::default()
            };
            // เรียก evaluate_single โดยตรง แต่ระวังการวนลูป (virtual_tool ไม่ใช่ run_shell_command)
            let decision = self.evaluate_single(&op_ctx).await;
            if decision > worst {
                worst = decision;
                if worst == PermissionDecision::Deny {
                    break;
                }
            }
        }
        worst
    }

    /// ประเมินคำสั่งผสม (compound command) โดยแยกเป็นคำสั่งย่อย
    /// ประเมินทีละคำสั่ง และเลือกผลลัพธ์ที่เข้มงวดที่สุด
    async fn evaluate_compound_command(
        &self,
        base_ctx: &PermissionCheckContext,
        sub_commands: &[String],
    ) -> PermissionDecision {
        let mut most_restrictive = PermissionDecision::Allow;

        for sub_cmd in sub_commands {
            let sub_ctx = PermissionCheckContext {
                command: Some(sub_cmd.clone()),
                ..base_ctx.clone()
            };
            let raw_decision = self.evaluate_single(&sub_ctx).await;

            // ถ้าได้ Default ให้ใช้การวิเคราะห์ AST เพื่อหาผลลัพธ์จริง
            let decision = if raw_decision == PermissionDecision::Default
                && base_ctx.tool_name == "run_shell_command"
            {
                self.resolve_default_permission(sub_cmd).await
            } else {
                raw_decision
            };

            if decision > most_restrictive {
                most_restrictive = decision;
            }
            if most_restrictive == PermissionDecision::Deny {
                break;
            }
        }

        most_restrictive
    }

    /// แปลงผลลัพธ์ Default เป็น Allow/Ask/Deny โดยวิเคราะห์คำสั่ง Shell
    /// - ตรวจพบ command substitution → Deny
    /// - คำสั่งอ่านอย่างเดียว → Allow
    /// - อื่น ๆ → Ask
    async fn resolve_default_permission(&self, command: &str) -> PermissionDecision {
        if detect_command_substitution(command) {
            return PermissionDecision::Deny;
        }

        let is_readonly = is_shell_command_read_only_ast(command).await;
        if is_readonly {
            PermissionDecision::Allow
        } else {
            PermissionDecision::Ask
        }
    }

    // ---------------------------------------------------------------------------
    // การตรวจสอบระดับ Registry (เครื่องมือถูกเปิดใช้งานหรือไม่)
    // ---------------------------------------------------------------------------

    /// รายชื่อเครื่องมือหลักที่ต้องตรวจสอบกับ coreTools allowlist
    const CORE_TOOLS: &'static [&'static str] = &[
        "read_file", "write_file", "edit", "glob", "grep_search", "run_shell_command",
        "list_directory", "web_fetch", "web_search", "todo_write", "save_memory", "lsp",
    ];

    fn is_core_tool(tool_name: &str) -> bool {
        Self::CORE_TOOLS.contains(&tool_name)
    }

    /// ตรวจสอบว่าเครื่องมือควรปรากฏใน registry หรือไม่
    /// - ถ้ามี coreTools allowlist และเครื่องมือไม่อยู่ในรายการ → false
    /// - ถ้ามีกฎ deny ทั้งเครื่องมือ (ไม่มี specifier) → false
    pub async fn is_tool_enabled(&self, tool_name: &str) -> bool {
        let canonical = resolve_tool_name(tool_name);

        // เครื่องมือที่ไม่ใช่ core จะข้ามการตรวจสอบ allowlist
        if !Self::is_core_tool(&canonical) {
            let ctx = PermissionCheckContext {
                tool_name: canonical,
                ..Default::default()
            };
            let decision = self.evaluate(&ctx).await;
            return decision != PermissionDecision::Deny;
        }

        // ถ้ามี coreTools allowlist และเครื่องมือไม่อยู่ในรายการ → ปิดใช้งาน
        if let Some(ref allow_list) = self.core_tools_allow_list {
            if !allow_list.is_empty() && !allow_list.contains(&canonical) {
                return false;
            }
        }

        // ประเมินโดยไม่มี command (จะจับเฉพาะกฎที่ไม่มี specifier)
        let ctx = PermissionCheckContext {
            tool_name: canonical,
            ..Default::default()
        };
        let decision = self.evaluate(&ctx).await;
        decision != PermissionDecision::Deny
    }

    /// ค้นหากฎ deny แรกที่ตรงกับบริบท (ใช้แสดงเหตุผลที่ปฏิเสธ)
    pub fn find_matching_deny_rule(&self, ctx: &PermissionCheckContext) -> Option<String> {
        let tool_name = &ctx.tool_name;
        let command = ctx.command.as_deref();
        let file_path = ctx.file_path.as_deref();
        let domain = ctx.domain.as_deref();
        let specifier = ctx.specifier.as_deref();

        let path_ctx = if let (Some(proj), Some(cwd)) = (
            self.config.get_project_root(),
            self.config.get_cwd(),
        ) {
            Some(PathMatchContext {
                project_root: proj.into(),
                cwd: cwd.into(),
            })
        } else {
            None
        };

        for rule in self.session_rules.deny.iter().chain(self.persistent_rules.deny.iter()) {
            if matches_rule(
                rule,
                tool_name,
                command,
                file_path,
                domain,
                path_ctx.as_ref(),
                specifier,
            ) {
                return Some(rule.raw.clone());
            }
        }
        None
    }

    // ---------------------------------------------------------------------------
    // ตัวช่วยสำหรับคำสั่ง Shell
    // ---------------------------------------------------------------------------

    /// ตรวจสอบสิทธิ์ของคำสั่ง Shell โดยเฉพาะ
    pub async fn is_command_allowed(&self, command: &str) -> PermissionDecision {
        self.evaluate(&PermissionCheckContext {
            tool_name: "run_shell_command".to_string(),
            command: Some(command.to_string()),
            ..Default::default()
        })
        .await
    }

    // ---------------------------------------------------------------------------
    // การตรวจสอบว่ามีกฎที่เกี่ยวข้องหรือไม่
    // ---------------------------------------------------------------------------

    /// ตรวจสอบว่ามีกฎใด ๆ (allow, ask, deny) ที่ตรงกับบริบทหรือไม่
    /// ใช้เพื่อให้ scheduler ข้ามการเรียก evaluate() เมื่อไม่มีกฎที่เกี่ยวข้อง
    pub fn has_relevant_rules(&self, ctx: &PermissionCheckContext) -> bool {
        let tool_name = &ctx.tool_name;
        let command = ctx.command.as_deref();
        let file_path = ctx.file_path.as_deref();
        let domain = ctx.domain.as_deref();
        let specifier = ctx.specifier.as_deref();

        // ถ้าเป็น Shell compound ให้ตรวจสอบทีละคำสั่งย่อย
        if tool_name == "run_shell_command" && command.is_some() {
            let sub_commands = split_compound_command(command.unwrap());
            if sub_commands.len() > 1 {
                return sub_commands.iter().any(|sub_cmd| {
                    let sub_ctx = PermissionCheckContext {
                        command: Some(sub_cmd.clone()),
                        ..ctx.clone()
                    };
                    self.has_relevant_rules(&sub_ctx)
                });
            }
        }

        let path_ctx = if let (Some(proj), Some(cwd)) = (
            self.config.get_project_root(),
            self.config.get_cwd(),
        ) {
            Some(PathMatchContext {
                project_root: proj.into(),
                cwd: cwd.into(),
            })
        } else {
            None
        };

        let all_rules: Vec<&PermissionRule> = self
            .session_rules
            .allow
            .iter()
            .chain(self.persistent_rules.allow.iter())
            .chain(self.session_rules.ask.iter())
            .chain(self.persistent_rules.ask.iter())
            .chain(self.session_rules.deny.iter())
            .chain(self.persistent_rules.deny.iter())
            .collect();

        if all_rules.iter().any(|rule| {
            matches_rule(
                rule,
                tool_name,
                command,
                file_path,
                domain,
                path_ctx.as_ref(),
                specifier,
            )
        }) {
            return true;
        }

        // สำหรับ Shell command ให้ตรวจสอบ virtual operations ด้วย
        if tool_name == "run_shell_command" && command.is_some() {
            let cwd = path_ctx
                .as_ref()
                .map(|p| p.cwd.clone())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let ops = extract_shell_operations(command.unwrap(), &cwd);
            if ops.iter().any(|op| {
                let op_ctx = PermissionCheckContext {
                    tool_name: op.virtual_tool.clone(),
                    file_path: op.file_path.clone(),
                    domain: op.domain.clone(),
                    ..Default::default()
                };
                self.has_relevant_rules(&op_ctx)
            }) {
                return true;
            }
        }

        false
    }

    /// ตรวจสอบว่ามีกฎ ask ตรงกับบริบทหรือไม่ (สำหรับ UI)
    pub fn has_matching_ask_rule(&self, ctx: &PermissionCheckContext) -> bool {
        // คล้ายกับ has_relevant_rules แต่ตรวจสอบเฉพาะกฎ ask
        // (ตัด implementation บางส่วนเพื่อความกระชับ)
        let ask_rules: Vec<&PermissionRule> = self
            .session_rules
            .ask
            .iter()
            .chain(self.persistent_rules.ask.iter())
            .collect();

        let tool_name = &ctx.tool_name;
        let command = ctx.command.as_deref();
        let file_path = ctx.file_path.as_deref();
        let domain = ctx.domain.as_deref();
        let specifier = ctx.specifier.as_deref();

        let path_ctx = if let (Some(proj), Some(cwd)) = (
            self.config.get_project_root(),
            self.config.get_cwd(),
        ) {
            Some(PathMatchContext {
                project_root: proj.into(),
                cwd: cwd.into(),
            })
        } else {
            None
        };

        ask_rules.iter().any(|rule| {
            matches_rule(
                rule,
                tool_name,
                command,
                file_path,
                domain,
                path_ctx.as_ref(),
                specifier,
            )
        })
    }

    // ---------------------------------------------------------------------------
    // การจัดการกฎชั่วคราว (session rules)
    // ---------------------------------------------------------------------------

    pub fn add_session_allow_rule(&mut self, raw: &str) {
        if !raw.trim().is_empty() {
            self.session_rules.allow.push(parse_rule(raw));
        }
    }

    pub fn add_session_deny_rule(&mut self, raw: &str) {
        if !raw.trim().is_empty() {
            self.session_rules.deny.push(parse_rule(raw));
        }
    }

    pub fn add_session_ask_rule(&mut self, raw: &str) {
        if !raw.trim().is_empty() {
            self.session_rules.ask.push(parse_rule(raw));
        }
    }

    // ---------------------------------------------------------------------------
    // การจัดการกฎถาวร (persistent rules)
    // ---------------------------------------------------------------------------

    pub fn add_persistent_rule(&mut self, raw: &str, rule_type: RuleType) -> PermissionRule {
        let rule = parse_rule(raw);
        let target = match rule_type {
            RuleType::Allow => &mut self.persistent_rules.allow,
            RuleType::Ask => &mut self.persistent_rules.ask,
            RuleType::Deny => &mut self.persistent_rules.deny,
        };
        if !target.iter().any(|r| r.raw == rule.raw) {
            target.push(rule.clone());
        }
        rule
    }

    pub fn remove_persistent_rule(&mut self, raw: &str, rule_type: RuleType) -> bool {
        let target = match rule_type {
            RuleType::Allow => &mut self.persistent_rules.allow,
            RuleType::Ask => &mut self.persistent_rules.ask,
            RuleType::Deny => &mut self.persistent_rules.deny,
        };
        if let Some(idx) = target.iter().position(|r| r.raw == raw) {
            target.remove(idx);
            true
        } else {
            false
        }
    }

    // ---------------------------------------------------------------------------
    // การแสดงรายการกฎ (สำหรับ /permissions)
    // ---------------------------------------------------------------------------

    pub fn list_rules(&self) -> Vec<RuleWithSource> {
        let mut result = Vec::new();

        let add_rules = |rules: &[PermissionRule], rule_type: RuleType, scope: RuleScope| {
            for rule in rules {
                result.push(RuleWithSource {
                    rule: rule.clone(),
                    rule_type,
                    scope,
                });
            }
        };

        add_rules(&self.session_rules.deny, RuleType::Deny, RuleScope::Session);
        add_rules(&self.persistent_rules.deny, RuleType::Deny, RuleScope::User);
        add_rules(&self.session_rules.ask, RuleType::Ask, RuleScope::Session);
        add_rules(&self.persistent_rules.ask, RuleType::Ask, RuleScope::User);
        add_rules(&self.session_rules.allow, RuleType::Allow, RuleScope::Session);
        add_rules(&self.persistent_rules.allow, RuleType::Allow, RuleScope::User);

        result
    }

    pub fn get_allow_raw_strings(&self) -> Vec<String> {
        self.session_rules
            .allow
            .iter()
            .chain(self.persistent_rules.allow.iter())
            .map(|r| r.raw.clone())
            .collect()
    }

    pub fn get_default_mode(&self) -> String {
        self.config.get_approval_mode().unwrap_or_else(|| "default".to_string())
    }

    pub fn update_persistent_rules(&mut self, allow: Option<Vec<PermissionRule>>, ask: Option<Vec<PermissionRule>>, deny: Option<Vec<PermissionRule>>) {
        if let Some(a) = allow {
            self.persistent_rules.allow = a;
        }
        if let Some(a) = ask {
            self.persistent_rules.ask = a;
        }
        if let Some(d) = deny {
            self.persistent_rules.deny = d;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule_parser::PermissionCheckContext;

    // ใช้ mock config สำหรับทดสอบ
    struct MockConfig;
    impl PermissionManagerConfig for MockConfig {
        fn get_permissions_allow(&self) -> Option<Vec<String>> { None }
        fn get_permissions_ask(&self) -> Option<Vec<String>> { None }
        fn get_permissions_deny(&self) -> Option<Vec<String>> { None }
        fn get_project_root(&self) -> Option<String> { Some(".".to_string()) }
        fn get_cwd(&self) -> Option<String> { Some(".".to_string()) }
        fn get_approval_mode(&self) -> Option<String> { Some("default".to_string()) }
        fn get_core_tools(&self) -> Option<Vec<String>> { None }
    }

    #[tokio::test]
    async fn test_empty_rules_returns_default() {
        let mut pm = PermissionManager::new(MockConfig);
        pm.initialize();
        let ctx = PermissionCheckContext {
            tool_name: "Bash".to_string(),
            command: Some("ls -la".to_string()),
            ..Default::default()
        };
        let decision = pm.evaluate(&ctx).await;
        // ls ถือว่าเป็น read-only → Allow
        assert_eq!(decision, PermissionDecision::Allow);
    }
}