# Architecture

AgentGuard is intended to coordinate several kinds of verification without hiding their differences.

## Current shape

The workspace starts with two crates:

- `agentguard-core` contains types shared across the pipeline.
- `agentguard` is the command-line application.

The core crate has no third-party dependencies. This is deliberate while the shared model is still small.

The CLI's `changes` command uses system Git to inspect a working tree. `check` selects changed Python and JavaScript/TypeScript files from that status output. `audit` detects supported languages from tracked and non-ignored untracked Git paths and lets Ruff and ESLint select files according to their own configurations. Each analyzer's JSON findings are mapped to diagnostic types without interpreting them as runtime proof. Missing tools and failed checks are reported distinctly from clean checks. ESLint can execute project configuration and plugins; AgentGuard does not install dependencies or apply fixes.

## Diagnostic evidence

A diagnostic records both a finding and the kind of evidence behind it:

- **Proven:** a property violation was established under explicit assumptions.
- **Reproduced:** an execution or test reproduced the failure.
- **Static finding:** an analyzer identified a potentially defective path.
- **Heuristic:** a suspicious pattern needs investigation.

These categories must not be collapsed into a single confidence score. In particular, a static finding is not proof that a bug will occur.

## Planned pipeline

The first useful pipeline should work on a Git working tree:

1. Discover the repository and changed files.
2. Parse the changed source.
3. Identify changed functions and relevant callers where supported.
4. Run selected analyzers and tests.
5. Convert their output into AgentGuard diagnostics.
6. Report findings with their evidence and limitations.

Each step should be independently testable. Tool adapters should preserve the original tool's useful details, including source locations and configuration.

## Language support

Language-specific frontends will own language semantics. A shared representation should contain only information that can be represented without misleading users. Language-specific facts must remain available rather than being discarded to force a universal model.

Python checks use Ruff; JavaScript and TypeScript checks use the target project's ESLint configuration. Neither integration performs language-wide semantic verification.

## Design constraints

- No mandatory cloud service or LLM API.
- No unsafe Rust unless a concrete requirement justifies it and the boundary is reviewed.
- No custom parser when a maintained parser is a better fit.
- No analyzer integration without fixtures that demonstrate its behaviour.
- No claim of complete correctness or complete bug detection.
