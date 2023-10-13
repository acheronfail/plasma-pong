use std::num::NonZeroU32;
use std::time::Instant;

use anyhow::Result;
use glam::Vec2;
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::GlWindow;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};

use crate::cli::Cli;
use crate::fps::FpsCounter;
use crate::renderer::Renderer;
use crate::state::{Rect, State};
use crate::window::create_window;

pub enum Interaction {
    Repel(Vec2),
    Suck(Vec2),
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
        let mut state = State::new();

        // create window and setup gl context
        let (window, event_loop, gl_display, gl_surface, mut not_current_gl_context) =
            create_window(LogicalSize::new(
                (state.bounding_box.w * State::PIXELS_PER_UNIT) as u32,
                (state.bounding_box.h * State::PIXELS_PER_UNIT) as u32,
            ));

        // engine state
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
                            let pos = map_window_pos_to_world_pos(
                                surface_dimensions,
                                cursor_pos,
                                state.bounding_box,
                            );
                            match cursor_button {
                                MouseButton::Right => Interaction::Suck(pos),
                                _ => Interaction::Repel(pos),
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

fn map_window_pos_to_world_pos(
    window_size: PhysicalSize<u32>,
    window_position: PhysicalPosition<f64>,
    bounding_box: Rect,
) -> Vec2 {
    Vec2::new(
        (bounding_box.x + (window_position.x as f32 / window_size.width as f32) * bounding_box.w)
            .clamp(bounding_box.left(), bounding_box.right()),
        (bounding_box.y + (window_position.y as f32 / window_size.height as f32) * bounding_box.h)
            .clamp(bounding_box.top(), bounding_box.bottom()),
    )
}
