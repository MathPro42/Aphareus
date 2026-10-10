use std::io::Write;

/// Runs the command-line mode. `args` are the arguments without the program
/// name. Returns the process exit code: 0 on success, 1 on failure.
pub fn run(args: &[String], _out: &mut impl Write, err: &mut impl Write) -> u8 {
    // An error message that cannot be written has nowhere else to go.
    let _ = match args.first() {
        Some(command) => writeln!(err, "unknown command: {command}"),
        None => writeln!(err, "missing command"),
    };
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `(exit code, stdout, stderr)` of `run` on `args`.
    fn run_with(args: &[&str]) -> (u8, String, String) {
        let args: Vec<String> = args.iter().map(|&arg| arg.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn unknown_command_fails() {
        let (code, out, err) = run_with(&["inconnue", "x"]);
        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert_eq!(err, "unknown command: inconnue\n");
    }

    #[test]
    fn missing_command_fails() {
        let (code, out, err) = run_with(&[]);
        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert_eq!(err, "missing command\n");
    }
}
