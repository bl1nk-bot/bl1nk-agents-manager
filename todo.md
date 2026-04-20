# TODO: bl1nk-agents

## [2026-04-20] High-Level Registry & Policy Refactor (v1.7.5.1)

- [x] **Universal Toolset Integration**
  - [x] Gather built-in tools from Gemini CLI (18 tools) and KiloCode (12 tools)
  - [x] Map universal tool policies for all 36 agents
- [x] **Registry Architecture Overhaul**
  - [x] Implement Source Abstraction (Polymorphic source object)
  - [x] Implement Nested Policy Map (O(1) lookup)
  - [x] Add per-agent versioning and trust scoring (Dynamic Weights)
- [x] **Infrastructure Consolidation**
  - [x] Rename and migrate `.config/` to visible `config/`
  - [x] Standardize schemas with versioning (v1.7) and purpose-based naming
  - [x] Consolidate Architect agents into a single `system-architect`
- [x] **Project Cleanup**
  - [x] Remove unused scripts and outdated agents
  - [x] Fix CI/CD submodule missing issue
  - [x] Modernize all documentation and fix language consistency

## [2026-04-18] CI/CD & Context Stabilization (v1.7.1)

- [x] Modernize GitHub Actions (Fix Node 20 & set-output warnings)
- [x] Implement Bl1nk Auto-Labeling system
- [x] Recreate core mandates in GEMINI.md

## [2026-04-17] Split Metadata & Discovery Upgrade (v1.7.0)

- [x] Port Skill Discovery from TS to Rust
- [x] Implement Split Metadata Architecture
- [x] Unified Registry & Schema Enforcement

---

## 🚀 Next Focus
- [x] **Tool Compaction TS→Rust**: ContentPart, MessageContent, compaction functions (12 tests)
- [x] **Context Management Phase 1-2**: Context structures + JSON storage (`.bl1nk/` design)
- [ ] **Context Management Phase 3: Token-based Compaction & History**
  - [x] Implement Token Budget Management (limit + auto-select)
  - [x] Implement Tool Use History Tracking
  - [x] Implement Context Offload to File (Archive old messages)
  - [ ] Implement Sliding Window Compaction (Retain top N%)
  - [ ] Integrate Compaction into Workspace lifecycle
- [ ] **Context Management Phase 4: Environment & Secrets Handling**
  - [ ] Implement Secrets Masking for safe logging
  - [ ] Implement Environment Variable Injection (APP_ prefix)
- [ ] **Context Management Phase 5: Integration with Orchestrator**
  - [ ] Create `ContextManager` High-level service
  - [ ] Integrate with `AgentExecutor` for task context injection
  - [ ] Final end-to-end verification and performance check
- [ ] **Multi-Source Loading**: Test loading agents from external Git URLs
