use super::*;

const DEBOUNCE: Duration = Duration::from_millis(300);
const MARGIN: Duration = Duration::from_millis(500);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn stops_once_per_burst_of_events() {
    let mut s = Scheduler::new(DEBOUNCE, MARGIN);
    let t = Instant::now();
    assert_eq!(s.on_event("a.mp3".into(), t), Some(Action::StopLoop));
    assert_eq!(s.on_event("a.mp3".into(), t + ms(50)), None);
}

#[test]
fn debounce_restarts_on_each_event() {
    let mut s = Scheduler::new(DEBOUNCE, MARGIN);
    let t = Instant::now();
    s.on_event("a.mp3".into(), t);
    s.on_event("a.mp3".into(), t + ms(200));
    assert_eq!(s.poll(t + ms(400)), None);
    assert_eq!(s.poll(t + ms(500)), Some(Action::Measure("a.mp3".into())));
}

#[test]
fn starts_after_duration_plus_margin_from_last_event() {
    let mut s = Scheduler::new(DEBOUNCE, MARGIN);
    let t = Instant::now();
    s.on_event("a.mp3".into(), t);
    s.poll(t + DEBOUNCE);
    s.schedule(ms(2000));
    assert_eq!(s.next_deadline(), Some(t + ms(2500)));
    assert_eq!(s.poll(t + ms(2499)), None);
    assert_eq!(
        s.poll(t + ms(2500)),
        Some(Action::StartLoop("a.mp3".into()))
    );
    assert_eq!(s.next_deadline(), None);
}

#[test]
fn newer_file_while_waiting_replaces_the_pending_one() {
    let mut s = Scheduler::new(DEBOUNCE, MARGIN);
    let t = Instant::now();
    s.on_event("a.mp3".into(), t);
    s.poll(t + DEBOUNCE);
    s.schedule(ms(5000));
    assert_eq!(
        s.on_event("b.mp3".into(), t + ms(1000)),
        Some(Action::StopLoop)
    );
    assert_eq!(s.poll(t + ms(1300)), Some(Action::Measure("b.mp3".into())));
}
