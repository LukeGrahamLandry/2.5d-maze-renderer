extern crate core;

use crate::game::GameState;
use crate::player::Player;
use std::ffi::{c_int, c_void};
use std::num::NonZeroU32;
use std::thread;
use instant::Instant;
use winit::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::camera::SoftBufferRender;
use crate::material::Colour;

mod camera;
mod camera2d;
mod camera3d;
mod entity;
mod game;
mod light_cache;
mod lighting;
mod material;
mod mth;
mod player;
mod ray;
mod world;
mod world_gen;

fn main() {
    let event_loop = EventLoop::new();
    let builder = WindowBuilder::new().with_title("2.5dmazerenderer");
    let window = builder.build(&event_loop).unwrap();
    let mut game = GameState::new();
    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();


    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let needs_render_update = game.tick();
                if needs_render_update {
                    println!("Render");
                    let (width, height) = {
                        let size = window.inner_size();
                        (size.width, size.height)
                    };
                    surface
                        .resize(
                            NonZeroU32::new(width).unwrap(),
                            NonZeroU32::new(height).unwrap(),
                        )
                        .unwrap();

                    let mut buffer = SoftBufferRender {
                        colour: Colour::black(),
                        buffer: surface.buffer_mut().unwrap(),
                        width: width as usize,
                        height: height as usize,
                    };

                    camera::render_scene(&mut buffer, &game.world);
                    game.render_frame_counter += 1;

                    buffer.buffer.present().unwrap();
                }
            }

            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => match key {
                    VirtualKeyCode::Space => game.toggle_camera_mode(),
                    VirtualKeyCode::R => game.reset_world(),
                    VirtualKeyCode::W => game.keys.w = state == ElementState::Pressed,
                    VirtualKeyCode::A => game.keys.a = state == ElementState::Pressed,
                    VirtualKeyCode::S => game.keys.s = state == ElementState::Pressed,
                    VirtualKeyCode::D => game.keys.d = state == ElementState::Pressed,
                    VirtualKeyCode::F => game.keys.f = state == ElementState::Pressed,
                    VirtualKeyCode::Escape => control_flow.set_exit(),
                    _ => {}
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    if state == ElementState::Pressed {
                        Player::mouse_click(&mut game.world, button);
                    }
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta: (x, _) } => {
                    game.delta_mouse += x as f32;
                }
                _ => {}
            },

            _ => {}
        }
    });
}
