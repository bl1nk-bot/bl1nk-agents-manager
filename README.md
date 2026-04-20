# 🤖 BL1NK Agents Manager

![Version](https://img.shields.io/badge/version-1.7.5.1-blue.svg)
![Status](https://img.shields.io/badge/status-Active--Development-brightgreen.svg)

> **Universal Agent Orchestration & Management System**

**BL1NK Agents Manager** คือระบบหัวใจหลักในการควบคุมและจัดการเอเจนต์อัจฉริยะ (Orchestrator) พัฒนาด้วยภาษา Rust (Edition 2024) ทำหน้าที่บริหารจัดการวงจรชีวิตของ AI workforce ตั้งแต่การค้นพบ การตัดสินใจเลือกเส้นทาง (Routing) ไปจนถึงการควบคุมสิทธิ์ที่เข้มงวดตามมาตรฐานสากล

## 🎯 ทิศทางการพัฒนา (Strategic Direction)
โปรเจกต์นี้มุ่งเน้นการเป็น **Unified Hub** สำหรับเอเจนต์ทุกค่าย โดยเน้นที่:
- **Hierarchical Governance**: ระบบควบคุมสิทธิ์แบบลำดับชั้น (Tier 1-5) ตามมาตรฐาน Gemini CLI
- **Universal Compatibility**: รองรับชุดเครื่องมือมาตรฐานจากทั้ง Gemini และ KiloCode
- **Dynamic Learning**: ปรับน้ำหนักการเลือกเอเจนต์อัตโนมัติอิงจากประวัติการทำงานจริง (Trust Score)

## 🚀 ฟีเจอร์เด่น (v1.7.5.1)
- **Universal Tool Policies**: รองรับและคุมสิทธิ์เครื่องมือกว่า 18 ชนิดผ่านโครงสร้าง Nested Map ความเร็วสูง
- **Source Abstraction**: ระบบ polymorphic source รองรับเอเจนต์ทั้งแบบ Built-in, Git, และ Remote URL
- **Path Consolidation**: ย้ายการตั้งค่าและสคีมาไปที่โฟลเดอร์ `config/` (Visible) เพื่อความเป็นระเบียบ
- **Atomic Persistence**: ระบบจัดเก็บสถานะบริบทแบบ JSON Atomic Write เพื่อความปลอดภัยของข้อมูล

## 📂 โครงสร้างโปรเจกต์
- `src/`: โค้ด Rust หลัก (Registry, Router, Executor)
- `config/`: สคีมาและไฟล์ตั้งค่ามาตรฐาน แบ่งตามเวอร์ชัน (v1.7+)
- `agents/`: คลังเอเจนต์อัจฉริยะ (.md & agents.json)
- `scripts/`: เครื่องมือช่วยพัฒนาและระบบ Build Release
- `conductor/`: แผนงานสถาปัตยกรรมและการจัดการ Track พัฒนา

## 🛠️ การเรียกใช้งานเบื้องต้น
```bash
# รันการค้นหาและตรวจสอบเอเจนต์
make agents-check

# สร้างชุดแจกจ่าย (Distribution) สำหรับเอเจนต์ทุกตัว
make release

# ตรวจสอบความถูกต้องของระบบทั้งหมด
make parallel
```

---
Built with 🦀 and ❤️ using Rust, Tokio, and PMCP
