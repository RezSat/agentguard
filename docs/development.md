# Development

## Requirements

- Rust stable
- Cargo
- Git
- Ruff on PATH when checking Python files
- ESLint installed locally in the target project (preferred) or on PATH, with a working configuration for JavaScript/TypeScript files

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
cargo run -p agentguard -- check
cargo run -p agentguard -- audit
```

`changes` reads the current working tree through Git's NUL-delimited porcelain status format. It does not edit source files or run project code. Git paths that are not valid UTF-8 currently produce an error rather than being silently modified in the report.

The `check` command selects changed Python and JavaScript/TypeScript files from Git status. Python files are passed to `ruff check --no-fix --no-fix-only --no-cache --output-format=json`. JavaScript/TypeScript files are passed to ESLint using its JSON formatter. Both commands avoid automatic fixes. ESLint uses the target project's configuration and plugins. Exit code 1 means findings; exit code 2 means an analyzer could not complete. A check of only one language does not require tools for the other language.

The `audit` command detects source languages from tracked and non-ignored untracked Git files, then runs the relevant analyzer(s) against `.` from the repository root. Ruff and ESLint use their own include and exclusion configuration. Skipped languages are not claimed to have been checked. No dependencies are installed. ESLint loads and executes project configuration/plugins, so use an isolated environment for untrusted repositories. These are static-analysis results, not application correctness or security guarantees.

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
