use bl1nk_core::{EventEmitter, GeminiBackend, GeminiAuthConfig, LLxprtConfig, QwenConfig};
use serde::Serialize;
use std::time::Duration;

#[derive(Clone)]
struct PrintEmitter;

impl EventEmitter for PrintEmitter {
    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> anyhow::Result<()> {
        let json = serde_json::to_string(&payload).unwrap_or_else(|_| "\"<unserializable>\"".into());
        println!("[EVENT] {event} {json}");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let emitter = PrintEmitter;
    let backend = GeminiBackend::new(emitter);

    let session_id = format!("session-{}", uuid::Uuid::new_v4());
    let working_directory = std::env::current_dir()
        .unwrap_or_else(|_| ".".into())
        .to_string_lossy()
        .to_string();
    let model = "auto".to_string();

    backend
        .initialize_session(
            session_id.clone(),
            working_directory,
            model,
            Some("codex".to_string()),
            None::<QwenConfig>,
            None::<GeminiAuthConfig>,
            None::<LLxprtConfig>,
        )
        .await?;

    backend
        .send_message(
            session_id.clone(),
            "ตอบแค่คำว่า OK ถ้าเห็นข้อความนี้".to_string(),
            "".to_string(),
        )
        .await?;

    tokio::time::sleep(Duration::from_secs(10)).await;

    let _ = backend.kill_process(&session_id);
    Ok(())
}
