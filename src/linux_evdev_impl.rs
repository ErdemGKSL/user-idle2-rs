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

fn update_last_input_time() {
    LAST_INPUT_TIME.store(current_time_millis(), Ordering::Relaxed);
}

fn ensure_listener_started() {
    LISTENER_STARTED.get_or_init(|| {
        START_TIME.get_or_init(Instant::now);
        LAST_INPUT_TIME.store(current_time_millis(), Ordering::Relaxed);

        thread::spawn(|| {
            if let Err(e) = run_evdev_listener() {
                eprintln!("evdev listener error: {:?}", e);
            }
        });
    });
}

fn run_evdev_listener() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let devices = evdev::enumerate().collect::<Vec<_>>();

    for (path, device) in devices {
        if device.supported_events().contains(evdev::EventType::KEY)
            || device.supported_events().contains(evdev::EventType::RELATIVE)
            || device.supported_events().contains(evdev::EventType::ABSOLUTE)
        {
            thread::spawn(move || {
                if let Ok(mut dev) = evdev::Device::open(&path) {
                    loop {
                        match dev.fetch_events() {
                            Ok(events) => {
                                for _ in events {
                                    update_last_input_time();
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
            });
        }
    }

    loop {
        thread::sleep(Duration::from_secs(60));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    const TEST_SECS: u64 = 5;

    #[test]
    fn test_evdev_idle_time() {
        ensure_listener_started();
        sleep(Duration::from_millis(500));

        let initial_idle = get_idle_time().unwrap();
        println!("Initial idle after listener start: {:?}", initial_idle);

        println!("Sleeping for {} seconds... (don't touch mouse/keyboard)", TEST_SECS);
        sleep(Duration::from_secs(TEST_SECS));

        let idle = get_idle_time().unwrap();
        let idle_secs = idle.as_secs();
        println!("Idle after sleep: {:?} ({} seconds)", idle, idle_secs);
        println!("LAST_INPUT_TIME: {}", LAST_INPUT_TIME.load(Ordering::Relaxed));
        println!("current_time_millis: {}", current_time_millis());

        assert!(
            idle_secs >= TEST_SECS - 1,
            "Expected idle time >= {} seconds, got {} seconds. If you moved mouse/keyboard, this is expected.",
            TEST_SECS - 1,
            idle_secs
        );
    }
}
