# Specification: JSON Schema Registry & Knowledge Backbone

## Overview

เพิ่มระบบ **Knowledge Registry** ที่ใช้ JSON Schema เป็น Single Source of Truth สำหรับเก็บ skills, projects, agents และ metadata อื่นๆ เพื่อให้การ routing ฉลาดขึ้น และ permission system มี context-aware decisions

## Goals

1. **Centralized Knowledge:** รวม keywords, skills, projects, agents ไว้ที่เดียว
2. **Intelligent Routing:** ใช้ registry data ช่วยเลือก agent ที่เหมาะสมตาม skills และ context
3. **Context-Aware Permissions:** กำหนด permission policies ตาม metadata ใน registry
4. **Extensible Schema:** เพิ่ม/ลด fields ได้ง่ายก่อนลง DB

## Functional Requirements

### 1. Registry Schema

#### 1.1 Core Structure
```json
{
  "version": "1.0.0",
  "metadata": {
    "last_updated": "2026-04-12T00:00:00Z",
    "description": "BL1NK Agents Knowledge Base",
    "owner": "bl1nk-agents-manager"
  },
  "groups": [
    {
      "groupId": "skills",
      "groupName": "Agent Skills",
      "description": "ความสามารถของ agents",
      "baseFieldsSchema": {
        "id": {"type": "string", "pattern": "^[a-z][a-z0-9-]*$"},
        "name": {"type": "string", "maxLength": 100},
        "description": {"type": "string"},
        "category": {"type": "string", "enum": ["code", "docs", "analysis", "creative"]},
        "relatedIds": {"type": "array", "items": {"type": "string"}}
      },
      "customFieldAllowed": {
        "enabled": true,
        "maxOne": false,
        "description": "อนุญาตให้เพิ่ม fields เฉพาะ skill ได้"
      },
      "entries": []
    },
    {
      "groupId": "projects",
      "groupName": "Project Templates",
      "description": "โครงสร้างโปรเจคที่รู้จัก",
      "baseFieldsSchema": {...},
      "entries": []
    },
    {
      "groupId": "agents",
      "groupName": "Agent Configurations",
      "description": "การตั้งค่า agents",
      "baseFieldsSchema": {...},
      "entries": []
    },
    {
      "groupId": "keywords",
      "groupName": "Keyword Registry",
      "description": "คำหลักสำหรับ routing และ search",
      "baseFieldsSchema": {
        "id": {"type": "string"},
        "term": {"type": "string"},
        "aliases": {"type": "array", "items": {"type": "string"}},
        "relatedSkills": {"type": "array", "items": {"type": "string"}},
        "relatedProjects": {"type": "array", "items": {"type": "string"}}
      },
      "entries": []
    }
  ]
}
```

#### 1.2 Validation Rules
```json
{
  "validation": {
    "rules": {
      "aliasMinLength": 2,
      "aliasMaxLength": 50,
      "descriptionMinLength": 10,
      "descriptionMaxLength": 500,
      "customFieldPerEntry": 5,
      "requiredBaseFields": ["id", "name", "description"]
    },
    "errorMessages": {
      "MISSING_FIELD": "ขาด field ที่จำเป็น: {field}",
      "INVALID_ID": "id ไม่ตรงรูปแบบ: ต้องเป็น {pattern}",
      "ALIAS_TOO_LONG": "alias '{alias}' ยาวเกิน {max} ตัวอักษร"
    }
  }
}
```

### 2. Registry Service

#### 2.1 Core API
```rust
pub struct RegistryService {
    registry: KnowledgeRegistry,
    watch_handle: Option<WatchHandle>, // สำหรับ hot-reload
}

impl RegistryService {
    /// โหลด registry จากไฟล์ JSON
    pub async fn load_from_file(path: &Path) -> Result<Self>;
    
    /// บันทึก registry กลับไปไฟล์ JSON
    pub async fn save_to_file(&self, path: &Path) -> Result<()>;
    
    /// ค้นหา entries ที่ตรงกับ query (ใช้ BM25)
    pub fn search(&self, query: &str, group_id: Option<&str>) -> Vec<&Entry>;
    
    /// หา entries ที่เกี่ยวข้องกับ entry นี้
    pub fn get_related(&self, entry_id: &str) -> Vec<&Entry>;
    
    /// สร้าง graph ของความสัมพันธ์
    pub fn build_knowledge_graph(&self) -> HashMap<String, Vec<String>>;
    
    /// ตรวจสอบ entry ใหม่ตาม validation rules
    pub fn validate_entry(&self, group_id: &str, entry: &Value) -> Result<(), ValidationError>;
    
    /// เพิ่ม entry ใหม่
    pub fn add_entry(&mut self, group_id: &str, entry: Value) -> Result<()>;
    
    /// ลบ entry
    pub fn remove_entry(&mut self, group_id: &str, entry_id: &str) -> Result<()>;
}
```

#### 2.2 Hot-Reload
- ใช้ `notify` crate เฝ้าดูการเปลี่ยนแปลงไฟล์ registry.json
- เมื่อไฟล์เปลี่ยน → โหลดใหม่และแจ้งเตือน subscribers

### 3. Integration Points

#### 3.1 Agent Routing
```rust
// ใน AgentRouter::route_task
let relevant_skills = registry.search(&task_type, Some("skills"));
let keyword_match = registry.search(&user_prompt, Some("keywords"));

// ใช้ relatedIds หา skills เพิ่มเติม
let mut skill_ids: HashSet<_> = relevant_skills.iter()
    .flat_map(|e| registry.get_related(&e.id))
    .collect();

// เลือก agent ที่มี skills ตรงกัน
let best_agent = self.select_agent_with_skills(&skill_ids)?;
```

#### 3.2 Permission System
```rust
// ใน PermissionManager
pub fn check_permission(&self, agent_id: &str, tool: &str) -> PermissionResult {
    let agent_entry = self.registry.get_agent_entry(agent_id);
    let skill = self.registry.get_related_skill(tool);
    
    // ตรวจสอบ metadata จาก registry
    if skill.category == "documentation" {
        return PermissionResult::Allow; // อนุญาตง่ายกว่า
    }
    
    if agent_entry.status == "archived" {
        return PermissionResult::RequireApproval;
    }
    
    // ... logic อื่นๆ
}
```

#### 3.3 Schema Validation
- ทุกครั้งที่เพิ่ม/แก้ไข entry ต้อง validate ตาม JSON Schema
- รองรับ custom fields ตามที่กำหนดใน `customFieldAllowed`

### 4. CLI Commands

```rust
#[derive(clap::Subcommand)]
pub enum RegistryCommand {
    /// เพิ่ม entry ใหม่
    Add {
        group: String,
        #[arg(short, long)]
        file: Option<PathBuf>,
        #[arg(short, long)]
        json: Option<String>,
    },
    
    /// ค้นหา entries
    Search {
        query: String,
        #[arg(short, long)]
        group: Option<String>,
        #[arg(long)]
        fuzzy: bool, // BM25 fuzzy search
    },
    
    /// แสดงความสัมพันธ์
    Graph {
        id: String,
        #[arg(long)]
        format: Option<String>, // "text", "mermaid", "json"
    },
    
    /// แสดง tree ของความสัมพันธ์
    Tree {
        id: String,
        #[arg(long, default_value = "2")]
        depth: usize,
    },
    
    /// ตรวจสอบความถูกต้อง
    Validate,
    
    /// Export registry
    Export {
        #[arg(long, default_value = "json")]
        format: String,
    },
}
```

#### 4.1 Output Examples

**Search:**
```
🔍 Searching for "api-docs" in all groups...

Found 3 matches:
1. [skills] write-api-reference - สร้างเอกสาร API
2. [projects] mintlify-docs-builder - เครื่องมือสร้าง docs
3. [keywords] api-documentation → aliases: [api-docs, swagger-docs]
```

**Graph:**
```
🕸️ Knowledge Graph for "write-api-reference":

write-api-reference
├── relates-to → mintlify-docs-builder
├── relates-to → code-analysis
└── relates-to → technical-writing
```

**Tree (nested):**
```
🌳 Skill Tree: "code-generation"

code-generation (depth: 0)
├── generates → web-apps
│   ├── relates-to → react-templates
│   └── relates-to → vue-templates
├── generates → api-endpoints
│   └── relates-to → openapi-specs
└── generates → tests
    └── relates-to → testing-frameworks
```

### 5. Non-Functional Requirements

- **Performance:** Search <50ms สำหรับ registry ขนาด 1000 entries
- **Memory:** ใช้ memory <10MB สำหรับ registry ปกติ
- **Hot-Reload:** ตรวจจับการเปลี่ยนแปลงไฟล์ภายใน 1 วินาที
- **Validation:** ตรวจสอบ entry ใหม่ภายใน 10ms

### 6. Acceptance Criteria

- [ ] โหลด registry.json ได้ถูกต้อง
- [ ] เพิ่ม/ลบ/แก้ไข entries ได้พร้อม validate
- [ ] Search ด้วย BM25 ได้ผลลัพธ์ถูกต้อง
- [ ] แสดง graph/tree ของความสัมพันธ์ได้
- [ ] Routing ใช้ registry data ช่วยเลือก agent
- [ ] Permission system ใช้ metadata กำหนด policy
- [ ] CLI commands ทำงานครบ
- [ ] Hot-reload ทำงานเมื่อไฟล์เปลี่ยน
- [ ] Tests >90% coverage

### 7. Out of Scope

- Database sync (จะทำใน track ถัดไปเมื่อ registry นิ่งแล้ว)
- Web UI สำหรับจัดการ registry
- Distributed registry (multi-instance sync)
- Real-time collaboration
