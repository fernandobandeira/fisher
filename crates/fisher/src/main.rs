use std::{env, io};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if bench::run_if_requested(env::args_os().nth(1).as_deref(), &mut output)? {
        return Ok(());
    }

    uci::run_session(io::stdin(), output, deepening::search)
}
