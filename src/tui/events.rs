use crossterm::event::{self, Event, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum AppEvent {
    Tick,
    Key(KeyEvent),
}

pub fn start_event_handler() -> mpsc::Receiver<AppEvent> {
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);

    thread::spawn(move || {
        let mut last_tick = std::time::Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    tx.send(AppEvent::Key(key)).unwrap();
                }
            }

            if last_tick.elapsed() >= tick_rate {
                tx.send(AppEvent::Tick).unwrap();
                last_tick = std::time::Instant::now();
            }
        }
    });

    rx
}
