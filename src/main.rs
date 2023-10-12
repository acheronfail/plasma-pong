mod cli;
mod fps;
mod renderer;
mod state;
mod window;

use std::num::NonZeroU32;
use std::time::Instant;

use clap::Parser;
use cli::Cli;
use glutin::prelude::*;
use glutin::surface::SwapInterval;
use glutin_winit::GlWindow;
use state::ApplicationState;
use window::create_window;
use winit::event::Event;

pub fn main() -> ! {
    let args = Cli::parse();

    // create window and setup gl context
    let (window, event_loop, gl_display, gl_surface, mut not_current_gl_context) = create_window();

    // don't allow the window to be dropped, since that closes the window
    let mut state = ApplicationState::new(window);
    let mut time = Instant::now();

    // surrender this thread to the window's event loop and run have it take over
    let mut gl_renderer = None;
    event_loop.run(move |event, _, control_flow| {
        // https://docs.rs/winit/latest/winit/index.html#event-handling
        control_flow.set_poll();

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => state.window_event(event, control_flow),
            Event::Resumed => {
                let gl_context = not_current_gl_context
                    .take()
                    .unwrap()
                    .make_current(&gl_surface)
                    .unwrap();

                // configure the swap interval to not wait for vsync
                gl_surface
                    .set_swap_interval(
                        &gl_context,
                        match args.vsync {
                            true => SwapInterval::Wait(NonZeroU32::MIN),
                            false => SwapInterval::DontWait,
                        },
                    )
                    .unwrap();

                gl_renderer = Some(renderer::Renderer::new(&gl_display, &state.window).unwrap());
                state.gl_context = Some(gl_context);
            }
            Event::MainEventsCleared => {
                // state update
                let delta_time = time.elapsed().as_secs_f32();
                time = Instant::now();
                state.update(delta_time);

                // render
                match (&state.gl_context, &mut gl_renderer) {
                    (Some(gl_context), Some(gl_renderer)) => {
                        let window_size = state.window.inner_size();
                        if state.surface_dimensions != window_size {
                            state.surface_dimensions = window_size;
                            state.window.resize_surface(&gl_surface, &gl_context);
                            unsafe {
                                gl::Viewport(
                                    0,
                                    0,
                                    state.surface_dimensions.width as _,
                                    state.surface_dimensions.height as _,
                                );
                            }
                        }

                        gl_renderer.draw(&state);
                        gl_surface.swap_buffers(&gl_context).unwrap();
                    }
                    _ => {}
                }
            }
            Event::Suspended => {
                let gl_context = state.gl_context.take().unwrap();
                assert!(not_current_gl_context
                    .replace(gl_context.make_not_current().unwrap())
                    .is_none());
            }
            _ => {}
        }
    });
}
