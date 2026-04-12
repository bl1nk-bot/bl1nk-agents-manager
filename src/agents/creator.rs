// src/agents/creator.rs
//! ตัวสร้างเอเจนต์ - สร้างข้อกำหนดเอเจนต์จากความต้องการด้วยภาษาธรรมชาติ
//! ไฟล์ผลลัพธ์เป็น Markdown (.md) พร้อม frontmatter แบบ YAML ตาม schema v1.2.1
//! และบันทึกประวัติการสร้างลงใน agent-log.json

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

// --- โครงสร้างข้อมูลตาม JSON Schema v1.2.1 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    Instruction,
    Explicit,
    Structure,
    Ctx,
    Assumption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weight {
    pub mode: f64,
    #[serde(rename = "type")]
    pub type_weight: f64,
    pub tool: f64,
    pub evidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub hierarchy: Vec<EvidenceType>,
    pub weight: Weight,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<Vec<EvidenceType>>,
}

/// ข้อกำหนดเอเจนต์ที่สร้างโดย AgentCreator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub identifier: String,          // ชื่อ slug สำหรับชื่อไฟล์
    pub name: String,                // ชื่อแสดงผล (ไม่เกิน 64 ตัวอักษร)
    pub description: String,         // คำอธิบาย (มี use case อย่างน้อย 2 อย่าง ไม่เกิน 300 ตัวอักษร)
    pub when_to_use: String,         // ข้อความดั้งเดิมที่ผู้ใช้ป้อน
    pub system_prompt: String,       // เนื้อหา system prompt ที่สร้าง
    pub model: Option<String>,       // โมเดลที่เลือก (optional)
    pub color: Option<String>,       // สี (optional)
    pub tools: Vec<String>,          // รายการเครื่องมือ
    pub mode: String,                // all, primary, subagent
    pub task_type: String,           // class, trans, code, plan, create, image, analysis, docs, assistant, general
    pub capabilities: Vec<String>,   // ความสามารถหลัก
    pub priority: u8,                // ระดับความสำคัญ (0-100)
    pub permission: u32,             // คะแนนสิทธิ์ 0-999
    pub permission_policy: PermissionPolicy,
}

/// ตัวสร้างเอเจนต์
pub struct AgentCreator {
    output_dir: String,
}

impl AgentCreator {
    pub fn new(_template_dir: String, output_dir: String) -> Self {
        Self {
            output_dir,
        }
    }

    /// จุดเริ่มต้นหลัก: สร้างเอเจนต์จากความต้องการของผู้ใช้
    pub async fn create_agent(
        &self,
        requirements: &str,
        _context: Option<Value>,
    ) -> Result<AgentSpec> {
        tracing::info!("🤖 Agent Creator: กำลังสร้างเอเจนต์จากความต้องการ");
        tracing::debug!("ความต้องการ: {}", requirements);

        // ตรวจสอบว่าไดเรกทอรีผลลัพธ์มีอยู่แล้ว
        fs::create_dir_all(&self.output_dir)
            .context("ไม่สามารถสร้างไดเรกทอรีผลลัพธ์ได้")?;

        // 1. แยกวิเคราะห์ความต้องการ
        let parsed = self.parse_requirements(requirements)?;
        tracing::debug!("ความต้องการที่ถูกแยกวิเคราะห์: {:?}", parsed);

        // 2. สร้างตัวระบุ (slug)
        let identifier = self.generate_identifier(&parsed.purpose)?;
        tracing::info!("ตัวระบุที่สร้างขึ้น: {}", identifier);

        // ตรวจสอบชื่อซ้ำ
        if self.agent_exists(&identifier) {
            bail!("เอเจนต์ชื่อ '{}' มีอยู่แล้วในไดเรกทอรีผลลัพธ์ กรุณาใช้ชื่ออื่น", identifier);
        }

        // 3. กำหนดโหมดและประเภทงาน
        let mode = self.determine_mode(&parsed);
        let task_type = self.determine_task_type(&parsed.purpose);
        let model = self.select_model(&parsed);
        let color = self.select_color(&parsed.purpose);

        // 4. สร้าง system prompt (แบบใหม่ที่คงเนื้อหาเดิม)
        let system_prompt = self.create_system_prompt(&parsed, &mode, &task_type)?;

        // 5. สร้างคำอธิบายที่มี use case
        let description = self.build_description(&parsed)?;

        // 6. คำนวณ permission (ตัวอย่างจาก priority)
        let permission = std::cmp::min(parsed.priority as u32 * 10, 999);

        // 7. สร้าง permission_policy (ใช้ค่าเริ่มต้น)
        let permission_policy = self.default_permission_policy();

        // 8. ประกอบข้อกำหนดเอเจนต์
        let spec = AgentSpec {
            identifier: identifier.clone(),
            name: parsed.name.clone(),
            description,
            when_to_use: requirements.to_string(),
            system_prompt,
            model,
            color,
            tools: parsed.tools.clone(),
            mode,
            task_type,
            capabilities: vec![parsed.purpose.clone()],
            priority: parsed.priority,
            permission,
            permission_policy,
        };

        // 9. ตรวจสอบความถูกต้อง
        self.validate_spec(&spec)?;

        // 10. เขียนลงไฟล์ Markdown
        let content_hash = self.write_agent_file(&spec)?;

        // 11. บันทึกประวัติลง agent-log.json
        self.update_agent_log(&spec, content_hash)?;

        tracing::info!("✅ สร้างเอเจนต์ '{}' สำเร็จ", identifier);
        Ok(spec)
    }

    /// แยกวิเคราะห์ความต้องการของผู้ใช้ให้เป็นข้อมูลที่มีโครงสร้าง
    fn parse_requirements(&self, requirements: &str) -> Result<ParsedRequirements> {
        let requirements_lower = requirements.to_lowercase();

        // กำหนดวัตถุประสงค์
        let purpose = if requirements_lower.contains("review") || requirements_lower.contains("analyze") {
            "การตรวจสอบและวิเคราะห์โค้ด"
        } else if requirements_lower.contains("generate") || requirements_lower.contains("create") {
            "การสร้างโค้ด"
        } else if requirements_lower.contains("test") {
            "การสร้างเทส"
        } else if requirements_lower.contains("validate") || requirements_lower.contains("check") {
            "การตรวจสอบความถูกต้อง"
        } else if requirements_lower.contains("doc") || requirements_lower.contains("เอกสาร") {
            "การเขียนเอกสาร"
        } else if requirements_lower.contains("plan") || requirements_lower.contains("วางแผน") {
            "การวางแผนโครงการ"
        } else if requirements_lower.contains("image") || requirements_lower.contains("รูป") {
            "การจัดการรูปภาพ"
        } else {
            "การดำเนินงานทั่วไป"
        };

        // กำหนดความซับซ้อน (ใช้เลือกโมเดล)
        let word_count = requirements.split_whitespace().count();
        let complexity = if word_count > 20 {
            AgentComplexity::Complex
        } else if word_count > 10 {
            AgentComplexity::Medium
        } else {
            AgentComplexity::Simple
        };

        // กำหนด priority จากความซับซ้อน (ตัวอย่าง)
        let priority = match complexity {
            AgentComplexity::Complex => 100,
            AgentComplexity::Medium => 80,
            AgentComplexity::Simple => 50,
        };

        // ดึงเครื่องมือที่ถูกกล่าวถึง
        let tools = if requirements_lower.contains("read") && requirements_lower.contains("write") {
            vec!["Read".to_string(), "Write".to_string()]
        } else if requirements_lower.contains("read") {
            vec!["Read".to_string()]
        } else if requirements_lower.contains("write") {
            vec!["Write".to_string()]
        } else {
            vec![]
        };

        Ok(ParsedRequirements {
            name: self.extract_agent_name(requirements)?,
            purpose: purpose.to_string(),
            when_to_use: requirements.to_string(),
            tools,
            complexity,
            priority,
        })
    }

    /// ดึงชื่อเอเจนต์จากความต้องการ (ไม่เกิน 64 ตัวอักษร)
    fn extract_agent_name(&self, requirements: &str) -> Result<String> {
        let words: Vec<&str> = requirements
            .split_whitespace()
            .filter(|w| !["a", "an", "the", "that", "this", "creates", "generates", "สร้าง", "เอเจนต์", "ที่", "เพื่อ", "สำหรับ"].contains(&w.to_lowercase().as_str()))
            .take(4)
            .collect();

        if words.is_empty() {
            return Ok("เอเจนต์ที่สร้างขึ้น".to_string());
        }

        let name = words.join(" ");
        // ใช้ char slicing แทน byte slicing เพื่อรองรับ UTF-8 หลาย byte
        if name.chars().count() > 64 {
            Ok(name.chars().take(64).collect())
        } else {
            Ok(name)
        }
    }

    /// สร้างตัวระบุ (slug) จากวัตถุประสงค์
    fn generate_identifier(&self, purpose: &str) -> Result<String> {
        let purpose_lower = purpose.to_lowercase();
        let words: Vec<&str> = purpose_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .take(4)
            .collect();

        let mut identifier = words.join("-");
        identifier = identifier
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();
        while identifier.contains("--") {
            identifier = identifier.replace("--", "-");
        }
        identifier = identifier.trim_matches('-').to_string();

        if identifier.len() < 3 {
            identifier = format!("{}-agent", identifier);
        }
        // ใช้ char count แทน byte len สำหรับ UTF-8
        if identifier.chars().count() > 50 {
            identifier = identifier.chars().take(50).collect();
            identifier = identifier.trim_end_matches('-').to_string();
        }
        if identifier.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            identifier = format!("agent-{}", identifier);
        }
        Ok(identifier)
    }

    /// สร้างคำอธิบายที่มี use case อย่างน้อย 2 อย่าง (ไม่เกิน 300 ตัวอักษร)
    fn build_description(&self, parsed: &ParsedRequirements) -> Result<String> {
        let base = format!("ใช้เมื่อต้องการ{}", parsed.when_to_use);
        let additional = match parsed.purpose.as_str() {
            "การตรวจสอบและวิเคราะห์โค้ด" => " เช่น ตรวจสอบคุณภาพโค้ด หรือวิเคราะห์ช่องโหว่",
            "การสร้างโค้ด" => " เช่น สร้างฟังก์ชันใหม่ หรือปรับโครงสร้างโค้ดเดิม",
            "การสร้างเทส" => " เช่น เขียน unit test หรือ integration test",
            "การเขียนเอกสาร" => " เช่น เขียน README หรือ API documentation",
            _ => " และงานอื่นที่เกี่ยวข้อง"
        };
        let mut desc = format!("{}{}", base, additional);
        if desc.len() > 300 {
            desc.truncate(297);
            desc.push_str("...");
        }
        Ok(desc)
    }

    /// กำหนดโหมดการทำงาน
    fn determine_mode(&self, parsed: &ParsedRequirements) -> String {
        let req = parsed.when_to_use.to_lowercase();
        if req.contains("primary") || req.contains("หลัก") {
            "primary".to_string()
        } else if req.contains("sub") || req.contains("รอง") || req.contains("ผู้ช่วย") {
            "subagent".to_string()
        } else {
            "all".to_string()
        }
    }

    /// กำหนดประเภทงาน
    fn determine_task_type(&self, purpose: &str) -> String {
        let p = purpose.to_lowercase();
        if p.contains("class") || p.contains("คลาส") { "class".to_string() }
        else if p.contains("trans") || p.contains("แปลง") { "trans".to_string() }
        else if p.contains("code") || p.contains("โค้ด") { "code".to_string() }
        else if p.contains("plan") || p.contains("วางแผน") { "plan".to_string() }
        else if p.contains("create") || p.contains("สร้าง") { "create".to_string() }
        else if p.contains("image") || p.contains("รูป") { "image".to_string() }
        else if p.contains("analysis") || p.contains("วิเคราะห์") { "analysis".to_string() }
        else if p.contains("docs") || p.contains("เอกสาร") { "docs".to_string() }
        else if p.contains("assistant") || p.contains("ผู้ช่วย") { "assistant".to_string() }
        else { "general".to_string() }
    }

    /// เลือกโมเดล (อาจคืน None)
    fn select_model(&self, parsed: &ParsedRequirements) -> Option<String> {
        match parsed.complexity {
            AgentComplexity::Complex => Some("opus".to_string()),
            AgentComplexity::Medium => Some("sonnet".to_string()),
            AgentComplexity::Simple => None,
        }
    }

    /// เลือกสี (optional)
    fn select_color(&self, purpose: &str) -> Option<String> {
        let purpose_lower = purpose.to_lowercase();
        if purpose_lower.contains("review") || purpose_lower.contains("analyz") {
            Some("#3B82F6".to_string())
        } else if purpose_lower.contains("generat") || purpose_lower.contains("creat") {
            Some("#10B981".to_string())
        } else if purpose_lower.contains("validat") || purpose_lower.contains("check") {
            Some("#F59E0B".to_string())
        } else if purpose_lower.contains("security") || purpose_lower.contains("critical") {
            Some("#EF4444".to_string())
        } else if purpose_lower.contains("doc") || purpose_lower.contains("เอกสาร") {
            Some("#38A3EE".to_string())
        } else {
            None
        }
    }

    /// สร้าง system prompt ที่รวมเนื้อหาละเอียดเดิมและรูปแบบใหม่
    fn create_system_prompt(&self, parsed: &ParsedRequirements, mode: &str, _task_type: &str) -> Result<String> {
        let header = "system prompt\n\n";

        let rule_section = format!(
            "## กฎ (Rule)\n\n\
            - คุณคือเอเจนต์ {} ผู้เชี่ยวชาญในโดเมนนี้\n\
            - ทำงานในโหมด {}\n\
            - ปฏิบัติตามกระบวนการ 5 ขั้นตอนอย่างเคร่งครัด\n\
            - ใช้ภาษาที่ชัดเจน กระชับ และเป็นมืออาชีพ\n\
            - ห้ามใช้การจัดรูปแบบตัวหนาหรือตัวเอียงโดยไม่จำเป็น\n\n",
            parsed.name, mode
        );

        let main_action = format!(
            "## หน้าที่หลัก (Main Action)\n\n{}\n\nคำอธิบาย: {}\n\n",
            parsed.purpose,
            parsed.when_to_use
        );

        let detailed_guide = format!(
            r#"## ความรับผิดชอบหลัก

### หน้าที่หลัก
{}

- ดำเนินงานด้วยความแม่นยำและความเชี่ยวชาญ
- ใช้แนวปฏิบัติที่ดีที่สุดเฉพาะโดเมน
- รับประกันผลลัพธ์คุณภาพสูง

### การประกันคุณภาพ
- ตรวจสอบความถูกต้องและความครบถ้วน
- ทำให้แน่ใจว่างานที่ส่งมอบสมบูรณ์
- รักษาความชัดเจนและความเป็นมืออาชีพ

### การจัดการข้อผิดพลาด
- ตรวจจับและจัดการกรณีขอบอย่างเหมาะสม
- ให้ข้อความแสดงข้อผิดพลาดที่ชัดเจน
- แนะนำการดำเนินการแก้ไขเมื่อจำเป็น

## กระบวนการดำเนินงาน

เมื่อคุณได้รับงาน ให้ปฏิบัติตามแนวทางอย่างเป็นระบบดังนี้:

### ขั้นตอนที่ 1: การวิเคราะห์
- อ่านและทำความเข้าใจคำขออย่างละเอียด
- ระบุข้อกำหนดและข้อจำกัดสำคัญ
- สังเกตความคลุมเครือหรือข้อมูลที่ขาดหายไป

### ขั้นตอนที่ 2: การตรวจสอบความถูกต้อง
- ตรวจสอบว่ามีข้อมูลนำเข้าที่จำเป็นทั้งหมด
- ตรวจสอบความขัดแย้งหรือปัญหาที่อาจเกิดขึ้น
- ถามคำถามเพื่อความชัดเจนหากจำเป็น

### ขั้นตอนที่ 3: การดำเนินการ
- ใช้ความเชี่ยวชาญของคุณในการทำงาน
- ปฏิบัติตามแนวปฏิบัติที่ดีที่สุดที่กำหนดไว้
- ใช้เครื่องมือและเทคนิคที่เหมาะสม

### ขั้นตอนที่ 4: การตรวจสอบ
- ทบทวนผลลัพธ์ของคุณเพื่อความถูกต้อง
- ตรวจสอบความครบถ้วน
- ตรวจสอบว่าได้มาตรฐานคุณภาพ

### ขั้นตอนที่ 5: การส่งมอบ
- นำเสนอผลลัพธ์ในรูปแบบที่ชัดเจน มีโครงสร้าง
- เน้นประเด็นสำคัญและการตัดสินใจ
- ให้บริบทและคำอธิบาย

## มาตรฐานผลลัพธ์

คำตอบของคุณควร:
- มีโครงสร้างที่ดีพร้อมส่วนที่ชัดเจน
- ใช้การจัดรูปแบบที่เหมาะสม (หัวข้อ รายการ บล็อกโค้ด)
- รวมตัวอย่างเมื่อเป็นประโยชน์
- ให้ขั้นตอนต่อไปที่สามารถดำเนินการได้
- กระชับแต่ครอบคลุม

## การจัดการกรณีขอบ

### คำขอที่คลุมเครือ
ถามคำถามเพื่อชี้แจงก่อนดำเนินการ

### ข้อมูลที่ขาดหายไป
ระบุอย่างชัดเจนว่าต้องการข้อมูลอะไรและเพราะเหตุใด

### ข้อผิดพลาดหรือความขัดแย้ง
อธิบายปัญหาอย่างชัดเจนและเสนอวิธีแก้ไขเฉพาะ

### นอกขอบเขต
แจ้งอย่างสุภาพหากคำขออยู่นอกเหนือความเชี่ยวชาญของคุณ

## รูปแบบการสื่อสาร

- เป็นมืออาชีพแต่เป็นกันเอง
- ชัดเจนและตรงประเด็น
- เป็นประโยชน์และสร้างสรรค์
- แม่นยำในรายละเอียดทางเทคนิค
- อดทนกับคำถามติดตาม

## เป้าหมายสูงสุด

เป้าหมายของคุณคือการให้บริการที่ยอดเยี่ยมผ่านความเชี่ยวชาญ ความแม่นยำ และความชัดเจน
"#,
            parsed.when_to_use
        );

        let full_prompt = format!("{}{}{}{}", header, rule_section, main_action, detailed_guide);

        let line_count = full_prompt.lines().filter(|l| !l.trim().is_empty()).count();
        if line_count > 500 {
            bail!("System prompt มี {} บรรทัด (เกิน 500 บรรทัด) กรุณาลดขนาด", line_count);
        }

        Ok(full_prompt)
    }

    /// ตรวจสอบความถูกต้องของข้อกำหนดเอเจนต์
    fn validate_spec(&self, spec: &AgentSpec) -> Result<()> {
        if spec.name.len() > 64 {
            bail!("ชื่อเอเจนต์ '{}' ยาวเกิน 64 ตัวอักษร", spec.name);
        }
        if spec.name.is_empty() {
            bail!("ชื่อเอเจนต์ต้องไม่ว่างเปล่า");
        }
        if spec.description.len() > 300 {
            bail!("คำอธิบายยาวเกิน 300 ตัวอักษร");
        }
        if spec.description.is_empty() {
            bail!("คำอธิบายต้องไม่ว่างเปล่าและควรมี use case อย่างน้อย 2 อย่าง");
        }

        let valid_modes = ["all", "primary", "subagent"];
        if !valid_modes.contains(&spec.mode.as_str()) {
            bail!("โหมด '{}' ไม่ถูกต้อง ต้องเป็น all, primary หรือ subagent", spec.mode);
        }

        let valid_types = ["class", "trans", "code", "plan", "create", "image", "analysis", "docs", "assistant", "general"];
        if !valid_types.contains(&spec.task_type.as_str()) {
            bail!("ประเภท '{}' ไม่ถูกต้อง", spec.task_type);
        }

        if spec.permission > 999 {
            bail!("permission ต้องไม่เกิน 999");
        }

        tracing::info!("✅ การตรวจสอบข้อกำหนดเอเจนต์ผ่าน");
        Ok(())
    }

    /// สร้างค่าเริ่มต้นสำหรับ PermissionPolicy
    fn default_permission_policy(&self) -> PermissionPolicy {
        PermissionPolicy {
            hierarchy: vec![
                EvidenceType::Instruction,
                EvidenceType::Explicit,
                EvidenceType::Structure,
                EvidenceType::Ctx,
                EvidenceType::Assumption,
            ],
            weight: Weight {
                mode: 0.3,
                type_weight: 0.3,
                tool: 0.2,
                evidence: 0.2,
            },
            threshold: Some(0.5),
            scope: None,
        }
    }

    /// เขียนข้อกำหนดเอเจนต์ลงไฟล์ Markdown (.md) พร้อม frontmatter แบบ YAML
    fn write_agent_file(&self, spec: &AgentSpec) -> Result<String> {
        let filename = format!("{}/{}.md", self.output_dir, spec.identifier);

        // สร้าง tool config
        let tool_config = ToolConfig {
            bash: false,
            write: spec.tools.contains(&"Write".to_string()),
            skill: true,
            ask: false,
        };

        // สร้าง frontmatter struct สำหรับ serialize
        let frontmatter = Frontmatter {
            name: &spec.name,
            description: &spec.description,
            mode: &spec.mode,
            task_type: &spec.task_type,
            model: spec.model.as_deref(),
            color: spec.color.as_deref(),
            tool: tool_config,
            permission: spec.permission,
            permission_policy: &spec.permission_policy,
        };

        let yaml_str = serde_yaml::to_string(&frontmatter)
            .context("ไม่สามารถ serialize frontmatter เป็น YAML")?;

        let content = format!("---\n{}---\n\n{}", yaml_str, spec.system_prompt);

        fs::write(&filename, &content)
            .with_context(|| format!("ไม่สามารถเขียนไฟล์เอเจนต์: {}", filename))?;

        let mut hasher = Sha256::new();
        hasher.update(spec.system_prompt.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        tracing::info!("📝 เขียนไฟล์เอเจนต์: {}", filename);
        Ok(hash)
    }

    /// อัปเดตไฟล์ agent-log.json
    fn update_agent_log(&self, spec: &AgentSpec, content_hash: String) -> Result<()> {
        let log_path = format!("{}/agent-log.json", self.output_dir);

        let mut log_entries: Vec<AgentLogEntry> = if Path::new(&log_path).exists() {
            let data = fs::read_to_string(&log_path)?;
            serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(existing) = log_entries.iter_mut().find(|e| e.identifier == spec.identifier) {
            existing.updated_at = now;
            existing.version += 1;
            existing.hash = content_hash;
            existing.name = spec.name.clone();
            existing.description = spec.description.clone();
        } else {
            log_entries.push(AgentLogEntry {
                identifier: spec.identifier.clone(),
                name: spec.name.clone(),
                description: spec.description.clone(),
                created_at: now,
                updated_at: now,
                version: 1,
                hash: content_hash,
            });
        }

        let json = serde_json::to_string_pretty(&log_entries)?;
        fs::write(&log_path, json)?;
        tracing::info!("📋 อัปเดต agent-log.json แล้ว");
        Ok(())
    }

    /// ตรวจสอบว่าเอเจนต์ชื่อนี้มีอยู่แล้วหรือไม่
    pub fn agent_exists(&self, identifier: &str) -> bool {
        let filename = format!("{}/{}.md", self.output_dir, identifier);
        Path::new(&filename).exists()
    }
}

// --- โครงสร้างภายใน ---

#[derive(Debug)]
struct ParsedRequirements {
    name: String,
    purpose: String,
    when_to_use: String,
    tools: Vec<String>,
    complexity: AgentComplexity,
    priority: u8,
}

#[derive(Debug)]
enum AgentComplexity {
    Simple,
    Medium,
    Complex,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentLogEntry {
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u32,
    pub hash: String,
}

#[derive(Debug, Serialize)]
struct Frontmatter<'a> {
    name: &'a str,
    description: &'a str,
    mode: &'a str,
    #[serde(rename = "type")]
    task_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<&'a str>,
    tool: ToolConfig,
    permission: u32,
    permission_policy: &'a PermissionPolicy,
}

#[derive(Debug, Serialize)]
struct ToolConfig {
    bash: bool,
    write: bool,
    skill: bool,
    ask: bool,
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_agent_with_policy() {
        let temp = TempDir::new().unwrap();
        let output_dir = temp.path().to_str().unwrap().to_string();

        let creator = AgentCreator::new("templates".to_string(), output_dir.clone());

        let spec = creator.create_agent(
            "Agent: Code Review & Quality",
            None,
        ).await.unwrap();

        assert!(spec.name.len() <= 64);
        assert!(spec.description.len() <= 300);
        assert!(spec.permission_policy.hierarchy.len() == 5);

        let file_path = format!("{}/{}.md", output_dir, spec.identifier);
        assert!(Path::new(&file_path).exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("permission_policy:"));
        assert!(content.contains("hierarchy:"));
        assert!(content.contains("system prompt"));
    }

    #[tokio::test]
    async fn test_duplicate_name_prevention() {
        let temp = TempDir::new().unwrap();
        let output_dir = temp.path().to_str().unwrap().to_string();
        let creator = AgentCreator::new("templates".to_string(), output_dir);

        let _ = creator.create_agent("เอเจนต์ทดสอบ", None).await.unwrap();
        let result = creator.create_agent("เอเจนต์ทดสอบ", None).await;
        assert!(result.is_err());
    }
}
