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
    accepts(&command().arg("thin-binary").output()?);
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

const FISHER_MANIFEST: &str = "crates/fisher/Cargo.toml";
const FISHER_MAIN: &str = "crates/fisher/src/main.rs";

fn fisher_workspace() -> io::Result<Workspace> {
    let workspace = Workspace::new()?;
    fs::create_dir_all(workspace.0.join("crates/fisher/src"))?;
    fs::write(
        workspace.0.join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"crates/fisher\"]\n",
    )?;
    fs::write(
        workspace.0.join(FISHER_MANIFEST),
        "[package]\nname = \"fisher\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )?;
    fs::write(
        workspace.0.join(FISHER_MAIN),
        include_str!("../fixtures/thin-binary/wiring.rs"),
    )?;
    Ok(workspace)
}

#[test]
fn thin_binary_reports_the_historical_main_and_accepts_wiring() -> io::Result<()> {
    let workspace = fisher_workspace()?;
    let tests = workspace.0.join("crates/fisher/tests");
    fs::create_dir(&tests)?;
    fs::write(tests.join("protocol.rs"), "fn helper() {}")?;
    fs::write(workspace.0.join("crates/fisher/src/notes.txt"), "notes")?;
    accepts(&workspace.run("thin-binary")?);

    let main = workspace.0.join(FISHER_MAIN);
    fs::write(&main, include_str!("../fixtures/thin-binary/bad.rs"))?;
    rejects(
        &workspace.run("thin-binary")?,
        "Only main may be declared as a named function. Move the logic to the library crate that owns it.",
    );

    fs::write(
        &main,
        include_str!("../fixtures/thin-binary/near_misses.rs"),
    )?;
    accepts(&workspace.run("thin-binary")?);
    Ok(())
}

#[test]
fn thin_binary_rejects_each_banned_source_form() -> io::Result<()> {
    let workspace = fisher_workspace()?;
    for (source, line, diagnostic) in [
        (
            "fn main() { fn r#helper() {} }",
            1,
            "Only main may be declared",
        ),
        ("fn main() { fn main() {} }", 1, "Only main may be declared"),
        ("fn main() {}\nmod parser;", 2, "Module declarations"),
        ("fn main() { include!(\"parser.rs\"); }", 1, "include!"),
        (
            "fn main() { let _ = include_str!(\"data\"); }",
            1,
            "include_str!",
        ),
        (
            "fn main() { let _ = r#include_bytes /* data */ !(\"data\"); }",
            1,
            "include_bytes!",
        ),
        (
            "#[ /* module path */ path = \"parser.rs\"]\nfn main() {}",
            1,
            "The #[path] attribute",
        ),
        (
            "#![path = \"parser.rs\"]\nfn main() {}",
            1,
            "The #[path] attribute",
        ),
        ("#[path]\nfn main() {}", 1, "The #[path] attribute"),
        (
            "\u{feff}#!/usr/bin/env script\nfn main() {}\nmod parser;",
            3,
            "Module declarations",
        ),
        (
            "fn main() {}\nmacro_rules! helper { () => {} }",
            2,
            "macro_rules!",
        ),
        ("fn main() {}\nimpl Parser {}", 2, "impl"),
        ("// No entry point", 1, "Declare fn main"),
    ] {
        fs::write(workspace.0.join(FISHER_MAIN), source)?;
        rejects(
            &workspace.run("thin-binary")?,
            &format!("{FISHER_MAIN}:{line}: {diagnostic}"),
        );
    }
    Ok(())
}

#[test]
fn thin_binary_rejects_extra_rust_source_in_nested_directories() -> io::Result<()> {
    let workspace = fisher_workspace()?;
    let nested = "crates/fisher/src/command";
    fs::create_dir(workspace.0.join(nested))?;
    fs::write(workspace.0.join(nested).join("parser.rs"), "fn parse() {}")?;
    rejects(
        &workspace.run("thin-binary")?,
        "crates/fisher/src/command/parser.rs: Keep src/main.rs as the only Rust source",
    );
    Ok(())
}

#[test]
fn thin_binary_checks_explicit_targets_and_build_scripts() -> io::Result<()> {
    let workspace = fisher_workspace()?;
    let manifest_path = workspace.0.join(FISHER_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path)?;
    fs::write(workspace.0.join("crates/fisher/other.rs"), "fn main() {}")?;
    for package_options in ["", "autobins = false\n"] {
        fs::write(
            &manifest_path,
            format!(
                "{manifest}{package_options}\n[[bin]]\nname = \"other\"\npath = \"other.rs\"\n"
            ),
        )?;
        rejects(
            &workspace.run("thin-binary")?,
            "fisher must have exactly one binary target at src/main.rs",
        );
    }
    fs::write(
        &manifest_path,
        format!("{manifest}autobins = false\n\n[lib]\npath = \"other.rs\"\n"),
    )?;
    rejects(
        &workspace.run("thin-binary")?,
        "fisher must have exactly one binary target at src/main.rs",
    );

    fs::write(&manifest_path, &manifest)?;
    fs::write(workspace.0.join("crates/fisher/build.rs"), "fn main() {}")?;
    rejects(
        &workspace.run("thin-binary")?,
        "fisher cannot have a build script",
    );
    fs::write(&manifest_path, format!("{manifest}build = \"other.rs\"\n"))?;
    rejects(
        &workspace.run("thin-binary")?,
        "fisher cannot have a build script",
    );
    fs::write(&manifest_path, format!("{manifest}build = false\n"))?;
    accepts(&workspace.run("thin-binary")?);
    Ok(())
}

#[test]
fn thin_binary_reports_missing_metadata_and_source() -> io::Result<()> {
    let empty = Workspace::new()?;
    rejects(&empty.run("thin-binary")?, "cargo metadata failed");
    fs::remove_dir_all(&empty.0)?;
    rejects(
        &empty.run("thin-binary")?,
        &format!("{}:", empty.0.display()),
    );
    let workspace = fisher_workspace()?;
    let manifest_path = workspace.0.join(FISHER_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path)?;
    fs::write(&manifest_path, manifest.replace("fisher", "other"))?;
    rejects(
        &workspace.run("thin-binary")?,
        "could not find the fisher package",
    );
    fs::write(&manifest_path, &manifest)?;
    fs::write(workspace.0.join(FISHER_MAIN), [0xff])?;
    rejects(
        &workspace.run("thin-binary")?,
        "src/main.rs: stream did not contain valid UTF-8",
    );
    fs::remove_dir_all(workspace.0.join("crates/fisher/src"))?;
    fs::write(workspace.0.join("crates/fisher/other.rs"), "fn main() {}")?;
    fs::write(
        &manifest_path,
        format!("{manifest}\n[[bin]]\nname = \"other\"\npath = \"other.rs\"\n"),
    )?;
    rejects(&workspace.run("thin-binary")?, "crates/fisher/src:");
    Ok(())
}
