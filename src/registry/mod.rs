// TODO: สร้าง RegistryService ที่ใช้ bl1nk-keyword-validator core library
// สำหรับ validate และ search keywords ใน registry

pub use bl1nk_keyword_core::{
    KeywordRegistry, KeywordSearch, Validator, ValidationError, ValidatorError, SearchResult,
    load_registry, save_registry, generate_markdown,
};

/// RegistryService wrapper สำหรับ bl1nk-agents-manager
/// TODO: เพิ่ม monitoring, weight calculation, honesty checks
pub struct RegistryService {
    registry: KeywordRegistry,
    search: KeywordSearch,
    validator: Validator,
}

impl RegistryService {
    /// โหลด registry จากไฟล์
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ValidatorError> {
        let registry = load_registry(&path)?;
        let search = KeywordSearch::new(registry.clone());
        let validator = Validator::new(registry.clone());
        
        Ok(Self {
            registry,
            search,
            validator,
        })
    }

    /// สร้าง registry ใหม่จากข้อมูลที่มี
    pub fn new(registry: KeywordRegistry) -> Self {
        let search = KeywordSearch::new(registry.clone());
        let validator = Validator::new(registry.clone());
        
        Self {
            registry,
            search,
            validator,
        }
    }

    /// ค้นหา keyword
    pub fn search(&self, query: &str, group_id: Option<&str>) -> Vec<SearchResult> {
        self.search.search(query, group_id)
    }

    /// Validate registry
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        self.validator.validate_registry()
    }

    /// Validate entry เดียว
    pub fn validate_entry(&self, group_id: &str, entry: &serde_json::Value) -> Result<(), Vec<ValidationError>> {
        self.validator.validate_entry(group_id, entry)
    }

    /// ดู registry data
    pub fn registry(&self) -> &KeywordRegistry {
        &self.registry
    }

    /// Generate markdown documentation
    pub fn to_markdown(&self) -> String {
        generate_markdown(&self.registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_test_registry() -> KeywordRegistry {
        // ใช้ example จาก vendor tests
        KeywordRegistry {
            version: "1.1.0".to_string(),
            metadata: bl1nk_keyword_core::Metadata {
                last_updated: "2026-04-12T00:00:00Z".to_string(),
                description: "Test Registry".to_string(),
                owner: "test".to_string(),
            },
            groups: vec![bl1nk_keyword_core::KeywordGroup {
                group_id: "skills".to_string(),
                group_name: "Skills".to_string(),
                description: "Test skills".to_string(),
                base_fields_schema: std::collections::HashMap::new(),
                custom_field_allowed: bl1nk_keyword_core::CustomFieldConfig {
                    enabled: false,
                    max_one: false,
                    description: "".to_string(),
                    examples: None,
                },
                entries: vec![
                    json!({
                        "id": "skill-test",
                        "aliases": ["test", "ทดสอบ"],
                        "description": "Test skill for validation"
                    }),
                ],
            }],
            validation: bl1nk_keyword_core::ValidationConfig {
                rules: bl1nk_keyword_core::ValidationRules {
                    alias_min_length: 2,
                    alias_max_length: 50,
                    description_min_length: 10,
                    description_max_length: 255,
                    custom_field_per_entry: 1,
                    required_base_fields: vec!["id".to_string(), "aliases".to_string(), "description".to_string()],
                },
                error_messages: std::collections::HashMap::new(),
            },
        }
    }

    #[test]
    fn test_registry_service_new() {
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        assert_eq!(service.registry().version, "1.1.0");
        assert_eq!(service.registry().groups.len(), 1);
    }

    #[test]
    fn test_registry_service_search() {
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        let results = service.search("test", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "skill-test");
        assert_eq!(results[0].match_type, "exact");
    }

    #[test]
    fn test_registry_service_validate() {
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        // Should pass validation
        assert!(service.validate().is_ok());
    }

    #[test]
    fn test_registry_service_validate_entry() {
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        let valid_entry = json!({
            "id": "skill-new",
            "aliases": ["new-skill", "ทักษะใหม่"],
            "description": "A new test skill for validation testing"
        });
        
        assert!(service.validate_entry("skills", &valid_entry).is_ok());
        
        // Test invalid entry (missing required field)
        let invalid_entry = json!({
            "id": "skill-invalid"
            // missing aliases and description
        });
        
        let result = service.validate_entry("skills", &invalid_entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_service_markdown() {
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        let md = service.to_markdown();
        assert!(md.contains("Test Registry"));
        assert!(md.contains("skill-test"));
    }
}
