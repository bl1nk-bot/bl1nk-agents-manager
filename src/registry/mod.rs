// =============================================================
// Registry Module - ระบบจัดการ Keyword Registry
// =============================================================
//
// จุดประสงค์:
// - ใช้ bl1nk-keyword-validator core library เป็นฐาน
// - เพิ่มความสามารถในการ validate, search, และจัดการ registry
// - TODO: เพิ่ม monitoring, weight calculation, honesty checks
//
// การใช้งาน:
//   use crate::registry::RegistryService;
//   let service = RegistryService::from_file("registry.json")?;
//   let results = service.search("visual-story", None);
// =============================================================

// TODO: เพิ่ม export types ที่จำเป็นจาก schema module (ใช้ใน tests)
#[cfg(test)]
use bl1nk_keyword_core::schema::{
    Metadata, KeywordGroup, FieldSchema, CustomFieldConfig,
    ValidationConfig, ValidationRules,
};

pub use bl1nk_keyword_core::{
    KeywordRegistry, KeywordSearch, Validator, 
    ValidationError, ValidatorError, SearchResult,
    load_registry, generate_markdown,
};

/// RegistryService - wrapper สำหรับจัดการ keyword registry
/// 
/// ใช้ library จาก vendor/bl1nk-keyword-validator/core
/// โดยมีหน้าที่หลัก:
/// - โหลด/บันทึก registry จากไฟล์ JSON
/// - ค้นหา keywords ด้วย exact, partial, fuzzy match
/// - Validate entries ตาม schema rules
/// 
/// TODO: เพิ่ม monitoring layer สำหรับติดตามพฤติกรรม
/// TODO: เพิ่ม weight calculation จาก monitoring data
/// TODO: เพิ่ม honesty checks สำหรับตรวจสอบความโปร่งใส
pub struct RegistryService {
    /// ข้อมูล registry ที่โหลดมา
    registry: KeywordRegistry,
    /// ตัวค้นหา keyword (ใช้ fuzzy-matcher)
    search: KeywordSearch,
    /// ตัวตรวจสอบความถูกต้องตาม schema
    validator: Validator,
}

impl RegistryService {
    // ---------------------------------------------------------
    // Constructors
    // ---------------------------------------------------------

    /// โหลด registry จากไฟล์ JSON
    /// 
    /// # Arguments
    /// * `path` - พาธไปยังไฟล์ registry JSON
    /// 
    /// # Returns
    /// * `Ok(RegistryService)` - โหลดสำเร็จ
    /// * `Err(ValidatorError)` - ไฟล์ไม่พบ หรือ JSON ผิดรูปแบบ
    /// 
    /// # Examples
    /// ```no_run
    /// let service = RegistryService::from_file(".config/registry.json")?;
    /// ```
    /// 
    /// TODO: เพิ่ม hot-reload เมื่อไฟล์เปลี่ยนแปลง
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ValidatorError> {
        // ใช้ load_registry จาก vendor library
        let registry = load_registry(&path)?;
        
        // สร้าง search และ validator instances
        let search = KeywordSearch::new(registry.clone());
        let validator = Validator::new(registry.clone());
        
        Ok(Self {
            registry,
            search,
            validator,
        })
    }

    /// สร้าง RegistryService ใหม่จากข้อมูลที่มี
    /// 
    /// # Arguments
    /// * `registry` - KeywordRegistry struct ที่สร้างไว้แล้ว
    pub fn new(registry: KeywordRegistry) -> Self {
        let search = KeywordSearch::new(registry.clone());
        let validator = Validator::new(registry.clone());
        
        Self {
            registry,
            search,
            validator,
        }
    }

    // ---------------------------------------------------------
    // Search Operations
    // ---------------------------------------------------------

    /// ค้นหา keyword จาก query
    /// 
    /// รองรับ:
    /// - Exact match: ค้นหาตรงตัว (score: 10000)
    /// - Partial match: ค้นหาเป็นส่วนหนึ่ง (score: 5000+)
    /// - Fuzzy match: ค้นหาแบบยืดหยุ่น typo tolerant (score: varies)
    /// 
    /// # Arguments
    /// * `query` - คำค้นหา (รองรับภาษาไทย/อังกฤษ)
    /// * `group_id` - จำกัดกลุ่ม (None = ค้นหาทั้งหมด)
    /// 
    /// # Returns
    /// Vec<SearchResult> เรียงตามคะแนน (มาก -> น้อย)
    /// 
    /// TODO: เพิ่ม semantic search (embeddings)
    pub fn search(&self, query: &str, group_id: Option<&str>) -> Vec<SearchResult> {
        self.search.search(query, group_id)
    }

    // ---------------------------------------------------------
    // Validation Operations
    // ---------------------------------------------------------

    /// Validate ทั้ง registry
    /// 
    /// ตรวจสอบ:
    /// - Version compatibility
    /// - Duplicate IDs ภายในกลุ่ม
    /// - Duplicate aliases ข้ามกลุ่ม
    /// - Broken relationships (relatedIds)
    /// - Required fields
    /// - Pattern matching
    /// - Enum values
    /// - Alias length constraints
    /// 
    /// # Returns
    /// * `Ok(())` - ผ่านการตรวจสอบ
    /// * `Err(Vec<ValidationError>)` - มี errors (อาจมีหลายตัว)
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        self.validator.validate_registry()
    }

    /// Validate entry เดียวตาม group_id
    /// 
    /// # Arguments
    /// * `group_id` - กลุ่มของ entry
    /// * `entry` - ข้อมูล entry ในรูปแบบ JSON Value
    pub fn validate_entry(
        &self, 
        group_id: &str, 
        entry: &serde_json::Value
    ) -> Result<(), Vec<ValidationError>> {
        self.validator.validate_entry(group_id, entry)
    }

    // ---------------------------------------------------------
    // Data Access
    // ---------------------------------------------------------

    /// ดู registry data
    pub fn registry(&self) -> &KeywordRegistry {
        &self.registry
    }

    /// Generate markdown documentation
    pub fn to_markdown(&self) -> String {
        generate_markdown(&self.registry)
    }

    // ---------------------------------------------------------
    // TODO: Future Features
    // ---------------------------------------------------------
    // TODO: pub fn record_monitoring(&mut self, record: MonitoringRecord)
    //   - บันทึกพฤติกรรมจาก people, model, tool, input, output
    //   - เก็บหลักฐานการทำงานจริง vs สิ่งที่โมเดลอ้าง
    //
    // TODO: pub fn calculate_weights(&mut self)
    //   - คำนวณน้ำหนัก keywords จาก monitoring data
    //   - ใช้ self-claim, actual performance, tool success rate
    //
    // TODO: pub fn check_honesty(&self, response: &ModelResponse) -> HonestyReport
    //   - ตรวจสอบว่าโมเดลแสดงหลักฐานหรือไม่
    //   - ตรวจจับ false completion, evidence missing
    //
    // TODO: pub fn generate_effectiveness_report(&self) -> EffectivenessReport
    //   - พิสูจน์ว่าระบบทำให้โมเดลดีขึ้นจริง
    //   - เปรียบเทียบ before/after metrics
}

// =============================================================
// Tests
// =============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper: สร้าง registry สำหรับทดสอบ
    // ใช้ types จาก vendor library ตรงๆ
    fn make_test_registry() -> KeywordRegistry {
        // สร้าง base_fields_schema สำหรับ group skills
        let mut base_fields: std::collections::HashMap<String, FieldSchema> = std::collections::HashMap::new();
        base_fields.insert(
            "id".to_string(),
            FieldSchema {
                field_type: "string".to_string(),
                item_type: None,
                pattern: Some("^skill-[a-z][a-z0-9-]*$".to_string()),
                values: None,
                required: Some(true), // required field
                max_length: None,
                description: "Skill ID".to_string(),
            },
        );
        base_fields.insert(
            "aliases".to_string(),
            FieldSchema {
                field_type: "array".to_string(),
                item_type: Some("string".to_string()),
                pattern: None,
                values: None,
                required: Some(true), // required field
                max_length: None,
                description: "Search aliases (aliases)".to_string(),
            },
        );
        base_fields.insert(
            "description".to_string(),
            FieldSchema {
                field_type: "string".to_string(),
                item_type: None,
                pattern: None,
                values: None,
                required: Some(true), // required field
                max_length: Some(255),
                description: "Skill description".to_string(),
            },
        );

        KeywordRegistry {
            version: "1.1.0".to_string(),
            metadata: Metadata {
                last_updated: "2026-04-12T00:00:00Z".to_string(),
                description: "Test Registry สำหรับทดสอบ".to_string(),
                owner: "test".to_string(),
            },
            groups: vec![KeywordGroup {
                group_id: "skills".to_string(),
                group_name: "Skills".to_string(),
                description: "Test skills สำหรับทดสอบการค้นหา".to_string(),
                base_fields_schema: base_fields,
                custom_field_allowed: CustomFieldConfig {
                    enabled: false,
                    max_one: false,
                    description: "".to_string(),
                    examples: None,
                },
                entries: vec![
                    json!({
                        "id": "skill-test",
                        "aliases": ["test", "ทดสอบ", "การทดสอบ"],
                        "description": "Test skill สำหรับ validation และ search"
                    }),
                    json!({
                        "id": "skill-visual",
                        "aliases": ["visual-story", "ภาพเรื่อง", "visual"],
                        "description": "Visual story generation skill"
                    }),
                ],
            }],
            validation: ValidationConfig {
                rules: ValidationRules {
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
        // ทดสอบ: สร้าง RegistryService ได้ถูกต้อง
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        assert_eq!(service.registry().version, "1.1.0");
        assert_eq!(service.registry().groups.len(), 1);
        assert_eq!(service.registry().groups[0].entries.len(), 2);
    }

    #[test]
    fn test_registry_service_search_exact() {
        // ทดสอบ: ค้นหาแบบ exact match
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        let results = service.search("test", None);
        
        // ต้องเจอ skill-test (exact match)
        assert!(!results.is_empty(), "ควรเจอผลลัพธ์อย่างน้อย 1 รายการ");
        assert_eq!(results[0].id, "skill-test");
        assert_eq!(results[0].match_type, "exact");
    }

    #[test]
    fn test_registry_service_search_thai() {
        // ทดสอบ: ค้นหาด้วยภาษาไทย
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        let results = service.search("ทดสอบ", None);
        
        // ต้องเจอ skill-test (alias ภาษาไทย)
        assert!(!results.is_empty(), "ควรเจอผลลัพธ์สำหรับคำค้นหาภาษาไทย");
    }

    #[test]
    fn test_registry_service_search_fuzzy() {
        // ทดสอบ: ค้นหาแบบ fuzzy (typo tolerant)
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        // พิมพ์ผิดเล็กน้อย (v-i-s-u-a-l -> vual)
        let results = service.search("vual", None);
        
        // ควรเจอผ่าน fuzzy match
        // หมายเหตุ: fuzzy-matcher ต้องการตัวอักษรเรียงลำดับ
        assert!(results.iter().any(|r| r.id == "skill-visual"));
    }

    #[test]
    fn test_registry_service_validate() {
        // ทดสอบ: validate registry ที่ถูกต้อง
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        let result = service.validate();
        assert!(result.is_ok(), "Registry ที่ถูกต้องควรผ่าน validation: {:?}", result);
    }

    #[test]
    fn test_registry_service_validate_entry_valid() {
        // ทดสอบ: validate entry ที่ถูกต้อง
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        let valid_entry = json!({
            "id": "skill-new",
            "aliases": ["new-skill", "ทักษะใหม่"],
            "description": "A new test skill สำหรับทดสอบ"
        });
        
        let result = service.validate_entry("skills", &valid_entry);
        assert!(result.is_ok(), "Entry ที่ถูกต้องควรผ่าน validation");
    }

    #[test]
    fn test_registry_service_validate_entry_invalid() {
        // ทดสอบ: validate entry ที่ผิด (missing required fields)
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        // entry นี้ไม่มี aliases และ description (required fields)
        let invalid_entry = json!({
            "id": "skill-invalid"
        });
        
        let result = service.validate_entry("skills", &invalid_entry);
        assert!(result.is_err(), "Entry ที่ขาด required fields ต้องไม่ผ่าน validation");
        
        let errors = result.unwrap_err();
        assert!(!errors.is_empty(), "ควรมีอย่างน้อย 1 error");
    }

    #[test]
    fn test_registry_service_markdown() {
        // ทดสอบ: generate markdown
        let registry = make_test_registry();
        let service = RegistryService::new(registry);
        
        let md = service.to_markdown();
        
        assert!(md.contains("Test Registry"), "Markdown ควรมีชื่อ registry");
        assert!(md.contains("skill-test"), "Markdown ควรมี entry IDs");
        assert!(md.contains("skills"), "Markdown ควรมี group names");
    }
}
