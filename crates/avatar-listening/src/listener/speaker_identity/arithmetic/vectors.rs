use super::hypot::python_hypot;
use super::sums::python_sum;

#[must_use]
pub fn cosine_similarity(first: &[f64], second: &[f64]) -> f64 {
    if first.len() != second.len() {
        return 0.0;
    }
    let norms = python_hypot(first) * python_hypot(second);
    if norms == 0.0 {
        return 0.0;
    }
    let pairs = first.iter().zip(second).map(|(left, right)| left * right);
    python_sum(pairs) / norms
}

#[must_use]
pub fn unit_vector(vector: &[f64]) -> Vec<f64> {
    let norm = python_hypot(vector);
    if norm == 0.0 {
        return vector.to_vec();
    }
    vector.iter().map(|value| value / norm).collect()
}

#[must_use]
pub fn mean_vector(vectors: &[Vec<f64>]) -> Vec<f64> {
    let count = f64::from(u32::try_from(vectors.len()).unwrap_or(u32::MAX));
    let width = vectors.first().map_or(0, Vec::len);
    (0..width)
        .map(|index| python_sum(vectors.iter().map(|vector| vector[index])) / count)
        .collect()
}
