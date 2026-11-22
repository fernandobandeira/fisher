use super::{parse, scan_source};
use std::path::Path;

const GLOSSARY: &str = include_str!("../../fixtures/glossary.md");

fn diagnostics(source: &str) -> Vec<String> {
    let banned = parse(GLOSSARY).unwrap();
    let mut errors = Vec::new();
    scan_source(Path::new("fixture.rs"), source, &banned, &mut errors).unwrap();
    errors
}

#[test]
fn matches_whole_segments_and_consecutive_phrases() {
    let errors = diagnostics(include_str!("../../fixtures/bad.rs"));
    for expected in [
        "identifier \"row\"",
        "identifier \"row_index\"",
        "identifier \"RowMask\"",
        "identifier \"first_row\"",
        "identifier \"r#row\"",
        "identifier \"half_move\"",
        "identifier \"HalfMove\"",
        "identifier \"halfMove\"",
        "identifier \"XMLRowMask\"",
    ] {
        assert!(
            errors.iter().any(|error| error.contains(expected)),
            "{expected}: {errors:?}"
        );
    }
    assert!(errors.iter().any(|error| error.contains("fixture.rs:1:4: identifier \"row\" uses \"row\". The glossary term is \"Rank\" (GLOSSARY.md:3). Rename it.")));
}

#[test]
fn ignores_near_misses_prose_literals_and_justified_allowances() {
    assert_eq!(
        diagnostics(include_str!("../../fixtures/near_misses.rs")),
        Vec::<String>::new()
    );
}

#[test]
fn invalid_and_unrelated_allowances_do_not_hide_violations() {
    let errors = diagnostics(include_str!("../../fixtures/bad.rs"));
    for line in [4, 5, 7, 8, 10] {
        assert!(
            errors
                .iter()
                .any(|error| error.starts_with(&format!("fixture.rs:{line}:"))
                    && error.contains("identifier \"row\"")),
            "{errors:?}"
        );
    }
    for line in [6, 8] {
        assert!(
            errors
                .iter()
                .any(|error| error.starts_with(&format!("fixture.rs:{line}:"))
                    && error.contains("with a word and a reason")),
            "{errors:?}"
        );
    }
}

#[test]
fn rejects_missing_definitions_and_banned_terms() {
    let missing = parse(include_str!("../../fixtures/no_definition.md")).unwrap_err();
    assert_eq!(missing, "GLOSSARY.md:3: add a definition for \"Rank\".");

    let conflict = parse(include_str!("../../fixtures/term_conflict.md")).unwrap_err();
    assert!(conflict.starts_with("GLOSSARY.md:5: \"HALF   MOVE\" is a defined term"));
}

#[test]
fn allows_definitions_with_guidance_and_without_bans() {
    let glossary = "**Rank**:\nA horizontal line.\n_Avoid_: row\n\n**Row**:\nA line in a table.";
    assert!(parse(glossary).unwrap().is_empty());
}
