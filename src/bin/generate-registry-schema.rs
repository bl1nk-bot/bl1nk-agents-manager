//! Generate JSON schema สำหรับ Registry (v1.7.2 Standard)
//!
//! รัน: cargo run --bin generate-registry-schema

use bl1nk_agents_manager::registry::schema::Registry;
use schemars::schema_for;
use std::fs;

fn main() -> anyhow::Result<()> {
    let schema = schema_for!(Registry);
    let json = serde_json::to_string_pretty(&schema)?;

    // อัปเดตพาธเป็นมาตรฐานใหม่ (v1.7)
    let schema_path = "config/v1.7/policy-schema.json";
    fs::create_dir_all("config/v1.7")?;
    fs::write(schema_path, &json)?;

    println!("✅ Generated updated registry schema at {}", schema_path);
    println!("📄 Schema size: {} bytes", json.len());

    Ok(())
}
