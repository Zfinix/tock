use super::*;

const SEC: Duration = Duration::from_secs(1);

fn five_minutes(t0: Instant) -> Timer {
    Timer::start(5 * MINUTE, t0)
}

#[test]
fn timer_counts_down() {
    let t0 = Instant::now();
    let timer = five_minutes(t0);
    assert_eq!(timer.remaining(t0 + 30 * SEC), 270 * SEC);
}

#[test]
fn timer_stops_at_zero() {
    let t0 = Instant::now();
    let timer = five_minutes(t0);
    assert_eq!(timer.remaining(t0 + 10 * MINUTE), Duration::ZERO);
    assert_eq!(timer.elapsed(t0 + 10 * MINUTE), 5 * MINUTE);
}

#[test]
fn timer_pause_freezes_remaining() {
    let t0 = Instant::now();
    let mut timer = five_minutes(t0);
    timer.toggle(t0 + 10 * SEC);
    assert_eq!(
        (timer.is_paused(), timer.remaining(t0 + 2 * MINUTE)),
        (true, 290 * SEC)
    );
}

#[test]
fn timer_resume_continues_from_pause() {
    let t0 = Instant::now();
    let mut timer = five_minutes(t0);
    timer.toggle(t0 + 10 * SEC);
    timer.toggle(t0 + MINUTE);
    assert_eq!(
        (timer.is_paused(), timer.remaining(t0 + MINUTE + 5 * SEC)),
        (false, 285 * SEC)
    );
}

#[test]
fn timer_restart_resets() {
    let t0 = Instant::now();
    let mut timer = five_minutes(t0);
    timer.toggle(t0 + MINUTE);
    timer.restart(t0 + 2 * MINUTE);
    assert_eq!(timer, five_minutes(t0 + 2 * MINUTE));
}

#[test]
fn timer_add_minute() {
    let t0 = Instant::now();
    let mut timer = five_minutes(t0);
    timer.add_minute();
    assert_eq!(timer.remaining(t0 + MINUTE), 5 * MINUTE);
}

#[test]
fn timer_remove_minute() {
    let t0 = Instant::now();
    let mut timer = five_minutes(t0);
    timer.remove_minute(t0);
    assert_eq!(timer.planned(), 4 * MINUTE);
}

#[test]
fn timer_remove_minute_clamps_at_zero() {
    let t0 = Instant::now();
    let mut timer = five_minutes(t0);
    let now = t0 + 270 * SEC;
    timer.remove_minute(now);
    assert_eq!(
        (timer.planned(), timer.remaining(now)),
        (270 * SEC, Duration::ZERO)
    );
}
