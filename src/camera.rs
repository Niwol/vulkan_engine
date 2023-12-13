use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec3};
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

pub trait Camera3DController {
    fn update_camera(&mut self, input: &WinitInputHelper, camera: &mut Camera3D, delta_time: f32);
}

pub struct Camera3D {
    position: Vec3,
    front: Vec3,
    right: Vec3,
    up: Vec3,

    world_up: Vec3,

    yaw: f32,
    pitch: f32,
}

impl Camera3D {
    pub fn new(position: Vec3, yaw: f32, pitch: f32, world_up: Vec3) -> Self {
        let world_up = world_up.normalize();

        let mut camera = Self {
            position,
            front: Vec3::ZERO,
            right: Vec3::ZERO,
            up: Vec3::ZERO,

            world_up,

            yaw,
            pitch,
        };

        camera.update_camera_vectors();

        camera
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn front(&self) -> Vec3 {
        self.front
    }

    pub fn right(&self) -> Vec3 {
        self.right
    }

    pub fn up(&self) -> Vec3 {
        self.up
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn move_up(&mut self, amount: f32) {
        self.position += self.up * amount;
    }

    pub fn move_world_up(&mut self, amount: f32) {
        self.position += self.world_up * amount;
    }

    pub fn move_world_down(&mut self, amount: f32) {
        self.position -= self.world_up * amount;
    }

    pub fn move_down(&mut self, amount: f32) {
        self.position -= self.up * amount;
    }

    pub fn move_left(&mut self, amount: f32) {
        self.position -= self.right * amount;
    }

    pub fn move_right(&mut self, amount: f32) {
        self.position += self.right * amount;
    }

    pub fn move_forwards(&mut self, amount: f32) {
        self.position += self.front * amount;
    }

    pub fn move_xz_forwards(&mut self, amount: f32) {
        self.position += Vec3::new(self.front.x, 0.0, self.front.z).normalize() * amount;
    }

    pub fn move_backwards(&mut self, amount: f32) {
        self.position -= self.front * amount;
    }

    pub fn move_xz_backwards(&mut self, amount: f32) {
        self.position -= Vec3::new(self.front.x, 0.0, self.front.z).normalize() * amount;
    }

    pub fn update_yaw(&mut self, amount: f32) {
        self.yaw += amount;
    }

    pub fn update_pitch(&mut self, amount: f32) {
        self.pitch += amount;
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
        self.update_camera_vectors();
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
        self.update_camera_vectors();
    }

    pub fn set_pitch_and_yaw(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch;
        self.update_camera_vectors();
    }

    pub(crate) fn get_view(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.front, self.up)
    }

    fn update_camera_vectors(&mut self) {
        let front_y = self.pitch.sin();

        let pitch_cos = self.pitch.cos();
        let front_x = self.yaw.cos() * pitch_cos;
        let front_z = self.yaw.sin() * pitch_cos;
        self.front = Vec3::new(front_x, front_y, front_z);
        self.right = self.front.cross(self.world_up);
        self.up = self.right.cross(self.front);
    }
}

pub struct DebugCamera3DController {
    camera_speed: f32,
    mouse_sensitivity: f32,
}

impl DebugCamera3DController {
    pub fn new() -> Self {
        Self {
            camera_speed: 10.0,
            mouse_sensitivity: 0.5,
        }
    }

    pub fn set_camera_speed(&mut self, camera_speed: f32) {
        self.camera_speed = camera_speed;
    }

    pub fn set_mouse_sensitivity(&mut self, mouse_sensitivity: f32) {
        self.mouse_sensitivity = mouse_sensitivity;
    }
}

impl Camera3DController for DebugCamera3DController {
    fn update_camera(&mut self, input: &WinitInputHelper, camera: &mut Camera3D, delta_time: f32) {
        if input.key_held(KeyCode::KeyW) {
            camera.move_xz_forwards(self.camera_speed * delta_time);
        }
        if input.key_held(KeyCode::KeyS) {
            camera.move_xz_backwards(self.camera_speed * delta_time);
        }
        if input.key_held(KeyCode::KeyA) {
            camera.move_left(self.camera_speed * delta_time);
        }
        if input.key_held(KeyCode::KeyD) {
            camera.move_right(self.camera_speed * delta_time);
        }
        if input.key_held(KeyCode::Space) {
            camera.move_world_up(self.camera_speed * delta_time);
        }
        if input.key_held(KeyCode::ControlLeft) {
            camera.move_world_down(self.camera_speed * delta_time);
        }

        if input.mouse_held(0) {
            let (mouse_diff_x, mouse_diff_y) = input.mouse_diff();
            let mut yaw = camera.yaw();
            let mut pitch = camera.pitch();

            yaw += mouse_diff_x * self.mouse_sensitivity * delta_time;
            pitch -= mouse_diff_y * self.mouse_sensitivity * delta_time;

            pitch = pitch.clamp(-FRAC_PI_2 + 0.1, FRAC_PI_2 - 0.1);

            camera.set_pitch_and_yaw(yaw, pitch);
        }
    }
}
