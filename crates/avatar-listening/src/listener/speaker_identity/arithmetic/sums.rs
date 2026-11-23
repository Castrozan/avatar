fn rounding_loss(total: f64, value: f64, next: f64) -> f64 {
    if total.abs() >= value.abs() {
        (total - next) + value
    } else {
        (value - next) + total
    }
}

fn corrected(total: f64, compensation: f64) -> f64 {
    if compensation != 0.0 && compensation.is_finite() {
        total + compensation
    } else {
        total
    }
}

struct Neumaier {
    total: f64,
    compensation: f64,
}

impl Neumaier {
    fn add(&mut self, value: f64) {
        let next = self.total + value;
        self.compensation += rounding_loss(self.total, value, next);
        self.total = next;
    }
}

#[must_use]
pub fn python_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut iterator = values.into_iter();
    let Some(first) = iterator.next() else {
        return 0.0;
    };
    let mut sum = Neumaier {
        total: 0.0 + first,
        compensation: 0.0,
    };
    iterator.for_each(|value| sum.add(value));
    corrected(sum.total, sum.compensation)
}
