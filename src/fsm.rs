use std::collections::VecDeque;
use std::f32::EPSILON;

use crate::param::*;
use crate::World;

trait StatePipe {
    fn new(period: u32) -> Self;
    fn loss(&self) -> u32;
    fn pop(&self) -> u32;
    fn step(&mut self, gain: u32);
    fn set_period(&mut self, period: u32);
    fn cull_pop(&mut self, remove_percent: f32);
}

#[derive(Default)]
struct GroundTruthStatePipe {
    age_list: VecDeque<u32>,
}

impl StatePipe for GroundTruthStatePipe {
    fn new(period: u32) -> Self {
        let mut queue = VecDeque::with_capacity(period as usize);
        for _ in 0..period {
            queue.push_back(0);
        }
        GroundTruthStatePipe { age_list: queue }
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

struct GaussianStatePipe {
    period: u32,
    pop: u32,
    age_avg: f32,
    age_var: f32,
}

impl GaussianStatePipe {
    fn retain_age_avg(&self) -> f32 {
        (self.pop as f32 * self.age_avg - (self.loss() * self.period) as f32)
            / ((self.pop - self.loss()) as f32)
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
    fn new(period: u32) -> Self {
        GaussianStatePipe {
            period: period,
            pop: 0,
            age_avg: 0.0,
            age_var: 0.0,
        }
    }

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
        self.period = period;
    }

    fn cull_pop(&mut self, remove_percent: f32) {
        self.pop = (self.pop as f32 * (1.0 - remove_percent)) as u32;
        self.age_var *= 1.0 / (1.0 - remove_percent);
    }
}

#[derive(Default)]
struct PlantStateMachine<T: StatePipe + Default> {
    immature_seeds: T,
    mature_seeds: T,
    dormant_seeds: u32,
    immature_plants: T,
    mature_plants: T,
    ready_to_flower_plants: u32,
    flowering_plants: T,
    flower_recovering_plants: T,
    dispersing_plants: T,
    disperse_recovering_plants: T,
    snags: T,
}

impl<T: StatePipe + Default> PlantStateMachine<T> {
    fn update_states(&mut self, param_vec: &SpecificParameterVector) {
        self.immature_seeds
            .set_period(param_vec.uint_param(PlantFsmParams::SeedMaturationPeriod));
        self.mature_seeds
            .set_period(param_vec.uint_param(PlantFsmParams::SeedGerminationPeriod));
        self.immature_plants
            .set_period(param_vec.uint_param(PlantFsmParams::PlantMaturationPeriod));
        self.mature_plants
            .set_period(param_vec.uint_param(PlantFsmParams::MaturePlantLifePeriod));
        self.flowering_plants
            .set_period(param_vec.uint_param(PlantFsmParams::FloweringPeriod));
        self.flower_recovering_plants
            .set_period(param_vec.uint_param(PlantFsmParams::FloweringRecoveryPeriod));
        self.dispersing_plants
            .set_period(param_vec.uint_param(PlantFsmParams::DispersionPeriod));
        self.disperse_recovering_plants
            .set_period(param_vec.uint_param(PlantFsmParams::DispersionRecoveryPeriod));
        self.snags
            .set_period(param_vec.uint_param(PlantFsmParams::SnagDecompositionPeriod));
    }

    // An unused variable in this function is a great indicator that the developer forgot
    // to connect something in the state machine!
    #[deny(unused_variables)]
    fn step(&mut self, param_vec: SpecificParameterVector) {
        self.update_states(&param_vec);

        let matured_seeds = self.immature_seeds.loss();
        let germinated_seeds = self.mature_seeds.loss();
        let matured_plants = self.immature_plants.loss();
        let dead_plants = self.mature_plants.loss();
        let flowered_plants = self.flowering_plants.loss();
        let dispersed_plants = self.dispersing_plants.loss();
        let total_seeds_dispersed = (self.dispersing_plants.pop() as f32
            * param_vec.float_param(PlantFsmParams::DispersionRate))
            as u32;
        let recovered_plants =
            self.flower_recovering_plants.loss() + self.disperse_recovering_plants.loss();

        // Process mature plant mortality
        let mature_die_percent = dead_plants as f32 / (self.mature_plants.pop() as f32);
        self.flowering_plants.cull_pop(mature_die_percent);
        self.dispersing_plants.cull_pop(mature_die_percent);
        self.flower_recovering_plants.cull_pop(mature_die_percent);
        self.disperse_recovering_plants.cull_pop(mature_die_percent);
        self.ready_to_flower_plants =
            (self.ready_to_flower_plants as f32 * (1.0 - mature_die_percent)) as u32;
        self.snags.step(dead_plants);

        // Process new seeds
        self.immature_seeds.step(total_seeds_dispersed);

        // Process ready-to-germinate seeds
        if param_vec.float_param(PlantFsmParams::SeedGerminationConditions) < 0.0 {
            // Not germinating conditions
            self.dormant_seeds += matured_seeds;
        } else {
            // Germinating conditions
            self.mature_seeds.step(matured_seeds + self.dormant_seeds);
            self.dormant_seeds = 0;
        }

        // Process germinated seeds
        self.immature_plants.step(germinated_seeds);

        // Process just matured plants
        self.mature_plants.step(matured_plants);
        self.ready_to_flower_plants += matured_plants;

        // Process recovered plants
        self.ready_to_flower_plants += recovered_plants;

        // Process ready-to-flower plants
        if param_vec.float_param(PlantFsmParams::FloweringConditions) < 0.0 {
            // Not flowering conditions, do nothing
        } else {
            self.flowering_plants.step(self.ready_to_flower_plants);
            self.ready_to_flower_plants = 0;
        }

        // Process flowering success
        let flower_success_pop = (param_vec.float_param(PlantFsmParams::FloweringSuccessRatio)
            * (flowered_plants as f32)) as u32;
        self.dispersing_plants.step(flower_success_pop);

        // Process flowering failure
        self.flower_recovering_plants
            .step(flowered_plants - flower_success_pop);

        // Process dispersal recovery
        self.disperse_recovering_plants.step(dispersed_plants);
    }

    pub fn update(&mut self, world: &World, plant_fsm_params: &ParameterMatrix) {
        assert!(
            plant_fsm_params.result_len() == PlantFsmParams::TotalParams as usize,
            "Passed plant fsm paramter matrix doesn't include required amount of paramters: {}!",
            PlantFsmParams::TotalParams as usize
        );
        self.step(plant_fsm_params * &world.resources);
    }
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn test_ground_truth_pipe() {}
}
