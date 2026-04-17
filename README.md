# 🤖 BL1NK Agents Manager

![Version](https://img.shields.io/badge/version-1.7.0-blue.svg)
![Status](https://img.shields.io/badge/status-Production--Ready-brightgreen.svg)

> **Intelligent MCP/ACP Orchestrator with Split Metadata Architecture**

BL1NK Agents Manager เป็นระบบจัดการเอเจนต์อัจฉริยะที่พัฒนาด้วย Rust ทำหน้าที่ประสานงานระหว่าง Gemini CLI และ specialized AI workforce

## 🚀 ฟีเจอร์หลัก (v1.7.0)
- **Split Metadata**: แยกความต้องการมนุษย์ (.md) และเทคนิค (.json) เพื่อความสะอาด
- **Auto-discovery**: ค้นหาเอเจนต์และทักษะใหม่ๆ อัตโนมัติ พร้อมการตรวจสอบ Schema
- **Smart Routing**: เลือกเอเจนต์ที่เหมาะสมตามงานและขีดความสามารถ
- **Security Policy**: ควบคุมสิทธิ์การใช้งานเครื่องมืออย่างเข้มงวด

## 📂 โครงสร้างโปรเจกต์
- `src/`: โค้ด Rust หลัก (Registry, Router, Executor)
- `agents/`: คลังเอเจนต์ 40+ ตัว (.md & agents.json)
- `scripts/`: เครื่องมือช่วยพัฒนา (ดู [scripts/README.md](scripts/README.md))
- `conductor/`: ระบบจัดการงานและแผนการพัฒนา

---
Built with ❤️ using Rust, Tokio, and PMCP
