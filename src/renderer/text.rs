use anyhow::Result;
use glyph_brush::ab_glyph::FontRef;
use glyph_brush::{BrushAction, BrushError, GlyphBrush, GlyphBrushBuilder, Section};
use winit::dpi::PhysicalSize;

use super::glyph::GlGlyphVertex;
use crate::gl_assert_ok;
use crate::renderer::glyph::{to_vertex, GlGlyphTexture, GlTextPipe};

const FONT: &[u8] = include_bytes!("gnu-freefont-FreeMono.ttf");

/// A wrapper around `glyph_brush` to expose a simple API for drawing text with GL.
pub struct GlText {
    max_image_dimension: u32,
    glyph_brush: GlyphBrush<GlGlyphVertex, glyph_brush::Extra, FontRef<'static>>,
    glyph_texture: GlGlyphTexture,
    text_pipe: GlTextPipe,
}

impl GlText {
    pub fn new(surface_dimensions: PhysicalSize<u32>) -> Result<GlText> {
        let max_image_dimension = {
            let mut value = 0;
            unsafe { gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut value) };
            value as u32
        };

        let font = FontRef::try_from_slice(FONT)?;
        let glyph_brush = GlyphBrushBuilder::using_font(font).build();
        let glyph_texture = GlGlyphTexture::new(glyph_brush.texture_dimensions());
        let text_pipe = GlTextPipe::new(surface_dimensions)?;

        Ok(GlText {
            max_image_dimension,
            glyph_brush,
            glyph_texture,
            text_pipe,
        })
    }

    pub fn update_geometry(&mut self, surface_dimensions: PhysicalSize<u32>) {
        self.text_pipe.update_geometry(surface_dimensions);
    }

    pub fn draw(&mut self, sections: &[Section]) {
        for section in sections {
            self.glyph_brush.queue(section);
        }

        // Tell glyph_brush to process the queued text
        let mut brush_action;
        loop {
            brush_action = self.glyph_brush.process_queued(
                |rect, tex_data| {
                    // Update part of gpu texture with new glyph alpha values
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, self.glyph_texture.gl_texture);
                        gl::TexSubImage2D(
                            gl::TEXTURE_2D,
                            0,
                            rect.min[0] as _,
                            rect.min[1] as _,
                            rect.width() as _,
                            rect.height() as _,
                            gl::RED,
                            gl::UNSIGNED_BYTE,
                            tex_data.as_ptr() as _,
                        );
                        gl_assert_ok!();
                    }
                },
                to_vertex,
            );

            // If the cache texture is too small to fit all the glyphs, resize and try again
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested, .. }) => {
                    let (new_width, new_height) = if (suggested.0 > self.max_image_dimension
                        || suggested.1 > self.max_image_dimension)
                        && (self.glyph_brush.texture_dimensions().0 < self.max_image_dimension
                            || self.glyph_brush.texture_dimensions().1 < self.max_image_dimension)
                    {
                        (self.max_image_dimension, self.max_image_dimension)
                    } else {
                        suggested
                    };

                    eprintln!("Resizing glyph texture -> {new_width}x{new_height}");

                    // Recreate texture as a larger size to fit more
                    self.glyph_texture = GlGlyphTexture::new((new_width, new_height));
                    self.glyph_brush.resize_texture(new_width, new_height);
                }
            }
        }
        // If the text has changed from what was last drawn, upload the new vertices to GPU
        match brush_action.unwrap() {
            BrushAction::Draw(vertices) => self.text_pipe.upload_vertices(&vertices),
            BrushAction::ReDraw => {}
        }

        self.text_pipe.draw();
    }
}
