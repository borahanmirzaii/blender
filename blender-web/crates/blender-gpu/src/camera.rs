use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// Orbit camera for the 3D viewport.
/// Supports orbit, pan, zoom — matching Blender's viewport navigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitCamera {
    /// Point the camera orbits around
    pub target: Vec3,
    /// Distance from target
    pub distance: f32,
    /// Horizontal angle in radians
    pub azimuth: f32,
    /// Vertical angle in radians (clamped to avoid gimbal lock)
    pub elevation: f32,
    /// Field of view in radians
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 10.0,
            azimuth: std::f32::consts::FRAC_PI_4,
            elevation: std::f32::consts::FRAC_PI_6,
            fov: 50.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl OrbitCamera {
    /// Get the camera's world position.
    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.elevation.cos() * self.azimuth.sin();
        let y = self.distance * self.elevation.sin();
        let z = self.distance * self.elevation.cos() * self.azimuth.cos();
        self.target + Vec3::new(x, y, z)
    }

    /// Compute the view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }

    /// Compute the projection matrix.
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect_ratio, self.near, self.far)
    }

    /// Orbit: rotate around the target.
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        self.azimuth += delta_x * 0.01;
        self.elevation = (self.elevation + delta_y * 0.01).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
    }

    /// Pan: move the target in the view plane.
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let view = self.view_matrix();
        let right = Vec3::new(view.col(0).x, view.col(1).x, view.col(2).x);
        let up = Vec3::new(view.col(0).y, view.col(1).y, view.col(2).y);

        let speed = self.distance * 0.002;
        self.target -= right * delta_x * speed;
        self.target += up * delta_y * speed;
    }

    /// Zoom: change distance to target.
    pub fn zoom(&mut self, delta: f32) {
        self.distance *= 1.0 - delta * 0.001;
        self.distance = self.distance.clamp(0.1, 500.0);
    }

    /// Get camera data packed for GPU uniform buffer.
    pub fn to_uniforms(&self, aspect_ratio: f32) -> CameraUniforms {
        let pos = self.position();
        CameraUniforms {
            view: self.view_matrix(),
            projection: self.projection_matrix(aspect_ratio),
            camera_pos: pos,
            _pad: 0.0,
        }
    }
}

/// GPU uniform buffer layout for camera data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CameraUniforms {
    pub view: Mat4,
    pub projection: Mat4,
    pub camera_pos: Vec3,
    pub _pad: f32,
}
