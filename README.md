# AgentGuard

AgentGuard is an open-source tool for checking code changes, with a focus on changes made by AI coding agents.

The project is in its early stages. The current codebase contains a Rust workspace, a basic command-line interface, and shared types for diagnostics. It does not analyse or verify source code yet.

## Development

You need a stable Rust toolchain.

Build the workspace:

```sh
cargo build --workspace
```

Run the tests:

```sh
cargo test --workspace
```

Run the CLI:

```sh
cargo run -p agentguard -- --help
```

At this stage, the CLI supports help and version information only.

## License

AgentGuard is available under either the MIT license or Apache License 2.0. You may choose which license applies to your use. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
