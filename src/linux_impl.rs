mod linux_dbus_impl;
mod linux_x11_impl;

#[cfg(feature = "evdev")]
mod linux_evdev_impl;

use linux_dbus_impl::{
    get_idle_time_from_mutter, get_idle_time_from_screensaver,
};
use linux_x11_impl::get_idle_time as get_idle_time_from_x11;

#[cfg(feature = "evdev")]
use linux_evdev_impl::get_idle_time as get_idle_time_from_evdev;

use crate::Error;
use std::time::Duration;

pub fn get_idle_time() -> Result<Duration, Error> {
    match get_idle_time_from_mutter() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }

    match get_idle_time_from_x11() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }

    match get_idle_time_from_screensaver() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }

    #[cfg(feature = "evdev")]
    {
        match get_idle_time_from_evdev() {
            Ok(duration) => return Ok(duration),
            Err(error) => return Err(error),
        }
    }

    #[cfg(not(feature = "evdev"))]
    {
        Err(Error::new(
            "No idle time provider available. Consider enabling the 'evdev' feature for Wayland support.",
        ))
    }
}
