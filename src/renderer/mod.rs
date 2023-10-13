mod fluid;
mod glyph;
mod particles;
mod text;
mod uniform;
mod utils;

use std::ffi::CString;

use anyhow::Result;
use glam::Vec2;
use glutin::display::Display;
use glutin::prelude::*;
use glyph_brush::{Section, Text};
use winit::window::Window;

use self::fluid::GlFluid;
use self::particles::GlParticles;
use self::text::GlText;
use self::utils::{compile_shader, link_program};
use crate::engine::EngineContext;
use crate::state::Rect;

pub struct Renderer {
    fluid: GlFluid,
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

        unsafe {
            // gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }

        Ok(Renderer {
            fluid: GlFluid::new()?,
            particles: GlParticles::new()?,
            text: GlText::new(dimensions)?,
        })
    }

    pub fn draw(&mut self, ctx: EngineContext) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // draw text on screen
            self.text.update_geometry(ctx.surface_dimensions);
            self.text.draw(&vec![
                // draw fps
                Section::default()
                    .add_text(
                        Text::new(&format!("FPS: {:.2} VSYNC: {}", ctx.fps, ctx.vsync))
                            .with_scale((18.0 * ctx.scale_factor).round())
                            .with_color([1.0, 1.0, 1.0, 1.0]),
                    )
                    .with_bounds((
                        ctx.surface_dimensions.width as f32,
                        ctx.surface_dimensions.height as f32,
                    )),
            ]);

            // draw particles
            self.particles.draw(&ctx);

            // draw pressure zones
            self.fluid.draw(&ctx);
        }
    }
}

#[inline]
pub fn world_pos_to_gl_pos(bounding_box: &Rect, world_pos: &Vec2) -> Vec2 {
    let x = (world_pos.x - bounding_box.x) / (bounding_box.w * 0.5) - 1.0;
    let y = (world_pos.y - bounding_box.y) / (bounding_box.h * 0.5) - 1.0;
    Vec2::new(x, -y)
}

#[allow(unused)]
#[inline]
pub fn world_len_to_gl_len(bounding_box: &Rect, world_len: f32) -> f32 {
    let world_min = f32::min(bounding_box.x, bounding_box.y);
    let world_max = f32::max(bounding_box.w, bounding_box.h);
    (world_len - world_min) / (world_max - world_min)
}
