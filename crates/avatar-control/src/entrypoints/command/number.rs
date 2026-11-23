use serde_json::{Number, Value};

const SAFE_INTEGER_LIMIT: f64 = 9_007_199_254_740_992.0;

fn is_javascript_space(character: char) -> bool {
    character.is_whitespace() || character == '\u{feff}'
}

fn radix_digits(text: &str) -> Option<(u32, &str)> {
    [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ]
    .into_iter()
    .find_map(|(prefix, radix)| text.strip_prefix(prefix).map(|digits| (radix, digits)))
}

fn radix_value(radix: u32, digits: &str) -> f64 {
    let accumulate = |total: f64, digit: char| {
        digit
            .to_digit(radix)
            .map(|value| total * f64::from(radix) + f64::from(value))
    };
    if digits.is_empty() {
        return f64::NAN;
    }
    digits.chars().try_fold(0.0, accumulate).unwrap_or(f64::NAN)
}

fn decimal_value(text: &str) -> f64 {
    let unsigned = text.trim_start_matches(['+', '-']);
    let sign_count = text.len() - unsigned.len();
    let starts_numeric = unsigned.starts_with(|first: char| first.is_ascii_digit() || first == '.');
    if sign_count > 1 || !starts_numeric {
        return f64::NAN;
    }
    text.parse().unwrap_or(f64::NAN)
}

#[must_use]
pub fn javascript_number(text: &str) -> f64 {
    let trimmed = text.trim_matches(is_javascript_space);
    match trimmed {
        "" => 0.0,
        "Infinity" | "+Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ => radix_digits(trimmed).map_or_else(
            || decimal_value(trimmed),
            |(radix, digits)| radix_value(radix, digits),
        ),
    }
}

#[must_use]
pub fn json_number(value: f64) -> Value {
    let integral = (value.fract() == 0.0 && value.abs() < SAFE_INTEGER_LIMIT)
        .then(|| format!("{value}").parse::<i64>().ok())
        .flatten();
    integral.map_or_else(|| fraction(value), Value::from)
}

fn fraction(value: f64) -> Value {
    Number::from_f64(value).map_or(Value::Null, Value::Number)
}
