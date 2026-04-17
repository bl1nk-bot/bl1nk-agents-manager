# 🤖 BL1NK Agents Manager

![Version](https://img.shields.io/badge/version-1.7.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Status](https://img.shields.io/badge/status-Production--Ready-brightgreen.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

> **Intelligent MCP/ACP Orchestrator with Split Metadata Architecture**
> **🚀 Powered by Unified Registry v1.7.0 & Schema Enforcement**

BL1NK Agents Manager เป็นระบบจัดการและประสานงานเอเจนต์ (Orchestrator) ประสิทธิภาพสูงที่พัฒนาด้วย Rust ทำหน้าที่เป็นตัวกลางเชื่อมต่อระหว่าง Gemini CLI (ผ่าน MCP) และ Sub-agents หลากหลายตัว (ผ่าน ACP/CLI) โดยเน้นความสะอาดของข้อมูลและความปลอดภัยสูงสุด

---

## ✨ Features

* **Split Metadata Architecture**: แยกข้อมูลเอเจนต์เป็น 2 ส่วน:
  * **Human-Readable (`.md`)**: มีเพียง 4 ฟิลด์หลัก (`name`, `description`, `mode`, `tool`) เพื่อความสะอาดและแก้ไขง่าย
  * **Technical Registry (`agents.json`)**: เก็บข้อมูลทางเทคนิคทั้งหมด (`model`, `permission`, `policy`) เพื่อการประมวลผลที่รวดเร็ว
* **Unified Registry v1.7.0**: ระบบฐานข้อมูลกลางที่ผสานข้อมูลจากไฟล์ Markdown และ JSON เข้าด้วยกันอัตโนมัติขณะรันไทม์
* **12 Core Tools Standard**: เอเจนต์ทุกตัวได้รับการติดตั้งชุดเครื่องมือมาตรฐาน 12+ ชนิด (Glob, Grep, ReadFile, ฯลฯ) เพื่อความพร้อมในการทำงานเต็มรูปแบบ
* **Intelligent Routing & Search**: ระบบค้นหาเอเจนต์ตามความสามารถ (Capabilities) และชื่อ (Slug) ที่แม่นยำด้วยการค้นหาแบบ Fuzzy Search
* **Security & Policy Enforcement**: ควบคุมสิทธิ์การใช้งาน (Bash, Write, Skill, Ask) ผ่านคะแนน Permission และ Decision Rules
* **Strict Schema Validation**: ตรวจสอบความถูกต้องของไฟล์เอเจนต์ด้วย JSON Schema ตลอดวงจรการพัฒนา

---

## 🏗️ Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    Gemini MCP Proxy                          │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: MCP Server (PMCP)                                  │
│  └── TypedTools, JSON-RPC 2.0, stdio transport               │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Unified Registry & Merge Engine                    │
│  └── .md (4 Fields) + .json (Metadata) => AgentConfig        │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Agent Management                                   │
│  ├── AgentRegistry     │ AgentRouter    │ AgentExecutor    │
│  └── PolicyEvaluator (Permission & Security)                 │
└─────────────────────────────────────────────────────────────┘
```

### Core Modules

| Module | Purpose |
|--------|---------|
| `src/registry/` | **Unified Registry v1.7.0**: จัดการ Schema และการค้นหาเอเจนต์ |
| `src/config/` | **Merge Engine**: ผสานข้อมูลจาก .md และ .json เป็นคอนฟิกสมบูรณ์ |
| `src/agents/` | **Execution Core**: จัดการการลงทะเบียน, การเลือก และการรันงาน |
| `src/mcp/` | **MCP Gateway**: ส่วนเชื่อมต่อกับ Gemini CLI ผ่าน PMCP SDK |

---

## 🤖 Agent Structure (v1.7.0)

เพื่อให้ระบบเป็นระเบียบและใช้งานง่าย เอเจนต์ในโฟลเดอร์ `agents/` จะถูกจัดโครงสร้างดังนี้:

### 1. Frontmatter ในไฟล์ `.md` (4 Fields Only)

```yaml
---
name: code-generator
description: Expert in production-ready and idiomatic code generation.
mode: subagent
tool:
- AskUserQuestion
- ExitPlanMode
- Glob
... (12 Core Tools)
---
```text
### 2. ข้อมูลใน `agents/agents.json`

เก็บ Metadata ทางเทคนิค เช่น `model`, `permission`, `permission_policy`, และ `tool_permissions` (Boolean booleans)

---

## 📂 Project Structure
```

bl1nk-agents-manager/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs             # Configuration & Merge Engine
│   ├── agents/
│   │   ├── register.rs      # Agent Registry logic
│   │   ├── router.rs        # Smart routing
│   │   ├── executor.rs      # Task execution (formerly extractor)
│   │   └── creator.rs        # Agent creation (Split-aware)
│   ├── registry/            # Unified Registry & Schema
├── agents/                  # 🤖 Managed Agents (.md & agents.json)
├── .config/                 # 🛡️ Strict Schemas
├── scripts/                 # 🛠️ Utility Tools (See scripts/README.md)
└── Cargo.toml

```text
---

## 🚀 Getting Started

### 1. Build the Project
```bash
cargo build --release
```text
### 2. Run Integration Tests
```bash
python3 scripts/test_integration.py
```text
### 3. Manage Agents

ใช้เครื่องมือในโฟลเดอร์ `scripts/` เพื่อจัดการเอเจนต์:
```bash
python3 scripts/agent_manager.py list
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [scripts/README.md](scripts/README.md) | **NEW**: คู่มือการใช้งานสคริปต์ทั้งหมด |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | รายละเอียดการออกแบบระบบภายใน |
| [AGENT_GUIDE.md](docs/AGENT_GUIDE.md) | วิธีการสร้างและปรับแต่งเอเจนต์ |

---

**Built with ❤️ using Rust, Tokio, and PMCP**
