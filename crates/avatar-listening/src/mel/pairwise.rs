use num_traits::AsPrimitive;

const PAIRWISE_BLOCK: usize = 128;
const LANES: usize = 8;
const VARIANCE_EPSILON: f32 = 1e-7;

fn pairwise_sum(values: &[f32]) -> f32 {
    let count = values.len();
    if count < LANES {
        return values.iter().fold(0.0, |total, &value| total + value);
    }
    if count <= PAIRWISE_BLOCK {
        return eight_lane_sum(values);
    }
    let mut half = count / 2;
    half -= half % LANES;
    pairwise_sum(&values[..half]) + pairwise_sum(&values[half..])
}

fn eight_lane_sum(values: &[f32]) -> f32 {
    let mut lanes = [0.0_f32; LANES];
    lanes.copy_from_slice(&values[..LANES]);
    let blocked_end = values.len() - values.len() % LANES;
    let lanes = values[LANES..blocked_end]
        .chunks_exact(LANES)
        .fold(lanes, add_block);
    let tail = &values[blocked_end..];
    tail.iter()
        .fold(combine_lanes(&lanes), |total, &value| total + value)
}

fn add_block(lanes: [f32; LANES], block: &[f32]) -> [f32; LANES] {
    std::array::from_fn(|lane| lanes[lane] + block[lane])
}

fn combine_lanes(lanes: &[f32; LANES]) -> f32 {
    ((lanes[0] + lanes[1]) + (lanes[2] + lanes[3]))
        + ((lanes[4] + lanes[5]) + (lanes[6] + lanes[7]))
}

pub(super) fn normalize(audio: &[f32]) -> Vec<f32> {
    let count: f32 = audio.len().as_();
    let mean = pairwise_sum(audio) / count;
    let deviations: Vec<f32> = audio.iter().map(|&sample| sample - mean).collect();
    let scale = standard_deviation(&deviations, count);
    deviations
        .iter()
        .map(|&deviation| deviation / scale)
        .collect()
}

fn standard_deviation(deviations: &[f32], count: f32) -> f32 {
    let squares: Vec<f32> = deviations
        .iter()
        .map(|&deviation| deviation * deviation)
        .collect();
    (pairwise_sum(&squares) / count + VARIANCE_EPSILON).sqrt()
}
