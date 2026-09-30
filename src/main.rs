//! tock, a pomodoro timer in your terminal. The countdown lives in a small
//! pane under your prompt and leaves one line in scrollback when it ends.

mod duration;
mod timer;
mod view;

use std::io::{self, IsTerminal, Write};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use crossterm::event::{KeyCode, KeyModifiers};
use kiln::guard::TuiGuard;
use kiln::render::Renderable;
use kiln::terminal::{Tui, TuiEvent, restore_raw};

use timer::Timer;
use view::Finish;

const TICK: Duration = Duration::from_millis(200);
const DEFAULT_LENGTH: Duration = Duration::from_secs(25 * 60);
const LABEL: &str = "focus";

const USAGE: &str = "A pomodoro timer in your terminal.

Usage: tock [length] [options]

  length           25m, 90s, 1h30m, 1:30 or a number of minutes (default 25m)

Options:
  -h, --help           show this help
  -V, --version        show the version

Keys: q quit
";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("tock: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
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

    if !io::stdout().is_terminal() {
        bail!(
            "the timer draws on the terminal, so run tock without capturing or redirecting its output"
        );
    }
    let _guard = TuiGuard::install(restore_raw);
    let mut tui = Tui::new(8)?;
    let frames = tui.frame_requester();
    let timer = Timer::start(length.unwrap_or(DEFAULT_LENGTH), Instant::now());

    let finish = loop {
        let now = Instant::now();
        if timer.remaining(now).is_zero() {
            break Finish::Done { at: None };
        }
        let width = tui.width();
        let pane = view::pane(&timer, now, LABEL, width);
        tui.draw(pane.desired_height(width), |frame| {
            pane.render(frame.area(), frame.buffer_mut());
        })?;
        frames.schedule_in(TICK);

        match tui.next_event().await {
            TuiEvent::Key(key) => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Char('c') if ctrl => break Finish::Stopped,
                    KeyCode::Char('q') => break Finish::Stopped,
                    _ => {}
                }
            }
            TuiEvent::Resize => tui.resized()?,
            TuiEvent::Mouse(_) | TuiEvent::Paste(_) | TuiEvent::Draw => {}
        }
    };

    let now = Instant::now();
    let finish = match finish {
        Finish::Done { .. } => {
            let mut out = io::stdout();
            write!(out, "\x07")?;
            out.flush()?;
            let at = Command::new("date")
                .arg("+%H:%M")
                .output()
                .ok()
                .filter(|run| run.status.success())
                .and_then(|run| String::from_utf8(run.stdout).ok())
                .map(|at| at.trim().to_string())
                .filter(|at| !at.is_empty());
            Finish::Done { at }
        }
        Finish::Stopped => Finish::Stopped,
    };

    let width = tui.width() as usize;
    tui.insert_history(view::summary(&finish, &timer, now, LABEL, width))?;
    Ok(())
}
