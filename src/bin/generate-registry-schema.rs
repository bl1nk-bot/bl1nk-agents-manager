//! Generate JSON schema สำหรับ Registry
//!
//! รัน: cargo run --bin generate-registry-schema

use bl1nk_agents_manager::registry::schema::Registry;
use schemars::schema_for;
use std::fs;

fn main() -> anyhow::Result<()> {
    let schema = schema_for!(Registry);
    let json = serde_json::to_string_pretty(&schema)?;

    let schema_path = ".config/registry-schema.json";
    fs::create_dir_all(".config")?;
    fs::write(schema_path, &json)?;

    println!("✅ Generated registry schema at {}", schema_path);
    println!("📄 Schema size: {} bytes", json.len());

    Ok(())
}
