pub struct EmptyTaskResponseDetectorHook;

impl EmptyTaskResponseDetectorHook {
    pub fn new() -> Self { Self }

    pub fn on_tool_execute_after(&self, tool: &str, output: &mut String) {
        if tool.to_lowercase() != "task" { return; }

        if output.trim().is_empty() {
            output.push_str("[Task Empty Response Warning]\nTask invocation completed but returned no response.");
        }
    }
}
