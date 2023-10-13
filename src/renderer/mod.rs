mod glyph;
mod particles;
mod text;
mod uniform;
mod utils;

use std::ffi::CString;

use anyhow::Result;
use glutin::display::Display;
use glutin::prelude::*;
use glyph_brush::{Section, Text};
use winit::window::Window;

use self::particles::GlParticles;
use self::text::GlText;
use self::utils::{compile_shader, link_program};
use crate::engine::EngineContext;
use crate::State;

pub struct Renderer {
    // renders the particles
    particles: GlParticles,
    // renders any text on the screen
    text: GlText,
}

impl Renderer {
    pub fn new(gl_display: &Display, window: &Window) -> Result<Renderer> {
        let dimensions = window.inner_size();

        // provide loader to link gl function pointers to the display
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        Ok(Renderer {
            particles: GlParticles::new()?,
            text: GlText::new(dimensions)?,
        })
    }

    pub fn draw(&mut self, ctx: EngineContext) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // draw particles
            self.particles.draw(
                State::PARTICLE_RADIUS,
                &ctx.state.positions,
                &ctx.state.velocities,
            );

            // draw text on screen
            self.text.update_geometry(ctx.surface_dimensions);
            self.text.draw(vec![
                // draw fps
                Section::default()
                    .add_text(
                        Text::new(&format!("FPS: {:.2} VSYNC: {}", ctx.fps, ctx.vsync))
                            .with_scale((18.0 * ctx.scale_factor).round())
                            .with_color([1.0, 1.0, 1.0, 1.0]),
                    )
                    .with_bounds((
                        ctx.surface_dimensions.width as f32 / 2.,
                        ctx.surface_dimensions.height as f32,
                    )),
            ]);
        }
    }
}
