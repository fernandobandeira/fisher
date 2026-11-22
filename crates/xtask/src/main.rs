mod glossary;

use std::env;
use std::path::Path;
use std::process::ExitCode;

#[expect(
    clippy::print_stderr,
    reason = "The xtask CLI reports failures to stderr"
)]
fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    if args.next().is_some_and(|arg| arg == "glossary") && args.next().is_none() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        match glossary::check(&root) {
            Ok(diagnostics) if diagnostics.is_empty() => return ExitCode::SUCCESS,
            Ok(diagnostics) => {
                for diagnostic in diagnostics {
                    eprintln!("{diagnostic}");
                }
            }
            Err(error) => eprintln!("{error}"),
        }
    } else {
        eprintln!("Usage: cargo xtask glossary");
    }
    ExitCode::FAILURE
}
