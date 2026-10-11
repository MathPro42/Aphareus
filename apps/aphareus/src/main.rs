use std::env;
use std::io::{self, BufReader};
use std::process::ExitCode;

use aphareus::{cli, uci};

fn main() -> ExitCode {
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    if args.is_empty() {
        uci::run(BufReader::new(io::stdin()), io::stdout());
        ExitCode::SUCCESS
    } else {
        ExitCode::from(cli::run(&args, &mut io::stdout(), &mut io::stderr()))
    }
}
