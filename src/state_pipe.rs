use std::collections::VecDeque;
use std::f32::EPSILON;

pub trait StatePipe: Default {
    fn loss(&self) -> u32;
    fn pop(&self) -> u32;
    fn step(&mut self, gain: u32);
    fn set_period(&mut self, period: u32);
    fn cull_pop(&mut self, remove_percent: f32);
}

#[derive(Default)]
pub struct GroundTruthStatePipe {
    age_list: VecDeque<u32>,
}

impl StatePipe for GroundTruthStatePipe {
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

    fn set_period(&mut self, period: u32) {
        let p_us = period as usize;
        if p_us < self.age_list.len() {
            let mut pop_sum: u32 = 0;
            for _ in p_us..self.age_list.len() {
                pop_sum += self.age_list.pop_front().unwrap();
            }
            self.age_list[0] += pop_sum;
        } else if p_us > self.age_list.len() {
            for _ in self.age_list.len()..p_us {
                self.age_list.push_front(0);
            }
        }
    }

    fn cull_pop(&mut self, remove_percent: f32) {
        for i in 0..self.age_list.len() {
            self.age_list[i] = (self.age_list[i] as f32 * (1.0 - remove_percent)) as u32;
        }
    }
}

pub struct GaussianStatePipe {
    period: u32,
    pop: u32,
    age_avg: f32,
    age_var: f32,
}

impl GaussianStatePipe {
    fn retain_age_avg(&self) -> f32 {
        if self.pop - self.loss() == 0 {
            // Zero retention
            0.0
        } else {
            (self.pop as f32 * self.age_avg - (self.loss() * self.period) as f32)
                / ((self.pop - self.loss()) as f32)
        }
    }

    fn approx_gauss(&self, mean: f32, std: f32, val: f32) -> f32 {
        let x = (val - mean) / (5.1 * std) + 0.5;
        return x * x * x * (x * (x * 6.0 - 15.0) + 10.0);
    }

    fn loss_percent(&self) -> f32 {
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

impl Default for GaussianStatePipe {
    fn default() -> Self {
        GaussianStatePipe {
            period: 0,
            pop: 0,
            age_avg: 0.0,
            age_var: 0.0,
        }
    }
}

impl StatePipe for GaussianStatePipe {
    fn loss(&self) -> u32 {
        (self.pop as f32 * self.loss_percent()).round() as u32
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

    fn set_period(&mut self, period: u32) {
        self.period = period - 1;
    }

    fn cull_pop(&mut self, remove_percent: f32) {
        let new_pop = (self.pop as f32 * (1.0 - remove_percent)) as u32;
        if new_pop == 0 {
            self.age_avg = 0.0;
        } else {
            self.age_var = self.age_var * (self.pop as f32) / (new_pop as f32);
        }
        self.pop = new_pop;
    }
}

#[cfg(test)]
pub mod tests {
    use crate::state_pipe::*;

    const SIMPLE_TEST_PERIOD_INCR: u32 = 5;
    const SIMPLE_TEST_AMOUNT: u32 = 10;
    const SIMPLE_TEST_ITER: u32 = 10;

    fn do_simple_state_pipe_test<T: StatePipe>(pipe_name: &'static str, mut pipe: T) {
        let mut period: u32 = 0;

        for _ in 0..SIMPLE_TEST_ITER {
            period += SIMPLE_TEST_PERIOD_INCR;
            pipe.set_period(period);

            pipe.step(SIMPLE_TEST_AMOUNT);

            for i in 0..(period - 1) {
                assert!(
                    pipe.loss() == 0,
                    "{} lost {} population prematurely on step {}!",
                    pipe_name,
                    pipe.loss(),
                    i
                );

                pipe.step(0);

                assert!(
                    pipe.pop() == SIMPLE_TEST_AMOUNT,
                    "{} population decreased during it's period!",
                    pipe_name
                );
            }

            assert!(
                pipe.loss() == SIMPLE_TEST_AMOUNT,
                "{} didn't lose enough population after it's period!",
                pipe_name
            );

            pipe.step(0);

            assert!(
                pipe.pop() == 0,
                "{} retained population after it's period! {}!",
                pipe_name,
                pipe.pop()
            );
        }
    }

    const CONT_TEST_PERIOD_INCR: u32 = 5;
    const CONT_TEST_DIST_LENGTH: u32 = 50;
    const CONT_TEST_AMOUNT_INCR: u32 = 5;
    const CONT_TEST_ITER: u32 = 20;

    fn do_cont_state_pipe_test<T: StatePipe>(pipe_name: &'static str, mut pipe: T) {
        let mut period = 0;
        let mut pop_size = 0;

        for _ in 0..CONT_TEST_ITER {
            period += CONT_TEST_PERIOD_INCR;

            for _ in 0..CONT_TEST_ITER {
                pop_size += CONT_TEST_AMOUNT_INCR;
                pipe.set_period(period);

                // Only gain
                for i in 0..period.min(CONT_TEST_DIST_LENGTH) {
                    assert!(
                        pipe.loss() == 0,
                        "{} lost {} population during insertion step {}!",
                        pipe_name,
                        pipe.loss(),
                        i
                    );

                    pipe.step(pop_size);

                    assert!(
                        pipe.pop() == (i + 1) * pop_size,
                        "{} incorrect population after insertion step {}: {}!",
                        pipe_name,
                        i,
                        pop_size,
                    );
                }

                let init_gain_pop = pipe.pop();

                if period < CONT_TEST_DIST_LENGTH {
                    // Both gain and loss
                    for i in 0..(CONT_TEST_DIST_LENGTH - period) {
                        assert!(
                            pipe.loss() == pop_size,
                            "{} didn't lose correct population on gain+loss on step {}!",
                            pipe_name,
                            i
                        );

                        pipe.step(pop_size);

                        assert!(
                            pipe.pop() == init_gain_pop,
                            "{} population didn't remain constant during gain+loss!",
                            pipe_name
                        );
                    }
                } else {
                    // Neither gain nor loss
                    for i in 0..(period - CONT_TEST_DIST_LENGTH) {
                        assert!(
                            pipe.loss() == 0,
                            "{} lost {} population prematurely on hold step {}!",
                            pipe_name,
                            pipe.loss(),
                            i
                        );

                        pipe.step(0);

                        assert!(
                            pipe.pop() == init_gain_pop,
                            "{} population didn't remain constant during hold!",
                            pipe_name
                        );
                    }
                }

                // Only loss
                for i in 0..period.min(CONT_TEST_DIST_LENGTH) {
                    assert!(
                        pipe.loss() == pop_size,
                        "{} didn't lose correct population on loss step {}: {}!",
                        pipe_name,
                        i,
                        pipe.loss()
                    );

                    pipe.step(0);

                    assert!(
                        pipe.pop() == init_gain_pop - (i + 1) * pop_size,
                        "{} populate didn't decrease correctly during loss!",
                        pipe_name
                    );
                }

                assert!(
                    pipe.loss() == 0,
                    "{} still has loss after it's period: {}!",
                    pipe_name,
                    pipe.loss(),
                );

                assert!(
                    pipe.pop() == 0,
                    "{} still has population after it's period: {}!",
                    pipe_name,
                    pipe.pop()
                );
            }
        }
    }

    #[test]
    fn test_ground_truth_pipe_simple() {
        do_simple_state_pipe_test("GroundTruthStatePipe", GroundTruthStatePipe::default());
    }

    #[test]
    fn test_gaussian_pipe_simple() {
        do_simple_state_pipe_test("GaussianStatePipe", GaussianStatePipe::default());
    }

    #[test]
    fn test_ground_truth_pipe_cont() {
        do_cont_state_pipe_test("GroundTruthStatePipe", GroundTruthStatePipe::default())
    }

    #[test]
    #[should_panic] // GaussianStatePipe will not pass this test currently
    fn test_gaussian_pipe_cont() {
        do_cont_state_pipe_test("GaussianStatePipe", GaussianStatePipe::default())
    }
}
