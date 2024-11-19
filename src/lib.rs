mod timer;

use std::{os::raw::c_int, ptr};

use classicube_sys::*;

extern "C" fn init() {
    timer::init();
}

extern "C" fn free() {
    timer::free();
}

extern "C" fn reset() {
    timer::stop();
}

extern "C" fn on_new_map() {
    timer::stop();
}

extern "C" fn on_new_map_loaded() {
    if unsafe { Server.Name.to_string() }.starts_with("Not Awesome 2") {
        timer::start();
    }
}

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
    // Called when the game is being loaded.
    Init: Some(init),
    // Called when the component is being freed. (e.g. due to game being closed)
    Free: Some(free),
    // Called to reset the component's state. (e.g. reconnecting to server)
    Reset: Some(reset),
    // Called to update the component's state when the user begins loading a new map.
    OnNewMap: Some(on_new_map),
    // Called to update the component's state when the user has finished loading a new map.
    OnNewMapLoaded: Some(on_new_map_loaded),
    // Next component in linked list of components.
    next: ptr::null_mut(),
};
