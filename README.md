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
4. **With the `evdev` feature enabled:** Fall back to using evdev for input event tracking. This works on Wayland compositors like **Hyprland**, **Sway**, and others.

## Features

### `evdev` (optional)

Enable the `evdev` feature to support Wayland compositors that don't expose idle time via DBus (like Hyprland):

```toml
[dependencies]
user-idle2 = { version = "0.6", features = ["evdev"] }
```

The `evdev` implementation works by running background threads that monitor all input devices for keyboard and mouse events, tracking the last input time. This approach works on any Linux desktop environment, including:
- Hyprland
- Sway
- wlroots-based compositors
- Any X11 or Wayland environment

**Note:** On Linux, the user must be in the `input` group to read input events:
```bash
sudo usermod -aG input $USER
# Log out and back in for the change to take effect
```

### Example

```rust
use user_idle2::UserIdle;

let idle = UserIdle::get_time().unwrap();

let idle_seconds = idle.as_seconds();
let idle_minutes = idle.as_minutes();
```

Check the documentation for more methods.
