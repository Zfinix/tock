//! tock, a pomodoro timer in your terminal. The countdown lives in a small
//! pane under your prompt and leaves one line in scrollback when it ends.

mod duration;
mod timer;
mod view;

use std::io::{self, IsTerminal, Write};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use crossterm::event::{KeyCode, KeyModifiers};
use kiln::guard::TuiGuard;
use kiln::render::Renderable;
use kiln::terminal::{Tui, TuiEvent, restore_raw};
use kiln::theme;

use timer::Timer;
use view::{Finish, Keys};

const TICK: Duration = Duration::from_millis(200);
const DEFAULT_LENGTH: Duration = Duration::from_secs(25 * 60);

const USAGE: &str = "A pomodoro timer in your terminal.

Usage: tock [length] [options]

  length           25m, 90s, 1h30m, 1:30 or a number of minutes (default 25m)

Options:
  -l, --label <text>   what the timer is for (default focus)
  -t, --theme <name>   colour theme (default ember, or $TOCK_THEME)
      --themes         list the themes
  -q, --quiet          leave nothing in scrollback when it ends
      --json           print one JSON line when it ends
  -h, --help           show this help
  -V, --version        show the version

Keys: space pause, r restart, + / - a minute, ? help, q quit
";

/// What tock leaves behind when the timer ends.
enum Output {
    Line,
    Quiet,
    Json,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("tock: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let mut length: Option<Duration> = None;
    let mut label = "focus".to_string();
    let mut theme_name = std::env::var("TOCK_THEME").unwrap_or_else(|_| "ember".into());
    let mut output = Output::Line;
    let names = || {
        theme::all()
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>()
    };

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-l" | "--label" => {
                label = args
                    .next()
                    .context("--label needs some text, like --label writing")?;
            }
            "-t" | "--theme" => {
                theme_name = args
                    .next()
                    .context("--theme needs a name, run tock --themes to see them")?;
            }
            "--themes" => {
                println!("{}", names().join("\n"));
                return Ok(());
            }
            "-q" | "--quiet" => output = Output::Quiet,
            "--json" => output = Output::Json,
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

    let Some(entry) = theme::named(&theme_name) else {
        bail!(
            "there is no theme called {theme_name}, try one of: {}",
            names().join(", ")
        );
    };
    theme::set(entry.theme);
    theme::settle();

    if !io::stdout().is_terminal() {
        bail!(
            "the timer draws on the terminal, so run tock without capturing or redirecting its output"
        );
    }
    let _guard = TuiGuard::install(restore_raw);
    let mut tui = Tui::new(8)?;
    let frames = tui.frame_requester();
    let mut timer = Timer::start(length.unwrap_or(DEFAULT_LENGTH), Instant::now());
    let mut keys = Keys::Footer;

    let finish = loop {
        let now = Instant::now();
        if timer.remaining(now).is_zero() {
            break Finish::Done { at: None };
        }
        let width = tui.width();
        let pane = view::pane(&timer, now, &label, keys, width);
        tui.draw(pane.desired_height(width), |frame| {
            pane.render(frame.area(), frame.buffer_mut());
        })?;
        if !timer.is_paused() {
            frames.schedule_in(TICK);
        }

        let event = tui.next_event().await;
        let now = Instant::now();
        match event {
            TuiEvent::Key(key) => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Char('c') if ctrl => break Finish::Stopped,
                    KeyCode::Char('q') => break Finish::Stopped,
                    KeyCode::Char(' ') => timer.toggle(now),
                    KeyCode::Char('r') => timer.restart(now),
                    KeyCode::Char('+' | '=') => timer.add_minute(),
                    KeyCode::Char('-' | '_') => timer.remove_minute(now),
                    KeyCode::Char('?') => keys = keys.toggle(),
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

    match output {
        Output::Line => {
            let width = tui.width() as usize;
            tui.insert_history(view::summary(&finish, &timer, now, &label, width))?;
        }
        Output::Quiet => {}
        Output::Json => {
            drop(tui);
            let completed = match finish {
                Finish::Done { .. } => true,
                Finish::Stopped => false,
            };
            let planned = timer.planned().as_secs();
            let elapsed = timer.elapsed(now).as_secs();
            println!(
                r#"{{"label":{},"planned_secs":{planned},"elapsed_secs":{elapsed},"completed":{completed}}}"#,
                serde_json::to_string(&label)?
            );
        }
    }
    Ok(())
}
