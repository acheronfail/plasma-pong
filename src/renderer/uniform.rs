#![allow(unused)]

use std::ffi::CString;

use anyhow::{anyhow, Result};

/// Small helper to create (and set defaults) for uniforms
pub enum Uniform {
    Vec2(f32, f32),
    F32(f32),
    Int(i32),
}

impl Uniform {
    pub unsafe fn create(self, program: u32, name: &str) -> Result<i32> {
        let c_str = CString::new(name).unwrap();
        let location = gl::GetUniformLocation(program, c_str.as_ptr());
        if location < 0 {
            return Err(anyhow!(r#"GetUniformLocation("{name}") -> {location}"#));
        }

        match self {
            Uniform::Vec2(x, y) => gl::Uniform2f(location, x, y),
            Uniform::F32(value) => gl::Uniform1f(location, value),
            Uniform::Int(value) => gl::Uniform1i(location, value),
        }

        Ok(location)
    }
}
