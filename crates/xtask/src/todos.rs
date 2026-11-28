use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

const REQUIRED_FORMAT: &str = "TODO(fisher-<id>): <nonblank description>";
type Finding = (PathBuf, usize, &'static str);

pub(crate) fn check(root: &Path) -> Result<Vec<String>, String> {
    let mut findings = Vec::new();
    scan_rust_directory(root, Path::new("crates"), &mut findings)?;
    scan_scripts(root, &mut findings)?;
    findings.sort_unstable();
    Ok(findings
        .into_iter()
        .map(|(path, line, marker)| {
            format!(
                "{}:{line}: replace {marker} with {REQUIRED_FORMAT}.",
                path.display()
            )
        })
        .collect())
}

fn scan_rust_directory(
    root: &Path,
    directory: &Path,
    findings: &mut Vec<Finding>,
) -> Result<(), String> {
    let directory_path = root.join(directory);
    let io_error = |error| format!("{}: {error}", directory_path.display());
    for entry in fs::read_dir(&directory_path).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let path = directory.join(entry.file_name());
        let kind = entry.file_type().map_err(io_error)?;
        if kind.is_dir() {
            if entry.file_name() != "target" && path != Path::new("crates/xtask/fixtures") {
                scan_rust_directory(root, &path, findings)?;
            }
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(entry.path())
                .map_err(|error| format!("{}: {error}", path.display()))?;
            scan_rust(&path, &source, findings)?;
        }
    }
    Ok(())
}

fn scan_rust(path: &Path, source: &str, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let shebang_len = strip_shebang(source).unwrap_or_default();
    let (shebang, mut remaining) = source
        .split_at_checked(shebang_len)
        .ok_or_else(|| format!("{}: invalid Rust shebang boundary", path.display()))?;
    let mut line = shebang.matches('\n').count() + 1;
    for token in tokenize(remaining, FrontmatterAllowed::Yes) {
        let (text, rest) = usize::try_from(token.len)
            .ok()
            .and_then(|len| remaining.split_at_checked(len))
            .ok_or_else(|| format!("{}:{line}: invalid Rust token boundary", path.display()))?;
        remaining = rest;
        if matches!(token.kind, TokenKind::LineComment { .. }) {
            check_comment(path, line, text, findings);
        }
        if matches!(token.kind, TokenKind::BlockComment { .. }) {
            for (offset, piece) in text.lines().enumerate() {
                // Nested delimiters and closing delimiters are not description text.
                for comment in piece.split("/*").flat_map(|part| part.split("*/")) {
                    check_comment(path, line + offset, comment, findings);
                }
            }
        }
        line += text.matches('\n').count();
    }
    Ok(())
}

fn scan_scripts(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let output = Command::new("python3")
        .args(["-I", "-c", include_str!("script_comments.py")])
        .current_dir(root)
        .output()
        .map_err(|error| format!("could not run python3 to read script comments: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "could not read script comments: {}",
            String::from_utf8_lossy(&output.stderr).trim_end()
        ));
    }
    let comments = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let mut records = comments.split_terminator('\0');
    while let Some(path) = records.next() {
        let (line, comment) = records
            .next()
            .zip(records.next())
            .ok_or("incomplete script comment record")?;
        let line = line.parse::<usize>().map_err(|error| error.to_string())?;
        check_comment(Path::new(path), line, comment, findings);
    }
    Ok(())
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn check_comment(path: &Path, line: usize, comment: &str, findings: &mut Vec<Finding>) {
    for marker in ["TODO", "FIXME"] {
        for (offset, _) in comment.match_indices(marker) {
            let (before, tail) = comment.split_at(offset);
            let after = tail.strip_prefix(marker).unwrap_or_default();
            if before.ends_with(is_word_character) || after.starts_with(is_word_character) {
                continue;
            }
            if marker == "FIXME" || !names_ticket(after) {
                findings.push((path.to_owned(), line, marker));
            }
        }
    }
}

fn names_ticket(after_marker: &str) -> bool {
    let Some((id, description)) = after_marker
        .strip_prefix("(fisher-")
        .and_then(|rest| rest.split_once("): "))
    else {
        return false;
    };

    let mut parts = id.split('.');
    let root = parts.next().unwrap_or_default();
    !description.trim().is_empty()
        && !root.is_empty()
        && root
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && parts.all(|child| !child.is_empty() && child.bytes().all(|byte| byte.is_ascii_digit()))
}
