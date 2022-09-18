use cli_clipboard::{ClipboardContext, ClipboardProvider};

pub struct EguiManager {
    screen_rect: Option<egui::Rect>,
    screen_dimensions: (f32, f32),
    events: Vec<egui::Event>,
    max_texture_side: usize,
    modifiers: egui::Modifiers,
    pixels_per_point: f32,
    start_time: std::time::Instant,

    last_known_cursor_position: egui::Pos2,
    clipboard_context: Option<ClipboardContext>,
}
impl EguiManager {

    #[must_use]
    pub fn new(window_width_px: f32, window_height_px: f32, pixels_per_point: f32, max_texture_side: usize) -> Self {
        EguiManager {
            screen_rect: Some(egui::Rect::from_two_pos(egui::pos2(0.0, 0.0), egui::pos2(window_width_px / pixels_per_point, window_height_px / pixels_per_point))),
            screen_dimensions: (window_width_px, window_height_px),
            pixels_per_point,
            start_time: std::time::Instant::now(),
            events: Vec::new(),
            modifiers: egui::Modifiers::default(),
            max_texture_side,
            last_known_cursor_position: egui::Pos2::default(),
            clipboard_context: ClipboardContext::new().ok(),
        }
    }

    pub fn pixels_per_point(&self) -> f32 {
        self.pixels_per_point
    }
    pub fn screen_dimensions(&self) -> (f32, f32) {
        self.screen_dimensions
    }




    pub fn handle_event(&mut self, event: &glfw::WindowEvent) {
        use glfw::WindowEvent as Ev;
        match *event {
            Ev::FramebufferSize(w, h) => {
                self.screen_dimensions = (w as f32, h as f32);
                self.screen_rect = Some(egui::Rect::from_two_pos(egui::pos2(0.0, 0.0), egui::pos2(w as f32 / self.pixels_per_point, h as f32 / self.pixels_per_point)));
            },
            Ev::CursorPos(x, y) => {
                self.last_known_cursor_position = egui::pos2(x as f32 / self.pixels_per_point, y as f32 / self.pixels_per_point);
                self.events.push(egui::Event::PointerMoved(self.last_known_cursor_position));
            },
            Ev::MouseButton(mouse_button, action, modifiers) => {
                self.update_modifiers(modifiers);
                let mouse_button = match mouse_button {
                    glfw::MouseButtonLeft => egui::PointerButton::Primary,
                    glfw::MouseButtonMiddle => egui::PointerButton::Middle,
                    glfw::MouseButtonRight => egui::PointerButton::Secondary,
                    glfw::MouseButton::Button4 => egui::PointerButton::Extra1,
                    _ => egui::PointerButton::Extra2,
                };
                let pressed = match action {
                    glfw::Action::Press => true,
                    glfw::Action::Release => false,
                    glfw::Action::Repeat => return,
                };
                self.events.push(egui::Event::PointerButton { pos: self.last_known_cursor_position, button: mouse_button, pressed, modifiers: self.modifiers });
            },
            Ev::Scroll(x, y) => {
                self.events.push(egui::Event::Scroll(egui::vec2(x as f32 / self.pixels_per_point, y as f32 / self.pixels_per_point)));
            },
            Ev::Char(c) => {
                self.events.push(egui::Event::Text(c.to_string()));
            },
            Ev::Key(key, _, action, modifiers) => {
                self.update_modifiers(modifiers);
                let pressed = !matches!(action, glfw::Action::Release);
                let key = match Self::translate_keycode(key) {
                    Some(key) => key,
                    None => return,
                };
                // Check for cut / copy / paste
                if pressed && self.modifiers.command_only() && key == egui::Key::X {
                    self.events.push(egui::Event::Cut);
                } else if pressed && self.modifiers.command_only() && key == egui::Key::C {
                    self.events.push(egui::Event::Copy);
                } else if pressed && self.modifiers.command_only() && key == egui::Key::V {
                    if let Some(ref mut clipboard_context) = self.clipboard_context {
                        if let Ok(contents) = clipboard_context.get_contents() {
                            self.events.push(egui::Event::Paste(contents));
                        }
                    }
                } else {
                    self.events.push(egui::Event::Key { key, pressed, modifiers: self.modifiers });
                }
            },
            _ => {},
        }
    }

    pub fn generate_raw_input(&mut self) -> egui::RawInput {
        egui::RawInput {
            screen_rect: self.screen_rect.take(),
            pixels_per_point: Some(self.pixels_per_point),
            modifiers: self.modifiers,
            max_texture_side: Some(self.max_texture_side),
            time: Some(self.start_time.elapsed().as_secs_f64()),
            predicted_dt: 1.0 / 60.0,   // Revisit this
            events: std::mem::take(&mut self.events),
            ..Default::default()
        }
    }

    fn update_modifiers(&mut self, modifiers: glfw::Modifiers) {
        self.modifiers = egui::Modifiers {
            alt: modifiers.contains(glfw::Modifiers::Alt),
            ctrl: modifiers.contains(glfw::Modifiers::Control),
            shift: modifiers.contains(glfw::Modifiers::Shift),
            mac_cmd: false,
            command: modifiers.contains(glfw::Modifiers::Control),
        };
    }

    fn translate_keycode(glfw_keycode: glfw::Key) -> Option<egui::Key> {
        match glfw_keycode {
            glfw::Key::Down => Some(egui::Key::ArrowDown),
            glfw::Key::Left => Some(egui::Key::ArrowLeft),
            glfw::Key::Right => Some(egui::Key::ArrowRight),
            glfw::Key::Up => Some(egui::Key::ArrowUp),
            glfw::Key::Escape => Some(egui::Key::Escape),
            glfw::Key::Tab => Some(egui::Key::Tab),
            glfw::Key::Backspace => Some(egui::Key::Backspace),
            glfw::Key::Enter => Some(egui::Key::Enter),
            glfw::Key::Space => Some(egui::Key::Space),
            glfw::Key::Insert => Some(egui::Key::Insert),
            glfw::Key::Delete => Some(egui::Key::Delete),
            glfw::Key::Home => Some(egui::Key::Home),
            glfw::Key::End => Some(egui::Key::End),
            glfw::Key::PageUp => Some(egui::Key::PageUp),
            glfw::Key::PageDown => Some(egui::Key::PageDown),
            glfw::Key::Kp0 | glfw::Key::Num0 => Some(egui::Key::Num0),
            glfw::Key::Kp1 | glfw::Key::Num1 => Some(egui::Key::Num1),
            glfw::Key::Kp2 | glfw::Key::Num2 => Some(egui::Key::Num2),
            glfw::Key::Kp3 | glfw::Key::Num3 => Some(egui::Key::Num3),
            glfw::Key::Kp4 | glfw::Key::Num4 => Some(egui::Key::Num4),
            glfw::Key::Kp5 | glfw::Key::Num5 => Some(egui::Key::Num5),
            glfw::Key::Kp6 | glfw::Key::Num6 => Some(egui::Key::Num6),
            glfw::Key::Kp7 | glfw::Key::Num7 => Some(egui::Key::Num7),
            glfw::Key::Kp8 | glfw::Key::Num8 => Some(egui::Key::Num8),
            glfw::Key::Kp9 | glfw::Key::Num9 => Some(egui::Key::Num9),
            glfw::Key::A => Some(egui::Key::A),
            glfw::Key::B => Some(egui::Key::B),
            glfw::Key::C => Some(egui::Key::C),
            glfw::Key::D => Some(egui::Key::D),
            glfw::Key::E => Some(egui::Key::E),
            glfw::Key::F => Some(egui::Key::F),
            glfw::Key::G => Some(egui::Key::G),
            glfw::Key::H => Some(egui::Key::H),
            glfw::Key::I => Some(egui::Key::I),
            glfw::Key::J => Some(egui::Key::J),
            glfw::Key::K => Some(egui::Key::K),
            glfw::Key::L => Some(egui::Key::L),
            glfw::Key::M => Some(egui::Key::M),
            glfw::Key::N => Some(egui::Key::N),
            glfw::Key::O => Some(egui::Key::O),
            glfw::Key::P => Some(egui::Key::P),
            glfw::Key::Q => Some(egui::Key::Q),
            glfw::Key::R => Some(egui::Key::R),
            glfw::Key::S => Some(egui::Key::S),
            glfw::Key::T => Some(egui::Key::T),
            glfw::Key::U => Some(egui::Key::U),
            glfw::Key::V => Some(egui::Key::V),
            glfw::Key::W => Some(egui::Key::W),
            glfw::Key::X => Some(egui::Key::X),
            glfw::Key::Y => Some(egui::Key::Y),
            glfw::Key::Z => Some(egui::Key::Z),
            glfw::Key::F1 => Some(egui::Key::F1),
            glfw::Key::F2 => Some(egui::Key::F2),
            glfw::Key::F3 => Some(egui::Key::F3),
            glfw::Key::F4 => Some(egui::Key::F4),
            glfw::Key::F5 => Some(egui::Key::F5),
            glfw::Key::F6 => Some(egui::Key::F6),
            glfw::Key::F7 => Some(egui::Key::F7),
            glfw::Key::F8 => Some(egui::Key::F8),
            glfw::Key::F9 => Some(egui::Key::F9),
            glfw::Key::F10 => Some(egui::Key::F10),
            glfw::Key::F11 => Some(egui::Key::F11),
            glfw::Key::F12 => Some(egui::Key::F12),
            glfw::Key::F13 => Some(egui::Key::F13),
            glfw::Key::F14 => Some(egui::Key::F14),
            glfw::Key::F15 => Some(egui::Key::F15),
            glfw::Key::F16 => Some(egui::Key::F16),
            glfw::Key::F17 => Some(egui::Key::F17),
            glfw::Key::F18 => Some(egui::Key::F18),
            glfw::Key::F19 => Some(egui::Key::F19),
            glfw::Key::F20 => Some(egui::Key::F20),
            _ => None,
        }
    }



}