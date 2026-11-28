use avatar_live::client::provisioning::contains;
use proptest::prelude::*;
use serde_json::{Value, json};

fn leaf() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<bool>().prop_map(Value::from),
        any::<i32>().prop_map(Value::from),
        "[a-z]{0,4}".prop_map(Value::from),
    ]
}

fn document() -> impl Strategy<Value = Value> {
    leaf().prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::from),
            prop::collection::btree_map("[a-c]", inner, 0..4)
                .prop_map(|fields| Value::Object(fields.into_iter().collect())),
        ]
    })
}

proptest! {
    #[test]
    fn a_document_contains_itself_and_survives_an_added_field(wanted in document()) {
        prop_assert!(contains(&wanted, &wanted));
        let wrapped = json!({"wanted": wanted.clone(), "extra": 1});
        let bare = json!({"wanted": wanted});
        prop_assert!(contains(&wrapped, &bare));
    }

    #[test]
    fn a_changed_or_missing_leaf_is_not_contained(wanted in leaf(), other in leaf()) {
        prop_assume!(wanted != other);
        let (changed, expected, empty) = (json!({"key": other}), json!({"key": wanted}), json!({}));
        prop_assert!(!contains(&changed, &expected));
        prop_assert!(!contains(&empty, &expected));
    }
}

#[test]
fn numbers_compare_by_value_and_arrays_by_length() {
    assert!(contains(
        &json!({"turn_timeout": 30.0}),
        &json!({"turn_timeout": 30})
    ));
    assert!(!contains(&json!(["a", "b"]), &json!(["a"])));
}
