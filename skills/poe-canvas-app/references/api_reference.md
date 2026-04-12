# Poe Embed API Reference
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

The Poe Embed API is accessible via the `window.Poe` global object.

## Core Methods

| Method | Description | Usage |
| :--- | :--- | :--- |
| `sendUserMessage` | Sends a message in chat on behalf of the user. | `window.Poe.sendUserMessage("@Bot text", options)` |
| `registerHandler` | Registers a callback to receive results from `sendUserMessage`. | `window.Poe.registerHandler("name", callback)` |
| `getTriggerMessage` | Returns the message that triggered the canvas. | `window.Poe.getTriggerMessage()` |
| `captureCost` | Charges the user for creator-defined paid events. | `window.Poe.captureCost(amounts, options)` |

## Method Details

### sendUserMessage
Sends a message to a bot. Returns a `Promise<{ success: boolean }>`.

**Options:**
- `attachments`: `File[]` to include.
- `stream`: `boolean` for streaming results.
- `openChat`: `boolean` to open the chat UI on send.
- `handler`: `string` name of the registered handler.
- `handlerContext`: `Record<string, any>` passed to the handler.

### registerHandler
Registers a function to handle bot responses. Returns a `VoidFunction` to unregister.

**Callback signature:** `(result: SendUserMessageResult, context: HandlerContext) => void`

### getTriggerMessage
Returns a `Promise<Message>` containing information about the triggering message, including content and attachments.

### captureCost
Implements custom pricing. Amounts are in USD milli-cents (1/100,000th of a dollar).
Example: `100000` = $1.00.

## Data Objects

### Message
| Property | Type | Description |
| :--- | :--- | :--- |
| `messageId` | `string` | Unique identifier for the message. |
| `senderId` | `string` | Name of the sender (e.g., "Assistant"). |
| `content` | `string` | Text content of the message. |
| `contentType` | `string` | `text/plain` or `text/markdown`. |
| `status` | `string` | `incomplete`, `complete`, or `error`. |
| `attachments` | `MessageAttachment[]` | List of attachments. |

### SendUserMessageResult
| Property | Type | Description |
| :--- | :--- | :--- |
| `status` | `string` | `incomplete`, `complete`, or `error`. |
| `responses` | `Message[]` | Array of bot response messages. |
