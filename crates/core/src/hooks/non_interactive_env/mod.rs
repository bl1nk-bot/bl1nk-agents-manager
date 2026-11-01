use std::io::IsTerminal;

pub struct NonInteractiveEnvHook;

impl NonInteractiveEnvHook {
    pub fn new() -> Self { Self }

    pub fn is_non_interactive(&self) -> bool {
        std::env::var("CI").is_ok() || 
        std::env::var("GITHUB_ACTIONS").is_ok() ||
        !std::io::stdout().is_terminal()
    }

    pub fn apply_to_command(&self, cmd: &mut std::process::Command) {
        if self.is_non_interactive() {
            cmd.env("DEBIAN_FRONTEND", "noninteractive");
            cmd.env("GIT_TERMINAL_PROMPT", "0");
            cmd.env("GIT_EDITOR", ":");
        }
    }
}
