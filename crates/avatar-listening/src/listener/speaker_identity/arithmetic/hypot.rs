use super::binary_scale::{exponent_of, power_of_two};

const SPLIT_LIMIT_EXPONENT: i32 = -1023;

struct DoubleLength {
    high: f64,
    low: f64,
}

fn fast_sum(first: f64, second: f64) -> DoubleLength {
    let high = first + second;
    DoubleLength {
        high,
        low: (first - high) + second,
    }
}

fn product(first: f64, second: f64) -> DoubleLength {
    let high = first * second;
    DoubleLength {
        high,
        low: first.mul_add(second, -high),
    }
}

struct Accumulator {
    checksum: f64,
    fraction_product: f64,
    fraction_sum: f64,
}

impl Accumulator {
    fn new() -> Self {
        Self {
            checksum: 1.0,
            fraction_product: 0.0,
            fraction_sum: 0.0,
        }
    }

    fn add_product(&mut self, first: f64, second: f64) {
        let pair = product(first, second);
        let sum = fast_sum(self.checksum, pair.high);
        self.checksum = sum.high;
        self.fraction_product += pair.low;
        self.fraction_sum += sum.low;
    }

    fn residual(&self) -> f64 {
        self.checksum - 1.0 + (self.fraction_product + self.fraction_sum)
    }
}

fn scaled_norm(values: &[f64], scale: f64) -> f64 {
    let mut total = Accumulator::new();
    for value in values {
        total.add_product(value * scale, value * scale);
    }
    let estimate = total.residual().sqrt();
    total.add_product(-estimate, estimate);
    estimate + total.residual() / (2.0 * estimate)
}

fn lifted_norm(values: &[f64], maximum: f64) -> f64 {
    let lifted: Vec<f64> = values
        .iter()
        .map(|value| value / f64::MIN_POSITIVE)
        .collect();
    f64::MIN_POSITIVE * vector_norm(&lifted, maximum / f64::MIN_POSITIVE)
}

fn vector_norm(values: &[f64], maximum: f64) -> f64 {
    if maximum == 0.0 || values.len() <= 1 {
        return maximum;
    }
    let exponent = exponent_of(maximum);
    if exponent < SPLIT_LIMIT_EXPONENT {
        return lifted_norm(values, maximum);
    }
    let scale = power_of_two(-exponent);
    scaled_norm(values, scale) / scale
}

fn is_infinite_or_nan(maximum: f64, magnitudes: &[f64]) -> Option<f64> {
    if maximum.is_infinite() {
        return Some(maximum);
    }
    magnitudes
        .iter()
        .any(|value| value.is_nan())
        .then_some(f64::NAN)
}

#[must_use]
pub fn python_hypot(coordinates: &[f64]) -> f64 {
    let magnitudes: Vec<f64> = coordinates.iter().map(|value| value.abs()).collect();
    let maximum = magnitudes.iter().copied().fold(0.0, f64::max);
    is_infinite_or_nan(maximum, &magnitudes).unwrap_or_else(|| vector_norm(&magnitudes, maximum))
}
