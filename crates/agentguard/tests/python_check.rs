#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct TempRepo {
    path: PathBuf,
}

impl TempRepo {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "agentguard-ruff-tests-{}-{timestamp}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        git(&path, &["init", "-q"]);
        Self { path }
    }

    fn make_ruff(&self, stdout: &str, stderr: &str, exit_code: i32) -> PathBuf {
        let bin = self.path.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let path = bin.join("ruff");
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nprintf '%s\\n' '{}'\nprintf '%s\\n' '{}' >&2\nexit {exit_code}\n",
            self.path.join("ruff-args.txt").display(),
            stdout,
            stderr
        );
        fs::write(&path, script).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        bin
    }

    fn check(&self, bin: Option<&Path>) -> Output {
        self.run("check", bin)
    }

    fn audit(&self, bin: Option<&Path>) -> Output {
        self.run("audit", bin)
    }

    fn run(&self, subcommand: &str, bin: Option<&Path>) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_agentguard"));
        command.arg(subcommand).current_dir(&self.path);
        if let Some(bin) = bin {
            let path = std::env::var_os("PATH").unwrap_or_default();
            command.env(
                "PATH",
                format!("{}:{}", bin.display(), path.to_string_lossy()),
            );
        }
        command.output().unwrap()
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn git(path: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn reports_ruff_findings_with_code_and_location() {
    let repo = TempRepo::new();
    fs::create_dir(repo.path.join("src")).unwrap();
    fs::write(repo.path.join("src/bad example.py"), "import os\n").unwrap();
    fs::write(repo.path.join("README.md"), "docs\n").unwrap();
    let bin = repo.make_ruff(
        r#"[{"code":"F401","message":"imported but unused","filename":"src/bad example.py","location":{"row":1,"column":1}}]"#,
        "",
        1,
    );

    let output = repo.check(Some(&bin));
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Ruff F401 (static finding): imported but unused"),
        "{stdout}"
    );
    assert!(stdout.contains("\"src/bad example.py\":1:1:"), "{stdout}");
    let args = fs::read_to_string(repo.path.join("ruff-args.txt")).unwrap();
    assert!(args.contains("--no-fix\n"), "{args}");
    assert!(args.contains("--no-cache\n"), "{args}");
    assert!(args.contains("--output-format=json\n"), "{args}");
    assert!(args.contains("./src/bad example.py\n"), "{args}");
    assert!(!args.contains("README.md"), "{args}");
}

#[test]
fn reports_when_ruff_cannot_complete() {
    let repo = TempRepo::new();
    fs::write(repo.path.join("bad.py"), "import os\n").unwrap();
    let bin = repo.make_ruff("[]", "invalid Ruff configuration", 2);

    let output = repo.check(Some(&bin));
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid Ruff configuration"));
    assert!(output.stdout.is_empty());
}

#[test]
fn clean_ruff_output_is_not_a_correctness_claim() {
    let repo = TempRepo::new();
    fs::write(repo.path.join("good.pyi"), "answer: int\n").unwrap();
    let bin = repo.make_ruff("[]", "", 0);

    let output = repo.check(Some(&bin));
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("no findings in 1 changed Python file")
    );
}

#[test]
fn skips_non_python_changes_without_running_ruff() {
    let repo = TempRepo::new();
    fs::write(repo.path.join("README.md"), "docs\n").unwrap();

    let output = repo.check(None);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "No changed Python files to check.\n"
    );
}

#[test]
fn real_ruff_reports_an_unused_import_when_installed() {
    if Command::new("ruff").arg("--version").output().is_err() {
        return;
    }

    let repo = TempRepo::new();
    fs::write(repo.path.join("bad.py"), "import os\n").unwrap();
    let output = repo.check(None);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Ruff F401"));
}

#[test]
fn audit_scans_a_clean_working_tree() {
    let repo = TempRepo::new();
    fs::write(repo.path.join("committed.py"), "import os\n").unwrap();
    git(&repo.path, &["add", "."]);
    git(
        &repo.path,
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

    let bin = repo.make_ruff(
        r#"[{"code":"F401","message":"unused import","filename":"committed.py","location":{"row":1,"column":1}}]"#,
        "",
        1,
    );

    let changed = repo.check(Some(&bin));
    assert!(changed.status.success());
    assert_eq!(
        String::from_utf8(changed.stdout).unwrap(),
        "No changed Python files to check.\n"
    );
    assert!(!repo.path.join("ruff-args.txt").exists());

    let audit = repo.audit(Some(&bin));
    assert_eq!(audit.status.code(), Some(1));
    let stdout = String::from_utf8(audit.stdout).unwrap();
    assert!(stdout.contains("Ruff F401 (static finding): unused import"));
    let args = fs::read_to_string(repo.path.join("ruff-args.txt")).unwrap();
    assert!(args.ends_with("--\n.\n"), "{args}");
    assert!(!args.contains("committed.py"), "{args}");
}

#[test]
fn audit_reports_clean_scope_without_claiming_correctness() {
    let repo = TempRepo::new();
    let bin = repo.make_ruff("[]", "", 0);

    let output = repo.audit(Some(&bin));
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Ruff: no findings in repository scope (Ruff exclusions apply).\n"
    );
}

#[test]
fn audit_propagates_ruff_configuration_failure() {
    let repo = TempRepo::new();
    let bin = repo.make_ruff("[]", "invalid Ruff configuration", 2);

    let output = repo.audit(Some(&bin));
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid Ruff configuration"));
    assert!(output.stdout.is_empty());
}

#[test]
fn real_ruff_audits_committed_python_files_when_installed() {
    if Command::new("ruff").arg("--version").output().is_err() {
        return;
    }

    let repo = TempRepo::new();
    fs::write(repo.path.join("committed.py"), "import os\n").unwrap();
    git(&repo.path, &["add", "."]);
    git(
        &repo.path,
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

    let output = repo.audit(None);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Ruff F401"));
}
