use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::Error;

static LAST_INPUT_TIME: AtomicU64 = AtomicU64::new(0);
static LISTENER_STARTED: OnceLock<()> = OnceLock::new();
static START_TIME: OnceLock<Instant> = OnceLock::new();

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

fn event_callback(_event: rdev::Event) {
    LAST_INPUT_TIME.store(current_time_millis(), Ordering::Relaxed);
}

fn ensure_listener_started() {
    LISTENER_STARTED.get_or_init(|| {
        START_TIME.get_or_init(Instant::now);
        LAST_INPUT_TIME.store(current_time_millis(), Ordering::Relaxed);

        thread::spawn(|| {
            if let Err(error) = rdev::listen(event_callback) {
                eprintln!("rdev listen error: {:?}", error);
            }
        });
    });
}

pub fn get_idle_time() -> Result<Duration, Error> {
    ensure_listener_started();

    let last_input = LAST_INPUT_TIME.load(Ordering::Relaxed);
    let now = current_time_millis();

    if last_input == 0 {
        let start = START_TIME
            .get()
            .ok_or_else(|| Error::new("Start time not initialized"))?;
        return Ok(start.elapsed());
    }

    if now >= last_input {
        Ok(Duration::from_millis(now - last_input))
    } else {
        Ok(Duration::ZERO)
    }
}
