use super::cases;
use avatar_holder::default_holder::DefaultHolder;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/default_holder_identity.json");

#[test]
fn pane_and_servant_match_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        let holder = DefaultHolder::new(case["profile"].as_str().unwrap_or_default(), None);
        assert_eq!(holder.pane(), case["pane"].as_str().unwrap_or_default());
        assert_eq!(
            holder.servant(),
            case["servant"].as_str().unwrap_or_default()
        );
    }
    Ok(())
}
