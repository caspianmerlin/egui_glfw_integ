use std::ffi::c_void;

use gl::types::{GLuint, GLsizei, GLfloat};

use crate::{shaders::ShaderProgram, input::EguiManager};

pub struct Painter {
    vertex_array_object: GLuint,
    vertex_buffer_object: GLuint,
    index_buffer_object: GLuint,
    shader_program: ShaderProgram,

    

}

impl Painter {
    pub fn new(window: &mut glfw::Window) -> Painter {
        // Load procedural addresses
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        // Compile shaders and get uniform location for screen_coordinates
        let shader_program = ShaderProgram::new("../shaders/vertex_shader.glsl", "../shaders/vertex_shader.glsl").unwrap();
        let screen_dimensions_uniform = shader_program.get_uniform_location("u_screen_dimensions");


        // Create vao, vbo and ibo, bind them and set attrib pointers
        let (vao, vbo, ibo) = unsafe {
            let (mut vao, mut vbo, mut ibo): (GLuint, GLuint, GLuint) = (0, 0, 0);
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ibo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLsizei, std::ptr::null());
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLsizei, (2 * std::mem::size_of::<GLfloat>()) as *const c_void);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLsizei, (4 * std::mem::size_of::<GLfloat>()) as *const c_void);

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            (vao, vbo, ibo)
        };

        todo!()
    }

    pub fn handle_full_output(&mut self, egui_manager: &EguiManager, egui_context: &egui::Context) {

    }

    fn set_textures(&mut self, textures_delta: &egui::TexturesDelta) {
        
    }
}

#[repr(C)]
struct Vertex {
    pos_coord: [f32; 2],
    tex_coord: [f32; 2],
    srgba_col: [f32; 4],
}