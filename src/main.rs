mod renderer;
mod state;
mod uniform;

use std::num::NonZeroU32;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext, Version};
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use state::ApplicationState;
use winit::dpi::PhysicalPosition;
use winit::event::Event;
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::window::{Window, WindowBuilder};

const WINDOW_TITLE: &str = "plasma-pong";
const WINDOW_X: i32 = 2230;
const WINDOW_Y: i32 = 50;
const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

/// Mostly all taken from:
/// https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs
fn prepare_gl_window() -> (
    Window,
    EventLoop<()>,
    Display,
    Surface<WindowSurface>,
    Option<NotCurrentContext>,
) {
    let event_loop = EventLoopBuilder::new().build();
    let window_builder = WindowBuilder::new()
        .with_position(PhysicalPosition::new(WINDOW_X, WINDOW_Y))
        .with_title(WINDOW_TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

    let (window, gl_config) = DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&event_loop, ConfigTemplateBuilder::new(), |targets| {
            // Find the config with the maximum number of samples
            targets
                .reduce(|curr, next| {
                    let transparency_check = next.supports_transparency().unwrap_or(false)
                        && !curr.supports_transparency().unwrap_or(false);

                    if transparency_check || next.num_samples() > curr.num_samples() {
                        next
                    } else {
                        curr
                    }
                })
                .unwrap()
        })
        .unwrap();

    let window = window.expect("failed to create window");
    let gl_display = gl_config.display();

    let attrs = window.build_surface_attributes(<_>::default());
    let gl_surface = unsafe {
        gl_display
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let raw_window_handle = Some(window.raw_window_handle());

    // The context creation part. It can be created before surface and that's how
    // it's expected in multithreaded + multiwindow operation mode, since you
    // can send NotCurrentContext, but not Surface.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    // Finally, we can create the gl context
    let not_current_gl_context: Option<glutin::context::NotCurrentContext> = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    });

    (
        window,
        event_loop,
        gl_display,
        gl_surface,
        not_current_gl_context,
    )
}

pub fn main() -> ! {
    // create window and setup gl context
    let (window, event_loop, gl_display, gl_surface, mut not_current_gl_context) =
        prepare_gl_window();

    // don't allow the window to be dropped, since that closes the window
    let mut state = ApplicationState::new(window);
    let enable_vsync = false;

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
                        match enable_vsync {
                            true => SwapInterval::Wait(NonZeroU32::MIN),
                            false => SwapInterval::DontWait,
                        },
                    )
                    .unwrap();

                gl_renderer = Some(renderer::Renderer::new(&gl_display, &state.window).unwrap());
                state.gl_context = Some(gl_context);
            }
            Event::MainEventsCleared => {
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

                state.after_update();
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
