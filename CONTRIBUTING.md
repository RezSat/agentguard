# Contributing

Thanks for taking an interest in AgentGuard.

The project is early. Before proposing a large feature, open an issue describing the problem it solves and how we could tell whether the solution works.

## Before submitting a change

Run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## What makes a good change

- It solves a specific problem.
- Its behaviour is covered by tests.
- Its limitations are documented where they matter.
- It keeps the code understandable.
- It does not claim stronger evidence than it actually provides.

Please avoid adding dependencies for functionality that is small enough to implement clearly with the standard library. For substantial dependencies, explain why they are needed.

## Code and comments

Follow the Google Rust Style Guide and Rust API Guidelines. Prefer clear names and direct control flow. Comments should explain intent, invariants, or non-obvious trade-offs.

## Commit messages

Write a short description of what changed. Use normal language. No particular prefix format is required.
