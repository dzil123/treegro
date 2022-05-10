use std::ops::Mul;

use crate::NUM_RESOURCES;

pub type ParamType = f32;
pub type ParamRowVector = [ParamType; NUM_RESOURCES + 1];
pub type ParamColumnVector = [ParamType; PlantFsmParams::TotalParams as usize];

pub struct ParameterMatrix {
    rows: [ParamRowVector; PlantFsmParams::TotalParams as usize],
}

impl Default for ParameterMatrix {
    fn default() -> Self {
        let mut row = [0.0_f32; NUM_RESOURCES + 1];
        row[NUM_RESOURCES] = 1.0;
        ParameterMatrix {
            rows: [row; PlantFsmParams::TotalParams as usize],
        }
    }
}

impl ParameterMatrix {
    pub fn set_offset(&mut self, param: PlantFsmParams, value: f32) {
        self.rows[param as usize][NUM_RESOURCES] = value;
    }

    pub fn map_offsets(&mut self, values: ParamColumnVector) {}
}

impl Mul<&ResourceVector> for &ParameterMatrix {
    type Output = SpecificParameterVector;

    fn mul(self, rhs: &ResourceVector) -> SpecificParameterVector {
        let result_iter = self.rows.iter().map(|row| {
            (0..(NUM_RESOURCES + 1))
                .map(|i| row[i] * rhs.columns[i])
                .sum()
        });
        let mut result_spec = SpecificParameterVector::default();
        for (i, val) in result_iter.enumerate() {
            result_spec.columns[i] = val;
        }
        result_spec
    }
}

pub struct ResourceVector {
    columns: [f32; NUM_RESOURCES as usize + 1],
}

impl Default for ResourceVector {
    fn default() -> Self {
        let mut vector = [0.0_f32; NUM_RESOURCES + 1];
        vector[NUM_RESOURCES] = 1.0;
        ResourceVector { columns: vector }
    }
}

#[derive(Default, Clone, Copy)]
pub struct SpecificParameterVector {
    columns: ParamColumnVector,
}

impl SpecificParameterVector {
    pub fn from_raw(columns: ParamColumnVector) -> Self {
        SpecificParameterVector { columns }
    }

    pub fn float_param(&self, param_type: PlantFsmParams) -> f32 {
        assert!((param_type as usize) < self.columns.len());
        self.columns[param_type as usize]
    }

    pub fn uint_param(&self, param_type: PlantFsmParams) -> u32 {
        assert!((param_type as usize) < self.columns.len());
        assert!(self.columns[param_type as usize] >= 0.0);
        self.columns[param_type as usize].trunc() as u32
    }

    pub fn total_maturation_period(&self) -> u32 {
        self.uint_param(PlantFsmParams::SeedMaturationPeriod)
            + self.uint_param(PlantFsmParams::SeedGerminationPeriod)
            + self.uint_param(PlantFsmParams::PlantMaturationPeriod)
    }
    
}

#[derive(Clone, Copy)]
pub enum PlantFsmParams {
    SeedMaturationPeriod = 0,
    SeedGerminationConditions,
    SeedGerminationPeriod,
    PlantMaturationPeriod,
    FloweringConditions,
    FloweringPeriod,
    FloweringRecoveryPeriod,
    FloweringSuccessRatio,
    DispersionRate,
    DispersionPeriod,
    DispersionRecoveryPeriod,
    MaturePlantLifePeriod,
    MinimumSurvivalConditions,
    SnagDecompositionPeriod,
    // Total non-trivial states: 9

    /*
     * WARNING!!!! DON"T PUT ANY VARIANTS AFTER THIS!!!!!!!
     */
    TotalParams,
}
