trait State {
    fn new(period: u32) -> Self;
    fn loss(&self) -> u32;
    fn pop(&self) -> u32;
    fn step(&mut self, gain: u32);
}

struct GaussianState {
    period: u32,
    pop: u32,
    age_avg: f32,
    age_var: f32,
}

impl GaussianState {
    fn retain_age_avg(&self) -> f32 {
        (self.pop as f32 * self.age_avg - (self.loss() * self.period) as f32)
            / ((self.pop - self.loss()) as f32)
    }

    // TODO: Implement these
    fn smoothstep(&self, min_edge: f32, max_edge: f32, val: f32) -> f32 {}

    fn lose_percent(&self) -> f32 {}
}

impl State for GaussianState {
    fn new(period: u32) -> Self {
        GaussianState {
            period: period,
            pop: 0,
            age_avg: 0.0,
            age_var: 0.0,
        }
    }

    fn loss(&self) -> u32 {
        (self.pop as f32 * self.lose_percent()).round() as u32
    }

    fn pop(&self) -> u32 {
        self.pop
    }

    fn step(&mut self, gain: u32) {
        let next_pop = self.pop + gain - self.loss();

        let next_age_avg =
            ((self.pop - self.loss()) as f32 * (self.age_avg + 1.0)) / (next_pop as f32);

        let loss_var_delta = self.loss() as f32 * (self.period as f32 - self.age_avg).powf(2.0);

        let avg_sft_var_delta = (self.pop - self.loss()) as f32
            * (next_age_avg - 1.0 - self.age_avg)
            * (next_age_avg - 1.0 + self.age_avg - 2.0 * self.retain_age_avg());

        let gain_var_delta = gain as f32 * next_age_avg.powf(2.0);

        let next_age_var = self.age_var
            + (gain_var_delta + avg_sft_var_delta - loss_var_delta) / next_pop as f32
            + avg_sft_var_delta;

        self.pop = next_pop;
        self.age_avg = next_age_avg;
        self.age_var = next_age_var;
    }
}
