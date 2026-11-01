use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub file: String,
    pub category: String,
    pub description: String,
    pub use_cases: Vec<String>,
    pub tags: Vec<String>,
    pub complexity: String,
    pub outputs: Vec<String>,
    #[serde(default)]
    pub personality: String,
}

impl Agent {
    /// Get a human-readable category name
    pub fn display_category(&self) -> String {
        // Convert common category abbreviations to full names
        match self.category.as_str() {
            "Available" => "Available".to_string(),
            "Dev" => "Development".to_string(),
            "Res" => "Research".to_string(),
            "Util" => "Utility".to_string(),
            "Creat" => "Creative".to_string(),
            "Plan" => "Planning".to_string(),
            "" => "Uncategorized".to_string(),
            _ => self.category.to_string(),
        }
    }

    /// Get a human-readable complexity level
    pub fn display_complexity(&self) -> String {
        match self.complexity.as_str() {
            "Unknown" => "Standard".to_string(),
            "" => "Standard".to_string(),
            _ => self.complexity.to_string(),
        }
    }
}
