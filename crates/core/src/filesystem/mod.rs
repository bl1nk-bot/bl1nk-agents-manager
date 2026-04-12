use anyhow::Result;
use ignore::WalkBuilder;
use pathdiff::diff_paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VolumeType {
    LocalDisk,
    RemovableDisk,
    NetworkDrive,
    CdDrive,
    RamDisk,
    FileSystem,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_directory: bool,
    pub full_path: String,
    pub size: Option<u64>,
    pub modified: Option<u64>,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub volume_type: Option<VolumeType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitInfo {
    pub current_directory: String,
    pub branch: String,
    pub status: String,
    pub is_clean: bool,
    pub has_uncommitted_changes: bool,
    pub has_untracked_files: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileContent {
    pub path: String,
    pub content: Option<String>,
    pub size: u64,
    pub modified: Option<u64>,
    pub encoding: String,
    pub is_text: bool,
    pub is_binary: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GeminiCommandScope {
    User,
    Project,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct GeminiCommand {
    pub name: String,
    pub description: Option<String>,
    pub scope: GeminiCommandScope,
    pub path: String,
}

pub async fn validate_directory(path: String) -> Result<bool> {
    let path = Path::new(&path);
    Ok(path.exists() && path.is_dir())
}

pub async fn is_home_directory(path: String) -> Result<bool> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    let home_path = Path::new(&home);
    let check_path = Path::new(&path);

    Ok(home_path == check_path)
}

pub async fn get_home_directory() -> Result<String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    Ok(home)
}

pub async fn get_parent_directory(path: String) -> Result<Option<String>> {
    let path = Path::new(&path);
    Ok(path.parent().map(|p| p.to_string_lossy().to_string()))
}

pub async fn list_directory_contents(path: String) -> Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    let dir_path = Path::new(&path);

    if !dir_path.exists() || !dir_path.is_dir() {
        return Ok(entries);
    }

    // Use the ignore crate's WalkBuilder for proper gitignore support
    let mut builder = WalkBuilder::new(dir_path);
    builder
        .max_depth(Some(1)) // Only list immediate children (not recursive)
        .git_ignore(true) // Respect .gitignore files
        .git_global(true) // Respect global git ignore
        .git_exclude(true) // Respect .git/info/exclude
        .hidden(false) // Show hidden files/directories (except .git which is handled by git_ignore)
        .parents(true); // Respect gitignore files in parent directories

    // Collect entries using the ignore crate
    for result in builder.build() {
        match result {
            Ok(entry) => {
                let entry_path = entry.path();

                // Skip the root directory itself
                if entry_path == dir_path {
                    continue;
                }

                // Only process immediate children (depth 1)
                if entry_path.parent() != Some(dir_path) {
                    continue;
                }

                let metadata = match entry_path.metadata() {
                    Ok(metadata) => metadata,
                    Err(_) => continue, // Skip files we can't read metadata for
                };

                let file_name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let full_path = entry_path.to_string_lossy().to_string();

                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs());

                let size = if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                };

                let is_symlink = metadata.is_symlink();
                let symlink_target = if is_symlink {
                    fs::read_link(entry_path)
                        .ok()
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    None
                };

                entries.push(DirEntry {
                    name: file_name,
                    is_directory: metadata.is_dir(),
                    full_path,
                    size,
                    modified,
                    is_symlink,
                    symlink_target,
                    volume_type: None,
                });
            }
            Err(_) => continue, // Skip entries we can't read
        }
    }

    entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

pub async fn list_files_recursive(path: String) -> Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    let root_path = Path::new(&path);

    if !root_path.exists() || !root_path.is_dir() {
        return Ok(entries);
    }

    // Limit the total number of directories to 200
    let mut directory_count = 0;
    const MAX_DIRECTORIES: usize = 200;

    // Use the ignore crate's WalkBuilder for proper gitignore support
    let mut builder = WalkBuilder::new(root_path);
    builder
        .git_ignore(true) // Respect .gitignore files
        .git_global(true) // Respect global git ignore
        .git_exclude(true) // Respect .git/info/exclude
        .hidden(true) // Hide hidden files/directories (like .git)
        .parents(true); // Respect gitignore files in parent directories

    // Collect entries using the ignore crate
    for result in builder.build() {
        match result {
            Ok(entry) => {
                let entry_path = entry.path();

                // Skip the root directory itself
                if entry_path == root_path {
                    continue;
                }

                let metadata = match entry_path.metadata() {
                    Ok(metadata) => metadata,
                    Err(_) => continue, // Skip files we can't read metadata for
                };

                // Check if this is a directory and increment counter if so
                let is_directory = metadata.is_dir();
                if is_directory {
                    directory_count += 1;
                    // Stop processing if we've reached our directory limit
                    if directory_count > MAX_DIRECTORIES {
                        break;
                    }
                }

                let file_name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let full_path = entry_path.to_string_lossy().to_string();

                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs());

                let size = if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                };

                let is_symlink = metadata.is_symlink();
                let symlink_target = if is_symlink {
                    fs::read_link(entry_path)
                        .ok()
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    None
                };

                entries.push(DirEntry {
                    name: file_name,
                    is_directory,
                    full_path,
                    size,
                    modified,
                    is_symlink,
                    symlink_target,
                    volume_type: None,
                });
            }
            Err(_) => continue, // Skip entries we can't read
        }
    }

    // Sort entries: directories first, then files, alphabetically within each group
    entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

fn command_name_from_path(commands_root: &Path, file_path: &Path) -> Option<String> {
    let rel = file_path.strip_prefix(commands_root).ok()?;
    let mut parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    if parts.is_empty() {
        return None;
    }
    if let Some(last) = parts.last_mut() {
        if let Some(stripped) = last.strip_suffix(".toml") {
            *last = stripped.to_string();
        } else {
            return None;
        }
    }
    Some(parts.join(":"))
}

fn read_command_description(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let parsed: toml::Value = toml::from_str(&content).ok()?;
    parsed
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn collect_gemini_commands(commands_root: &Path, scope: GeminiCommandScope) -> Vec<GeminiCommand> {
    if !commands_root.exists() || !commands_root.is_dir() {
        return Vec::new();
    }

    let mut commands = Vec::new();
    let mut builder = WalkBuilder::new(commands_root);
    builder
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .hidden(false)
        .parents(false);

    for result in builder.build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        let name = match command_name_from_path(commands_root, path) {
            Some(name) => name,
            None => continue,
        };
        let description = read_command_description(path);
        commands.push(GeminiCommand {
            name,
            description,
            scope: scope.clone(),
            path: path.to_string_lossy().to_string(),
        });
    }

    commands
}

pub async fn list_gemini_commands(working_directory: String) -> Result<Vec<GeminiCommand>> {
    let mut merged = std::collections::HashMap::<String, GeminiCommand>::new();

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let user_root = PathBuf::from(home).join(".gemini").join("commands");
    for command in collect_gemini_commands(&user_root, GeminiCommandScope::User) {
        merged.insert(command.name.clone(), command);
    }

    let project_root = PathBuf::from(working_directory)
        .join(".gemini")
        .join("commands");
    for command in collect_gemini_commands(&project_root, GeminiCommandScope::Project) {
        merged.insert(command.name.clone(), command);
    }

    let mut commands: Vec<GeminiCommand> = merged.into_values().collect();
    commands.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(commands)
}

pub async fn list_volumes() -> Result<Vec<DirEntry>> {
    let mut volumes = Vec::new();

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        let output = Command::new("wmic")
            .args(["logicaldisk", "get", "name,volumename,drivetype,size"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let drive_letter = parts[0];
                let drive_type = parts
                    .get(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);

                let volume_type = match drive_type {
                    2 => VolumeType::RemovableDisk,
                    3 => VolumeType::LocalDisk,
                    4 => VolumeType::NetworkDrive,
                    5 => VolumeType::CdDrive,
                    6 => VolumeType::RamDisk,
                    _ => VolumeType::LocalDisk,
                };

                let volume_name = parts.get(2..).map(|p| p.join(" ")).unwrap_or_default();
                let display_name = if volume_name.is_empty() {
                    drive_letter.to_string()
                } else {
                    format!("{volume_name} ({drive_letter})")
                };

                volumes.push(DirEntry {
                    name: display_name,
                    is_directory: true,
                    full_path: format!("{drive_letter}\\"),
                    size: None,
                    modified: None,
                    is_symlink: false,
                    symlink_target: None,
                    volume_type: Some(volume_type),
                });
            }
        }
    }

    #[cfg(not(windows))]
    {
        volumes.push(DirEntry {
            name: "Root (/)".to_string(),
            is_directory: true,
            full_path: "/".to_string(),
            size: None,
            modified: None,
            is_symlink: false,
            symlink_target: None,
            volume_type: Some(VolumeType::FileSystem),
        });

        if let Ok(home) = std::env::var("HOME") {
            volumes.push(DirEntry {
                name: "Home".to_string(),
                is_directory: true,
                full_path: home,
                size: None,
                modified: None,
                is_symlink: false,
                symlink_target: None,
                volume_type: Some(VolumeType::FileSystem),
            });
        }
    }

    Ok(volumes)
}

pub async fn get_git_info(directory: String) -> Result<Option<GitInfo>> {
    let path = Path::new(&directory);
    if !path.exists() || !path.is_dir() {
        return Ok(None);
    }

    // Check if this is a git repository by looking for .git directory
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        return Ok(None);
    }

    let current_directory = path.to_string_lossy().to_string();

    // Get current branch
    let mut branch_output_cmd = Command::new("git");
    branch_output_cmd
        .arg("branch")
        .arg("--show-current")
        .current_dir(path);
    #[cfg(windows)]
    branch_output_cmd.creation_flags(CREATE_NO_WINDOW);
    let branch_output = branch_output_cmd.output().await;

    let branch = if let Ok(output) = branch_output {
        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            // Fallback to symbolic-ref if branch --show-current fails
            let mut symbolic_ref_cmd = Command::new("git");
            symbolic_ref_cmd
                .arg("symbolic-ref")
                .arg("--short")
                .arg("HEAD")
                .current_dir(path);
            #[cfg(windows)]
            symbolic_ref_cmd.creation_flags(CREATE_NO_WINDOW);
            let symbolic_ref_output = symbolic_ref_cmd.output().await;

            if let Ok(ref_output) = symbolic_ref_output {
                if ref_output.status.success() {
                    String::from_utf8_lossy(&ref_output.stdout)
                        .trim()
                        .to_string()
                } else {
                    "HEAD".to_string() // Detached HEAD state
                }
            } else {
                "unknown".to_string()
            }
        }
    } else {
        "unknown".to_string()
    };

    // Get git status
    let mut status_output_cmd = Command::new("git");
    status_output_cmd
        .arg("status")
        .arg("--porcelain")
        .arg("--branch")
        .current_dir(path);
    #[cfg(windows)]
    status_output_cmd.creation_flags(CREATE_NO_WINDOW);
    let status_output = status_output_cmd.output().await;

    let (status, is_clean, has_uncommitted_changes, has_untracked_files) =
        if let Ok(output) = status_output {
            if output.status.success() {
                let status_text = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = status_text.lines().collect();

                // Parse the status output
                let mut has_changes = false;
                let mut has_untracked = false;

                for line in &lines {
                    if line.starts_with("##") {
                        // Branch line - we don't need to store this for now
                    } else if !line.is_empty() {
                        // File status line
                        if line.starts_with("??") {
                            has_untracked = true;
                        } else {
                            has_changes = true;
                        }
                    }
                }

                let is_clean = !has_changes && !has_untracked;
                let status_desc = if is_clean {
                    "clean".to_string()
                } else {
                    let mut parts = Vec::new();
                    if has_changes {
                        parts.push("modified files");
                    }
                    if has_untracked {
                        parts.push("untracked files");
                    }
                    parts.join(", ")
                };

                (status_desc, is_clean, has_changes, has_untracked)
            } else {
                ("unknown".to_string(), false, false, false)
            }
        } else {
            ("unknown".to_string(), false, false, false)
        };

    Ok(Some(GitInfo {
        current_directory,
        branch,
        status,
        is_clean,
        has_uncommitted_changes,
        has_untracked_files,
    }))
}

pub async fn write_file_content(path: String, content: String) -> Result<FileContent> {
    use std::io::Write;

    let file_path = Path::new(&path);

    // Ensure the parent directory exists
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            return Ok(FileContent {
                path: path.clone(),
                content: None,
                size: 0,
                modified: None,
                encoding: "unknown".to_string(),
                is_text: false,
                is_binary: false,
                error: Some("Parent directory does not exist".to_string()),
            });
        }
    }

    // Write the content to the file
    match std::fs::File::create(file_path) {
        Ok(mut file) => {
            match file.write_all(content.as_bytes()) {
                Ok(_) => {
                    // Return the updated file info
                    read_file_content(path).await
                }
                Err(e) => Ok(FileContent {
                    path: path.clone(),
                    content: None,
                    size: 0,
                    modified: None,
                    encoding: "unknown".to_string(),
                    is_text: false,
                    is_binary: false,
                    error: Some(format!("Failed to write file: {}", e)),
                }),
            }
        }
        Err(e) => Ok(FileContent {
            path: path.clone(),
            content: None,
            size: 0,
            modified: None,
            encoding: "unknown".to_string(),
            is_text: false,
            is_binary: false,
            error: Some(format!("Failed to create file: {}", e)),
        }),
    }
}

pub async fn read_file_content_with_options(path: String, force_text: bool) -> Result<FileContent> {
    use std::io::Read;

    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Ok(FileContent {
            path: path.clone(),
            content: None,
            size: 0,
            modified: None,
            encoding: "unknown".to_string(),
            is_text: false,
            is_binary: false,
            error: Some("File does not exist".to_string()),
        });
    }

    if file_path.is_dir() {
        return Ok(FileContent {
            path: path.clone(),
            content: None,
            size: 0,
            modified: None,
            encoding: "unknown".to_string(),
            is_text: false,
            is_binary: false,
            error: Some("Path is a directory, not a file".to_string()),
        });
    }

    let metadata = match file_path.metadata() {
        Ok(metadata) => metadata,
        Err(e) => {
            return Ok(FileContent {
                path: path.clone(),
                content: None,
                size: 0,
                modified: None,
                encoding: "unknown".to_string(),
                is_text: false,
                is_binary: false,
                error: Some(format!("Cannot read file metadata: {}", e)),
            });
        }
    };

    let size = metadata.len();
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());

    // Limit file size to 10MB for safety
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
    if size > MAX_FILE_SIZE {
        return Ok(FileContent {
            path: path.clone(),
            content: None,
            size,
            modified,
            encoding: "unknown".to_string(),
            is_text: false,
            is_binary: true,
            error: Some(format!(
                "File too large ({} bytes). Maximum size is {} bytes",
                size, MAX_FILE_SIZE
            )),
        });
    }

    // Read file content
    let mut file = match std::fs::File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            return Ok(FileContent {
                path: path.clone(),
                content: None,
                size,
                modified,
                encoding: "unknown".to_string(),
                is_text: false,
                is_binary: false,
                error: Some(format!("Cannot open file: {}", e)),
            });
        }
    };

    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        return Ok(FileContent {
            path: path.clone(),
            content: None,
            size,
            modified,
            encoding: "unknown".to_string(),
            is_text: false,
            is_binary: false,
            error: Some(format!("Cannot read file content: {}", e)),
        });
    }

    // Check if content is valid UTF-8
    let is_valid_utf8 = std::str::from_utf8(&buffer).is_ok();
    let is_binary = !is_valid_utf8;

    // If force_text is true, always try to show content even for binary files
    let show_as_text = is_valid_utf8 || force_text;

    let (content, encoding) = if show_as_text {
        if is_valid_utf8 {
            // We already know it's valid UTF-8, so this should succeed
            match String::from_utf8(buffer) {
                Ok(text) => (Some(text), "UTF-8".to_string()),
                Err(utf8_error) => {
                    // If for some reason it still fails, use the original bytes from the error
                    let original_bytes = utf8_error.into_bytes();
                    (
                        Some(String::from_utf8_lossy(&original_bytes).into_owned()),
                        "UTF-8 (with replacements)".to_string(),
                    )
                }
            }
        } else {
            // Binary file shown as text (forced)
            (
                Some(String::from_utf8_lossy(&buffer).into_owned()),
                "binary (forced as text)".to_string(),
            )
        }
    } else {
        // For binary files, don't include content when not forced
        (None, "binary".to_string())
    };

    Ok(FileContent {
        path: path.clone(),
        content,
        size,
        modified,
        encoding,
        is_text: show_as_text,
        is_binary,
        error: None,
    })
}

pub async fn read_file_content(path: String) -> Result<FileContent> {
    read_file_content_with_options(path, false).await
}

pub async fn read_binary_file_as_base64(path: String) -> Result<String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::io::Read;

    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("File does not exist"));
    }

    if file_path.is_dir() {
        return Err(anyhow::anyhow!("Path is a directory, not a file"));
    }

    let mut file = std::fs::File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(general_purpose::STANDARD.encode(&buffer))
}

pub fn get_relative_path(from: &str, to: &str) -> Option<String> {
    diff_paths(to, from).map(|p| p.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    #[test]
    fn test_volume_type_serialization() {
        let volume_types = vec![
            VolumeType::LocalDisk,
            VolumeType::RemovableDisk,
            VolumeType::NetworkDrive,
            VolumeType::CdDrive,
            VolumeType::RamDisk,
            VolumeType::FileSystem,
        ];

        for volume_type in volume_types {
            let json = serde_json::to_string(&volume_type).unwrap();
            let deserialized: VolumeType = serde_json::from_str(&json).unwrap();
            assert_eq!(volume_type, deserialized);
        }
    }

    #[test]
    fn test_dir_entry_creation() {
        let entry = DirEntry {
            name: "test_file.txt".to_string(),
            is_directory: false,
            full_path: "/path/to/test_file.txt".to_string(),
            size: Some(1024),
            modified: Some(1640995200), // 2022-01-01 00:00:00 UTC
            is_symlink: false,
            symlink_target: None,
            volume_type: None,
        };

        assert_eq!(entry.name, "test_file.txt");
        assert!(!entry.is_directory);
        assert_eq!(entry.full_path, "/path/to/test_file.txt");
        assert_eq!(entry.size, Some(1024));
        assert_eq!(entry.modified, Some(1640995200));
        assert!(!entry.is_symlink);
        assert!(entry.symlink_target.is_none());
        assert!(entry.volume_type.is_none());
    }

    #[test]
    fn test_dir_entry_serialization() {
        let entry = DirEntry {
            name: "test".to_string(),
            is_directory: true,
            full_path: "/test".to_string(),
            size: None,
            modified: Some(123456789),
            is_symlink: false,
            symlink_target: None,
            volume_type: Some(VolumeType::LocalDisk),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: DirEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.name, deserialized.name);
        assert_eq!(entry.is_directory, deserialized.is_directory);
        assert_eq!(entry.full_path, deserialized.full_path);
        assert_eq!(entry.size, deserialized.size);
        assert_eq!(entry.modified, deserialized.modified);
        assert_eq!(entry.is_symlink, deserialized.is_symlink);
        assert_eq!(entry.symlink_target, deserialized.symlink_target);
    }

    #[tokio::test]
    async fn test_validate_directory_existing() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        let result = validate_directory(dir_path).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_validate_directory_nonexistent() {
        let result = validate_directory("/path/that/does/not/exist".to_string()).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_validate_directory_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let result = validate_directory(file_path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false because it's a file, not a directory
    }

    #[tokio::test]
    async fn test_get_home_directory() {
        let result = get_home_directory().await;
        assert!(result.is_ok());
        let home = result.unwrap();
        assert!(!home.is_empty());

        // Should be either HOME or USERPROFILE environment variable
        let expected = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        assert_eq!(home, expected);
    }

    #[tokio::test]
    async fn test_is_home_directory() {
        let home = get_home_directory().await.unwrap();

        let result = is_home_directory(home.clone()).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test with a non-home directory
        let result = is_home_directory("/definitely/not/home".to_string()).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_get_parent_directory() {
        // Test normal path
        let result = get_parent_directory("/path/to/file".to_string()).await;
        assert!(result.is_ok());
        let parent = result.unwrap();
        assert!(parent.is_some());

        #[cfg(windows)]
        {
            // On Windows, the path handling might be different
            let parent_path = parent.unwrap();
            assert!(parent_path.contains("path"));
        }
        #[cfg(not(windows))]
        {
            assert_eq!(parent.unwrap(), "/path/to");
        }
    }

    #[tokio::test]
    async fn test_get_parent_directory_root() {
        let result = get_parent_directory("/".to_string()).await;
        assert!(result.is_ok());
        let parent = result.unwrap();
        assert!(parent.is_none()); // Root has no parent
    }

    #[tokio::test]
    async fn test_list_directory_contents() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files and subdirectories
        let file_path = dir_path.join("test_file.txt");
        fs::write(&file_path, "test content").unwrap();

        let subdir_path = dir_path.join("test_subdir");
        fs::create_dir(&subdir_path).unwrap();

        let result = list_directory_contents(dir_path.to_string_lossy().to_string()).await;
        assert!(result.is_ok());

        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);

        // Check that directory comes before file (directories are sorted first)
        let dir_entry = &entries[0];
        let file_entry = &entries[1];

        assert!(dir_entry.is_directory);
        assert_eq!(dir_entry.name, "test_subdir");
        assert!(dir_entry.full_path.ends_with("test_subdir"));
        assert!(dir_entry.size.is_none());
        assert!(!dir_entry.is_symlink);

        assert!(!file_entry.is_directory);
        assert_eq!(file_entry.name, "test_file.txt");
        assert!(file_entry.full_path.ends_with("test_file.txt"));
        assert_eq!(file_entry.size, Some("test content".len() as u64));
        assert!(!file_entry.is_symlink);
    }
}
