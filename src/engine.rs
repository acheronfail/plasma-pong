use std::num::NonZeroU32;
use std::time::Instant;

use anyhow::Result;
use glam::Vec2;
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::GlWindow;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};

use crate::cli::Cli;
use crate::fps::FpsCounter;
use crate::renderer::Renderer;
use crate::state::State;
use crate::window::create_window;

pub enum Interaction {
    Repel { pos: Vec2, radius: f32 },
    Suck { pos: Vec2, radius: f32 },
}

impl Interaction {
    const RADIUS: f32 = 0.25;
}

pub struct EngineContext<'a> {
    pub surface_dimensions: PhysicalSize<u32>,
    pub scale_factor: f32,
    pub state: &'a State,
    pub vsync: bool,
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
        let mut cursor_pos = PhysicalPosition::default();
        let mut cursor_button = MouseButton::Left;
        let mut cursor_pressed = false;
        let mut vsync = args.vsync;

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
                        // toggle vsync
                        Some(VirtualKeyCode::V) if input.state == ElementState::Pressed => {
                            vsync = !vsync;
                            set_vsync(&gl_surface, gl_context.as_ref().unwrap(), vsync).unwrap();
                        }

                        _ => {}
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_pos = position;
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        cursor_pressed = matches!(state, ElementState::Pressed);
                        cursor_button = button;
                    }
                    _ => (),
                },
                Event::Resumed => {
                    gl_context = not_current_gl_context
                        .take()
                        .unwrap()
                        .make_current(&gl_surface)
                        .ok();

                    // configure the swap interval to not wait for vsync
                    set_vsync(&gl_surface, gl_context.as_ref().unwrap(), vsync).unwrap();

                    gl_renderer = Some(Renderer::new(&gl_display, &window).unwrap());
                }
                Event::MainEventsCleared => {
                    if paused {
                        return;
                    }

                    // state update
                    let delta_time = time.elapsed().as_secs_f32();
                    time = Instant::now();
                    state.update(
                        delta_time,
                        cursor_pressed.then(|| {
                            let radius = Interaction::RADIUS;
                            let pos = map_window_pos_to_gl_pos(surface_dimensions, cursor_pos);
                            match cursor_button {
                                MouseButton::Right => Interaction::Suck { pos, radius },
                                _ => Interaction::Repel { pos, radius },
                            }
                        }),
                    );

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
                                vsync,
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

fn set_vsync(
    gl_surface: &Surface<WindowSurface>,
    gl_context: &PossiblyCurrentContext,
    vsync: bool,
) -> Result<()> {
    gl_surface.set_swap_interval(
        &gl_context,
        match vsync {
            true => SwapInterval::Wait(NonZeroU32::MIN),
            false => SwapInterval::DontWait,
        },
    )?;

    Ok(())
}

fn map_window_pos_to_gl_pos(
    window_dimensions: PhysicalSize<u32>,
    position: PhysicalPosition<f64>,
) -> Vec2 {
    let w = window_dimensions.width as f32;
    let h = window_dimensions.height as f32;

    // convert to normalized device coordinates
    let x_ndc = 2.0 * (position.x as f32 / w) - 1.0;
    let y_ndc = 1.0 - 2.0 * (position.y as f32 / h);

    // assuming the viewport is the same size as the window
    Vec2::new(x_ndc, y_ndc).clamp(Vec2::NEG_ONE, Vec2::ONE)
}
