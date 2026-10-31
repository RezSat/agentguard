use std::path::Path;
use std::process::Command;

use agentguard_core::{Diagnostic, Evidence, SourceLocation};
use serde::Deserialize;

#[derive(Deserialize)]
struct EslintFile {
    #[serde(rename = "filePath")]
    file_path: String,
    messages: Vec<EslintMessage>,
}

#[derive(Deserialize)]
struct EslintMessage {
    #[serde(rename = "ruleId")]
    rule_id: Option<String>,
    message: String,
    line: Option<u32>,
    column: Option<u32>,
    severity: u8,
    #[serde(default)]
    fatal: bool,
}

pub(crate) fn run_eslint(root: &Path, targets: &[String]) -> Result<Vec<Diagnostic>, String> {
    let local = root.join("node_modules").join(".bin").join("eslint");
    let executable = if local.is_file() {
        local.as_os_str().to_owned()
    } else {
        "eslint".into()
    };
    let output = Command::new(executable)
        .args([
            "--format",
            "json",
            "--no-warn-ignored",
            "--no-error-on-unmatched-pattern",
            "--ext",
            ".js,.jsx,.mjs,.cjs,.ts,.tsx,.mts,.cts",
            "--",
        ])
        .args(targets)
        .current_dir(root)
        .output()
        .map_err(|error| {
            format!("could not start ESLint ({error}); install it in the project or on PATH")
        })?;

    let status = output.status.code();
    if !matches!(status, Some(0 | 1)) {
        return Err(format!(
            "ESLint failed (status {:?}): {}",
            status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let files: Vec<EslintFile> = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("could not parse ESLint JSON: {error}"))?;
    let mut diagnostics = Vec::new();
    for file in files {
        let path = Path::new(&file.file_path);
        let relative = path.strip_prefix(root).unwrap_or(path);
        for message in file.messages {
            let label = match message.severity {
                1 => "warning",
                2 => "error",
                _ => "unknown severity",
            };
            let code = message.rule_id.unwrap_or_else(|| {
                if message.fatal {
                    "parse-error".to_owned()
                } else {
                    "unclassified".to_owned()
                }
            });
            let mut diagnostic = Diagnostic::new(
                code,
                format!("{label}: {}", message.message),
                Evidence::StaticFinding,
            );
            if let (Some(line), Some(column)) = (message.line, message.column) {
                diagnostic = diagnostic.at(SourceLocation {
                    path: relative.to_string_lossy().into_owned(),
                    line,
                    column,
                });
            } else {
                diagnostic.message = format!("{:?}: {}", relative, diagnostic.message);
            }
            diagnostics.push(diagnostic);
        }
    }
    if status == Some(1) && diagnostics.is_empty() {
        return Err("ESLint exited with status 1 but returned no findings".to_owned());
    }
    Ok(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::EslintFile;

    #[test]
    fn parses_eslint_json_with_warnings_and_missing_locations() {
        let result: Vec<EslintFile> = serde_json::from_str(
            r#"[{"filePath":"/tmp/app.ts","messages":[{"ruleId":"no-unused-vars","message":"Unused","line":2,"column":1,"severity":1},{"ruleId":null,"message":"Parsing error","severity":2,"fatal":true}]}]"#,
        )
        .unwrap();
        assert_eq!(result[0].messages.len(), 2);
        assert_eq!(result[0].messages[0].severity, 1);
        assert!(result[0].messages[1].line.is_none());
    }
}
