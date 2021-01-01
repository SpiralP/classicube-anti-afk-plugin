use classicube_helpers::tick::TickEventHandler;
use classicube_sys::*;
use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};

const INTERVAL: Duration = Duration::from_secs(14 * 10);

thread_local!(
    static TICK_HANDLER: RefCell<Option<TickEventHandler>> = Default::default();
);

thread_local!(
    static ENABLED: Cell<bool> = Cell::new(false);
);

pub fn start() {
    ENABLED.with(|cell| cell.set(true));
}
pub fn stop() {
    ENABLED.with(|cell| cell.set(false));
}

pub fn init() {
    if unsafe { Server.IsSinglePlayer } != 0 {
        return;
    }

    TICK_HANDLER.with(|cell| {
        let mut tick_handler = TickEventHandler::new();

        tick_handler.on(move |_task| {
            if ENABLED.with(|c| c.get()) {
                check();
            }
        });

        *cell.borrow_mut() = Some(tick_handler);
    });
}

pub fn free() {
    TICK_HANDLER.with(|cell| drop(cell.borrow_mut().take()));
}

fn check() {
    thread_local!(
        static NEXT: Cell<Instant> = Cell::new(Instant::now());
    );

    let now = Instant::now();
    if now >= NEXT.with(|c| c.get()) {
        NEXT.with(move |cell| {
            cell.set(now + INTERVAL);
        });

        if let Some(send_position) = unsafe { Server.SendPosition } {
            let local_player = unsafe { &*Entities.List[ENTITIES_SELF_ID as usize] };

            unsafe {
                // 180 is upside-down which isn't normally possible
                // client will reset to normal angles after another tick
                send_position(local_player.Position, local_player.Yaw, 180.0);
            }
        }
    }
}
