# AgentGuard

Reliable verification for AI-generated software.

AgentGuard is an open-source project for checking code changes with evidence developers can inspect. The goal is not to have one AI model review another. The goal is to combine established analysis tools, targeted tests, and explicit verification results in a workflow that developers and coding agents can use.

The project is at an early stage. It does not yet verify application code.

## Principles

- **Evidence over confidence scores.** Every finding should explain what was checked and what evidence supports it.
- **Honest results.** A reproduced failure, a static-analysis warning, and a proven property are different things. AgentGuard must keep those distinctions visible.
- **Useful before clever.** Prefer a small feature that works reliably over a broad feature that only works in a demo.
- **Local first.** The basic workflow should not require an account, hosted service, or LLM API.
- **Readable code.** Keep the design understandable. Add abstractions when they solve a real problem.
- **Measure before optimizing.** Correctness and maintainability come first. Optimize against real workloads and benchmarks.

## Current status

The first development milestone is a code-change verification pipeline:

1. Read a Git working tree and identify changed files.
2. Determine which parts of the program are affected.
3. Run suitable analysis and tests.
4. Normalize results into a consistent diagnostic format.
5. Report what was checked, what failed, and what remains unknown.

The implementation will grow in small, testable steps. Language support will be added through dedicated frontends rather than pretending that every language has identical semantics.

## Development

The workspace requires a stable Rust toolchain.

```sh
cargo build --workspace
cargo test --workspace
cargo run -p agentguard -- --help
```

## Project status

AgentGuard is experimental software. Do not treat its output as a guarantee that code is free of defects. Its reports should state the scope and limitations of each check.

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option. See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
