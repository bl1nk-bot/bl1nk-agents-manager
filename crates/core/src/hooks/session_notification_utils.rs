use std::process::Command;

pub fn find_command(cmd: &str) -> bool {
    let check_cmd = if cfg!(windows) { "where" } else { "which" };
    Command::new(check_cmd).arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
}

pub async fn send_notification(title: &str, message: &str) {
    if cfg!(target_os = "macos") {
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(format!("display notification \"{}\" with title \"{}\"", message, title))
            .spawn();
    } else if cfg!(target_os = "linux") {
        let _ = Command::new("notify-send").arg(title).arg(message).spawn();
    }
}

