use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "agentguard-tests-{}-{timestamp}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn git(directory: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn agentguard(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agentguard"))
        .args(args)
        .current_dir(directory)
        .output()
        .unwrap()
}

#[test]
fn changes_reports_staged_renamed_and_untracked_files() {
    let repo = TemporaryDirectory::new();
    git(&repo.path, &["init", "-q"]);
    fs::write(repo.path.join("changed.py"), "before\n").unwrap();
    fs::write(repo.path.join("renamed.py"), "same content\n").unwrap();
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

    fs::write(repo.path.join("changed.py"), "after\n").unwrap();
    fs::rename(repo.path.join("renamed.py"), repo.path.join("moved.py")).unwrap();
    git(&repo.path, &["add", "-A"]);
    fs::write(repo.path.join("untracked file.py"), "print('hi')\n").unwrap();

    let output = agentguard(&repo.path, &["changes"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("M  \"changed.py\""), "{stdout}");
    assert!(
        stdout.contains("R  \"renamed.py\" -> \"moved.py\""),
        "{stdout}"
    );
    assert!(stdout.contains("?? \"untracked file.py\""), "{stdout}");
}

#[test]
fn changes_handles_clean_and_non_repository_directories() {
    let directory = TemporaryDirectory::new();

    let outside = agentguard(&directory.path, &["changes"]);
    assert_eq!(outside.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&outside.stderr).contains("could not inspect Git changes"));

    git(&directory.path, &["init", "-q"]);
    let clean = agentguard(&directory.path, &["changes"]);
    assert!(clean.status.success());
    assert_eq!(
        String::from_utf8(clean.stdout).unwrap(),
        "No changes found.\n"
    );
}

#[test]
fn cli_rejects_unknown_and_extra_arguments() {
    let directory = TemporaryDirectory::new();

    let unknown = agentguard(&directory.path, &["not-a-command"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown command"));

    let extra = agentguard(&directory.path, &["--version", "unexpected"]);
    assert_eq!(extra.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&extra.stderr).contains("unexpected argument"));
}

#[test]
fn cli_reports_help_and_version() {
    let directory = TemporaryDirectory::new();

    let help = agentguard(&directory.path, &["--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("changes"));

    let version = agentguard(&directory.path, &["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "agentguard 0.1.0\n"
    );
}
