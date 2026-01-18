mod linux_dbus_impl;
mod linux_x11_impl;

#[cfg(feature = "rdev")]
mod linux_rdev_impl;

use linux_dbus_impl::{
    get_idle_time_from_mutter, get_idle_time_from_screensaver,
};
use linux_x11_impl::get_idle_time as get_idle_time_from_x11;

#[cfg(feature = "rdev")]
use linux_rdev_impl::get_idle_time as get_idle_time_from_rdev;

use crate::Error;
use std::time::Duration;

pub fn get_idle_time() -> Result<Duration, Error> {
    // Try Mutter first (GNOME on Wayland/X11)
    match get_idle_time_from_mutter() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }

    // Try X11 (works on X11 sessions and XWayland)
    match get_idle_time_from_x11() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }

    // Try screensaver DBus interfaces
    match get_idle_time_from_screensaver() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }

    // If rdev feature is enabled, try rdev as last resort
    // This works on Wayland compositors like Hyprland
    #[cfg(feature = "rdev")]
    {
        match get_idle_time_from_rdev() {
            Ok(duration) => return Ok(duration),
            Err(error) => return Err(error),
        }
    }

    #[cfg(not(feature = "rdev"))]
    {
        Err(Error::new(
            "No idle time provider available. Consider enabling the 'rdev' feature for Wayland support.",
        ))
    }
}
