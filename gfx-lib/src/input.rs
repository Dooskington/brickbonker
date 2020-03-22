use ::winit::{event::ElementState, event::KeyboardInput};
use std::collections::HashMap;

pub use ::winit::event::VirtualKeyCode;

pub struct InputState {
    current_keys: HashMap<VirtualKeyCode, bool>,
    pressed_keys: HashMap<VirtualKeyCode, bool>,
    released_keys: HashMap<VirtualKeyCode, bool>,
    pub input_string: String,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            current_keys: HashMap::new(),
            pressed_keys: HashMap::new(),
            released_keys: HashMap::new(),
            input_string: String::new(),
        }
    }

    pub fn clear_pressed_and_released(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }

    pub fn clear_input_string(&mut self) {
        self.input_string = String::new();
    }

    pub fn process_input(&mut self, input: &KeyboardInput) {
        let keycode: VirtualKeyCode = input.virtual_keycode.unwrap();

        match input.state {
            ElementState::Pressed => {
                if !self.is_key_held(keycode) {
                    self.pressed_keys.insert(keycode, true);
                }

                self.current_keys.insert(keycode, true);

                if keycode == VirtualKeyCode::Back {
                    self.input_string.pop();
                }

                // Add the key to the input string if possible
                if let Some(key_char) = keycode_to_char(keycode, input.modifiers.shift()) {
                    self.input_string.push(key_char);
                }
            }
            ElementState::Released => {
                self.released_keys.insert(keycode, true);
                self.current_keys.insert(keycode, false);
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        *self.pressed_keys.get(&keycode).unwrap_or(&false)
    }

    #[allow(dead_code)]
    pub fn is_key_released(&self, keycode: VirtualKeyCode) -> bool {
        *self.released_keys.get(&keycode).unwrap_or(&false)
    }

    #[allow(dead_code)]
    pub fn is_key_held(&self, keycode: VirtualKeyCode) -> bool {
        *self.current_keys.get(&keycode).unwrap_or(&false)
    }
}

fn keycode_to_char(keycode: VirtualKeyCode, is_upper: bool) -> Option<char> {
    let character = match keycode {
        VirtualKeyCode::A => 'a',
        VirtualKeyCode::B => 'b',
        VirtualKeyCode::C => 'c',
        VirtualKeyCode::D => 'd',
        VirtualKeyCode::E => 'e',
        VirtualKeyCode::F => 'f',
        VirtualKeyCode::G => 'g',
        VirtualKeyCode::H => 'h',
        VirtualKeyCode::I => 'i',
        VirtualKeyCode::J => 'j',
        VirtualKeyCode::K => 'k',
        VirtualKeyCode::L => 'l',
        VirtualKeyCode::M => 'm',
        VirtualKeyCode::N => 'n',
        VirtualKeyCode::O => 'o',
        VirtualKeyCode::P => 'p',
        VirtualKeyCode::Q => 'q',
        VirtualKeyCode::R => 'r',
        VirtualKeyCode::S => 's',
        VirtualKeyCode::T => 't',
        VirtualKeyCode::U => 'u',
        VirtualKeyCode::V => 'v',
        VirtualKeyCode::W => 'w',
        VirtualKeyCode::X => 'x',
        VirtualKeyCode::Y => 'y',
        VirtualKeyCode::Z => 'z',
        VirtualKeyCode::Space => ' ',
        VirtualKeyCode::Period => '.',
        VirtualKeyCode::Comma => ',',
        _ => return None,
    };

    if is_upper {
        return Some(character.to_ascii_uppercase());
    }

    Some(character)
}
