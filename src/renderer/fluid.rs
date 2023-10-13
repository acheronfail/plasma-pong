use std::mem::{size_of, transmute};
use std::ptr;

use anyhow::Result;
use gl::types::*;

use super::utils::{compile_shader, link_program};
use super::{world_len_to_gl_len, world_pos_to_gl_pos};
use crate::engine::EngineContext;
use crate::gl_assert_ok;

pub struct GlFluid {
    vao: u32,
    vbo: u32,
    ebo: u32,
    program: u32,
}

impl GlFluid {
    pub fn new() -> Result<GlFluid> {
        let vs = compile_shader(include_str!("fluid.vert"), gl::VERTEX_SHADER)?;
        let fs = compile_shader(include_str!("fluid.frag"), gl::FRAGMENT_SHADER)?;
        let program = link_program(vs, fs)?;

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::VertexAttribPointer(
                0,
                4,
                gl::FLOAT,
                gl::FALSE,
                4 * size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );

            gl::EnableVertexAttribArray(0);

            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

            gl_assert_ok!();
        }

        Ok(GlFluid {
            vao,
            vbo,
            ebo,
            program,
        })
    }

    pub fn draw(&self, ctx: &EngineContext) {
        let radius_normalised =
            world_len_to_gl_len(&ctx.state.bounding_box, ctx.state.smoothing_radius()) * 3.0;
        let vertices = ctx
            .state
            .positions
            .iter()
            .flat_map(|p| {
                let p = world_pos_to_gl_pos(&ctx.state.bounding_box, p);
                [
                    // top left
                    p.x - radius_normalised,
                    p.y + radius_normalised,
                    p.x,
                    p.y,
                    // top-right
                    p.x + radius_normalised,
                    p.y + radius_normalised,
                    p.x,
                    p.y,
                    // bottom-right
                    p.x + radius_normalised,
                    p.y - radius_normalised,
                    p.x,
                    p.y,
                    // bottom-left
                    p.x - radius_normalised,
                    p.y - radius_normalised,
                    p.x,
                    p.y,
                ]
            })
            .collect::<Vec<f32>>();

        let indices = (0..ctx.state.positions.len())
            .into_iter()
            .flat_map(|i| {
                let offset = i as u32 * 4;
                [
                    0 + offset,
                    1 + offset,
                    2 + offset,
                    0 + offset,
                    2 + offset,
                    3 + offset,
                ]
            })
            .collect::<Vec<u32>>();

        unsafe {
            gl::UseProgram(self.program);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * size_of::<f32>()) as GLsizeiptr,
                transmute(&vertices[0]),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * size_of::<GLuint>()) as GLsizeiptr,
                transmute(&indices[0]),
                gl::STATIC_DRAW,
            );

            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            gl_assert_ok!();
        }
    }
}
