use super::*;

const SEC: Duration = Duration::from_secs(1);
const MINUTE: Duration = Duration::from_secs(60);

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
