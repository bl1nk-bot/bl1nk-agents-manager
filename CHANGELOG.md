# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.7.5.1] - 2026-04-20

### Added
- **Universal Toolset**: รองรับและคุมสิทธิ์เครื่องมือมาตรฐานครบชุดจากทั้ง Gemini CLI (18 ตัว) และ KiloCode (12 ตัว)
- **Source Abstraction**: เปลี่ยนระบบอ้างอิงเอเจนต์เป็นแบบ Polymorphic Object รองรับประเภท Built-in และเตรียมพร้อมสำหรับ Git/URL ในอนาคต
- **Dynamic Learning (Trust Score)**: ระบบคำนวณน้ำหนักการเลือกเอเจนต์อิงจากประวัติการทำงานจริง (Success Rate) และการตอบรับจากผู้ใช้ (Approval Rate)
- **Nested Policy Architecture**: จัดเก็บสิทธิ์เครื่องมือในรูปแบบ Key-Value Map เพื่อประสิทธิภาพในการเรียกดูข้อมูลระดับ O(1)
- **Agent Versioning**: เพิ่มเลขเวอร์ชันกำกับรายเอเจนต์ใน Registry เพื่อการจัดการวงจรชีวิตที่แม่นยำ

### Changed
- **Directory Restructuring**: ย้ายจาก `.config/` เป็นโฟลเดอร์ `config/` ที่เปิดเผยชัดเจน เพื่อลดความสับสนกับไฟล์ระบบ
- **Permission Downgrade**: ปรับระดับสิทธิ์พื้นฐานของ Extension ลงมาที่เพดาน 300 (จาก 1000) ตามมารยาทของ Gemini CLI (Extension Etiquette)
- **Unified Schemas**: ตั้งชื่อไฟล์สคีมาใหม่ตามหน้าที่ (Capability, Policy, App) พร้อมระบุเวอร์ชัน v1.7 ชัดเจน
- **System Architect**: รวมร่างเอเจนต์ `kiro-workflow`, `workflow-diagrams` และ `code-architect` เข้าเป็น `system-architect` ตัวเดียวที่เป็นกลางและทรงพลัง

### Fixed
- **CI/CD Reliability**: แก้ไขปัญหา Submodule หายบน GitHub Actions โดยการเปิดใช้ `submodules: recursive`
- **Git Recovery**: ซ่อมแซมฐานข้อมูล Git ในกรณีเกิดปัญหา Corrupt Objects (Empty objects) ระหว่างทำงาน
- **Japanese Language Consistency**: แก้ไขคำว่า '実装' เป็น 'Implement/Implemented' ตลอดทั้งเอกสารหลัก

## [1.7.1] - 2026-04-18

### Added
- **JSON Context Persistence**: Implement ระบบจัดเก็บข้อมูลบริบท (Context) โดยใช้ไฟล์ JSON ในโฟลเดอร์ `.omg/state/` พร้อมระบบบันทึกแบบ Atomic เพื่อความปลอดภัยของข้อมูล.
- **Enhanced Bash Tool**: เพิ่มความสามารถในการรันคำสั่ง Shell แบบ Async พร้อมระบบ Timeout และการจับสัญญาณ Error ที่ละเอียดขึ้น.
- **Observability**: ผสานระบบ `tracing` เข้ากับระบบจัดเก็บข้อมูลและเครื่องมือใหม่เพื่อการตรวจสอบสถานะ (Observability) ที่ดีขึ้น.
- **Track Plans**: เพิ่มแผนงานการพัฒนาเชิงลึกสำหรับ `context_management` และ `tool_compaction` เพื่อเป็นแนวทางให้เอเจนต์ทำงานตามลำดับความสำคัญ.

## [1.7.0] - 2026-04-17

### Added
- **Rust Discovery Engine**: พอร์ตระบบค้นหาเอเจนต์จาก TypeScript มาเป็น Rust แบบ Async 100%.
- **Split Metadata**: สถาปัตยกรรมแยกส่วนระหว่างเนื้อหา Prompt (.md) และ ข้อมูลเทคนิค (.json).
- **Validation**: ระบบตรวจสอบ Schema ทันทีที่ค้นพบไฟล์เอเจนต์ใหม่.
