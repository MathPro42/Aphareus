//! Runs the real binary: the only tests that go through `main.rs`.

use std::process::{Command, Output, Stdio};

const BINARY: &str = env!("CARGO_BIN_EXE_aphareus");

/// Runs the binary with `args` and an empty, closed standard input.
fn run(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(BINARY)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("the binary runs")
}

#[test]
fn unknown_argument_fails() {
    let output = run(&["inconnue".as_ref()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"unknown command: inconnue\n");
}

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_an_unknown_command() {
    use std::os::unix::ffi::OsStrExt;
    let output = run(&[std::ffi::OsStr::from_bytes(b"perf\xFF")]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "unknown command: perf\u{FFFD}\n"
    );
}

#[test]
fn no_argument_runs_uci_until_the_input_ends() {
    let output = run(&[]);
    assert!(output.status.success(), "{output:?}");
}
