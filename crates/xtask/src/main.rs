mod glossary;
mod interfaces;

use std::env;
use std::path::Path;
use std::process::ExitCode;

#[expect(
    clippy::print_stderr,
    reason = "The xtask CLI reports failures to stderr"
)]
fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let check = match (args.next(), args.next()) {
        (Some(command), None) if command == "glossary" => glossary::check,
        (Some(command), None) if command == "interfaces" => interfaces::check,
        _ => {
            eprintln!("Usage: cargo xtask <glossary|interfaces>");
            return ExitCode::FAILURE;
        }
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
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
