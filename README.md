# 🤖 BL1NK Agents Manager

![Version](https://img.shields.io/badge/version-1.7.1-blue.svg)
![Status](https://img.shields.io/badge/status-Active--Development-brightgreen.svg)

> **Agent Orchestration & Management Center**

**BL1NK Agents Manager** คือระบบหัวใจหลักในการควบคุมและจัดการเอเจนต์อัจฉริยะ (Orchestrator) พัฒนาด้วยภาษา Rust ทำหน้าที่บริหารจัดการวงจรชีวิตของ AI workforce ตั้งแต่การค้นพบ การตัดสินใจเลือกเส้นทาง (Routing) ไปจนถึงการจัดการบริบท (Context Management) ที่ซับซ้อน

## 🎯 ทิศทางการพัฒนา (Strategic Direction)
โปรเจกต์นี้มุ่งเน้นการเป็น **Agent Hub** ที่สมบูรณ์แบบ โดยให้ความสำคัญกับ:
- **Orchestration**: การประสานงานระหว่างเอเจนต์หลายตัว
- **Context Integrity**: ระบบจัดเก็บและบีบอัดบริบท (Context Persistence & Compaction)
- **Lifecycle Management**: การจัดการสถานะของงานและเอเจนต์อย่างเป็นระบบ

## 🚀 ฟีเจอร์หลัก
- **Split Metadata Architecture**: แยกข้อมูลสำหรับมนุษย์ (.md) และข้อมูลเทคนิค (.json) อย่างชัดเจน
- **Intelligent Context Management**: จัดเก็บสถานะแบบ JSON และรองรับการ Archive บริบทเก่า
- **Dynamic Auto-discovery**: ค้นหาเอเจนต์และทักษะใหม่เข้าสู่ระบบอัตโนมัติพร้อมการตรวจ Schema
- **Secure Execution**: ควบคุมการใช้เครื่องมือ (Tools) และสิทธิ์การเข้าถึงผ่านระบบ Policy

## 📂 โครงสร้างโปรเจกต์
- `src/`: โค้ด Rust หลัก (Registry, Router, Executor)
- `agents/`: คลังเอเจนต์ 40+ ตัว (.md & agents.json)
- `scripts/`: เครื่องมือช่วยพัฒนา (ดู [scripts/README.md](scripts/README.md))
- `conductor/`: ระบบจัดการงานและแผนการพัฒนา

---
Built with ❤️ using Rust, Tokio, and PMCP
