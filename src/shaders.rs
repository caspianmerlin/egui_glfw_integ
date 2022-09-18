use std::{path::Path, error::Error, fs, ffi::CString};

use gl::{types::*, GetShaderiv};

pub struct ShaderProgram {
    inner: u32,
}
impl ShaderProgram {
    pub fn new<T: AsRef<Path>>(vertex: T, fragment: T) -> Result<Self, Box<dyn Error>> {
        let vertex_shader = compile_shader(vertex, ShaderType::Vertex)?;
        let fragment_shader = compile_shader(fragment, ShaderType::Fragment)?;
        let shader_program = link_shader_program(vertex_shader, fragment_shader)?;
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }
        Ok( Self { inner: shader_program } )
    }
    pub fn bind(&self) {
        unsafe { gl::UseProgram(self.inner) }
    }
    pub fn unbind(&self) {
        unsafe { gl::UseProgram(0) }
    }
    pub fn get_uniform_location(&self, uniform_name: &str) -> i32 {
        let name_as_c_string = CString::new(uniform_name).unwrap();
        let location = unsafe {
            gl::GetUniformLocation(self.inner, name_as_c_string.as_ptr())
        };
        location
    }
}



fn link_shader_program(vertex_shader: u32, fragment_shader: u32) -> Result<u32, Box<dyn Error>> {

    let shader_program = unsafe {
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        shader_program
    };
    
    match check_for_gl_error(InfoLogType::ShaderProgram(shader_program)) {
        Ok(_) => Ok(shader_program),
        Err(info_log) => Err(info_log.into())
    }
}

fn compile_shader<T: AsRef<Path>>(path: T, shader_type: ShaderType) -> Result<u32, Box<dyn Error>> {
    let shader_type = match shader_type {
        ShaderType::Vertex => gl::VERTEX_SHADER,
        ShaderType::Fragment => gl::FRAGMENT_SHADER,
    };
    
    let shader = unsafe { gl::CreateShader(shader_type) };
    let shader_source = fs::read_to_string(path)?;
    let shader_c_string = CString::new(shader_source)?;
    unsafe { 
        gl::ShaderSource(shader, 1, &shader_c_string.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
    }
    match check_for_gl_error(InfoLogType::Shader(shader)) {
        Ok(_) => Ok(shader),
        Err(info_log) => Err(info_log.into()),
    }
}

fn check_for_gl_error(input: InfoLogType) -> Result<(), String> {
    let mut success: i32 = 0;
    match input {
        InfoLogType::Shader(id) => unsafe { gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success) },
        InfoLogType::ShaderProgram(id) => unsafe { gl::GetProgramiv(id, gl::LINK_STATUS, &mut success) },
    }
    if success == 1 {
        Ok(())
    } else {
        let info_log = match input {
            InfoLogType::Shader(id) => {
                unsafe {
                    let mut info_log_len: i32 = 0;
                    gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut info_log_len);
                    let mut info_log_vec = Vec::with_capacity(info_log_len as usize);
                    gl::GetShaderInfoLog(id, info_log_len, std::ptr::null_mut(), info_log_vec.as_mut_ptr() as *mut GLchar);
                    info_log_vec.set_len(info_log_len as usize - 1);
                    String::from_utf8(info_log_vec).expect("Error converting shader info log")
                }
            },
            InfoLogType::ShaderProgram(id) => {
                unsafe {
                    let mut info_log_len: i32 = 0;
                    gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut info_log_len);
                    let mut info_log_vec = Vec::with_capacity(info_log_len as usize);
                    gl::GetProgramInfoLog(id, info_log_len, std::ptr::null_mut(), info_log_vec.as_mut_ptr() as *mut GLchar);
                    info_log_vec.set_len(info_log_len as usize - 1);
                    String::from_utf8(info_log_vec).expect("Error converting program info log")
                }
            },
        };
        Err(info_log)
    }
}

enum ShaderType {
    Fragment,
    Vertex,
}

enum InfoLogType {
    Shader(u32),
    ShaderProgram(u32)
}

