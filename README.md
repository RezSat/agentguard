# AgentGuard

AgentGuard is an open-source tool for checking code changes, with a focus on changes made by AI coding agents.

The project is in its early stages. The current codebase contains a Rust workspace, a basic command-line interface, and shared types for diagnostics. It can list Git working-tree changes and run Ruff or ESLint checks on changed files or repository files. Test execution, runtime verification, and regression analysis are not implemented yet.

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

`check` runs Ruff on changed `.py` and `.pyi` files, and ESLint on changed JavaScript/TypeScript files (`.js`, `.jsx`, `.mjs`, `.cjs`, `.ts`, `.tsx`, `.mts`, and `.cts`). It reports Ruff's rule codes and source locations as static findings. Exit code 0 means no lint findings were reported for the selected files; 1 means Ruff found violations; 2 means the check could not complete. Ruff must be on `PATH` for Python checks. ESLint is resolved from the repository's `node_modules/.bin` first, then `PATH`, and requires a working project configuration for the languages being checked. Only analyzers relevant to the selected files are invoked. The command does not fix files or run application tests. Deleted, ignored, and unchanged files are not selected. A clean result does not establish that the application is correct.

`audit` identifies Python and JavaScript/TypeScript files among tracked and non-ignored untracked files, then runs the relevant analyzers from the Git root, even on a clean working tree. Ruff and ESLint determine their own scan scope, including project exclusions. It is not an exhaustive scan of every file. Exit code 0 means completed checks reported no findings, 1 means findings, and 2 means at least one check could not complete. ESLint warnings are reported as findings even when ESLint exits successfully. AgentGuard does not automatically install dependencies or apply fixes. ESLint can execute project configuration and plugins, so only audit codebases you trust in a suitably isolated environment. Neither command proves the application is correct.

## License

AgentGuard is available under either the MIT license or Apache License 2.0. You may choose which license applies to your use. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
