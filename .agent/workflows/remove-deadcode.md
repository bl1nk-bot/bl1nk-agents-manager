---
name: remove-deadcode
description: Remove unused code from this project with ultrawork mode, LSP-verified safety, atomic commits
---

You are a dead code removal specialist. Execute the FULL dead code removal workflow using ultrawork mode.

Your core weapon: **LSP FindReferences**. If a symbol has ZERO external references, it's dead. Remove it.

## CRITICAL RULES

1. **LSP is law.** Never guess. Always verify with `LspFindReferences` before removing ANYTHING.
2. **One removal = one commit.** Every dead code removal gets its own atomic commit.
3. **Test after every removal.** Run `just test` after each. If it fails, REVERT and skip.
4. **Leaf-first order.** Remove deepest unused symbols first, then work up the dependency chain. Removing a leaf may expose new dead code upstream.
5. **Never remove entry points.** `crates/core/src/lib.rs`, `crates/server/src/main.rs`, test files, config files, and `gemini-extension.json` are off-limits unless explicitly targeted.

---

## STEP 0: REGISTER TODO LIST (MANDATORY FIRST ACTION)

```
TodoWrite([
  {"id": "scan", "content": "PHASE 1: Scan codebase for dead code candidates using LSP + explore agents", "status": "pending", "priority": "high"},
  {"id": "verify", "content": "PHASE 2: Verify each candidate with LspFindReferences - zero false positives", "status": "pending", "priority": "high"},
  {"id": "plan", "content": "PHASE 3: Plan removal order (leaf-first dependency order)", "status": "pending", "priority": "high"},
  {"id": "remove", "content": "PHASE 4: Remove dead code one-by-one (remove -> test -> commit loop)", "status": "pending", "priority": "high"},
  {"id": "final", "content": "PHASE 5: Final verification - full test suite + build + typecheck", "status": "pending", "priority": "high"}
])
```

---

## PHASE 1: SCAN FOR DEAD CODE CANDIDATES

**Mark scan as in_progress.**

### 1.1: Launch Parallel Explore Agents (ALL BACKGROUND)

Fire ALL simultaneously:

```
// Agent 1: Find all exported symbols
task(subagent_type="explore", run_in_background=true,
  prompt="Find ALL exported functions, structs, enums, traits, types, and constants across crates/.
  List each with: file path, line number, symbol name, export type.
  EXCLUDE: crates/core/src/lib.rs root exports, test files.
  Return as structured list.")

// Agent 2: Find potentially unused files
task(subagent_type="explore", run_in_background=true,
  prompt="Find files in crates/ that are NOT imported/mod-ed by any other file.
  Check mod statements and use statements across the entire codebase.
  EXCLUDE: lib.rs, main.rs, test files, entry points, config files, .md files.
  Return list of potentially orphaned files.")

// Agent 3: Find unused imports within files
task(subagent_type="explore", run_in_background=true,
  prompt="Find unused imports across crates/**/*.rs files.
  Look for use statements where the imported symbol is never referenced in the file body.
  Return: file path, line number, imported symbol name.")

// Agent 4: Find functions/variables only used in their own declaration
task(subagent_type="explore", run_in_background=true,
  prompt="Find private/non-exported functions, variables, and types in crates/**/*.rs that appear
  to have zero usage beyond their declaration. Return: file path, line number, symbol name.")
```

### 1.2: Direct AST-Grep Scans (WHILE AGENTS RUN)

```rust
// Find unused imports pattern (Rust use statements)
ast_grep_search(pattern="use $PATH::$NAME;", lang="rust", paths=["crates/"])

// Find empty pub(crate) or private items that might be dead
ast_grep_search(pattern="pub(crate) fn $NAME($$$) { $$$ }", lang="rust", paths=["crates/"])
```

### 1.3: Collect All Results

Collect background agent results. Compile into a master candidate list:

```
## DEAD CODE CANDIDATES

| # | File | Line | Symbol | Type | Confidence |
|---|------|------|--------|------|------------|
| 1 | crates/core/src/foo.rs | 42 | unused_func | function | HIGH |
| 2 | crates/core/src/bar.rs | 10 | OldStruct | struct | MEDIUM |
```

**Mark scan as completed.**

---

## PHASE 2: VERIFY WITH LSP (ZERO FALSE POSITIVES)

**Mark verify as in_progress.**

For EVERY candidate from Phase 1, run this verification:

### 2.1: The LSP Verification Protocol

For each candidate symbol:

```rust
// Step 1: Find the symbol's exact position
LspDocumentSymbols(filePath)  // Get line/character of the symbol

// Step 2: Find ALL references across the ENTIRE workspace
LspFindReferences(filePath, line, character, includeDeclaration=false)
// includeDeclaration=false → only counts USAGES, not the definition itself

// Step 3: Evaluate
// 0 references → CONFIRMED DEAD CODE
// 1+ references → NOT dead, remove from candidate list
```

### 2.2: False Positive Guards

**NEVER mark as dead code if:**

- Symbol is in `crates/core/src/lib.rs` or `crates/server/src/main.rs`
- Symbol is re-exported in a public module (check `pub use`)
- Symbol is referenced in test files (tests are valid consumers)
- Symbol has `#[no_mangle]` or is part of a public API
- Symbol is a hook registration or tool factory mentioned in project documentation
- Symbol is an agent definition in `agents/`

### 2.3: Build Confirmed Dead Code List

After verification, produce:

```
## CONFIRMED DEAD CODE (LSP-verified, 0 external references)

| # | File | Line | Symbol | Type | Safe to Remove |
|---|------|------|--------|------|----------------|
| 1 | crates/core/src/foo.rs | 42 | unused_func | function | YES |
```

**If ZERO confirmed dead code found: Report "No dead code found" and STOP.**

**Mark verify as completed.**

---

## PHASE 3: PLAN REMOVAL ORDER

**Mark plan as in_progress.**

### 3.1: Dependency Analysis

For each confirmed dead symbol:

1. Check if removing it would expose other dead code
2. Check if other dead symbols depend on this one
3. Build removal dependency graph

### 3.2: Order by Leaf-First

```
Removal Order:
1. [Leaf symbols - no other dead code depends on them]
2. [Intermediate symbols - depended on only by already-removed dead code]
3. [Dead files - entire files with no live exports]
```

### 3.3: Register Granular Todos

Create one todo per removal:

```
TodoWrite([
  {"id": "remove-1", "content": "Remove unused_func from crates/core/src/foo.rs:42", "status": "pending", "priority": "high"},
  {"id": "remove-2", "content": "Remove OldStruct from crates/core/src/bar.rs:10", "status": "pending", "priority": "high"},
  // ... one per confirmed dead symbol
])
```

**Mark plan as completed.**

---

## PHASE 4: ITERATIVE REMOVAL LOOP

**Mark remove as in_progress.**

For EACH dead code item, execute this exact loop:

### 4.1: Pre-Removal Check

```rust
// Re-verify it's still dead (previous removals may have changed things)
LspFindReferences(filePath, line, character, includeDeclaration=false)
// If references > 0 now → SKIP (previous removal exposed a new consumer)
```

### 4.2: Remove the Dead Code

Use appropriate tool:

**For unused imports:**

```rust
Edit(filePath, oldString="use path::to::DeadSymbol;\n", newString="")
// Or if it's one of many imports, remove just the symbol from the import list
```

**For unused functions/structs/enums:**

```rust
// Read the full symbol extent first
Read(filePath, offset=startLine, limit=endLine-startLine+1)
// Then remove it
Edit(filePath, oldString="[full symbol text]", newString="")
```

**For dead files:**

```bash
# Only after confirming ZERO imports point to this file
rm "path/to/dead-file.rs"
```

**After removal, also clean up:**

- Remove any imports (`use`) that were ONLY used by the removed code
- Fix any trailing whitespace / double blank lines left behind

### 4.3: Post-Removal Verification

```bash
# 1. LSP diagnostics/Check
just check

# 2. Run tests
just test

# 3. Clippy/Lint
just clippy
```

### 4.4: Handle Failures

If ANY verification fails:

1. **REVERT** the change immediately (`git checkout -- [file]`)
2. Mark this removal todo as `cancelled` with note: "Removal caused [error]. Skipped."
3. Proceed to next item

### 4.5: Commit

```bash
git add [changed-files]
git commit -m "refactor: remove unused [symbolType] [symbolName] from [filePath]"
```

Mark this removal todo as `completed`.

### 4.6: Re-scan After Removal

After removing a symbol, check if its removal exposed NEW dead code:

- Were there `use` statements that only existed to serve the removed symbol?
- Are there other symbols in the same file now unreferenced?

If new dead code is found, add it to the removal queue.

**Repeat 4.1-4.6 for every item. Mark remove as completed when done.**

---

## PHASE 5: FINAL VERIFICATION

**Mark final as in_progress.**

### 5.1: Full Test Suite

```bash
just test
```

### 5.2: Full Typecheck

```bash
just check
```

### 5.3: Full Build

```bash
just build
```

### 5.4: Summary Report

```markdown
## Dead Code Removal Complete

### Removed
| # | Symbol | File | Type | Commit |
|---|--------|------|------|--------|
| 1 | unused_func | crates/core/src/foo.rs | function | abc1234 |

### Skipped (caused failures)
| # | Symbol | File | Reason |
|---|--------|------|--------|
| 1 | risky_func | crates/core/src/bar.rs | Test failure: [details] |

### Verification
- Tests: PASSED
- Check: CLEAN
- Build: SUCCESS
- Total dead code removed: N symbols across M files
- Total commits: K atomic commits
```

**Mark final as completed.**

---

## SCOPE CONTROL

**If $ARGUMENTS is provided**, narrow the scan to the specified scope:

- File path: Only scan that file
- Directory: Only scan that directory (e.g., `crates/core`)
- Symbol name: Only check that specific symbol
- "all" or empty: Full project scan (default)

## ABORT CONDITIONS

**STOP and report to user if:**

- 3 consecutive removals cause test failures
- Build breaks and cannot be fixed by reverting
- More than 50 candidates found (ask user to narrow scope)

## LANGUAGE

Use English for commit messages and technical output.
