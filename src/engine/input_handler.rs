use std::collections::HashMap;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug, PartialEq, Eq)]
enum InputState {
    Pressed,
    Released,
    Held,
}

#[derive(Debug)]
struct MouseState {
    button_state: HashMap<MouseButton, InputState>,
    current_position: (f32, f32),
    previous_position: (f32, f32),
}

#[derive(Debug)]
pub struct InputHandler {
    keyboard_state: HashMap<KeyCode, InputState>,
    mouse_state: MouseState,
}

impl InputHandler {
    pub(crate) fn new() -> Self {
        Self {
            keyboard_state: HashMap::new(),
            mouse_state: MouseState::new(),
        }
    }

    pub(crate) fn update(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => {
                self.update_window_event(event);
            }

            Event::DeviceEvent { event, .. } => {
                self.update_device_event(event);
            }

            _ => (),
        }
    }

    fn update_window_event(&mut self, window_event: &WindowEvent) {
        match window_event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => self.update_key_press(*key_code),
                ElementState::Released => self.update_key_release(*key_code),
            },

            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_state.update_input(state, button);
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_state.update_position(position);
            }

            _ => (),
        }
    }

    fn update_device_event(&mut self, _device_event: &DeviceEvent) {}

    pub(crate) fn step(&mut self) {
        self.keyboard_state = self
            .keyboard_state
            .iter()
            .filter_map(|(key_code, key_state)| match key_state {
                InputState::Pressed => Some((*key_code, InputState::Held)),
                InputState::Held => Some((*key_code, InputState::Held)),
                _ => None,
            })
            .collect();

        self.mouse_state.step();
    }

    fn update_key_press(&mut self, key_code: KeyCode) {
        self.keyboard_state.insert(key_code, InputState::Pressed);
    }

    fn update_key_release(&mut self, key_code: KeyCode) {
        self.keyboard_state.insert(key_code, InputState::Released);
    }

    pub fn key_pressed(&self, key_code: KeyCode) -> bool {
        if let Some(key_state) = self.keyboard_state.get(&key_code) {
            return *key_state == InputState::Pressed;
        }

        false
    }

    pub fn key_released(&self, key_code: KeyCode) -> bool {
        if let Some(key_state) = self.keyboard_state.get(&key_code) {
            return *key_state == InputState::Released;
        }

        false
    }

    pub fn key_held(&self, key_code: KeyCode) -> bool {
        if let Some(key_state) = self.keyboard_state.get(&key_code) {
            return *key_state == InputState::Held || *key_state == InputState::Pressed;
        }

        false
    }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_state.button_pressed(button)
    }

    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_state.button_released(button)
    }

    pub fn mouse_held(&self, button: MouseButton) -> bool {
        self.mouse_state.button_held(button)
    }

    pub fn mouse_diff(&self) -> (f32, f32) {
        self.mouse_state.mouse_diff()
    }
}

impl MouseState {
    fn new() -> Self {
        Self {
            button_state: HashMap::new(),
            current_position: (0.0, 0.0),
            previous_position: (0.0, 0.0),
        }
    }

    fn update_input(&mut self, state: &ElementState, button: &MouseButton) {
        match state {
            ElementState::Pressed => self.button_state.insert(*button, InputState::Pressed),
            ElementState::Released => self.button_state.insert(*button, InputState::Released),
        };
    }

    fn update_position(&mut self, position: &PhysicalPosition<f64>) {
        self.current_position = (position.x as f32, position.y as f32);
    }

    fn step(&mut self) {
        self.button_state = self
            .button_state
            .iter()
            .filter_map(|(button, button_state)| match button_state {
                InputState::Pressed => Some((*button, InputState::Held)),
                InputState::Held => Some((*button, InputState::Held)),
                _ => None,
            })
            .collect();

        self.previous_position = self.current_position;
    }

    fn button_pressed(&self, button: MouseButton) -> bool {
        if let Some(button_state) = self.button_state.get(&button) {
            return *button_state == InputState::Pressed;
        }

        false
    }

    fn button_released(&self, button: MouseButton) -> bool {
        if let Some(button_state) = self.button_state.get(&button) {
            return *button_state == InputState::Released;
        }

        false
    }

    fn button_held(&self, button: MouseButton) -> bool {
        if let Some(button_state) = self.button_state.get(&button) {
            return *button_state == InputState::Pressed || *button_state == InputState::Held;
        }

        false
    }

    fn mouse_diff(&self) -> (f32, f32) {
        (
            self.current_position.0 - self.previous_position.0,
            self.current_position.1 - self.previous_position.1,
        )
    }
}
