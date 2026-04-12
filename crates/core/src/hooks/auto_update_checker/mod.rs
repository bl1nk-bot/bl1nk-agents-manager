pub mod constants;

use serde::{Deserialize, Serialize};
use crate::hooks::auto_update_checker::constants::{GITHUB_REPO_URL, version_file};
use std::fs;

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
}

pub struct AutoUpdateCheckerHook;

impl AutoUpdateCheckerHook {
    pub fn new() -> Self { Self }

    pub async fn check_for_update(&self, current_version: &str) -> Option<String> {
        let client = reqwest::Client::new();
        let res = client.get(GITHUB_REPO_URL)
            .header("User-Agent", "bl1nk-agents-manager")
            .send()
            .await
            .ok()?;

        if let Ok(release) = res.json::<GithubRelease>().await {
            let latest = release.tag_name.trim_start_matches('v');
            if latest != current_version {
                return Some(latest.to_string());
            }
        }
        None
    }

    pub fn get_cached_version(&self) -> Option<String> {
        fs::read_to_string(version_file()).ok().map(|s| s.trim().to_string())
    }
}
