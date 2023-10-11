use std::mem::{size_of, transmute};

use anyhow::Result;
use gl::types::*;

use super::utils::{compile_shader, link_program};
use crate::gl_assert_ok;
use crate::state::Vec2;

pub struct GlParticles {
    vao: u32,
    vbo: u32,
    program: u32,
}

impl GlParticles {
    pub fn new() -> Result<GlParticles> {
        let vs = compile_shader(include_str!("particle.vert"), gl::VERTEX_SHADER)?;
        let fs = compile_shader(include_str!("particle.frag"), gl::FRAGMENT_SHADER)?;
        let program = link_program(vs, fs)?;

        let (vao, vbo) = unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                2 * size_of::<GLfloat>() as GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            (vao, vbo)
        };

        Ok(GlParticles { vao, vbo, program })
    }

    pub fn draw(&self, particles: &[Vec2]) {
        unsafe {
            gl::UseProgram(self.program);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            let points = particles
                .iter()
                .flat_map(|v| vec![v.x, v.y])
                .collect::<Vec<f32>>();

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (points.len() * size_of::<f32>()) as GLsizeiptr,
                transmute(&points[0]),
                gl::STATIC_DRAW,
            );

            gl::PointSize(10.0);
            gl::DrawArrays(gl::POINTS, 0, particles.len() as GLsizei);

            gl_assert_ok!();
        }
    }
}
