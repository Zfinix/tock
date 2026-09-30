//! tock, a pomodoro timer in your terminal.

mod duration;

use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};

const TICK: Duration = Duration::from_millis(200);
const DEFAULT_LENGTH: Duration = Duration::from_secs(25 * 60);

const USAGE: &str = "A pomodoro timer in your terminal.

Usage: tock [length] [options]

  length           25m, 90s, 1h30m, 1:30 or a number of minutes (default 25m)

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
    let mut length: Option<Duration> = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            "-V" | "--version" => {
                println!("tock {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            flag if flag.starts_with('-') => {
                bail!("there is no {flag} option, run tock --help to see them")
            }
            text => {
                if length.is_some() {
                    bail!("tock takes one length, but got another: {text}");
                }
                length = Some(duration::parse(text)?);
            }
        }
    }

    let length = length.unwrap_or(DEFAULT_LENGTH);
    let started = Instant::now();
    let mut out = io::stdout();
    loop {
        let remaining = length.saturating_sub(started.elapsed());
        let secs = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
        write!(out, "\r{}  ", duration::clock(secs))?;
        out.flush()?;
        if remaining.is_zero() {
            break;
        }
        thread::sleep(TICK.min(remaining));
    }
    writeln!(
        out,
        "\r\x07focus done ({})",
        duration::short(length.as_secs())
    )?;
    Ok(())
}
