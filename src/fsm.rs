use std::collections::VecDeque;
use std::f32::EPSILON;

trait State {
    fn new(period: u32) -> Self;
    fn loss(&self) -> u32;
    fn pop(&self) -> u32;
    fn step(&mut self, gain: u32);
}

struct GroundTruthState {
    age_list: VecDeque<u32>,
}

impl State for GroundTruthState {
    fn new(period: u32) -> Self {
        let mut queue = VecDeque::with_capacity(period as usize);
        for _ in 0..period {
            queue.push_back(0);
        }
        GroundTruthState { age_list: queue }
    }

    fn loss(&self) -> u32 {
        self.age_list[0]
    }

    fn pop(&self) -> u32 {
        self.age_list.iter().sum()
    }

    fn step(&mut self, gain: u32) {
        self.age_list.pop_front();
        self.age_list.push_back(gain);
    }
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

    fn approx_gauss(&self, mean: f32, std: f32, val: f32) -> f32 {
        let x = (val - mean) / (5.1 * std) + 0.5;
        return x * x * x * (x * (x * 6.0 - 15.0) + 10.0);
    }

    fn lose_percent(&self) -> f32 {
        // FIXME: self.age_var.abs()? Needs testing
        if self.age_var < EPSILON {
            if self.age_avg >= self.period as f32 {
                1.0
            } else {
                0.0
            }
        } else {
            1.0 - self.approx_gauss(self.age_avg, self.age_var.sqrt(), self.period as f32)
        }
    }
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

        let next_age_avg = {
            if next_pop == 0 {
                0.0
            } else {
                ((self.pop - self.loss()) as f32 * (self.age_avg + 1.0)) / (next_pop as f32)
            }
        };

        let loss_var_delta = self.loss() as f32 * (self.period as f32 - self.age_avg).powf(2.0);

        let avg_sft_var_delta = (self.pop - self.loss()) as f32
            * (next_age_avg - 1.0 - self.age_avg)
            * (next_age_avg - 1.0 + self.age_avg - 2.0 * self.retain_age_avg());

        let gain_var_delta = gain as f32 * next_age_avg.powf(2.0);

        let next_age_var = {
            if next_pop == 0 {
                0.0
            } else {
                self.age_var
                    + (gain_var_delta + avg_sft_var_delta - loss_var_delta) / next_pop as f32
            }
        };

        self.pop = next_pop;
        self.age_avg = next_age_avg;
        self.age_var = next_age_var;
    }
}
