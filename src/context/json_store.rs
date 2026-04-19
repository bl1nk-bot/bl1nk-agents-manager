//! ระบบจัดเก็บบริบทแบบไฟล์ JSON (JSON File-based Context Store)
//!
//! ดำเนินการตาม ContextStore trait โดยใช้ไฟล์ JSON ในโฟลเดอร์ .omg/state/

use crate::context::{secrets_file_path, workspace_file_path, Secrets, Workspace, WORKSPACES_INDEX_FILE};
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

/// ระบบจัดเก็บบริบทโดยใช้ไฟล์ JSON
pub struct JsonContextStore {
    base_path: PathBuf,
}

impl JsonContextStore {
    /// สร้างอินสแตนซ์ใหม่ของ JSON context store
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// รับพาธเต็มสำหรับไฟล์ที่ระบุ (สัมพัทธ์กับโฟลเดอร์เก็บสถานะ)
    fn path(&self, relative: &str) -> PathBuf {
        self.base_path.join(".omg").join("state").join(relative)
    }

    /// ตรวจสอบและสร้างไดเรกทอรีหลักหากยังไม่มี
    async fn ensure_base_dir(&self) -> Result<()> {
        let state_dir = self.base_path.join(".omg").join("state");
        if !state_dir.exists() {
            fs::create_dir_all(&state_dir)
                .await
                .with_context(|| format!("ไม่สามารถสร้างไดเรกทอรีสถานะ: {:?}", state_dir))?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::context::ContextStore for JsonContextStore {
    async fn save_workspace(&self, workspace: &Workspace) -> Result<()> {
        self.ensure_base_dir().await?;

        // สร้างไดเรกทอรีย่อยสำหรับ workspaces
        let workspaces_dir = self.path("workspaces");
        if !workspaces_dir.exists() {
            fs::create_dir_all(&workspaces_dir).await?;
        }

        // บันทึกข้อมูล workspace
        let path = self.path(&workspace_file_path(workspace.id));
        let content = serde_json::to_string_pretty(workspace).context("ไม่สามารถแปลงข้อมูล workspace เป็น JSON")?;

        // การเขียนไฟล์แบบ Atomic เพื่อความปลอดภัย
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, &content).await?;
        fs::rename(&temp_path, &path).await?;

        // อัปเดตดัชนีรวม
        self.update_workspace_index(workspace.id).await?;

        tracing::info!(workspace_id = %workspace.id, "✅ บันทึก Workspace สำเร็จ");
        Ok(())
    }

    async fn load_workspace(&self, id: Uuid) -> Result<Option<Workspace>> {
        let path = self.path(&workspace_file_path(id));

        match fs::read_to_string(&path).await {
            Ok(content) => {
                let workspace: Workspace = serde_json::from_str(&content)
                    .with_context(|| format!("ไม่สามารถอ่านข้อมูล workspace จาก {:?}", path))?;
                Ok(Some(workspace))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).context(format!("เกิดข้อผิดพลาดในการอ่านไฟล์ workspace: {:?}", path)),
        }
    }

    async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let index_path = self.path(WORKSPACES_INDEX_FILE);

        match fs::read_to_string(&index_path).await {
            Ok(content) => {
                let ids: Vec<Uuid> = serde_json::from_str(&content).with_context(|| "ไม่สามารถอ่านดัชนี workspace")?;

                let mut workspaces = Vec::new();
                for id in ids {
                    if let Some(workspace) = self.load_workspace(id).await? {
                        workspaces.push(workspace);
                    }
                }
                Ok(workspaces)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e).context("ไม่สามารถอ่านดัชนี workspace"),
        }
    }

    async fn delete_workspace(&self, id: Uuid) -> Result<()> {
        let path = self.path(&workspace_file_path(id));

        // ลบไฟล์ workspace
        match fs::remove_file(&path).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).context(format!("ไม่สามารถลบ workspace: {:?}", path)),
        }

        // ลบข้อมูลลับ (Secrets)
        let secrets_path = self.path(&secrets_file_path(id));
        match fs::remove_file(&secrets_path).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).context(format!("ไม่สามารถลบข้อมูลลับ: {:?}", secrets_path)),
        }

        // อัปเดตดัชนี
        self.remove_from_workspace_index(id).await?;

        tracing::info!(workspace_id = %id, "🗑️ ลบ Workspace สำเร็จ");
        Ok(())
    }

    async fn save_secrets(&self, workspace_id: Uuid, secrets: &Secrets) -> Result<()> {
        self.ensure_base_dir().await?;

        let path = self.path(&secrets_file_path(workspace_id));
        let content = serde_json::to_string_pretty(secrets).context("ไม่สามารถแปลงข้อมูลลับเป็น JSON")?;

        // ตรวจสอบว่าไดเรกทอรีสำหรับเก็บข้อมูลลับมีอยู่จริง
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }

        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, &content).await?;
        fs::rename(&temp_path, &path).await?;

        tracing::info!(workspace_id = %workspace_id, "🔐 บันทึกข้อมูลลับสำเร็จ");
        Ok(())
    }

    async fn load_secrets(&self, workspace_id: Uuid) -> Result<Option<Secrets>> {
        let path = self.path(&secrets_file_path(workspace_id));

        match fs::read_to_string(&path).await {
            Ok(content) => {
                let secrets: Secrets =
                    serde_json::from_str(&content).with_context(|| format!("ไม่สามารถอ่านข้อมูลลับจาก {:?}", path))?;
                Ok(Some(secrets))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).context(format!("เกิดข้อผิดพลาดในการอ่านไฟล์ข้อมูลลับ: {:?}", path)),
        }
    }
}

impl JsonContextStore {
    /// อัปเดตดัชนี Workspace เพื่อรวม Workspace ใหม่
    async fn update_workspace_index(&self, id: Uuid) -> Result<()> {
        let index_path = self.path(WORKSPACES_INDEX_FILE);

        let mut ids = match fs::read_to_string(&index_path).await {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e).context("ไม่สามารถอ่านดัชนี workspace"),
        };

        if !ids.contains(&id) {
            ids.push(id);
        }

        let content = serde_json::to_string_pretty(&ids).context("ไม่สามารถแปลงดัชนี workspace เป็น JSON")?;
        fs::write(&index_path, content).await?;

        Ok(())
    }

    /// ลบ Workspace ออกจากดัชนี
    async fn remove_from_workspace_index(&self, id: Uuid) -> Result<()> {
        let index_path = self.path(WORKSPACES_INDEX_FILE);

        let mut ids: Vec<Uuid> = match fs::read_to_string(&index_path).await {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e).context("ไม่สามารถอ่านดัชนี workspace"),
        };

        ids.retain(|i| *i != id);

        let content = serde_json::to_string_pretty(&ids).context("ไม่สามารถแปลงดัชนี workspace เป็น JSON")?;
        fs::write(&index_path, content).await?;

        Ok(())
    }

    /// ถ่ายโอนบริบทของ Workspace ไปยังไฟล์เก็บถาวร (Archive) ที่อ่านได้ง่าย
    pub async fn offload_workspace(&self, workspace_id: Uuid) -> Result<PathBuf> {
        let workspace = <Self as crate::context::ContextStore>::load_workspace(self, workspace_id)
            .await?
            .context("ไม่พบ Workspace")?;

        let archive_dir = self.base_path.join(".omg").join("state").join("archives");
        if !archive_dir.exists() {
            fs::create_dir_all(&archive_dir).await?;
        }

        let filename = format!("{}_{}.md", workspace.name, workspace_id);
        let path = archive_dir.join(&filename);

        let mut content = String::new();
        content.push_str(&format!("# Context Archive: {}\n", workspace.name));
        content.push_str(&format!("Created: {}\n\n", chrono::Utc::now()));

        for (id, conv) in &workspace.conversations {
            content.push_str(&format!("## Conversation: {}\n", id));
            for msg in &conv.messages {
                content.push_str(&format!(
                    "[{}] {:?}: {}\n",
                    msg.timestamp.format("%Y-%m-%d %H:%M"),
                    msg.role,
                    msg.content
                ));
            }
            content.push('\n');
        }

        fs::write(&path, content).await?;
        tracing::info!(workspace_id = %workspace_id, path = %path.display(), "📦 ถ่ายโอน Workspace ไปยัง Archive สำเร็จ");
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ContextStore, Conversation};
    use tempfile::TempDir;

    fn create_test_store(temp_dir: &TempDir) -> JsonContextStore {
        JsonContextStore::new(temp_dir.path().to_path_buf())
    }

    #[tokio::test]
    async fn test_save_and_load_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let store = create_test_store(&temp_dir);

        let mut workspace = Workspace::new("Test".to_string());
        workspace.add_conversation(Conversation::new(workspace.id));

        store.save_workspace(&workspace).await.unwrap();

        let loaded = store.load_workspace(workspace.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "Test");
    }

    #[tokio::test]
    async fn test_load_nonexistent_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let store = create_test_store(&temp_dir);

        let loaded = store.load_workspace(Uuid::new_v4()).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_list_workspaces() {
        let temp_dir = TempDir::new().unwrap();
        let store = create_test_store(&temp_dir);

        let workspace1 = Workspace::new("Workspace 1".to_string());
        let workspace2 = Workspace::new("Workspace 2".to_string());

        store.save_workspace(&workspace1).await.unwrap();
        store.save_workspace(&workspace2).await.unwrap();

        let workspaces = store.list_workspaces().await.unwrap();
        assert_eq!(workspaces.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let store = create_test_store(&temp_dir);

        let workspace = Workspace::new("To Delete".to_string());
        store.save_workspace(&workspace).await.unwrap();

        store.delete_workspace(workspace.id).await.unwrap();

        let loaded = store.load_workspace(workspace.id).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_save_and_load_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let store = create_test_store(&temp_dir);

        let workspace_id = Uuid::new_v4();
        let mut secrets = Secrets::new();
        secrets.set("api_key", "secret123");

        store.save_secrets(workspace_id, &secrets).await.unwrap();

        let loaded = store.load_secrets(workspace_id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().get("api_key"), Some(&"secret123".to_string()));
    }

    #[tokio::test]
    async fn test_load_nonexistent_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let store = create_test_store(&temp_dir);

        let loaded = store.load_secrets(Uuid::new_v4()).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_empty_list_workspaces() {
        let temp_dir = TempDir::new().unwrap();
        let store = create_test_store(&temp_dir);

        let workspaces = store.list_workspaces().await.unwrap();
        assert!(workspaces.is_empty());
    }

    #[tokio::test]
    async fn test_offload_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let store = create_test_store(&temp_dir);

        let mut workspace = Workspace::new("Test Offload".to_string());
        workspace.add_conversation(Conversation::new(workspace.id));
        store.save_workspace(&workspace).await.unwrap();

        let path = store.offload_workspace(workspace.id).await.unwrap();
        assert!(path.exists());

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(content.contains("Context Archive"));
        assert!(content.contains("Test Offload"));
    }
}
