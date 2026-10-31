use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use agentguard_core::{Diagnostic, Evidence, SourceLocation};
use serde::Deserialize;

use crate::changes::{collect_changes, Change};

#[derive(Deserialize)]
struct RuffLocation {
    row: u32,
    column: u32,
}

#[derive(Deserialize)]
struct RuffFinding {
    code: Option<String>,
    message: String,
    filename: String,
    location: RuffLocation,
}

pub(crate) fn run() -> ExitCode {
    match check_changed_python() {
        Ok(CheckOutcome::NoFiles) => {
            println!("No changed Python files to check.");
            ExitCode::SUCCESS
        }
        Ok(CheckOutcome::Clean(count)) => {
            println!("Ruff: no findings in {count} changed Python file(s).");
            ExitCode::SUCCESS
        }
        Ok(CheckOutcome::Findings(diagnostics)) => report_findings(diagnostics),
        Err(error) => report_error(error),
    }
}

pub(crate) fn run_audit() -> ExitCode {
    match repository_root().and_then(|root| run_ruff(&root, &[".".to_owned()])) {
        Ok(diagnostics) if diagnostics.is_empty() => {
            println!("Ruff: no findings in repository scope (Ruff exclusions apply).");
            ExitCode::SUCCESS
        }
        Ok(diagnostics) => report_findings(diagnostics),
        Err(error) => report_error(error),
    }
}

fn report_findings(diagnostics: Vec<Diagnostic>) -> ExitCode {
    for diagnostic in &diagnostics {
        let location = diagnostic
            .location
            .as_ref()
            .expect("Ruff provides a location");
        println!(
            "{:?}:{}:{}: Ruff {} (static finding): {}",
            location.path, location.line, location.column, diagnostic.code, diagnostic.message
        );
    }
    println!("Ruff: {} finding(s).", diagnostics.len());
    ExitCode::from(1)
}

fn report_error(error: String) -> ExitCode {
    eprintln!("Python check could not complete: {error}");
    ExitCode::from(2)
}

enum CheckOutcome {
    NoFiles,
    Clean(usize),
    Findings(Vec<Diagnostic>),
}

fn check_changed_python() -> Result<CheckOutcome, String> {
    let changes = collect_changes()?;
    let root = repository_root()?;
    let files = changed_python_paths(&root, &changes);
    if files.is_empty() {
        return Ok(CheckOutcome::NoFiles);
    }

    let diagnostics = run_ruff(&root, &files)?;
    if diagnostics.is_empty() {
        Ok(CheckOutcome::Clean(files.len()))
    } else {
        Ok(CheckOutcome::Findings(diagnostics))
    }
}

fn run_ruff(root: &Path, files: &[String]) -> Result<Vec<Diagnostic>, String> {
    let output = Command::new("ruff")
        .args([
            "check",
            "--no-fix",
            "--no-fix-only",
            "--no-cache",
            "--output-format=json",
            "--",
        ])
        .args(files)
        .current_dir(&root)
        .output()
        .map_err(|error| {
            format!("could not start Ruff ({error}); install Ruff and make it available on PATH")
        })?;

    let status = output.status.code();
    if !matches!(status, Some(0 | 1)) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Ruff failed (status {:?}): {}",
            status,
            stderr.trim()
        ));
    }

    let findings: Vec<RuffFinding> = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("could not parse Ruff JSON: {error}"))?;
    if status == Some(1) && findings.is_empty() {
        return Err("Ruff exited with status 1 but returned no findings".to_owned());
    }

    let diagnostics = findings
        .into_iter()
        .map(|finding| {
            let path = Path::new(&finding.filename);
            let displayed_path = path.strip_prefix(&root).unwrap_or(path);
            Diagnostic::new(
                finding.code.unwrap_or_else(|| "unknown".to_owned()),
                finding.message,
                Evidence::StaticFinding,
            )
            .at(SourceLocation {
                path: displayed_path.to_string_lossy().into_owned(),
                line: finding.location.row,
                column: finding.location.column,
            })
        })
        .collect();

    Ok(diagnostics)
}

fn repository_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| format!("could not start Git: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "could not locate Git repository: {}",
            stderr.trim()
        ));
    }

    let root = std::str::from_utf8(&output.stdout)
        .map_err(|_| "Git repository path is not valid UTF-8".to_owned())?
        .trim_end_matches(['\r', '\n']);
    if root.is_empty() {
        return Err("Git returned an empty repository path".to_owned());
    }
    Ok(PathBuf::from(root))
}

fn changed_python_paths(root: &Path, changes: &[Change]) -> Vec<String> {
    changes
        .iter()
        .filter(|change| change.path.ends_with(".py") || change.path.ends_with(".pyi"))
        .filter(|change| !change.status.contains(&b'D'))
        .filter(|change| {
            fs::symlink_metadata(root.join(&change.path))
                .is_ok_and(|metadata| metadata.file_type().is_file())
        })
        .map(|change| format!("./{}", change.path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{changed_python_paths, RuffFinding};
    use crate::changes::Change;

    #[test]
    fn accepts_ruff_json_findings() {
        let findings: Vec<RuffFinding> = serde_json::from_str(
            r#"[{"code":"F401","message":"Unused import","filename":"src/main.py","location":{"row":3,"column":1}}]"#,
        )
        .unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code.as_deref(), Some("F401"));
        assert_eq!(findings[0].location.row, 3);
    }

    #[test]
    fn does_not_scan_missing_or_non_python_files() {
        let root = std::env::temp_dir().join("agentguard-nonexistent-fixture");
        let changes = vec![
            Change {
                path: "removed.py".to_owned(),
                status: *b" D",
                original_path: None,
            },
            Change {
                path: "README.md".to_owned(),
                status: *b"??",
                original_path: None,
            },
        ];
        assert!(changed_python_paths(&root, &changes).is_empty());
    }
}
