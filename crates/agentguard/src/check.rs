use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use agentguard_core::{Diagnostic, Evidence, SourceLocation};
use serde::Deserialize;

use crate::changes::{collect_changes, Change};
use crate::eslint::run_eslint;

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

#[derive(Clone, Copy)]
enum Language {
    Python,
    JavaScript,
}

impl Language {
    fn matches(self, path: &str) -> bool {
        let extension = Path::new(path).extension().and_then(|value| value.to_str());
        match self {
            Self::Python => matches!(extension, Some("py" | "pyi")),
            Self::JavaScript => matches!(
                extension,
                Some("js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts")
            ),
        }
    }
}

pub(crate) fn run() -> ExitCode {
    let result = (|| {
        let changes = collect_changes()?;
        let root = repository_root()?;
        Ok::<_, String>((root, changes))
    })();
    let (root, changes) = match result {
        Ok(result) => result,
        Err(error) => return report_error("Git", error),
    };

    let python = changed_paths(&root, &changes, Language::Python);
    let javascript = changed_paths(&root, &changes, Language::JavaScript);
    if python.is_empty() && javascript.is_empty() {
        println!("No changed Python or JavaScript/TypeScript files to check.");
        return ExitCode::SUCCESS;
    }

    let mut status = 0;
    if !python.is_empty() {
        status = status.max(report(
            "Ruff",
            &format!("{} changed Python file(s)", python.len()),
            run_ruff(&root, &python),
        ));
    }
    if !javascript.is_empty() {
        status = status.max(report(
            "ESLint",
            &format!("{} changed JavaScript/TypeScript file(s)", javascript.len()),
            run_eslint(&root, &javascript),
        ));
    }
    ExitCode::from(status)
}

pub(crate) fn run_audit() -> ExitCode {
    let root = match repository_root() {
        Ok(root) => root,
        Err(error) => return report_error("Git", error),
    };
    let files = match repository_files(&root) {
        Ok(files) => files,
        Err(error) => return report_error("Git", error),
    };
    let python = files.iter().any(|file| Language::Python.matches(file));
    let javascript = files.iter().any(|file| Language::JavaScript.matches(file));
    if !python && !javascript {
        println!("No supported Python or JavaScript/TypeScript files to audit.");
        return ExitCode::SUCCESS;
    }

    let mut status = 0;
    if python {
        status = status.max(report(
            "Ruff",
            "repository scope (Ruff exclusions apply)",
            run_ruff(&root, &[".".to_owned()]),
        ));
    }
    if javascript {
        status = status.max(report(
            "ESLint",
            "repository scope (ESLint exclusions apply)",
            run_eslint(&root, &[".".to_owned()]),
        ));
    }
    ExitCode::from(status)
}

fn report(tool: &str, scope: &str, result: Result<Vec<Diagnostic>, String>) -> u8 {
    match result {
        Ok(diagnostics) if diagnostics.is_empty() => {
            println!("{tool}: no findings in {scope}.");
            0
        }
        Ok(diagnostics) => {
            let count = diagnostics.len();
            for diagnostic in diagnostics {
                if let Some(location) = diagnostic.location {
                    println!(
                        "{:?}:{}:{}: {tool} {} (static finding): {}",
                        location.path, location.line, location.column, diagnostic.code, diagnostic.message
                    );
                } else {
                    println!("{tool} {} (static finding): {}", diagnostic.code, diagnostic.message);
                }
            }
            println!("{tool}: {count} finding(s).");
            1
        }
        Err(error) => {
            eprintln!("{tool} check could not complete: {error}");
            2
        }
    }
}

fn report_error(tool: &str, error: String) -> ExitCode {
    eprintln!("{tool} check could not complete: {error}");
    ExitCode::from(2)
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
        .current_dir(root)
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

    Ok(findings
        .into_iter()
        .map(|finding| {
            let path = Path::new(&finding.filename);
            let displayed_path = path.strip_prefix(root).unwrap_or(path);
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
        .collect())
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

fn repository_files(root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args([
            "--no-optional-locks",
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "--full-name",
            "-z",
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("could not list Git files: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    if !output.stdout.is_empty() && output.stdout.last() != Some(&0) {
        return Err("Git file list is missing its final delimiter".to_owned());
    }
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .map(|record| {
            let path = std::str::from_utf8(record)
                .map_err(|_| "Git returned a path that is not valid UTF-8".to_owned())?;
            Ok(path.to_owned())
        })
        .collect()
}

fn changed_paths(root: &Path, changes: &[Change], language: Language) -> Vec<String> {
    changes
        .iter()
        .filter(|change| language.matches(&change.path))
        .filter(|change| {
            fs::symlink_metadata(root.join(&change.path))
                .is_ok_and(|metadata| metadata.file_type().is_file())
        })
        .map(|change| format!("./{}", change.path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{changed_paths, Language, RuffFinding};
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
    fn recognizes_supported_source_extensions() {
        for extension in ["js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts"] {
            assert!(Language::JavaScript.matches(&format!("src/a.{extension}")));
        }
        assert!(Language::Python.matches("src/a.pyi"));
        assert!(!Language::JavaScript.matches("src/a.py"));
        assert!(!Language::Python.matches("src/a.ts"));
        assert!(!Language::JavaScript.matches("README.md"));
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
        assert!(changed_paths(&root, &changes, Language::Python).is_empty());
    }
}
