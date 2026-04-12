# Java / Spring Data JPA Analysis
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

## 1. N+1 Query Detection

The "N+1 problem" occurs when an application loads a collection of N objects and then performs a separate query for each object to fetch related data.

### Detection
- Look for `@OneToMany` or `@ManyToMany` relationships with `FetchType.EAGER` (bad practice) or `LAZY` (default, but risky if accessed in a loop).
- Check repository methods. Standard `findAll()` does not fetch associations.
- Review Service logic:
  ```java
  List<Author> authors = authorRepository.findAll(); // Query 1
  for (Author a : authors) {
      a.getBooks().size(); // Query N (one per author)
  }
  ```

### Solutions
- **Entity Graph:** Use `@EntityGraph` on repository methods to fetch associations in a single query.
- **Join Fetch:** Use JPQL `JOIN FETCH`.
  ```java
  @Query("SELECT a FROM Author a JOIN FETCH a.books")
  List<Author> findAllWithBooks();
  ```

## 2. Transaction Management

- **@Transactional:** Ensure service methods modifying data are annotated.
- **Read-Only:** Use `@Transactional(readOnly = true)` for fetch-only operations to allow DB optimizations (e.g., avoiding dirty checking in Hibernate).
- **Boundaries:** Be aware that `@Transactional` only works on *external* method calls. Calling a transactional method from within the same class (using `this.method()`) bypasses the proxy and the transaction.

## 3. Repository Best Practices

- **Pagination:** Ensure large result sets use `Pageable` instead of returning `List<?>`.
- **Projections:** Use Interfaces or DTOs (Records) for read-only views to avoid fetching entire Entities when only a few columns are needed.
  ```java
  // Efficient
  List<UserNameOnly> findByActiveTrue();
  ```
