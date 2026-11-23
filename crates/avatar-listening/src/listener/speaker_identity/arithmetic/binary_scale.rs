const SUBNORMAL_LIFT: f64 = 18_014_398_509_481_984.0;

pub(super) fn exponent_of(value: f64) -> i32 {
    if value < f64::MIN_POSITIVE {
        return exponent_of(value * SUBNORMAL_LIFT) - 54;
    }
    let field = i32::try_from((value.to_bits() >> 52) & 0x7ff).unwrap_or(0);
    field - 1022
}

pub(super) fn power_of_two(exponent: i32) -> f64 {
    if exponent >= -1022 {
        let field = u64::try_from(exponent + 1023).unwrap_or(0);
        return f64::from_bits(field << 52);
    }
    let shift = u32::try_from(exponent + 1074).unwrap_or(0);
    f64::from_bits(1_u64 << shift)
}
