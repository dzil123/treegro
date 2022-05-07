use std::f32::consts::E;

/**
 * Approximation of normal CDF.
 *
 * -1.65451 is a magic constant that has the lowest deviation from the actual normal CDF.
 */
fn normal_cdf(mean: f32, std: f32, x: f32) -> f32 {
    1.0 / (1.0 + E.powf(-1.65451 * std * (x - mean)))
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

    const STD_TOO_BIG_RATIO: f32 = 5.0 / 6.0;
    const STD_TOO_BIG_POP_INCR: u32 = 50;
    const STD_TOO_BIG_ITER: u32 = 20;

    #[test]
    fn test_distrib_gen_std_too_big() {
        let mut pop_size: u32 = 0;
        for _ in 0..STD_TOO_BIG_ITER {
            pop_size += STD_TOO_BIG_POP_INCR;
            let distrib = get_pop_normal_distrib(pop_size, pop_size as f32 * STD_TOO_BIG_RATIO);
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
            let distrib = get_pop_normal_distrib(pop_size, pop_size as v32)
            // TODO
        }
    }
}
