pub const HOOK_NAME: &str = "start-work";

pub const KEYWORD_PATTERN: &str = r"\b(ultrawork|ulw)\b";

pub const CONTEXT_TAG: &str = "<session-context>";

pub const SYSTEM_REMINDER_OPEN: &str = "<system-reminder>";

pub const SYSTEM_REMINDER_CLOSE: &str = "</system-reminder>";

pub const PLAN_FILE_EXTENSION: &str = ".md";

pub const BOULDER_STATE_FILE: &str = "boulder.json";

pub const PROMETHEUS_PLANS_DIR: &str = ".sisyphus/plans/";

pub const SESSION_ID_PLACEHOLDER: &str = "$SESSION_ID";

pub const TIMESTAMP_PLACEHOLDER: &str = "$TIMESTAMP";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(HOOK_NAME, "start-work");
        assert_eq!(CONTEXT_TAG, "<session-context>");
        assert_eq!(SYSTEM_REMINDER_OPEN, "<system-reminder>");
        assert_eq!(SYSTEM_REMINDER_CLOSE, "</system-reminder>");
        assert_eq!(PLAN_FILE_EXTENSION, ".md");
        assert_eq!(BOULDER_STATE_FILE, "boulder.json");
        assert_eq!(PROMETHEUS_PLANS_DIR, ".sisyphus/plans/");
        assert_eq!(SESSION_ID_PLACEHOLDER, "$SESSION_ID");
        assert_eq!(TIMESTAMP_PLACEHOLDER, "$TIMESTAMP");
    }
}