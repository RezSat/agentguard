#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "agentguard-eslint-{}-{stamp}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        Self { root }
    }

    fn tool(&self, name: &str, json: &str, exit_code: u8) -> PathBuf {
        let bin = self.root.join("fake-bin");
        let file = bin.join(name);
        fs::create_dir_all(&bin).unwrap();
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\ncat <<'AG_JSON'\n{json}\nAG_JSON\nexit {exit_code}\n",
            self.root.join(format!("{name}-args.txt")).display()
        );
        fs::write(&file, script).unwrap();
        let mut permissions = fs::metadata(&file).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&file, permissions).unwrap();
        bin
    }

    fn command(&self, mode: &str, bin: Option<&Path>) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_agentguard"));
        cmd.arg(mode).current_dir(&self.root);
        if let Some(bin) = bin {
            let inherited = std::env::var_os("PATH").unwrap_or_default();
            cmd.env("PATH", format!("{}:{}", bin.display(), inherited.to_string_lossy()));
        }
        cmd.output().unwrap()
    }

    fn commit(&self) {
        git(&self.root, &["add", "."]);
        git(
            &self.root,
            &[
                "-c",
                "user.name=AgentGuard tests",
                "-c",
                "user.email=tests@example.invalid",
                "commit",
                "-q",
                "-m",
                "initial",
            ],
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn git(root: &Path, args: &[&str]) {
    let result = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn check_reports_eslint_warnings_even_when_eslint_exits_zero() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.root.join("src")).unwrap();
    fs::write(fixture.root.join("src/App.tsx"), "export default function App() {}\n").unwrap();
    let bin = fixture.tool(
        "eslint",
        r#"[{"filePath":"src/App.tsx","messages":[{"ruleId":"@typescript-eslint/no-unused-vars","message":"Unused value","severity":1,"line":1,"column":5}]}]"#,
        0,
    );

    let output = fixture.command("check", Some(&bin));
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ESLint @typescript-eslint/no-unused-vars (static finding): warning: Unused value"), "{stdout}");
    assert!(stdout.contains("\"src/App.tsx\":1:5:"), "{stdout}");
    let args = fs::read_to_string(fixture.root.join("eslint-args.txt")).unwrap();
    assert!(args.contains("--format\njson\n"), "{args}");
    assert!(args.contains("--\n./src/App.tsx\n"), "{args}");
    assert!(!args.contains("--fix\n"), "{args}");
}

#[test]
fn audit_runs_on_committed_typescript_in_a_clean_working_tree() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("app.ts"), "const answer: number = 42;\n").unwrap();
    fixture.commit();
    let bin = fixture.tool("eslint", "[]", 0);

    let output = fixture.command("audit", Some(&bin));
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("ESLint: no findings in repository scope")
    );
    let args = fs::read_to_string(fixture.root.join("eslint-args.txt")).unwrap();
    assert!(args.ends_with("--\n.\n"), "{args}");
    assert!(!args.contains("--\n./app.ts\n"), "{args}");
}

#[test]
fn mixed_language_checks_continue_after_one_tool_fails() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("a.py"), "import os\n").unwrap();
    fs::write(fixture.root.join("b.js"), "unknown();\n").unwrap();
    let bin = fixture.tool("ruff", "[]", 2);
    fixture.tool(
        "eslint",
        r#"[{"filePath":"b.js","messages":[{"ruleId":"no-undef","message":"Unknown name","severity":2,"line":1,"column":1}]}]"#,
        1,
    );
    let output = fixture.command("check", Some(&bin));
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stdout.contains("ESLint no-undef (static finding): error: Unknown name"), "{stdout}");
    assert!(stderr.contains("Ruff check could not complete"), "{stderr}");
    assert!(fixture.root.join("ruff-args.txt").exists());
    assert!(fixture.root.join("eslint-args.txt").exists());
}

#[test]
fn audit_reports_no_supported_files_without_running_analyzers() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("README.md"), "docs\n").unwrap();
    let output = fixture.command("audit", None);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "No supported Python or JavaScript/TypeScript files to audit.\n"
    );
}

#[test]
fn audit_uses_project_local_eslint_when_present() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("app.js"), "call();\n").unwrap();
    let local = fixture.root.join("node_modules/.bin");
    fs::create_dir_all(&local).unwrap();
    let tool = local.join("eslint");
    fs::write(
        &tool,
        "#!/bin/sh\nprintf '[{\"filePath\":\"app.js\",\"messages\":[{\"ruleId\":\"local-rule\",\"message\":\"Local ESLint\",\"severity\":2,\"line\":1,\"column\":1}]}]\\n'\nexit 1\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&tool).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&tool, permissions).unwrap();
    let bin = fixture.tool("eslint", "[]", 0);

    let output = fixture.command("audit", Some(&bin));
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("ESLint local-rule"));
    assert!(!fixture.root.join("eslint-args.txt").exists());
}

#[test]
fn real_eslint_detects_undefined_name_when_installed() {
    if Command::new("eslint").arg("--version").output().is_err() {
        return;
    }
    let fixture = Fixture::new();
    fs::write(
        fixture.root.join("eslint.config.mjs"),
        "export default [{ files: ['**/*.js'], rules: { 'no-undef': 'error' } }];\n",
    )
    .unwrap();
    fs::write(fixture.root.join("bad.js"), "doesNotExist();\n").unwrap();
    fixture.commit();

    let output = fixture.command("audit", None);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("ESLint no-undef"));
}
