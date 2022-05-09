use std::f32::consts::E;

const NORMAL_CDF_APPROX_CONST: f32 = 1.65451;

/**
 * Approximation of normal CDF.
 *
 * -1.65451 is a magic constant that has the lowest deviation from the actual normal CDF.
 */
fn normal_cdf(mean: f32, std: f32, x: f32) -> f32 {
    1.0 / (1.0 + E.powf((-NORMAL_CDF_APPROX_CONST / std) * (x - mean)))
}

fn normal_prob(mean: f32, std: f32, min: f32, max: f32) -> f32 {
    normal_cdf(mean, std, max) - normal_cdf(mean, std, min)
}

fn get_pop_normal_distrib(pop_size: u32, age_std: f32) -> Vec<u32> {
    // Find maximum negative Z where the PDF is less than 0.5 e.g. rounds down to zero
    let mut neg_age: i32 = 0;
    while normal_prob(
        0.0,
        age_std,
        ((neg_age - 1) as f32 + 0.5) / age_std,
        (neg_age as f32 + 0.5) / age_std,
    ) * pop_size as f32
        >= 0.5
    {
        neg_age -= 1;
    }

    if neg_age == 0 {
        // Age standard deviation is too large, discrete CDF becomes linear
        (0..pop_size).map(|_| 1).collect::<Vec<u32>>()
    } else if neg_age == 1 {
        // Age standard deviation is too small, discrete CDF becomes step function
        vec![pop_size; 1]
    } else {
        // Can construct something resembling a normal distribution
        (neg_age..=-neg_age)
            .map(|age| {
                (normal_prob(
                    0.0,
                    age_std,
                    (age as f32 + 0.5) / age_std,
                    ((age + 1) as f32 + 0.5) / age_std,
                ) * pop_size as f32) as u32
            })
            .collect()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::distrib_util::*;

    fn calc_distrib_pop_minimum(std: f32) -> u32 {
        let r = NORMAL_CDF_APPROX_CONST / (std * 2.0);
        let min_pop = 0.5 * (E.powf(r) + 1.0) / (E.powf(r) - 1.0);
        min_pop.ceil() as u32
    }

    const STD_TOO_BIG_STD_INCR: f32 = 2.0;
    const STD_TOO_BIG_ITER: u32 = 20;

    #[test]
    fn test_distrib_gen_std_way_too_big() {
        let mut std: f32 = 0.0;
        for _ in 0..STD_TOO_BIG_ITER {
            std += STD_TOO_BIG_STD_INCR;
            let pop_size = calc_distrib_pop_minimum(std);
            println!("min pop size is {} for std {}", pop_size, std);
            let distrib = get_pop_normal_distrib(pop_size, std);
            assert!(
                distrib.iter().all(|pop| *pop == 1),
                "Failed to generate flat distribution with population size: {}!",
                pop_size
            );
        }
    }

    const STD_TOO_SMALL_RATIO: f32 = 5.0 / 6.0;
    const STD_TOO_SMALL_POP_INCR: u32 = 50;
    const STD_TOO_SMALL_ITER: u32 = 20;

    #[test]
    fn test_distrib_gen_std_too_small() {
        let mut pop_size: u32 = 0;
        for _ in 0..STD_TOO_SMALL_ITER {
            pop_size += STD_TOO_SMALL_POP_INCR;
            let distrib = get_pop_normal_distrib(pop_size, pop_size as f32);
            // TODO
        }
    }
}
