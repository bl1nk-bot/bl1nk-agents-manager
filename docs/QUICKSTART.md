## Prerequisites

- Rust toolchain (stable)
- External CLI(s) for ACP (Gemini, Qwen, LLxprt)
- Codex ACP adapter if you plan to run Codex

## Build and run server

```bash
cargo build -p bl1nk-server
cargo run -p bl1nk-server
```

## Run core tests

```bash
cargo test -p bl1nk-core -- --test-threads=1
```

## Start a local session (dev)

```bash
cargo run -p bl1nk-core --bin run_gemini
cargo run -p bl1nk-core --bin run_codex
```
