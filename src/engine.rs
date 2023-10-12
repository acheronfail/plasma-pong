use std::num::NonZeroU32;
use std::time::Instant;

use glutin::prelude::*;
use glutin::surface::SwapInterval;
use glutin_winit::GlWindow;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};

use crate::cli::Cli;
use crate::fps::FpsCounter;
use crate::renderer::Renderer;
use crate::state::State;
use crate::window::create_window;

pub struct EngineContext<'a> {
    pub surface_dimensions: PhysicalSize<u32>,
    pub scale_factor: f32,
    pub state: &'a State,
    pub fps: f32,
}

pub struct Engine;

impl Engine {
    pub fn run(args: Cli) -> ! {
        // create window and setup gl context
        let (window, event_loop, gl_display, gl_surface, mut not_current_gl_context) =
            create_window();

        // engine state
        let mut state = State::new();
        let mut time = Instant::now();
        let mut paused = false;
        let mut fps_counter = FpsCounter::new();
        let mut surface_dimensions = window.inner_size();

        // gl state
        let mut gl_renderer = None;
        let mut gl_context = None;

        // surrender this thread to the window's event loop and run have it take over
        event_loop.run(move |event, _, control_flow| {
            // https://docs.rs/winit/latest/winit/index.html#event-handling
            control_flow.set_poll();

            macro_rules! set_pause {
                ($paused:expr) => {{
                    paused = $paused;
                    if paused {
                        control_flow.set_wait();
                    } else {
                        control_flow.set_poll();
                        time = Instant::now();
                    }
                }};
            }

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::Focused(focused) => {
                        set_pause!(!focused);
                    }
                    WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        // close and exit when escape is pressed
                        Some(VirtualKeyCode::Escape) => control_flow.set_exit(),
                        // pause waveform render when space is pressed
                        Some(VirtualKeyCode::Space) if input.state == ElementState::Pressed => {
                            set_pause!(!paused);
                        }

                        _ => {}
                    },
                    _ => (),
                },
                Event::Resumed => {
                    gl_context = not_current_gl_context
                        .take()
                        .unwrap()
                        .make_current(&gl_surface)
                        .ok();

                    // configure the swap interval to not wait for vsync
                    gl_surface
                        .set_swap_interval(
                            &gl_context.as_ref().unwrap(),
                            match args.vsync {
                                true => SwapInterval::Wait(NonZeroU32::MIN),
                                false => SwapInterval::DontWait,
                            },
                        )
                        .unwrap();

                    gl_renderer = Some(Renderer::new(&gl_display, &window).unwrap());
                }
                Event::MainEventsCleared => {
                    if paused {
                        return;
                    }

                    // state update
                    let delta_time = time.elapsed().as_secs_f32();
                    time = Instant::now();
                    state.update(delta_time);

                    // render
                    match (&gl_context, &mut gl_renderer) {
                        (Some(gl_context), Some(gl_renderer)) => {
                            let window_size = window.inner_size();
                            if surface_dimensions != window_size {
                                surface_dimensions = window_size;
                                window.resize_surface(&gl_surface, &gl_context);
                                unsafe {
                                    gl::Viewport(
                                        0,
                                        0,
                                        surface_dimensions.width as _,
                                        surface_dimensions.height as _,
                                    );
                                }
                            }

                            gl_renderer.draw(EngineContext {
                                surface_dimensions,
                                scale_factor: window.scale_factor() as f32,
                                state: &state,
                                fps: fps_counter.fps(),
                            });
                            gl_surface.swap_buffers(&gl_context).unwrap();
                        }
                        _ => {}
                    }

                    fps_counter.update();
                }
                Event::Suspended => {
                    let gl_context = gl_context.take().unwrap();
                    assert!(not_current_gl_context
                        .replace(gl_context.make_not_current().unwrap())
                        .is_none());
                }
                _ => {}
            }
        });
    }
}
