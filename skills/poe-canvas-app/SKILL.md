## 📌 Project Status (Feb 7, 2026)

Bl1nk Agents Manager is in active development and is not feature‑complete yet.
This repo contains a working extension shell and a Rust core that is being
brought to feature parity with existing TypeScript logic.

**What works now**
- Extension manifest and Gemini CLI scaffolding are present.
- Core Rust modules exist for agents, hooks, MCP/ACP, sessions, and RPC.
- Command and documentation sets are present (currently being refreshed).

**In progress**
- TypeScript → Rust parity for large subsystems (background agents, config,
  ACP normalization).
- End‑to‑end session flows for Gemini/Codex/Qwen within a unified adapter.
- Validation of hook behavior and task orchestration across agents.

**Known gaps**
- Some Rust modules compile but are not fully wired end‑to‑end.
- Configuration loading/migration is still being aligned to actual runtime.
- Authentication flows for some CLIs still require manual steps.

**What to expect right now**
- You can explore the architecture, commands, and agent catalogs.
- Some workflows will still require manual setup or troubleshooting.

For a complete non‑developer overview, see `docs/PROJECT_STATUS.md`.

---
name: poe-canvas-app
description: Building beautiful, standards-compliant single-file HTML applications for the Poe Canvas platform. Use for creating interactive UI, data visualizations, or mini-apps within Poe chat that interact with bots via the Poe Embed API.
---

# Poe Canvas App Skill

This skill provides comprehensive guidance for developing single-file HTML applications optimized for the Poe Canvas platform. It integrates Poe's specific API standards with professional UI/UX principles.

## Core Principles

- **Single-File Architecture**: Everything (HTML, CSS, JS) must be in one file.
- **Client-Side Only**: No server-side execution; use CDNs for external libraries.
- **Poe API Integration**: Use `window.Poe` for chat interaction and monetization.
- **Adaptive Design**: Support light/dark modes and responsive layouts.

## Workflow

### 1. Project Initialization
- Use the starter template: `/home/ubuntu/skills/poe-canvas-app/templates/poe_canvas_starter.html`.
- Plan the UI structure based on the bot's intended functionality.

### 2. UI/UX Design (Powered by ui-ux-pro-max)
- **Style**: Use `html-tailwind` for rapid, single-file development.
- **Theme**: Implement `prefers-color-scheme` to match Poe's light/dark settings.
- **Standards**: Ensure touch targets are ≥44px and contrast ratios are ≥4.5:1.
- **Animations**: Use CSS transitions or lightweight libraries from trusted CDNs.

### 3. API Implementation
- **Registration**: Always `registerHandler` before calling `sendUserMessage`.
- **Context**: Use `getTriggerMessage` to initialize the app with user data or files.
- **Monetization**: Use `captureCost` for premium features (requires revenue sharing enrollment).

### 4. Handling Limitations
- Read `/home/ubuntu/skills/poe-canvas-app/references/limitations.md` for security restrictions.
- Avoid `alert()`, `localStorage`, or `History API`.
- Use `blob://` for Web Workers if needed.

## Reference Documentation

- **API Methods**: `/home/ubuntu/skills/poe-canvas-app/references/api_reference.md`
- **Platform Limitations**: `/home/ubuntu/skills/poe-canvas-app/references/limitations.md`

## Trusted External Resources

Only use resources from these trusted origins to avoid security prompts:
- `https://cdn.jsdelivr.net`
- `https://cdnjs.cloudflare.com`

## Example: Sending a Message

```javascript
// 1. Register handler
window.Poe.registerHandler("imageGen", (result) => {
  if (result.status === "complete") {
    const response = result.responses[0].content;
    // Process response...
  }
});

// 2. Send message
await window.Poe.sendUserMessage("@ImageBot generate a cat", {
  handler: "imageGen",
  openChat: true
});
```
