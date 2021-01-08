use std::mem;
use std::ptr;
use std::str;
use std::os::raw::c_void;
use std::ffi::CString;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor; // Specify a vertex attribute for color
out vec3 color;
void main()
{
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
	color = aColor; // pass the color along to the fragment shader
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
out vec4 FragColor;
in vec3 color;
void main()
{
   // Set the fragment color to the color passed from the vertex shader
   FragColor = vec4(color, 1.0);
}
"#;

fn doit() {
    let mut events_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_title("Glutin Triangle")
        .with_inner_size(glutin::dpi::Size::Physical(glutin::dpi::PhysicalSize::new(1, 1)));
    let gl_window = glutin::ContextBuilder::new().build_headless(&*events_loop, glutin::dpi::PhysicalSize::new(1, 1)).unwrap();


    let gl_window = unsafe {
        let gl_window = gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl_window
    };

    unsafe {
        // Setup shader compilation checks
        let mut success = i32::from(gl::FALSE);

        // Vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        // Check for shader compilation errors
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            let mut info_len: gl::types::GLint = 0;
            gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut info_len);
            let mut info = vec![0u8; info_len as usize];
            gl::GetShaderInfoLog(vertex_shader, info_len as gl::types::GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut gl::types::GLchar);
            info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
            println!(
                "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}",
                str::from_utf8_unchecked(&info)
            );
        }

        // Fragment shader
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        // Check for shader compilation errors
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            let mut info_len: gl::types::GLint = 0;
            gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut info_len);
            let mut info = vec![0u8; info_len as usize];
            gl::GetShaderInfoLog(fragment_shader, info_len as gl::types::GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut gl::types::GLchar);
            info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
            println!(
                "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}",
                str::from_utf8_unchecked(&info)
            );
        }

        // Link Shaders
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        // Check for linking errors
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            let mut info_len: gl::types::GLint = 0;
            gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut info_len);
            let mut info = vec![0u8; info_len as usize];
            gl::GetProgramInfoLog(shader_program, info_len as gl::types::GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut gl::types::GLchar);
            info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
            println!(
                "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}",
                str::from_utf8_unchecked(&info)
            );
        }
        /* gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);*/
    };
}