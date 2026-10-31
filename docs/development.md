# Development

## Requirements

- Rust stable
- Cargo
- Git

## Build and test

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Run the CLI from a Git repository:

```sh
cargo run -p agentguard -- --help
cargo run -p agentguard -- --version
cargo run -p agentguard -- changes
```

`changes` reads the current working tree through Git's NUL-delimited porcelain status format. It does not edit source files or run project code. Git paths that are not valid UTF-8 currently produce an error rather than being silently modified in the report.

## Code style

Follow the [Google Rust Style Guide](https://google.github.io/rust-style-guide/) alongside the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

A few project rules matter just as much:

- Prefer straightforward code over clever abstractions.
- Use comments to explain decisions and constraints, not to restate the next line.
- Keep functions focused, but do not split simple logic into needless layers.
- Make errors explicit and useful to the person running the tool.
- Avoid unsafe Rust.
- Add tests for behaviour and failure cases.
- Benchmark before spending time on performance work.

Commit messages should describe the change in ordinary language. There is no required prefix convention.
