use std::time::Duration;

use dbus::blocking::Connection;

use crate::error::Error;

// Based on https://bitbucket.org/pidgin/main/src/default/pidgin/gtkidle.c

const SCREENSAVERS: &[&[&str]] = &[
    &[
        "org.freedesktop.ScreenSaver",
        "/org/freedesktop/ScreenSaver",
    ],
    &["org.gnome.ScreenSaver", "/org/gnome/ScreenSaver"],
    &["org.kde.ScreenSaver", "/org/kde/ScreenSaver"],
];

pub fn get_idle_time_from_mutter() -> Result<Duration, Error> {
    let Ok(conn) = Connection::new_session() else {
        return Err(Error::new("Cannot connect to DBus"));
    };

    let proxy = conn.with_proxy(
        "org.gnome.Mutter.IdleMonitor",
        "/org/gnome/Mutter/IdleMonitor/Core",
        Duration::from_millis(5000),
    );

    let (time,): (u64,) = match proxy.method_call(
        "org.gnome.Mutter.IdleMonitor",
        "GetIdletime",
        (),
    ) {
        Ok(v) => v,
        Err(_) => {
            return Err(Error::new("Mutter not available"));
        }
    };
    return Ok(Duration::from_millis(u64::from(time)));
}

pub fn get_idle_time_from_screensaver() -> Result<Duration, Error> {
    let Ok(conn) = Connection::new_session() else {
        return Err(Error::new("Cannot connect to DBus"));
    };

    for screensaver in SCREENSAVERS {
        let proxy = conn.with_proxy(
            screensaver[0],
            screensaver[1],
            Duration::from_millis(5000),
        );

        let (time,): (u32,) =
            match proxy.method_call(screensaver[0], "GetActiveTime", ()) {
                Ok(v) => v,
                Err(_) => continue,
            };

        // freedesktop seems to return the time in milliseconds??
        if screensaver[0] == "org.freedesktop.ScreenSaver" {
            return Ok(Duration::from_millis(u64::from(time)));
        }

        return Ok(Duration::from_secs(u64::from(time)));
    }

    Err(Error::new("No screensaver available"))
}
