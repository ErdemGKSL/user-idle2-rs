# user-idle2

This is a fork of [user-idle-rs](https://github.com/olback/user-idle-rs), which had been idle for a while and did not support Wayland.

| OS              | Supported |
| --------------- | --------- |
| Linux           | ✔️*       |
| Windows         | ✔️        |
| MacOS           | ✔️        |

\* The Linux implementation will do the following:
1. Try to get the idle time from Mutter via DBus. This should work on GNOME desktops with Wayland or X11.
2. Try to get the idle time from X11. This will not work on Wayland.
3. Try to get the screensaver's idle time via DBus. Note that the screensaver may report a value of 0ns when it's not active.
4. **With the `rdev` feature enabled:** Fall back to using the `rdev` crate for input event tracking. This works on Wayland compositors like **Hyprland**, **Sway**, and others.

## Features

### `rdev` (optional)

Enable the `rdev` feature to support Wayland compositors that don't expose idle time via DBus (like Hyprland):

```toml
[dependencies]
user-idle2 = { version = "0.6", features = ["rdev"] }
```

The `rdev` implementation works by running a background thread that listens for all keyboard and mouse events, tracking the last input time. This approach works on any Linux desktop environment, including:
- Hyprland
- Sway
- wlroots-based compositors
- Any X11 or Wayland environment

**Note:** The `rdev` crate may require additional permissions on some systems to capture global input events.

### Example

```rust
use user_idle2::UserIdle;

let idle = UserIdle::get_time().unwrap();

let idle_seconds = idle.as_seconds();
let idle_minutes = idle.as_minutes();
```

Check the documentation for more methods.
