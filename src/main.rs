extern crate sdl2;
extern crate core;

use std::ffi::{c_int, c_void};
use std::thread;

mod player;
mod camera3d;
mod mth;
mod world_gen;
mod ray;
mod material;
mod camera2d;
mod camera;
mod light_cache;
mod world;
mod lighting;
mod entity;
mod game;



fn main() -> Result<(), String> {
    run()?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn run() -> Result<(), String> {
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn run() -> Result<(), String> {
    setup_mainloop(-1, 1, move || {});
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[allow(non_camel_case_types)]
type em_callback_func = unsafe extern "C" fn(context: *mut c_void);

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn emscripten_set_main_loop_arg(
        func: em_callback_func,
        arg: *mut c_void,
        fps: c_int,
        simulate_infinite_loop: c_int,
    );
}

#[cfg(target_arch = "wasm32")]
pub fn setup_mainloop<F: FnMut() + 'static>(
    fps: c_int,
    simulate_infinite_loop: c_int,
    callback: F,
) {
    let on_the_heap = Box::new(callback);
    let leaked_pointer = Box::into_raw(on_the_heap);
    let untyped_pointer = leaked_pointer as *mut c_void;

    unsafe {
        emscripten_set_main_loop_arg(wrapper::<F>, untyped_pointer, fps, simulate_infinite_loop)
    }

    extern "C" fn wrapper<F: FnMut() + 'static>(untyped_pointer: *mut c_void) {
        let leaked_pointer = untyped_pointer as *mut F;
        let callback_ref = unsafe { &mut *leaked_pointer };
        callback_ref()
    }
}
