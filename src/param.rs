use std::ops::Mul;

use crate::NUM_RESOURCES;

pub struct ParameterMatrix {
    rows: Vec<[f32; NUM_RESOURCES + 1]>,
}

impl ParameterMatrix {
    pub fn new(num_rows: usize) -> Self {
        ParameterMatrix {
            rows: (0..num_rows).map(|_| [0.0; 5]).collect(),
        }
    }

    pub fn result_len(&self) -> usize {
        self.rows.len()
    }
}

impl Mul<&ResourceVector> for &ParameterMatrix {
    type Output = SpecificParameterVector;

    fn mul(self, rhs: &ResourceVector) -> SpecificParameterVector {
        let result_vec = self
            .rows
            .iter()
            .map(|row| {
                (0..(NUM_RESOURCES + 1))
                    .map(|i| row[i] * rhs.columns[i])
                    .sum()
            })
            .collect::<Vec<f32>>();
        SpecificParameterVector {
            columns: result_vec,
        }
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

pub struct SpecificParameterVector {
    columns: Vec<f32>,
}

impl SpecificParameterVector {
    pub fn float_param(&self, param_type: PlantFsmParams) -> f32 {
        assert!((param_type as usize) < self.columns.len());
        self.columns[param_type as usize]
    }

    pub fn uint_param(&self, param_type: PlantFsmParams) -> u32 {
        assert!((param_type as usize) < self.columns.len());
        assert!(self.columns[param_type as usize] >= 0.0);
        self.columns[param_type as usize].trunc() as u32
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
    TotalParams,
}
