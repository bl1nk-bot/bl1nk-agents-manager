pub mod constants;
pub mod storage;

pub struct SessionRecoveryHook;

impl SessionRecoveryHook {
    pub fn new() -> Self { Self }

    pub fn detect_error_type(&self, error_msg: &str) -> Option<&str> {
        let msg = error_msg.to_lowercase();
        if msg.contains("thinking") && (msg.contains("first block") || msg.contains("must start with")) {
            return Some(constants::THINKING_BLOCK_ORDER_ERROR);
        }
        if msg.contains("tool_use") && msg.contains("tool_result") {
            return Some(constants::TOOL_RESULT_MISSING_ERROR);
        }
        None
    }
}
