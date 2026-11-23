#[must_use]
pub fn python_float(number: f64) -> String {
    if number.is_finite() {
        finite_float(number)
    } else {
        non_finite_float(number)
    }
}

fn non_finite_float(number: f64) -> String {
    let text = if number.is_nan() {
        "NaN"
    } else if number > 0.0 {
        "Infinity"
    } else {
        "-Infinity"
    };
    text.to_owned()
}

fn finite_float(number: f64) -> String {
    let scientific = format!("{number:e}");
    let (mantissa, exponent) = split_scientific(&scientific);
    if (-4..16).contains(&exponent) {
        fixed_notation(number)
    } else {
        scientific_notation(mantissa, exponent)
    }
}

fn split_scientific(scientific: &str) -> (&str, i32) {
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((scientific, "0"));
    (mantissa, exponent.parse().unwrap_or(0))
}

fn scientific_notation(mantissa: &str, exponent: i32) -> String {
    let sign = if exponent < 0 { '-' } else { '+' };
    format!("{mantissa}e{sign}{:02}", exponent.unsigned_abs())
}

fn fixed_notation(number: f64) -> String {
    let plain = format!("{number}");
    if plain.contains('.') {
        plain
    } else {
        format!("{plain}.0")
    }
}
