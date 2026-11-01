use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (tx, rx) = mpsc::channel();
        let _tx = tx.clone();

        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if let Ok(true) = event::poll(Duration::from_millis(250)) {
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        if tx.send(Event::Input(key)).is_err() {
                            return;
                        }
                    }
                }
            })
        };

        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(tick_rate);
            })
        };

        Self {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_enum() {
        let tick: Event<KeyEvent> = Event::Tick;
        match tick {
            Event::Tick => assert!(true),
            _ => assert!(false),
        }
    }
}
