use avatar_listening::conversation_addressee::Addressee;
use avatar_listening::conversation_addressee::file_store::FileAddresseeStore;
use avatar_listening::conversation_addressee::store::AddresseeStore;
use proptest::prelude::*;
use tempfile::TempDir;

mod concurrency;
mod oracle;

fn addressee(pane: &str, servant: &str, heard_through: i64, host: Option<String>) -> Addressee {
    Addressee {
        pane: pane.to_owned(),
        servant: servant.to_owned(),
        heard_through,
        host,
    }
}

proptest! {
    #[test]
    fn a_handed_addressee_reads_back_unchanged(
        pane in ".*",
        servant in ".*",
        heard_through in any::<i64>(),
        host in proptest::option::of(".*"),
    ) {
        let root = TempDir::new().map_err(|error| TestCaseError::fail(error.to_string()))?;
        let store = FileAddresseeStore::new(root.path());
        let handed = addressee(&pane, &servant, heard_through, host);
        store.hand_to(&handed).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let read_back = store.current().map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(read_back, Some(handed));
    }
}
