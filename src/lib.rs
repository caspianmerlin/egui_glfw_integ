#![warn(clippy::pedantic)]

use std::{rc::Rc, cell::RefCell};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use egui::{Pos2, RawInput, TexturesDelta, ClippedPrimitive, PlatformOutput, CursorIcon, TextureFilter, Color32, TextureId};
use glfw::{WindowEvent, Window, StandardCursor};
use input::InputHandler;
use painter::Painter;

pub use egui;

mod input;
mod painter;
mod shaders;


pub struct EguiManager {
    window_dimensions_px: Pos2,
    pixels_per_point: f32,
    clipboard_context: Rc<RefCell<ClipboardContext>>,
    input_handler: InputHandler,
    painter: Painter,
    current_cursor: StandardCursor,
}

impl EguiManager {
    pub fn new(glfw_window: &mut Window, window_dimensions_px: Pos2, pixels_per_point: f32, max_texture_side: usize) -> EguiManager {
        // Initialise clipboard context
        let clipboard_context = Rc::new(RefCell::new(ClipboardContext::new().expect("Unable to create clipboard context")));
        let input_handler = InputHandler::new(window_dimensions_px, pixels_per_point, max_texture_side, Rc::clone(&clipboard_context));
        let painter = Painter::new(glfw_window);
        painter.update_window_dimensions(window_dimensions_px, pixels_per_point);
        
        EguiManager { window_dimensions_px, pixels_per_point, clipboard_context, input_handler, painter, current_cursor: StandardCursor::Arrow }
    }

    pub fn handle_window_event(&mut self, window_event: &WindowEvent) {
        if let WindowEvent::FramebufferSize(w, h) = window_event {
            self.window_dimensions_px.x = *w as f32;
            self.window_dimensions_px.y = *h as f32;

            self.painter.update_window_dimensions(self.window_dimensions_px, self.pixels_per_point);
        }
        self.input_handler.handle_event(window_event, self.pixels_per_point);
    }

    pub fn generate_raw_input(&mut self) -> RawInput {
        self.input_handler.generate_raw_input(self.pixels_per_point)
    }

    pub fn set_pixels_per_point(&mut self, pixels_per_point: f32) {
        self.pixels_per_point = pixels_per_point;
    }


    pub fn render(&mut self, textures_delta: &TexturesDelta, clipped_primitives: &Vec<ClippedPrimitive>) {
        self.painter.handle_paint_call(textures_delta, clipped_primitives, self.window_dimensions_px, self.pixels_per_point);
    }

    pub fn handle_platform_output(&mut self, glfw_window: &mut Window, platform_output: &PlatformOutput) {
        // Set cursor
        let cursor = translate_cursor(platform_output.cursor_icon);
        if self.current_cursor != cursor {
            println!("Setting cursor...");
            glfw_window.set_cursor(Some(glfw::Cursor::standard(cursor)));
            self.current_cursor = cursor;
        }

        if !platform_output.copied_text.is_empty() {
            let mut ctx = self.clipboard_context.borrow_mut();
            _ = ctx.set_contents(platform_output.copied_text.clone());
        }
    }

    pub fn new_user_texture(&mut self, width: usize, height: usize, filtering: TextureFilter, pixels: &[Color32]) -> TextureId {
        self.painter.new_user_texture(width, height, filtering, pixels)
    }

    pub fn update_user_texture_subregion(&mut self, texture_id: &TextureId, start_position: [usize; 2], width: usize, height: usize, pixels: &[Color32]) {
        self.painter.update_user_texture_subregion(texture_id, start_position, width, height, pixels);
    }

}


pub fn translate_cursor(cursor_icon: egui::CursorIcon) -> glfw::StandardCursor {
    match cursor_icon {
        CursorIcon::Default => glfw::StandardCursor::Arrow,

        CursorIcon::PointingHand => glfw::StandardCursor::Hand,

        CursorIcon::ResizeHorizontal => glfw::StandardCursor::HResize,
        CursorIcon::ResizeVertical => glfw::StandardCursor::VResize,
        // TODO: GLFW doesnt have these specific resize cursors, so we'll just use the HResize and VResize ones instead
        CursorIcon::ResizeNeSw => glfw::StandardCursor::HResize,
        CursorIcon::ResizeNwSe => glfw::StandardCursor::VResize,

        CursorIcon::Text => glfw::StandardCursor::IBeam,
        CursorIcon::Crosshair => glfw::StandardCursor::Crosshair,

        CursorIcon::Grab | CursorIcon::Grabbing => glfw::StandardCursor::Hand,

        // TODO: Same for these
        CursorIcon::NotAllowed | CursorIcon::NoDrop => glfw::StandardCursor::Arrow,
        CursorIcon::Wait => glfw::StandardCursor::Arrow,
        _ => glfw::StandardCursor::Arrow,
    }
}