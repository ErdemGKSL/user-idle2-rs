mod linux_dbus;
mod linux_wayland;
mod linux_x11;

use linux_dbus::{get_idle_time_from_mutter, get_idle_time_from_screensaver};
use linux_x11::get_idle_time as get_idle_time_from_x11;

use crate::{
    idle::linux_wayland::get_idle_time_from_wayland_compositor, Error,
};
use std::time::Duration;

pub fn get_idle_time() -> Result<Duration, Error> {
    match get_idle_time_from_mutter() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }
    match get_idle_time_from_wayland_compositor() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }
    match get_idle_time_from_x11() {
        Ok(duration) => return Ok(duration),
        Err(_) => {}
    }
    match get_idle_time_from_screensaver() {
        Ok(duration) => return Ok(duration),
        Err(error) => Err(error),
    }
}
