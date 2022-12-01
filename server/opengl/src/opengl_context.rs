use std::ffi::{CStr, CString};
use std::ptr;

use glutin::event_loop::EventLoopBuilder;
use glutin::platform::unix::EventLoopBuilderExtUnix;
use logging::info;

use crate::ShaderValidator;

pub(crate) struct Context {
    _ctx: glutin::Context<glutin::PossiblyCurrent>,
}

impl Context {
    pub fn default() -> Context {
        let events_loop = EventLoopBuilder::new().with_any_thread(true).build();
        let gl_window = glutin::ContextBuilder::new()
            .build_headless(&*events_loop, glutin::dpi::PhysicalSize::new(1, 1))
            .unwrap();

        let gl_window = unsafe {
            let gl_window = gl_window.make_current().unwrap();
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            gl_window
        };

        let gl_ctx = Context { _ctx: gl_window };

        unsafe {
            info!(
                "OpenGL device";
                "vendor" => gl_ctx.vendor(),
                "version" => String::from_utf8(CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_bytes().to_vec()).unwrap(),
                "renderer" => String::from_utf8(CStr::from_ptr(gl::GetString(gl::RENDERER) as *const _).to_bytes().to_vec()).unwrap()
            );
        }
        gl_ctx
    }

    unsafe fn compile_and_get_shader_log(&self, shader: gl::types::GLuint, source: &str) -> Option<String> {
        let mut success = i32::from(gl::FALSE);
        let c_str_frag = CString::new(source).unwrap();
        gl::ShaderSource(shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check for shader compilation errors
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        let result = if success != i32::from(gl::TRUE) {
            let mut info_len: gl::types::GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_len);
            let mut info = vec![0u8; info_len as usize];
            gl::GetShaderInfoLog(
                shader,
                info_len as gl::types::GLsizei,
                ptr::null_mut(),
                info.as_mut_ptr() as *mut gl::types::GLchar,
            );
            info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
            Some(String::from_utf8(info).unwrap())
        } else { None };
        gl::DeleteShader(shader);
        result
    }
}

impl ShaderValidator for Context {
    fn validate(&self, tree_type: super::TreeType, source: &str) -> Option<String> {
        unsafe {
            match tree_type {
                crate::TreeType::Fragment => {
                    // Fragment shader
                    let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
                    self.compile_and_get_shader_log(fragment_shader, source)
                }
                crate::TreeType::Vertex => {
                    // Vertex shader
                    let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
                    self.compile_and_get_shader_log(vertex_shader, source)
                }
                crate::TreeType::Geometry => {
                    // Geometry shader
                    let geometry_shader = gl::CreateShader(gl::GEOMETRY_SHADER);
                    self.compile_and_get_shader_log(geometry_shader, source)
                }
                crate::TreeType::Compute => {
                    // Compute shader
                    let compute_shader = gl::CreateShader(gl::COMPUTE_SHADER);
                    self.compile_and_get_shader_log(compute_shader, source)
                }
            }
        }
    }

    fn vendor(&self) -> String {
        unsafe { String::from_utf8(CStr::from_ptr(gl::GetString(gl::VENDOR) as *const _).to_bytes().to_vec()).unwrap() }
    }
}
