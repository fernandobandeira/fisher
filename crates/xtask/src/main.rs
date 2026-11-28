mod glossary;
mod interfaces;
mod thin_binary;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[expect(
    clippy::print_stderr,
    reason = "The xtask CLI reports failures to stderr"
)]
fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let (command, root) = match args.as_slice() {
        [command] => (command, Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")),
        [command, option, root] if option == "--root" => (command, PathBuf::from(root)),
        _ => return usage(),
    };
    let check = match command.to_str() {
        Some("glossary") => glossary::check,
        Some("interfaces") => interfaces::check,
        Some("thin-binary") => thin_binary::check,
        _ => return usage(),
    };
    match check(&root) {
        Ok(diagnostics) if diagnostics.is_empty() => return ExitCode::SUCCESS,
        Ok(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("{diagnostic}");
            }
        }
        Err(error) => eprintln!("{error}"),
    }
    ExitCode::FAILURE
}

#[expect(clippy::print_stderr, reason = "The xtask CLI prints usage to stderr")]
fn usage() -> ExitCode {
    eprintln!("Usage: cargo xtask <glossary|interfaces|thin-binary> [--root <path>]");
    ExitCode::FAILURE
}
