use egui::lerp;
use ultraviolet::{Mat4, Vec4};

#[derive(Clone, Default)]
pub struct Cell {
    // 0.0 == no resources
    // 1.0 == unlimited resources
    pub resources: Vec4,

    // 0.0 == extinct
    // 1.0 == maximum possible density
    pub density: Vec4,
}

impl Cell {
    pub fn step(&mut self, mat: Mat4, dt: f32) {
        // let rate = mat * self.density;
        // let d_density_dt = rate * self.density * (Vec4::one() - self.density / self.resources);
        // self.density += d_density_dt * dt;

        let new_density = mat * self.density;
        self.density = lerp(self.density..=new_density, dt);
        // self.density.clamp(Vec4::zero(), Vec4::one());
    }
}
