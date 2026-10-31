use std::process::{Command, ExitCode};

#[derive(Debug, Eq, PartialEq)]
struct Change {
    status: [u8; 2],
    path: String,
    original_path: Option<String>,
}

pub(crate) fn run() -> ExitCode {
    match collect_changes() {
        Ok(changes) if changes.is_empty() => {
            println!("No changes found.");
            ExitCode::SUCCESS
        }
        Ok(changes) => {
            for change in changes {
                let status = String::from_utf8_lossy(&change.status);
                if let Some(original_path) = change.original_path {
                    println!("{status} {:?} -> {:?}", original_path, change.path);
                } else {
                    println!("{status} {:?}", change.path);
                }
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not inspect Git changes: {error}");
            ExitCode::from(1)
        }
    }
}

fn collect_changes() -> Result<Vec<Change>, String> {
    let output = Command::new("git")
        .args([
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--renames",
        ])
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git status failed: {}", stderr.trim()));
    }

    parse_changes(&output.stdout)
}

fn parse_changes(output: &[u8]) -> Result<Vec<Change>, String> {
    if !output.is_empty() && output.last() != Some(&0) {
        return Err("Git status output is missing its final delimiter".to_owned());
    }

    let mut records = output.split(|byte| *byte == 0);
    let mut changes = Vec::new();

    while let Some(record) = records.next() {
        if record.is_empty() {
            if records.next().is_some() {
                return Err("Git status output contains an empty record".to_owned());
            }
            break;
        }

        if record.len() < 4 || record[2] != b' ' {
            return Err("Git status output contains a malformed record".to_owned());
        }

        let path = std::str::from_utf8(&record[3..])
            .map_err(|_| "Git reported a path that is not valid UTF-8".to_owned())?
            .to_owned();

        let has_original_path =
            matches!(record[0], b'R' | b'C') || matches!(record[1], b'R' | b'C');
        let original_path = if has_original_path {
            let original = records
                .next()
                .ok_or_else(|| "Git status omitted the original path".to_owned())?;
            if original.is_empty() {
                return Err("Git status supplied an empty original path".to_owned());
            }
            Some(
                std::str::from_utf8(original)
                    .map_err(|_| "Git reported a path that is not valid UTF-8".to_owned())?
                    .to_owned(),
            )
        } else {
            None
        };

        changes.push(Change {
            status: [record[0], record[1]],
            path,
            original_path,
        });
    }

    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::{parse_changes, Change};

    #[test]
    fn parses_modified_untracked_and_renamed_files() {
        let parsed =
            parse_changes(b" M src/app.py\0?? new file.py\0R  src/new.py\0src/old.py\0").unwrap();

        assert_eq!(
            parsed,
            vec![
                Change {
                    status: [b' ', b'M'],
                    path: "src/app.py".to_owned(),
                    original_path: None,
                },
                Change {
                    status: [b'?', b'?'],
                    path: "new file.py".to_owned(),
                    original_path: None,
                },
                Change {
                    status: [b'R', b' '],
                    path: "src/new.py".to_owned(),
                    original_path: Some("src/old.py".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn keeps_newlines_in_file_names() {
        let parsed = parse_changes(b"?? file\nname.py\0").unwrap();

        assert_eq!(parsed[0].path, "file\nname.py");
    }

    #[test]
    fn rejects_truncated_renames_and_non_utf8_paths() {
        assert!(parse_changes(b"R  moved.py\0").is_err());
        assert!(parse_changes(b"?? invalid\xff.py\0").is_err());
        assert!(parse_changes(b"?? without-delimiter").is_err());
    }

    #[test]
    fn accepts_a_clean_repository() {
        assert!(parse_changes(b"").unwrap().is_empty());
    }
}
