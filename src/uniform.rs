use std::ffi::CString;

/// Small helper to create (and set defaults) for uniforms
pub enum Uniform {
    F32(f32),
    Int(i32),
}

impl Uniform {
    pub unsafe fn create(self, program: u32, name: &str) -> i32 {
        let c_str = CString::new(name).unwrap();
        let location = gl::GetUniformLocation(program, c_str.as_ptr());
        match self {
            Uniform::F32(value) => gl::Uniform1f(location, value),
            Uniform::Int(value) => gl::Uniform1i(location, value),
        }

        location
    }
}
