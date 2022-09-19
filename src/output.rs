use std::{collections::HashMap, ffi::c_void};

use gl::types::{GLfloat, GLint, GLsizei, GLuint};

use crate::{input::EguiManager, shaders::ShaderProgram};

pub struct Painter {
    vertex_array_object: GLuint,
    vertex_buffer_object: GLuint,
    index_buffer_object: GLuint,
    shader_program: ShaderProgram,
    screen_dimensions_uniform: GLint,
    last_screen_dimensions: [i32; 2],
    textures: HashMap<egui::TextureId, Texture>,
}

impl Painter {
    pub fn new(window: &mut glfw::Window) -> Painter {
        // Load procedural addresses
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        
        // Compile shaders and get uniform location for screen_coordinates
        let shader_program = ShaderProgram::new(
            "../shaders/vertex_shader.glsl",
            "../shaders/vertex_shader.glsl",
        )
        .unwrap();
        let screen_dimensions_uniform = shader_program
            .get_uniform_location("u_screen_dimensions")
            .unwrap();

        // Create vao, vbo and ibo, bind them and set attrib pointers
        let (vao, vbo, ibo) = unsafe {
            let (mut vao, mut vbo, mut ibo): (GLuint, GLuint, GLuint) = (0, 0, 0);
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ibo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as GLsizei,
                std::ptr::null(),
            );
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as GLsizei,
                (2 * std::mem::size_of::<GLfloat>()) as *const c_void,
            );
            gl::VertexAttribPointer(
                2,
                4,
                gl::UNSIGNED_BYTE,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as GLsizei,
                (4 * std::mem::size_of::<GLfloat>()) as *const c_void,
            );

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            (vao, vbo, ibo)
        };

        Painter {
            vertex_array_object: vao,
            vertex_buffer_object: vbo,
            index_buffer_object: ibo,
            shader_program,
            screen_dimensions_uniform,
            last_screen_dimensions: [0; 2],
            textures: HashMap::new(),
        }
    }

    pub fn handle_full_output(&mut self, egui_manager: &EguiManager, egui_context: &egui::Context, full_output: &egui::FullOutput) {
        //Paint

        // Set the textures to be set
        self.set_textures(&full_output.textures_delta);

        //Draw

        // Free the textures to be freed
        self.free_textures(&full_output.textures_delta);
    }





    fn free_textures(&mut self, textures_delta: &egui::TexturesDelta) {
        for texture_id in &textures_delta.free {
            _ = self.textures.remove(texture_id).expect("Cannot remove this texture as it is not in the HashMap");
        }
    }

    fn set_textures(&mut self, textures_delta: &egui::TexturesDelta) {
        for (texture_id, image_delta) in &textures_delta.set {
            match image_delta.pos {
                Some(pos) => {
                    // Only a certain portion of an existing texture needs updating
                    if let Some(texture) = self.textures.get_mut(&texture_id) {
                        texture.update_subregion(&image_delta.image, pos);
                        texture.register();
                    } else {
                        panic!("Can't find texture of which we were meant to be updating a subregion!");
                    }
                },
                None => {
                    assert!(!self.textures.contains_key(&texture_id), "This is supposed to be a new texture but it's already stored!");
                    let mut texture = Texture::new(&image_delta.image, image_delta.filter);
                    texture.register();
                    self.textures.insert(*texture_id, texture);
                },
            }
        }
    }

    fn draw(&mut self, egui_manager: &EguiManager) {
        let (w, h) = egui_manager.screen_dimensions();
        let w = w as i32;
        let h = h as i32;
        if w != self.last_screen_dimensions[0] || h != self.last_screen_dimensions[1] {
            unsafe {
                gl::Uniform2i(self.screen_dimensions_uniform, w, h);
                todo!()
            }
            self.last_screen_dimensions = [w, h];
        }

        // Set blend function and enable sRGB conversion
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::SCISSOR_TEST);
            self.shader_program.bind();
            gl::ActiveTexture(gl::TEXTURE0);
        }



    }



}

#[repr(C)]
struct Vertex {
    pos_coord: [f32; 2],
    tex_coord: [f32; 2],
    srgba_col: egui::Color32,
}

struct Texture {
    opengl_status: TextureOpenGLStatus,
    filter: egui::TextureFilter,
    update_status: TextureUpdateStatus,
}
impl Texture {
    fn new(image_data: &egui::ImageData, filter: egui::TextureFilter) -> Texture {
        let [width, height] = image_data.size();
        let texture_data = TextureData {
            start_position: [0, 0],
            width,
            height,
            srgba_pixels: match image_data {
                egui::ImageData::Color(color_image) => {
                    color_image.pixels.clone()
                },
                egui::ImageData::Font(font_image) => {
                    font_image.srgba_pixels(1.0).collect()
                }
            },
        };
        Texture {
            opengl_status: TextureOpenGLStatus::Unregistered,
            filter,
            update_status: TextureUpdateStatus::New(texture_data)
        }
        
    }

    fn update_subregion(&mut self, image_data: &egui::ImageData, start_position: [usize; 2]) {
        let [width, height] = image_data.size();
        let new_texture_data = TextureData{
            start_position,
            width,
            height,
            srgba_pixels: match image_data {
                egui::ImageData::Color(color_image) => {
                    color_image.pixels.clone()
                },
                egui::ImageData::Font(font_image) => {
                    font_image.srgba_pixels(1.0).collect()
                }
            },
        };
        self.update_status = TextureUpdateStatus::Update(new_texture_data);
    }

    fn register(&mut self) {
        match &self.update_status {
            TextureUpdateStatus::None => panic!("You shouldn't have reached this point (trying to register a texture that doesn't need it"),
            TextureUpdateStatus::New(texture_data) => {
                assert_eq!(self.opengl_status, TextureOpenGLStatus::Unregistered, "This texture for some reason is registered");
                let texture_opengl_status = unsafe {
                    let mut opengl_texture_id = 0;
                    gl::GenTextures(1, &mut opengl_texture_id);
                    gl::BindTexture(gl::TEXTURE_2D, opengl_texture_id);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                    match self.filter {
                        egui::TextureFilter::Nearest => {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                        },
                        egui::TextureFilter::Linear => {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                        },
                    };

                    if !texture_data.srgba_pixels.is_empty() {
                        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB_ALPHA as i32, texture_data.width as i32, texture_data.height as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, texture_data.srgba_pixels.as_ptr() as *const c_void);
                    }

                    TextureOpenGLStatus::Registered(opengl_texture_id)
                };
                self.opengl_status = texture_opengl_status;
            },
            TextureUpdateStatus::Update(texture_data) => {
                assert_ne!(self.opengl_status, TextureOpenGLStatus::Unregistered, "Trying to update subregion of texture but it doesn't have an opengl id");
                if let TextureOpenGLStatus::Registered(opengl_id) = self.opengl_status {
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, opengl_id);
                        gl::TexSubImage2D(gl::TEXTURE_2D, 0, texture_data.start_position[0] as i32, texture_data.start_position[1] as i32, texture_data.width as i32, texture_data.height as i32, gl::RGBA, gl::UNSIGNED_BYTE, texture_data.srgba_pixels.as_ptr() as *const c_void);
                    }
                } 
            },
        }
        self.update_status = TextureUpdateStatus::None;
    }

    fn needs_updating(&self) -> bool {
        !matches!(self.update_status, TextureUpdateStatus::None)
    }

}
impl Drop for Texture {
    fn drop(&mut self) {
        if let TextureOpenGLStatus::Registered(opengl_id) = self.opengl_status {
            unsafe {
                gl::DeleteTextures(1, opengl_id as *const _)
            }
        }
    }
}

struct TextureData {
    start_position: [usize; 2],
    width: usize,
    height: usize,
    srgba_pixels: Vec<egui::Color32>,
}


enum TextureUpdateStatus {
    New(TextureData),
    Update(TextureData),
    None,
}

#[derive(Debug, PartialEq, Eq)]
enum TextureOpenGLStatus {
    Unregistered,
    Registered(GLuint),
}