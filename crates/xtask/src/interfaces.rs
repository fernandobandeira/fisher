use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

pub(crate) fn check(root: &Path) -> Result<Vec<String>, String> {
    let crates = root.join("crates");
    let entries =
        fs::read_dir(&crates).map_err(|error| format!("{}: {error}", crates.display()))?;
    let mut diagnostics = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", crates.display()))?;
        if !entry.path().is_dir() {
            continue;
        }
        let path = entry.path().join("src/lib.rs");
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        let path = path.strip_prefix(root).unwrap_or(&path);
        for line in
            public_modules(&source).map_err(|error| format!("{}: {error}", path.display()))?
        {
            diagnostics.push(format!(
                "{}:{line}: use a private module and explicitly re-export its public items at the crate root.",
                path.display(),
            ));
        }
    }
    diagnostics.sort_unstable();
    Ok(diagnostics)
}

fn public_modules(source: &str) -> Result<Vec<usize>, &'static str> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let (shebang, mut remaining) = source
        .split_at_checked(strip_shebang(source).unwrap_or_default())
        .ok_or("invalid Rust shebang boundary")?;
    let mut line = 1 + shebang.matches('\n').count();
    let mut nesting = 0_usize;
    let mut public = None;
    let mut lines = Vec::new();
    for token in tokenize(remaining, FrontmatterAllowed::Yes) {
        let (text, rest) = usize::try_from(token.len)
            .ok()
            .and_then(|len| remaining.split_at_checked(len))
            .ok_or("invalid Rust token boundary")?;
        remaining = rest;
        let token_line = line;
        line += text.matches('\n').count();

        if matches!(
            token.kind,
            TokenKind::Whitespace | TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        ) {
            continue;
        }
        nesting = next_nesting(nesting, token.kind);
        if text == "mod"
            && let Some(line) = public
        {
            lines.push(line);
        }
        public = (nesting == 0 && text == "pub").then_some(token_line);
    }
    Ok(lines)
}

fn next_nesting(nesting: usize, kind: TokenKind) -> usize {
    if matches!(
        kind,
        TokenKind::OpenBrace | TokenKind::OpenBracket | TokenKind::OpenParen
    ) {
        nesting + 1
    } else if matches!(
        kind,
        TokenKind::CloseBrace | TokenKind::CloseBracket | TokenKind::CloseParen
    ) {
        nesting.saturating_sub(1)
    } else {
        nesting
    }
}

#[cfg(test)]
mod tests {
    use super::public_modules;

    #[test]
    fn reports_a_public_implementation_module() -> Result<(), &'static str> {
        let source = "#[doc(hidden)]\npub /* implementation */\nmod implementation;";
        assert_eq!(public_modules(source)?, [2]);
        Ok(())
    }

    #[test]
    fn permits_root_re_exports_and_restricted_modules() -> Result<(), &'static str> {
        let source = r#"
mod implementation {
    pub struct Position;
    pub mod nested {}
}
pub use implementation::Position;
pub(crate) mod helpers {}
// pub mod fake {}
const TEXT: &str = "pub mod fake {}";
macro_rules! tokens { () => { pub mod generated {} }; }
macro_rules! tokens_to_discard { ($($token:tt)*) => {}; }
tokens_to_discard!(pub mod in_arguments {});
"#;
        assert_eq!(public_modules(source)?, Vec::<usize>::new());
        Ok(())
    }
}
