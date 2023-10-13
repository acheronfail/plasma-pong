use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext, Version};
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::window::{Window, WindowBuilder};

const WINDOW_TITLE: &str = "plasma-pong";
const WINDOW_X: i32 = 2000;
const WINDOW_Y: i32 = 50;

/// Mostly all taken from:
/// https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs
pub fn create_window(
    window_size: LogicalSize<u32>,
) -> (
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
        .with_inner_size(window_size);

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
