use avatar_listening::listener::speaker_identity::{cosine_similarity, python_hypot, python_sum};
use proptest::prelude::*;

fn finite_vector() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(-1.0e6_f64..1.0e6, 1..40)
}

proptest! {
    #[test]
    fn hypot_of_one_coordinate_is_its_magnitude(value in -1.0e300_f64..1.0e300) {
        prop_assert_eq!(python_hypot(&[value]).to_bits(), value.abs().to_bits());
    }

    #[test]
    fn hypot_ignores_order_and_sign_within_one_rounding_step(vector in finite_vector()) {
        let negated: Vec<f64> = vector.iter().rev().map(|value| -value).collect();
        let (first, second) = (python_hypot(&vector), python_hypot(&negated));
        prop_assert!((first - second).abs() <= first * 1e-15);
    }

    #[test]
    fn cosine_similarity_is_bounded_and_symmetric(left in finite_vector(), shift in -1.0e3_f64..1.0e3) {
        let right: Vec<f64> = left.iter().map(|value| value + shift).collect();
        let score = cosine_similarity(&left, &right);
        prop_assert!(score.abs() <= 1.0 + 1e-9);
        prop_assert!((score - cosine_similarity(&right, &left)).abs() <= 1e-9);
    }

    #[test]
    fn a_sum_stays_within_rounding_of_the_exact_total(values in prop::collection::vec(-1.0e3_f64..1.0e3, 0..60)) {
        let plain: f64 = values.iter().sum();
        prop_assert!((python_sum(values.iter().copied()) - plain).abs() <= 1e-9);
    }
}
