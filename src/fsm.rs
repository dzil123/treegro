use crate::param::*;
use crate::state_pipe::*;
use crate::World;

#[derive(Default)]
struct PlantStateMachine<T: StatePipe> {
    inserted_seeds: u32,
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
        self.immature_seeds
            .step(total_seeds_dispersed + self.inserted_seeds);
        self.inserted_seeds = 0;

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
        self.step(plant_fsm_params * &world.resources);
    }

    pub fn insert_seeds(&mut self, gain: u32) {
        self.inserted_seeds += gain;
    }

    pub fn immature_pop(&self) -> u32 {
        self.immature_seeds.pop()
            + self.mature_seeds.pop()
            + self.dormant_seeds
            + self.immature_plants.pop()
    }

    pub fn mature_pop(&self) -> u32 {
        self.mature_plants.pop()
    }

    pub fn snag_pop(&self) -> u32 {
        self.snags.pop()
    }

    pub fn total_pop(&self) -> u32 {
        self.immature_pop() + self.mature_pop() + self.snag_pop()
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{distrib_util::*, fsm::*};
    use std::f32::consts::{E, PI};

    fn generate_test_param_vec() -> SpecificParameterVector {
        SpecificParameterVector::from_raw([
            2.0,  // Seed mature period
            1.0,  // Seed germination conditions
            3.0,  // Seed germination period
            4.0,  // Plant maturation period
            1.0,  // Flowering conditions
            2.0,  // Flowering period
            5.0,  // Flowering recovery period
            0.8,  // Flowering success ratio
            0.27, // Dispersion rate
            2.0,  // Dispersion period
            4.0,  // Dispersion recovery period
            20.0, // Mature plant lifetime
            1.0,  // Minimum survivial conditions
            10.0, // Snag decomposition period
        ])
    }

    const FSM_TEST_START_POP: u32 = 100;

    #[test]
    fn test_fsm_continuity() {
        let mut fsm: PlantStateMachine<GroundTruthStatePipe> = PlantStateMachine::default();

        let params = generate_test_param_vec();

        fsm.insert_seeds(FSM_TEST_START_POP);

        println!(
            "        |  is  |  ms  |  ip  |  fp  |  frp  |  dp  |  drp  |  sn  |  mp  |  total  |"
        );

        for i in 0..200 {
            fsm.step(params);

            println!(
                "step {:3}:{:6} {:6} {:6} {:6}  {:6} {:6}  {:6} {:6} {:6} {:6}",
                i + 1,
                fsm.immature_seeds.pop(),
                fsm.mature_seeds.pop(),
                fsm.immature_plants.pop(),
                fsm.flowering_plants.pop(),
                fsm.flower_recovering_plants.pop(),
                fsm.dispersing_plants.pop(),
                fsm.disperse_recovering_plants.pop(),
                fsm.snags.pop(),
                fsm.mature_pop(),
                fsm.total_pop()
            );
        }
    }
}
