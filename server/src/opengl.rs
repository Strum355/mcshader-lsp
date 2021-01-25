use std::ptr;
use std::ffi::{CString, CStr};

#[cfg(test)]
use mockall::automock;
#[cfg_attr(test, automock)]
pub trait ShaderValidator {
    fn validate(&self, tree_type: super::TreeType, source: String) -> Option<String>;
}

pub struct OpenGLContext {
    _ctx: glutin::Context<glutin::PossiblyCurrent>
}

impl OpenGLContext {
    pub fn new() -> OpenGLContext {
        let events_loop = glutin::event_loop::EventLoop::new();
        let gl_window = glutin::ContextBuilder::new().build_headless(&*events_loop, glutin::dpi::PhysicalSize::new(1, 1)).unwrap();
    
        let gl_window = unsafe {
            let gl_window = gl_window.make_current().unwrap();
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            gl_window
        };

        unsafe {
            eprintln!(
                "Using OpenGL device {} {} {}", 
                String::from_utf8(CStr::from_ptr(gl::GetString(gl::VENDOR) as *const _).to_bytes().to_vec()).unwrap(),
                String::from_utf8(CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_bytes().to_vec()).unwrap(),
                String::from_utf8(CStr::from_ptr(gl::GetString(gl::RENDERER) as *const _).to_bytes().to_vec()).unwrap()
            );
        }
        OpenGLContext{
            _ctx: gl_window,
        }
    }
}

impl ShaderValidator for OpenGLContext {
    fn validate(&self, tree_type: super::TreeType, source: String) -> Option<String> {
        unsafe {
            let mut success = i32::from(gl::FALSE);
    
            match tree_type {
                crate::TreeType::Fragment => {
                    // Fragment shader
                    let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
                    let c_str_frag = CString::new(source).unwrap();
                    gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
                    gl::CompileShader(fragment_shader);
    
                    // Check for shader compilation errors
                    gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
                    let result = if success != i32::from(gl::TRUE) {
                        let mut info_len: gl::types::GLint = 0;
                        gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut info_len);
                        let mut info = vec![0u8; info_len as usize];
                        gl::GetShaderInfoLog(fragment_shader, info_len as gl::types::GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut gl::types::GLchar);
                        info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
                        Some(String::from_utf8(info).unwrap())
                    } else {
                        None
                    };
                    gl::DeleteShader(fragment_shader);
                    result
                }
                crate::TreeType::Vertex => {
                    // Vertex shader
                    let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
                    let c_str_vert = CString::new(source).unwrap();
                    gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
                    gl::CompileShader(vertex_shader);
    
                    // Check for shader compilation errors
                    gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
                    let result = if success != i32::from(gl::TRUE) {
                        let mut info_len: gl::types::GLint = 0;
                        gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut info_len);
                        let mut info = vec![0u8; info_len as usize];
                        gl::GetShaderInfoLog(vertex_shader, info_len as gl::types::GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut gl::types::GLchar);
                        info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
                        Some(String::from_utf8(info).unwrap())
                    } else {
                        None
                    };
                    gl::DeleteShader(vertex_shader);
                    result
                }
            }
        }
    }
}
