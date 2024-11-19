use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};

use byteorder::{BigEndian, WriteBytesExt};
use classicube_helpers::tick::TickEventHandler;
use classicube_sys::*;

// 10 minutes afk delay
const INTERVAL: Duration = Duration::from_secs(9 * 60);

thread_local!(
    static TICK_HANDLER: RefCell<Option<TickEventHandler>> = Default::default();
);

thread_local!(
    static ENABLED: Cell<bool> = const { Cell::new(false) };
);
thread_local!(
    static NEXT: Cell<Instant> = Cell::new(Instant::now());
);

pub fn start() {
    NEXT.with(move |cell| {
        let now = Instant::now();
        cell.set(now + INTERVAL);
    });
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
    let now = Instant::now();
    if now >= NEXT.with(|c| c.get()) {
        NEXT.with(move |cell| {
            cell.set(now + INTERVAL);
        });

        if let Some(send_data) = unsafe { Server.SendData } {
            let local_player = unsafe { &*Entities.List[ENTITIES_SELF_ID as usize] };
            match create_packet(local_player) {
                Ok(data) => unsafe {
                    send_data(data.as_ptr(), data.len() as _);
                },
                e => {
                    eprintln!("create_packet: {e:#?}");
                }
            }
        } else {
            eprintln!("Server.SendData is None");
        }
    }
}

#[allow(non_snake_case)]
fn Math_Deg2Packed(x: f32) -> u8 {
    (x * 256.0 / 360.0) as u8
}

const OPCODE_ENTITY_TELEPORT: u8 = 8;

fn create_packet(local_player: &Entity) -> Result<Vec<u8>, std::io::Error> {
    let mut data = vec![];
    data.write_u8(OPCODE_ENTITY_TELEPORT)?;
    // u16 if ExtendedBlocks, else u8
    data.write_u16::<BigEndian>(ENTITIES_SELF_ID as _)?;

    // u32 if ExtEntityPositions, else u16
    data.write_u32::<BigEndian>((local_player.next.pos.x * 32.0) as u32)?;
    data.write_u32::<BigEndian>(((local_player.next.pos.y * 32.0) + 51.0) as u32)?;
    data.write_u32::<BigEndian>((local_player.next.pos.z * 32.0) as u32)?;

    data.write_u8(Math_Deg2Packed(local_player.Yaw))?;

    // 180 is upside-down which isn't normally possible
    // client will reset to normal angles after another tick
    data.write_u8(Math_Deg2Packed(180.0))?;
    Ok(data)
}
