use std::mem::{size_of, transmute};
use std::ptr;

use anyhow::Result;
use gl::types::*;
use glam::Vec2;

use super::utils::{compile_shader, link_program};
use crate::gl_assert_ok;
use crate::state::Rect;

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

        let mut vao = 0;
        let mut vbo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            let n_values = 3;
            gl::VertexAttribPointer(
                0,
                n_values,
                gl::FLOAT,
                gl::FALSE,
                n_values * size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
            gl_assert_ok!();
        }

        Ok(GlParticles { vao, vbo, program })
    }

    pub fn draw(&self, radius: f32, bounding_box: &Rect, positions: &[Vec2], velocities: &[Vec2]) {
        let points = positions
            .iter()
            .zip(velocities)
            .flat_map(|(p, v)| {
                let p = world_pos_to_gl_pos(bounding_box, p);
                [p.x, p.y, v.length() / 4.0]
            })
            .collect::<Vec<f32>>();

        unsafe {
            gl::UseProgram(self.program);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (points.len() * size_of::<f32>()) as GLsizeiptr,
                transmute(&points[0]),
                gl::STATIC_DRAW,
            );

            gl::PointSize(radius);
            gl::DrawArrays(gl::POINTS, 0, positions.len() as GLsizei);

            gl_assert_ok!();
        }
    }
}

#[inline]
fn world_pos_to_gl_pos(bounding_box: &Rect, world_pos: &Vec2) -> Vec2 {
    let x = (world_pos.x - bounding_box.x) / (bounding_box.w * 0.5) - 1.0;
    let y = (world_pos.y - bounding_box.y) / (bounding_box.h * 0.5) - 1.0;
    Vec2::new(x, -y)
}
