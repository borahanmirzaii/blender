use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// Camera data block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov_y: 50.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Orbit camera for viewport navigation (not a data block — runtime only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 5.0,
            azimuth: std::f32::consts::FRAC_PI_4,
            elevation: std::f32::consts::FRAC_PI_6,
            fov_y: 50.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl OrbitCamera {
    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.elevation.cos() * self.azimuth.sin();
        let y = self.distance * self.elevation.sin();
        let z = self.distance * self.elevation.cos() * self.azimuth.cos();
        self.target + Vec3::new(x, y, z)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, aspect, self.near, self.far)
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.azimuth += dx * 0.01;
        self.elevation = (self.elevation + dy * 0.01).clamp(-1.5, 1.5);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        let view = self.view_matrix();
        let right = Vec3::new(view.col(0).x, view.col(1).x, view.col(2).x);
        let up = Vec3::new(view.col(0).y, view.col(1).y, view.col(2).y);
        let speed = self.distance * 0.002;
        self.target -= right * dx * speed;
        self.target += up * dy * speed;
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance *= 1.0 - delta * 0.001;
        self.distance = self.distance.clamp(0.1, 500.0);
    }
}
