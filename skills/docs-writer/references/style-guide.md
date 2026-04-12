# Documentation style guide
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

## I. Core principles

1.  **Clarity:** Write for easy understanding. Prioritize clear, direct, and
    simple language.
2.  **Consistency:** Use consistent terminology, formatting, and style
    throughout the documentation.
3.  **Accuracy:** Ensure all information is technically correct and up-to-date.
4.  **Accessibility:** Design documentation to be usable by everyone. Focus on
    semantic structure, clear link text, and image alternatives.
5.  **Global audience:** Write in standard US English. Avoid slang, idioms, and
    cultural references.
6.  **Prescriptive:** Guide the reader by recommending specific actions and
    paths, especially for complex tasks.

## II. Voice and tone

- **Professional yet friendly:** Maintain a helpful, knowledgeable, and
  conversational tone without being frivolous.
- **Direct:** Get straight to the point. Keep paragraphs short and focused.
- **Second person:** Address the reader as "you."
- **Present tense:** Use the present tense to describe functionality (e.g., "The
  API returns a JSON object.").
- **Avoid:** Jargon, slang, marketing hype, and overly casual language.

## III. Language and grammar

- **Active voice:** Prefer active voice over passive voice.
  - _Example:_ "The system sends a notification." (Not: "A notification is sent
    by the system.")
- **Contractions:** Use common contractions (e.g., "don't," "it's") to maintain
  a natural tone.
- **Simple vocabulary:** Use common words. Define technical terms when
  necessary.
- **Conciseness:** Keep sentences short and focused, but don't omit helpful
  information.
- **"Please":** Avoid using the word "please."

## IV. Procedures and steps

- Start each step with an imperative verb (e.g., "Connect to the database").
- Number steps sequentially.
- Introduce lists of steps with a complete sentence.
- Put conditions before instructions, not after.
- Provide clear context for where the action takes place (e.g., "In the
  administration console...").
- Indicate optional steps clearly (e.g., "Optional: ...").

## V. Formatting and punctuation

- **Text wrap:** Wrap all text at 80 characters, with exceptions for long links
  or tables.
- **Headings, titles, and bold text:** Use sentence case. Structure headings
  hierarchically.
- **Lists:** Use numbered lists for sequential steps and bulleted lists for all
  other lists. Keep list items parallel in structure.
- **Serial comma:** Use the serial comma (e.g., "one, two, and three").
- **Punctuation:** Use standard American punctuation. Place periods inside
  quotation marks.
- **Dates:** Use unambiguous date formatting (e.g., "January 22, 2026").

## VI. UI, code, and links

- **UI elements:** Put UI elements in **bold**. Focus on the task when
  discussing interaction.
- **Code:** Use `code font` for filenames, code snippets, commands, and API
  elements. Use code blocks for multi-line samples.
- **Links:** Use descriptive link text that indicates what the link leads to.
  Avoid "click here."

## VII. Word choice and terminology

- **Consistent naming:** Use product and feature names consistently. Always
  refer to Gemini CLI as `Gemini CLI`, never `the Gemini CLI`.
- **Specific verbs:** Use precise verbs.
- **Avoid:**
  - Latin abbreviations (e.g., use "for example" instead of "e.g.").
  - Placeholder names like "foo" and "bar" in examples; use meaningful names
    instead.
  - Anthropomorphism (e.g., "The server thinks...").
  - "Should": Be clear about requirements ("must") vs. recommendations ("we
    recommend").

## VIII. Files and media

- **Filenames:** Use lowercase letters, separate words with hyphens (-), and use
  standard ASCII characters.
- **Images:** Provide descriptive alt text for all images. Provide
  high-resolution or vector images when practical.

## IX. Accessibility quick check

- Provide descriptive alt text for images.
- Ensure link text makes sense out of context.
- Use semantic HTML elements correctly (headings, lists, tables).
