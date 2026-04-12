use bl1nk_agents_manager::system::discovery::AgentDiscovery;
use bl1nk_agents_manager::registry::RegistryService;
use bl1nk_agents_manager::config::AgentConfig;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Starting Real-World Validation for bl1nk-agents...");

    // 1. Agent Discovery from Markdown Files
    println!("\n--- 1. Agent Discovery (from agents/*.md) ---");
    let agents_dir = "agents/";
    match AgentDiscovery::discover_agents(agents_dir) {
        Ok(agents) => {
            println!("✅ Discovered {} valid agents from markdown files", agents.len());
            
            // 2. Build Registry Context
            println!("\n--- 2. Building Registry Context ---");
            let registry_path = "agents/agents.json";
            match RegistryService::from_file(registry_path) {
                Ok(service) => {
                    println!("✅ Registry Service Ready");
                    
                    // 3. Agent Coverage Analysis
                    println!("\n--- 3. Agent Coverage Analysis (Real Data) ---");
                    let agents_slice: &[AgentConfig] = &agents;
                    let report = service.analyze_agent_coverage(agents_slice);
                    println!("{}", report.summary());
                    
                    if !report.missing_coverage.is_empty() {
                        println!("⚠️ Missing Coverage for Registry IDs:");
                        for id in &report.missing_coverage {
                            println!("   - {}", id);
                        }
                    }

                    if !report.agent_mapping.is_empty() {
                        println!("🔗 Top Agent Mappings:");
                        for (reg_id, agent_ids) in report.agent_mapping.iter().take(5) {
                            println!("   - Registry: {} -> Agents: {:?}", reg_id, agent_ids);
                        }
                    }

                    // 4. Agent Spec Validation
                    println!("\n--- 4. Agent Spec Validation (vs .config/schema-agent.json) ---");
                    let mut valid_specs = 0;
                    for agent in &agents {
                        let agent_json = serde_json::to_value(agent)?;
                        match service.validate_agent_spec(&agent_json) {
                            Ok(_) => valid_specs += 1,
                            Err(e) => println!("❌ Validation failed for {}: {}", agent.id, e),
                        }
                    }
                    println!("✅ Validated Agent Specs: {}/{}", valid_specs, agents.len());

                    // 5. Reputation & Score Report (Dynamic & Behavioral)
                    println!("\n--- 5. Behavioral Integrity Ledger (Persistent) ---");
                    let weight_registry = bl1nk_agents_manager::registry::WeightRegistry::load().await?;
                    println!("{:<20} | {:<6} | {:<6} | {:<6} | {:<6} | {:<10}", 
                        "Agent ID", "Succ", "Hidden", "Rules", "Bypass", "Trust Score");
                    println!("{:-<80}", "");
                    for agent in &agents {
                        let stats = weight_registry.stats.get(&agent.id)
                            .cloned()
                            .unwrap_or_default();
                        let trust = weight_registry.get_trust_score(&agent.id);
                        
                        println!("{:<20} | {:<6} | {:<6} | {:<6} | {:<6} | {:<10.2}", 
                            agent.id, 
                            stats.success_count, 
                            stats.hidden_error_count, 
                            stats.rule_violation_count,
                            stats.bypassed_ask_user_count,
                            trust
                        );
                    }
                },
                Err(e) => println!("❌ Registry Service Failed: {} (Ensure agents/agents.json is valid)", e),
            }
        },
        Err(e) => println!("❌ Discovery Failed: {}", e),
    }

    // 6. Content Integrity Check
    println!("\n--- 6. Content Integrity Check ---");
    let entries = fs::read_dir(agents_dir)?;
    let mut total = 0;
    let mut thai_count = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            total += 1;
            let content = fs::read_to_string(&path)?;
            if content.chars().any(|c| (c as u32) >= 0x0E00 && (c as u32) <= 0x0E7F) {
                thai_count += 1;
            }
        }
    }

    println!("📊 Total Markdown Files: {}", total);
    println!("🇹🇭 Files with Thai Content: {}/{}", thai_count, total);
    
    if thai_count < total {
        println!("⚠️ Warning: {} files are missing Thai descriptions/logic (Rule bl1nk-standard)", total - thai_count);
    }

    println!("\n✅ Validation Cycle Complete.");
    Ok(())
}
