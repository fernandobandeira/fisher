use std::{
    fs, io,
    path::PathBuf,
    process::{self, Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> io::Result<Self> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "fisher-xtask-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        let workspace = Self(path);
        fs::create_dir_all(workspace.0.join("crates/example/src"))?;
        fs::write(
            workspace.0.join("GLOSSARY.md"),
            include_str!("../fixtures/glossary.md"),
        )?;
        Ok(workspace)
    }

    fn write_root(&self, source: &str) -> io::Result<()> {
        fs::write(self.0.join("crates/example/src/lib.rs"), source)
    }

    fn run(&self, check: &str) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args([check, "--root"])
            .arg(&self.0)
            .output()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

fn accepts(output: &Output) {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

fn rejects(output: &Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(diagnostic), "{stderr}");
}

#[test]
fn glossary_reports_identifiers_and_accepts_near_misses() -> io::Result<()> {
    let workspace = Workspace::new()?;
    workspace.write_root(include_str!("../fixtures/bad.rs"))?;
    rejects(
        &workspace.run("glossary")?,
        "crates/example/src/lib.rs:1:4: identifier \"row\" uses \"row\". The glossary term is \"Rank\" (GLOSSARY.md:3). Rename it.",
    );
    workspace.write_root(include_str!("../fixtures/near_misses.rs"))?;
    accepts(&workspace.run("glossary")?);
    Ok(())
}

#[test]
fn interfaces_reports_public_modules_and_accepts_re_exports() -> io::Result<()> {
    let workspace = Workspace::new()?;
    workspace.write_root("pub mod implementation;")?;
    rejects(
        &workspace.run("interfaces")?,
        "crates/example/src/lib.rs:1: use a private module and explicitly re-export its public items at the crate root.",
    );
    workspace.write_root("mod implementation; pub use implementation::Position;")?;
    accepts(&workspace.run("interfaces")?);
    Ok(())
}

#[test]
fn reports_missing_inputs_and_invalid_commands() -> io::Result<()> {
    let workspace = Workspace::new()?;
    fs::remove_dir_all(workspace.0.join("crates"))?;
    rejects(&workspace.run("interfaces")?, "/crates:");
    fs::remove_file(workspace.0.join("GLOSSARY.md"))?;
    rejects(&workspace.run("glossary")?, "/GLOSSARY.md:");
    rejects(&workspace.run("unknown")?, "Usage: cargo xtask");
    Ok(())
}

#[test]
fn uses_the_workspace_by_default_and_rejects_an_unknown_option() -> io::Result<()> {
    let command = || Command::new(env!("CARGO_BIN_EXE_xtask"));
    accepts(&command().arg("glossary").output()?);
    accepts(&command().arg("interfaces").output()?);
    let workspace = Workspace::new()?;
    rejects(
        &command()
            .args(["glossary", "--other"])
            .arg(&workspace.0)
            .output()?,
        "Usage: cargo xtask",
    );
    Ok(())
}

#[test]
fn glossary_checks_only_rust_sources_and_skips_fixture_and_build_paths() -> io::Result<()> {
    let workspace = Workspace::new()?;
    for directory in ["crates/example/target", "crates/xtask/fixtures"] {
        fs::create_dir_all(workspace.0.join(directory))?;
        fs::write(workspace.0.join(directory).join("bad.rs"), "fn row() {}")?;
    }
    fs::write(workspace.0.join("crates/example/notes.txt"), "fn row() {}")?;
    accepts(&workspace.run("glossary")?);
    Ok(())
}

#[test]
fn interfaces_skips_binary_crates_but_reports_other_read_errors() -> io::Result<()> {
    let workspace = Workspace::new()?;
    fs::write(
        workspace.0.join("crates/example/src/main.rs"),
        "fn main() {}",
    )?;
    accepts(&workspace.run("interfaces")?);
    fs::create_dir(workspace.0.join("crates/example/src/lib.rs"))?;
    rejects(&workspace.run("interfaces")?, "crates/example/src/lib.rs:");
    Ok(())
}

#[test]
fn interfaces_counts_the_shebang_before_the_public_module() -> io::Result<()> {
    let workspace = Workspace::new()?;
    workspace.write_root("#!/usr/bin/env script\npub mod implementation;")?;
    rejects(
        &workspace.run("interfaces")?,
        "crates/example/src/lib.rs:2: use a private module",
    );
    Ok(())
}

fn todo_workspace() -> io::Result<Workspace> {
    let workspace = Workspace::new()?;
    for directory in [
        "scripts",
        "strength/subdirectory",
        "crates/xtask/fixtures",
        "crates/target",
    ] {
        fs::create_dir_all(workspace.0.join(directory))?;
    }
    Ok(workspace)
}

fn rejected_lines(output: &Output, expected: &[(&str, usize)]) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let locations: Vec<_> = stderr
        .lines()
        .map(|line| line.splitn(3, ':').take(2).collect::<Vec<_>>().join(":"))
        .collect();
    let expected: Vec<_> = expected
        .iter()
        .map(|(path, line)| format!("{path}:{line}"))
        .collect();
    assert_eq!(locations, expected, "{stderr}");
    assert!(
        stderr.contains("TODO(fisher-<id>): <nonblank description>"),
        "{stderr}"
    );
}

#[test]
fn todos_rejects_each_bad_rust_comment() -> io::Result<()> {
    let workspace = todo_workspace()?;
    workspace.write_root(include_str!("../fixtures/todo_bad.rs"))?;
    rejected_lines(
        &workspace.run("todos")?,
        &[1, 2, 3, 5, 6, 8, 9, 10, 11, 12, 13, 14].map(|line| ("crates/example/src/lib.rs", line)),
    );
    Ok(())
}

#[test]
fn todos_checks_script_comments_by_extension_and_shebang() -> io::Result<()> {
    let workspace = todo_workspace()?;
    let python = include_str!("../fixtures/todo_bad.py");
    let shell = include_str!("../fixtures/todo_bad.sh");
    fs::write(workspace.0.join("scripts/example.py"), python)?;
    fs::write(workspace.0.join("scripts/example.sh"), shell)?;
    fs::write(workspace.0.join("strength/subdirectory/python"), python)?;
    fs::write(
        workspace.0.join("strength/subdirectory/shell.script"),
        shell,
    )?;
    let python_lines: &[usize] = &[2, 3, 4];
    let shell_lines: &[usize] = &[2, 3, 4, 7];
    let expected: Vec<_> = [
        ("scripts/example.py", python_lines),
        ("scripts/example.sh", shell_lines),
        ("strength/subdirectory/python", python_lines),
        ("strength/subdirectory/shell.script", shell_lines),
    ]
    .into_iter()
    .flat_map(|(path, lines)| lines.iter().map(move |&line| (path, line)))
    .collect();
    rejected_lines(&workspace.run("todos")?, &expected);
    Ok(())
}

#[test]
fn todos_accepts_near_misses_and_ignores_out_of_scope_paths() -> io::Result<()> {
    let workspace = todo_workspace()?;
    workspace.write_root(include_str!("../fixtures/todo_near_misses.rs"))?;
    for (path, source) in [
        (
            "scripts/example",
            include_str!("../fixtures/todo_near_misses.py"),
        ),
        (
            "strength/example.sh",
            include_str!("../fixtures/todo_near_misses.sh"),
        ),
        ("crates/xtask/fixtures/skipped.rs", "// TODO"),
        ("crates/target/skipped.rs", "// TODO"),
        ("crates/example/notes.txt", "// TODO"),
        ("scripts/notes.md", "#!/bin/sh\n# TODO"),
        ("scripts/notes", "# An extensionless document\n# TODO"),
        ("strength/example.rs", "// TODO"),
    ] {
        fs::write(workspace.0.join(path), source)?;
    }
    accepts(&workspace.run("todos")?);
    Ok(())
}

#[test]
fn todos_counts_shebang_and_reports_missing_inputs() -> io::Result<()> {
    let workspace = todo_workspace()?;
    workspace.write_root("\u{feff}#!/usr/bin/env rust-script\n/** TODO */")?;
    rejected_lines(
        &workspace.run("todos")?,
        &[("crates/example/src/lib.rs", 2)],
    );
    fs::remove_dir_all(workspace.0.join("crates"))?;
    rejects(&workspace.run("todos")?, "/crates:");
    fs::create_dir(workspace.0.join("crates"))?;
    fs::remove_dir_all(workspace.0.join("scripts"))?;
    rejects(&workspace.run("todos")?, "scripts");
    Ok(())
}

#[test]
fn todos_reports_an_unreadable_rust_source() -> io::Result<()> {
    let workspace = todo_workspace()?;
    fs::write(workspace.0.join("crates/example/src/lib.rs"), [0xFF])?;
    rejects(&workspace.run("todos")?, "crates/example/src/lib.rs:");
    Ok(())
}

#[test]
fn todos_reports_a_missing_python_interpreter() -> io::Result<()> {
    let workspace = todo_workspace()?;
    fs::create_dir(workspace.0.join("empty-path"))?;
    rejects(
        &Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["todos", "--root"])
            .arg(&workspace.0)
            .env("PATH", workspace.0.join("empty-path"))
            .output()?,
        "could not run python3 to read script comments:",
    );
    Ok(())
}
