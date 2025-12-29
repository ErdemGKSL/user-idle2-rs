use std::sync::{LazyLock, RwLock};

use crate::error::Error;
use chrono::{DateTime, Local, Utc};
use wayland_client::{
    globals::GlobalListContents,
    protocol::{
        wl_registry,
        wl_seat::{self, WlSeat},
    },
    Connection, Dispatch, EventQueue, QueueHandle,
};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{self, ExtIdleNotificationV1},
    ext_idle_notifier_v1::ExtIdleNotifierV1,
};

static IDLE_STATE: LazyLock<RwLock<WaylandIdleState>> =
    LazyLock::new(|| RwLock::new(WaylandIdleState::default()));

struct WaylandIdleState {
    last_fetch: DateTime<Utc>,
    last_change: DateTime<Utc>,
    was_active: bool,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents>
    for WaylandIdleState
{
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandIdleState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotifierV1, ()> for WaylandIdleState {
    fn event(
        _state: &mut Self,
        _proxy: &ExtIdleNotifierV1,
        _event: <ExtIdleNotifierV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotificationV1, ()> for WaylandIdleState {
    fn event(
        state: &mut Self,
        _proxy: &ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!(
            "Wayland idle event: {}",
            match event {
                ext_idle_notification_v1::Event::Idled => "idled",
                ext_idle_notification_v1::Event::Resumed => "resumed",
                _ => "??",
            }
        );
        let old_active_state = state.was_active;
        let new_active_state = match event {
            ext_idle_notification_v1::Event::Idled => false,
            ext_idle_notification_v1::Event::Resumed => true,
            _ => panic!("Unknown idle notification event variant encountered"),
        };
        let now = Local::now().to_utc();
        if old_active_state != new_active_state {
            state.was_active = new_active_state;
            state.last_change = now;
        }
        state.last_fetch = now;
    }
}

impl WaylandIdleState {
    fn default() -> Self {
        let now = Local::now().to_utc();

        WaylandIdleState {
            last_fetch: now,
            last_change: now,
            was_active: true,
        }
    }
}

pub(crate) fn get_idle_time_from_wayland_compositor(
) -> Result<std::time::Duration, Error> {
    let Ok(conn) = Connection::connect_to_env() else {
        return Err(Error::new("Connection failed"));
    };
    let mut event_queue: EventQueue<WaylandIdleState> = conn.new_event_queue();
    let event_queue_handle = event_queue.handle();

    let (globals, _) = wayland_client::globals::registry_queue_init::<
        WaylandIdleState,
    >(&conn)
    .map_err(|e| Error::new(format!("Failed to init registry: {:?}", e)))?;

    let Ok(seat): Result<WlSeat, _> =
        globals.bind(&event_queue_handle, 1..=10, ())
    else {
        return Err(Error::new("Could not get Wayland seat"));
    };

    let Ok(notifier): Result<ExtIdleNotifierV1, _> =
        globals.bind(&event_queue_handle, 1..=2, ())
    else {
        return Err(Error::new("Could not get Wayland notifier"));
    };
    notifier.get_input_idle_notification(0, &seat, &event_queue_handle, ());

    let mut idle_state = IDLE_STATE
        .write()
        .expect("Could not get lock on idle state");
    event_queue.roundtrip(&mut idle_state).map_err(|e| {
        Error::new(format!("Failed to dispatch events: {:?}", e))
    })?;

    match idle_state.was_active {
        true => Ok(std::time::Duration::from_secs(0)),
        false => Ok(Local::now()
            .to_utc()
            .signed_duration_since(idle_state.last_change)
            .to_std()
            .expect("Idle time conversion failed")),
    }
}
