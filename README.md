# AgentGuard

AgentGuard is an open-source tool for checking code changes, with a focus on changes made by AI coding agents.

The project is in its early stages. The current codebase contains a Rust workspace, a basic command-line interface, and shared types for diagnostics. It can list Git working-tree changes and run Ruff checks on either changed Python files or all Python files in Ruff's repository scan scope. Test execution, runtime verification, and regression analysis are not implemented yet.

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
cargo run -p agentguard -- check
cargo run -p agentguard -- audit
```

`changes` reports staged, unstaged, renamed, and untracked files in the current Git repository. Each line begins with Git's two-character index and working-tree status. Paths are quoted so unusual characters remain visible. Ignored files are not reported. A clean working tree prints `No changes found.`

`check` runs the installed Ruff executable on existing changed `.py` and `.pyi` files. It reports Ruff's rule codes and source locations as static findings. Exit code 0 means no lint findings were reported for the selected files; 1 means Ruff found violations; 2 means the check could not complete. Ruff must be installed separately and available on `PATH`. The command does not fix files or run application code. Deleted files, ignored files, and unchanged files are not scanned. A clean result does not establish that the application is correct.

`audit` runs Ruff from the Git repository root, even when the working tree is clean. Ruff selects Python files using its normal configuration, include patterns, and exclusions; it is not an exhaustive scan of every file. Like `check`, it reports static findings, uses exit code 1 for findings and 2 for tool failures, and does not fix files or execute application code. Neither command proves the application is correct.

## License

AgentGuard is available under either the MIT license or Apache License 2.0. You may choose which license applies to your use. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
