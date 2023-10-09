use std::ffi::CString;

use glutin::display::Display;
use glutin::prelude::*;

use crate::ApplicationState;

pub struct Renderer {}

impl Renderer {
    pub fn new(gl_display: &Display) -> Renderer {
        unsafe {
            // provide loader to link gl function pointers to the display
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            // compile shaders
            let vertex_source = CString::new(include_str!("./vertex.vert")).unwrap();
            let fragment_source = CString::new(include_str!("./fragment.frag")).unwrap();

            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex_shader, 1, &vertex_source.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(
                fragment_shader,
                1,
                &fragment_source.as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(fragment_shader);

            // link shaders into a program
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);

            // we can delete the shaders now, since they're linked into the program now
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            // set the program we just created to be the active one
            gl::UseProgram(shader_program);

            Renderer {}
        }
    }

    pub fn draw(&mut self, _: &ApplicationState) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // TODO: draw something
        }
    }
}
