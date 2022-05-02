#[derive(Clone, Default)]
pub struct Cell {
    // 0.0 == no resources
    // 1.0 == unlimited resources
    pub resources: f32,

    // 0.0 == extinct
    // 1.0 == maximum possible density
    pub density: f32,
}

impl Cell {
    pub fn step(&mut self, gr: f32, dt: f32) {
        self.density += gr * self.density * (1.0 - self.density / self.resources) * dt;
    }
}
