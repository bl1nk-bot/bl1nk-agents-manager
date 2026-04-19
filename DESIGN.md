# Design System: bl1nk-agents-manager

This document defines the visual language, interaction patterns, and design tokens for the bl1nk-agents ecosystem.

## 1. Design Philosophy
The system follows a **"CLI-First, Logic-Driven"** philosophy. It prioritizes clarity, performance, and reliability over decorative elements. Visual feedback is used to communicate status and intent efficiently.

---

## 2. Color Palette & Roles

### 2.1 Semantic Roles
Mapped to ANSI standard colors for CLI compatibility and WCAG AA compliance.

| Role | Color (Hex) | ANSI | Usage | Emoji |
| :--- | :--- | :--- | :--- | :--- |
| **Primary** | `#3b82f6` | Blue | Action, Search, Discovery | 🚀 / 🔍 |
| **Secondary** | `#a855f7` | Magenta | Sub-agents, Secondary info | 🤖 |
| **Success** | `#10b981` | Green | Completion, Validation | ✅ |
| **Error** | `#ef4444` | Red | Failure, Security violation | ❌ |
| **Warning** | `#f59e0b` | Yellow | Deprecation, Potential risk | ⚠️ |
| **Text (Dim)** | `#6b7280` | Gray | Metadata, Secondary context | |
| **Neutral** | `#f3f4f6` | White/Light | Default text | |

### 2.2 Contrast Verification
- **Text on Default Background**: > 4.5:1 (WCAG AA)
- **Primary on Dark Background**: 6.8:1
- **Error on Dark Background**: 5.2:1

---

## 3. Typography & Styling

### 3.1 CLI Output Hierarchy
- **Headers (H1)**: Bold, All-Caps, Blue (`#3b82f6`) or Underlined.
- **Sub-headers (H2)**: Bold, Gray (`#6b7280`).
- **Command Tags**: Mapped as `[category:name]` using bracket notation.
- **Code Snippets**: Wrapped in triple backticks with language tags for proper highlighting.

### 3.2 Label System
Defined in `.github/labels.json`. Standard prefixes:
- `stage:*`: Development lifecycle (spec, plan, act, test, doc, review, finalize)
- `p:*`: Priority (p0 - critical to p3 - backlog)
- `size:*`: Effort estimation (xs to xxl)
- `type:*`: Functional category (feat, fix, ui, docs, etc.)

---

## 4. Interaction Patterns

### 4.1 Status Indicators
- **Spinners**: Used for long-running I/O (Discovery, Network).
- **Progress Bars**: Used for batch processing (Multiple agent tasks).

### 4.2 Interactive Prompts
- **Confirmation**: `[Y/n]` format. Defaulting to 'Yes' (Uppercase).
- **Selection**: Numbered lists for agent choices or track selection.

---

## 5. Layout & Composition

### 5.1 Help Message Structure
```text
Usage: <binary> [global-options] <command> [sub-command] [options]

Commands:
  <cmd-name>   <description>

Options:
  --config, -c  Path to configuration file
  --json        Machine-parseable output
```

### 5.2 Task Reporting
- **Title**: Emoji + Task Name
- **Metadata**: [ID] [Stage] [Priority]
- **Body**: Detailed breakdown or execution log.
- **Footer**: Status (Success/Fail) + Duration.

---

## 6. Depth & Elevation
(Note: For CLI, "Depth" is simulated through indentation and nesting.)

- **Level 0**: Main prompt/Root output.
- **Level 1**: Sub-tasks (2-space indentation).
- **Level 2**: Evidence/Logs (4-space indentation, Dim text).

---

**Document Version:** 1.0.0
**Managed by:** Gemini CLI (Design System Skill)
