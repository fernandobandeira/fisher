use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};
use serde::Deserialize;

const MANIFEST: &str = "crates/fisher/Cargo.toml";
const SOURCE_DIRECTORY: &str = "crates/fisher/src";
const MAIN: &str = "crates/fisher/src/main.rs";
const MOVE_LOGIC: &str = "Move the logic to the library crate that owns it.";

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    manifest_path: PathBuf,
    targets: Vec<Target>,
}

#[derive(Deserialize)]
struct Target {
    kind: Vec<String>,
    src_path: PathBuf,
}

pub(crate) fn check(root: &Path) -> Result<Vec<String>, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", root.display()))?;
    let mut diagnostics = target_diagnostics(&root)?;
    scan_directory(&root, Path::new(SOURCE_DIRECTORY), &mut diagnostics)?;

    let source = fs::read_to_string(root.join(MAIN)).map_err(|error| format!("{MAIN}: {error}"))?;
    let tokens = lex(&source).map_err(|error| format!("{MAIN}: {error}"))?;
    diagnostics.extend(source_diagnostics(&tokens));
    diagnostics.sort_unstable();
    Ok(diagnostics)
}

fn metadata(root: &Path) -> Result<Metadata, String> {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version=1", "--no-deps", "--locked"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("could not run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("could not read cargo metadata: {error}"))
}

fn target_diagnostics(root: &Path) -> Result<Vec<String>, String> {
    let manifest = root.join(MANIFEST);
    let package = metadata(root)?
        .packages
        .into_iter()
        .find(|package| package.name == "fisher" && package.manifest_path == manifest)
        .ok_or_else(|| format!("could not find the fisher package at {MANIFEST}"))?;
    let mut diagnostics = Vec::new();
    let binaries = package
        .targets
        .iter()
        .filter(|target| target.kind == ["bin"])
        .map(|target| &target.src_path);
    if !binaries.eq([&root.join(MAIN)]) {
        diagnostics.push(format!(
            "{MANIFEST}: fisher must have exactly one binary target at src/main.rs. {MOVE_LOGIC}"
        ));
    }
    if package
        .targets
        .iter()
        .any(|target| target.kind == ["custom-build"])
    {
        diagnostics.push(format!(
            "{MANIFEST}: fisher cannot have a build script. {MOVE_LOGIC}"
        ));
    }
    Ok(diagnostics)
}

fn scan_directory(
    root: &Path,
    directory: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<(), String> {
    let io_error = |error| format!("{}: {error}", directory.display());
    for entry in fs::read_dir(root.join(directory)).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let path = directory.join(entry.file_name());
        if entry.file_type().map_err(io_error)?.is_dir() {
            scan_directory(root, &path, diagnostics)?;
        } else if path != Path::new(MAIN) && path.extension().is_some_and(|ext| ext == "rs") {
            diagnostics.push(format!(
                "{}: Keep src/main.rs as the only Rust source under crates/fisher/src/. {MOVE_LOGIC}",
                path.display(),
            ));
        }
    }
    Ok(())
}

struct Token<'a> {
    text: &'a str,
    kind: TokenKind,
    line: usize,
}

fn lex(source: &str) -> Result<Vec<Token<'_>>, &'static str> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut remaining = source
        .get(strip_shebang(source).unwrap_or_default()..)
        .ok_or("invalid Rust shebang boundary")?;
    let mut line = 1;
    let mut tokens = Vec::new();
    for token in tokenize(remaining, FrontmatterAllowed::Yes) {
        let (text, rest) = usize::try_from(token.len)
            .ok()
            .and_then(|len| remaining.split_at_checked(len))
            .ok_or("invalid Rust token boundary")?;
        remaining = rest;
        if !matches!(
            token.kind,
            TokenKind::Whitespace | TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        ) {
            tokens.push(Token {
                text,
                kind: token.kind,
                line,
            });
        }
        line += text.matches('\n').count();
    }
    Ok(tokens)
}

fn source_diagnostics(mut tokens: &[Token<'_>]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut main_seen = false;
    while let [token, rest @ ..] = tokens {
        tokens = rest;
        let reason = match (token.text, rest) {
            ("fn", [name, ..]) if matches!(name.kind, TokenKind::Ident | TokenKind::RawIdent) => {
                if name.text.trim_start_matches("r#") == "main" && !main_seen {
                    main_seen = true;
                    continue;
                }
                Some("Only main may be declared as a named function.")
            }
            _ => banned_form(token.text, rest),
        };
        if let Some(reason) = reason {
            diagnostics.push(format!("{MAIN}:{}: {reason} {MOVE_LOGIC}", token.line));
        }
    }
    if !main_seen {
        diagnostics.push(format!(
            "{MAIN}:1: Declare fn main to wire the library crates together. {MOVE_LOGIC}"
        ));
    }
    diagnostics
}

fn banned_form(text: &str, rest: &[Token<'_>]) -> Option<&'static str> {
    match (text, rest) {
        ("mod", _) => Some("Module declarations are forbidden."),
        ("impl", _) => Some("impl is forbidden."),
        (
            "#",
            [
                Token { text: "[", .. },
                name,
                Token {
                    text: "=" | "]" | "(",
                    ..
                },
                ..,
            ]
            | [
                Token { text: "!", .. },
                Token { text: "[", .. },
                name,
                Token {
                    text: "=" | "]" | "(",
                    ..
                },
                ..,
            ],
        ) if name.text.trim_start_matches("r#") == "path" => {
            Some("The #[path] attribute is forbidden.")
        }
        (name, [Token { text: "!", .. }, ..]) => match name.trim_start_matches("r#") {
            "include" => Some("include! is forbidden."),
            "include_str" => Some("include_str! is forbidden."),
            "include_bytes" => Some("include_bytes! is forbidden."),
            "macro_rules" => Some("macro_rules! is forbidden."),
            _ => None,
        },
        _ => None,
    }
}
