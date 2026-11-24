use std::collections::HashMap;
use std::fs;
use std::path::Path;

use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

#[derive(Debug)]
struct Banned<'a> {
    phrase: &'a str,
    segments: Vec<String>,
    term: &'a str,
    term_line: usize,
}

#[derive(Debug)]
struct Entry<'a> {
    term: &'a str,
    line: usize,
    has_definition: bool,
}

pub(crate) fn check(root: &Path) -> Result<Vec<String>, String> {
    let glossary = read(&root.join("GLOSSARY.md"))?;
    let banned = parse(&glossary)?;
    let mut diagnostics = Vec::new();
    scan_directory(root, &root.join("crates"), &banned, &mut diagnostics)?;
    diagnostics.sort_unstable();
    Ok(diagnostics)
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn parse(glossary: &str) -> Result<Vec<Banned<'_>>, String> {
    let mut entries: Vec<Entry<'_>> = Vec::new();
    let mut banned = Vec::new();
    let mut in_entry = false;
    let mut in_definition = false;
    for (index, line) in glossary.lines().enumerate() {
        let line_number = index + 1;
        let line = line.trim();
        if let Some(term) = line
            .strip_prefix("**")
            .and_then(|line| line.strip_suffix("**:"))
        {
            if term.trim().is_empty() {
                return Err(format!(
                    "GLOSSARY.md:{line_number}: name the glossary term."
                ));
            }
            entries.push(Entry {
                term: term.trim(),
                line: line_number,
                has_definition: false,
            });
            in_entry = true;
            in_definition = true;
        } else if line.starts_with('#') {
            in_entry = false;
            in_definition = false;
        } else if line.starts_with("_Avoid_:") {
            in_definition = false;
        } else if let Some(list) = line.strip_prefix("_Banned in code_:") {
            in_definition = false;
            let Some(entry) = entries.last().filter(|_| in_entry) else {
                return Err(format!(
                    "GLOSSARY.md:{line_number}: put banned wording under a glossary term."
                ));
            };
            banned.extend(parse_bans(list, entry, line_number)?);
        } else if in_definition
            && !line.is_empty()
            && let Some(entry) = entries.last_mut()
        {
            entry.has_definition = true;
        }
    }

    check_definitions(entries, &banned)?;
    Ok(banned)
}

fn parse_bans<'a>(
    list: &'a str,
    entry: &Entry<'a>,
    line: usize,
) -> Result<Vec<Banned<'a>>, String> {
    list.split(',')
        .map(str::trim)
        .map(|phrase| {
            let segments = phrase_segments(phrase);
            if segments.is_empty() {
                return Err(format!(
                    "GLOSSARY.md:{line}: list a banned word or remove the empty item."
                ));
            }
            Ok(Banned {
                phrase,
                segments,
                term: entry.term,
                term_line: entry.line,
            })
        })
        .collect()
}

fn check_definitions(entries: Vec<Entry<'_>>, banned: &[Banned<'_>]) -> Result<(), String> {
    let mut defined = HashMap::new();
    for entry in entries {
        if !entry.has_definition {
            return Err(format!(
                "GLOSSARY.md:{}: add a definition for {:?}.",
                entry.line, entry.term
            ));
        }
        defined.insert(phrase_segments(entry.term), entry.line);
    }
    for ban in banned {
        if let Some(line) = defined.get(&ban.segments) {
            return Err(format!(
                "GLOSSARY.md:{line}: {:?} is a defined term and is banned under {:?} on line {}. Remove the ban or resolve the conflicting terms.",
                ban.phrase, ban.term, ban.term_line
            ));
        }
    }
    Ok(())
}

fn phrase_segments(phrase: &str) -> Vec<String> {
    phrase.split_whitespace().map(str::to_lowercase).collect()
}

fn scan_directory(
    root: &Path,
    directory: &Path,
    banned: &[Banned<'_>],
    diagnostics: &mut Vec<String>,
) -> Result<(), String> {
    let io_error = |error| format!("{}: {error}", directory.display());
    for entry in fs::read_dir(directory).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let path = entry.path();
        let kind = entry.file_type().map_err(io_error)?;
        if kind.is_dir() {
            if entry.file_name() != "target" && path != root.join("crates/xtask/fixtures") {
                scan_directory(root, &path, banned, diagnostics)?;
            }
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            let source = read(&path)?;
            let relative = path.strip_prefix(root).unwrap_or(&path);
            scan_source(relative, &source, banned, diagnostics)?;
        }
    }
    Ok(())
}

fn identifier_segments(identifier: &str) -> Vec<String> {
    let mut segments = Vec::new();
    for part in identifier.split('_').filter(|part| !part.is_empty()) {
        let mut start = 0;
        let mut previous = None;
        let mut chars = part.char_indices().peekable();
        while let Some((offset, ch)) = chars.next() {
            // Keep an acronym together and split before the following capitalized word.
            if ch.is_uppercase()
                && previous.is_some_and(|prev: char| {
                    prev.is_lowercase()
                        || prev.is_ascii_digit()
                        || (prev.is_uppercase()
                            && chars.peek().is_some_and(|(_, next)| next.is_lowercase()))
                })
            {
                segments.extend(part.get(start..offset).map(str::to_lowercase));
                start = offset;
            }
            previous = Some(ch);
        }
        segments.extend(part.get(start..).map(str::to_lowercase));
    }
    segments
}

fn is_keyword(word: &str) -> bool {
    // Strict and reserved Rust 2024 keywords. Weak keywords can be identifiers.
    const KEYWORDS: &str = "as break const continue crate else enum extern false fn for if impl in
        let loop match mod move mut pub ref return self Self static struct super trait true type
        unsafe use where while async await dyn abstract become box do final gen macro override
        priv try typeof unsized virtual yield";
    KEYWORDS
        .split_ascii_whitespace()
        .any(|keyword| word == keyword)
}

fn scan_source(
    path: &Path,
    source: &str,
    banned: &[Banned<'_>],
    diagnostics: &mut Vec<String>,
) -> Result<(), String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut remaining = source
        .get(strip_shebang(source).unwrap_or_default()..)
        .ok_or_else(|| format!("{}: invalid shebang boundary", path.display()))?;
    let mut identifiers = Vec::new();
    let mut allowances: HashMap<usize, Vec<Vec<String>>> = HashMap::new();
    let (mut line, mut character_number) = (1, 1);
    for token in tokenize(remaining, FrontmatterAllowed::Yes) {
        let (text, rest) = usize::try_from(token.len)
            .ok()
            .and_then(|len| remaining.split_at_checked(len))
            .ok_or_else(|| {
                format!(
                    "{}:{line}:{character_number}: invalid Rust token boundary",
                    path.display()
                )
            })?;
        remaining = rest;

        if let Some(directive) = allowance_directive(token.kind, text) {
            if let Some(phrase) = allowed_phrase(directive) {
                allowances.entry(line).or_default().push(phrase);
            } else {
                diagnostics.push(format!(
                    "{}:{line}:{character_number}: use // glossary-allow: <word> -- <reason> with a word and a reason.",
                    path.display()
                ));
            }
        }

        if let Some(identifier) = identifier(token.kind, text) {
            let segments = identifier_segments(identifier);
            identifiers.extend(
                matching_bans(banned, segments).map(|ban| (line, character_number, text, ban)),
            );
        }

        let mut pieces = text.split('\n');
        character_number += pieces.next().unwrap_or_default().chars().count();
        for piece in pieces {
            line += 1;
            character_number = piece.chars().count() + 1;
        }
    }

    for (line, character_number, identifier, ban) in identifiers {
        if [line, line - 1].iter().any(|line| {
            allowances
                .get(line)
                .is_some_and(|phrases| phrases.contains(&ban.segments))
        }) {
            continue;
        }
        diagnostics.push(format!(
            "{}:{line}:{character_number}: identifier {identifier:?} uses {:?}. The glossary term is {:?} (GLOSSARY.md:{}). Rename it. If this is a different concept, stop and raise it instead of inventing a word.",
            path.display(), ban.phrase, ban.term, ban.term_line
        ));
    }
    Ok(())
}

fn allowance_directive(kind: TokenKind, text: &str) -> Option<&str> {
    if !matches!(kind, TokenKind::LineComment { doc_style: None }) {
        return None;
    }
    text.strip_prefix("//")?
        .trim_start()
        .strip_prefix("glossary-allow:")
}

fn allowed_phrase(directive: &str) -> Option<Vec<String>> {
    let (phrase, reason) = directive.split_once("--")?;
    (!phrase.trim().is_empty() && !reason.trim().is_empty()).then(|| phrase_segments(phrase))
}

fn identifier(kind: TokenKind, text: &str) -> Option<&str> {
    if kind == TokenKind::Ident && !is_keyword(text) {
        Some(text)
    } else if matches!(
        kind,
        TokenKind::RawIdent
            | TokenKind::RawLifetime
            | TokenKind::Lifetime {
                starts_with_number: false
            }
    ) && text != "'static"
    {
        Some(text.trim_start_matches('\'').trim_start_matches("r#"))
    } else {
        None
    }
}

fn matching_bans<'a, 'b>(
    banned: &'a [Banned<'b>],
    segments: Vec<String>,
) -> impl Iterator<Item = &'a Banned<'b>> {
    banned.iter().filter(move |ban| {
        segments
            .windows(ban.segments.len())
            .any(|window| window == ban.segments)
    })
}

#[cfg(test)]
mod tests;
