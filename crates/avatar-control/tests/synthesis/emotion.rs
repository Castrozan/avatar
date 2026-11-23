use std::collections::HashMap;

use avatar_control::synthesis::media_speech::emotion::delivery_for;

const NATURAL: &str = "natural and conversational";

#[test]
fn every_emotion_in_the_data_file_maps_to_its_delivery() -> Result<(), serde_json::Error> {
    let deliveries: HashMap<String, String> =
        serde_json::from_str(include_str!("../../data/emotion-delivery.json"))?;
    assert_eq!(deliveries.len(), 12);
    for (emotion, delivery) in &deliveries {
        assert_eq!(delivery_for(Some(emotion.as_str())), delivery.as_str());
    }
    Ok(())
}

#[test]
fn missing_and_unknown_emotions_use_natural_delivery() {
    assert_eq!(delivery_for(None), NATURAL);
    assert_eq!(delivery_for(Some("bewildered")), NATURAL);
}

#[test]
fn happy_maps_to_warm_and_cheerful() {
    assert_eq!(delivery_for(Some("happy")), "warm and cheerful, smiling");
}
