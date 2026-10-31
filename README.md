# AgentGuard

AgentGuard is an open-source tool for checking code changes, with a focus on changes made by AI coding agents.

The project is in its early stages. The current codebase contains a Rust workspace, a basic command-line interface, and shared types for diagnostics. It can list changes in a Git working tree, but it does not analyse or verify application source code yet.

## Development

You need a stable Rust toolchain and Git.

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
cargo run -p agentguard -- changes
```

`changes` reports staged, unstaged, renamed, and untracked files in the current Git repository. Each line begins with Git's two-character index and working-tree status. Paths are quoted so unusual characters remain visible. Ignored files are not reported. A clean working tree prints `No changes found.`

The CLI also supports help and version information. Analysis and verification commands are not implemented yet.

## License

AgentGuard is available under either the MIT license or Apache License 2.0. You may choose which license applies to your use. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
