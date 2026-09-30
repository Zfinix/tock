//! tock, a pomodoro timer in your terminal.

use anyhow::{Result, bail};

const USAGE: &str = "A pomodoro timer in your terminal.

Usage: tock [options]

Options:
  -h, --help           show this help
  -V, --version        show the version
";

fn main() {
    if let Err(e) = run() {
        eprintln!("tock: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match std::env::args().nth(1).as_deref() {
        None | Some("-h" | "--help") => print!("{USAGE}"),
        Some("-V" | "--version") => println!("tock {}", env!("CARGO_PKG_VERSION")),
        Some(other) => bail!("there is no {other} option, run tock --help to see them"),
    }
    Ok(())
}
