//! What tock draws: the live pane while the timer runs, and the one line it
//! leaves in scrollback when it ends.

use std::time::Instant;

use kiln::keys::{self, Binding};
use kiln::render::{Inset, Insets};
use kiln::{bigtext, cells, progress, theme};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::duration;
use crate::timer::Timer;

const BAR_MIN: usize = 8;
const BAR_MAX: usize = 32;

const HELP: [Binding; 5] = [
    ("space", "pause or resume"),
    ("r", "start over"),
    ("+ / -", "add or take off a minute"),
    ("q", "quit"),
    ("?", "close this help"),
];

/// Whether the pane ends in the one-row key hints or the full help list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keys {
    Footer,
    Help,
}

impl Keys {
    /// Switch between the footer and the help list.
    pub fn toggle(self) -> Self {
        match self {
            Keys::Footer => Keys::Help,
            Keys::Help => Keys::Footer,
        }
    }
}

/// How a run ended, for the line left in scrollback.
pub enum Finish {
    /// Time ran out; `at` is the wall clock time, when it could be read.
    Done { at: Option<String> },
    /// The user quit first.
    Stopped,
}

/// The live pane, eight rows so it fits the viewport of a 14-row terminal:
/// the big countdown (or the help list behind `?`), then the progress bar
/// with the label beside it, then the key hints.
pub fn pane(
    timer: &Timer,
    now: Instant,
    label: &str,
    keys: Keys,
    width: u16,
) -> Inset<Vec<Line<'static>>> {
    let t = theme::get();
    let remaining = timer.remaining(now);
    let clock = duration::clock(remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0));
    let planned = timer.planned().as_secs_f64();
    let ratio = match planned > 0.0 {
        true => timer.elapsed(now).as_secs_f64() / planned,
        false => 1.0,
    };

    let mut status = vec![Span::styled(
        label.to_string(),
        t.text_style().add_modifier(Modifier::BOLD),
    )];
    if timer.is_paused() {
        status.push(Span::styled(" · ", t.faint_style()));
        status.push(Span::styled(
            "paused",
            t.amber_style().add_modifier(Modifier::BOLD),
        ));
    }
    let detail = match keys {
        Keys::Footer => format!("{} total", duration::short(timer.planned().as_secs())),
        Keys::Help => format!("{clock} left"),
    };
    status.push(Span::styled(format!("  {detail}"), t.faint_style()));

    let status_width = status.iter().map(Span::width).sum::<usize>();
    let room = usize::from(width.saturating_sub(4)).saturating_sub(status_width + 2);
    let mut row = progress::bar(ratio, room.clamp(BAR_MIN, BAR_MAX));
    row.push(Span::raw("  "));
    row.extend(status);

    let mut lines = match keys {
        Keys::Footer => {
            let digits = match timer.is_paused() {
                true => t.dim_style(),
                false => t.accent_bold(),
            };
            bigtext::lines(&clock, digits)
        }
        Keys::Help => keys::help(&HELP),
    };
    lines.push(Line::from(""));
    lines.push(Line::from(row));
    match keys {
        Keys::Footer => {
            let space = match timer.is_paused() {
                true => ("space", "resume"),
                false => ("space", "pause"),
            };
            lines.push(Line::from(""));
            lines.push(keys::footer(&[
                space,
                ("r", "restart"),
                ("+/-", "minute"),
                ("?", "help"),
                ("q", "quit"),
            ]));
        }
        Keys::Help => {}
    }
    Inset::new(lines, Insets::tlbr(1, 2, 0, 2))
}

/// The line left in scrollback: `✓ focus done at 14:32 (25m)`, or
/// `stopped focus at 12:04 of 25:00` when the user quit early.
pub fn summary(
    finish: &Finish,
    timer: &Timer,
    now: Instant,
    label: &str,
    width: usize,
) -> Vec<Line<'static>> {
    let planned = timer.planned().as_secs();
    let at = match finish {
        Finish::Done { at } => at,
        Finish::Stopped => {
            let elapsed = duration::clock(timer.elapsed(now).as_secs());
            let text = format!(
                "stopped {label} at {elapsed} of {}",
                duration::clock(planned)
            );
            return cells::notice(&text, width);
        }
    };
    let t = theme::get();
    let when = at
        .as_ref()
        .map(|at| format!(" at {at}"))
        .unwrap_or_default();
    vec![Line::from(vec![
        Span::styled("✓ ", Style::default().fg(t.success)),
        Span::styled(format!("{label} done{when}"), t.text_style()),
        Span::styled(format!(" ({})", duration::short(planned)), t.dimmer_style()),
    ])]
}
