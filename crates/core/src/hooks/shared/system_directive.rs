pub const SYSTEM_DIRECTIVE_PREFIX: &str = "[SYSTEM_DIRECTIVE:";

pub enum SystemDirectiveType {
    User,
    System
}

pub fn create_system_directive(name: &str, _type: SystemDirectiveType) -> String {
    format!("[SYSTEM_DIRECTIVE:{}]", name)
}
