use std::collections::{BTreeMap, BTreeSet};

use avatar_conversation::talk::ports::{Clock, Console, Environment, Files, ListeningRequest};

const GRANTED: &str = r#"{"pane": "p1", "servant": "Mara", "heard_through": 0}"#;

#[derive(Default)]
pub(crate) struct Fake {
    pub(crate) files: BTreeMap<String, String>,
    pub(crate) environment: BTreeMap<String, String>,
    pub(crate) active: BTreeSet<String>,
    pub(crate) outputs: BTreeMap<String, (i32, String)>,
    pub(crate) ran: Vec<Vec<String>>,
    pub(crate) out: String,
    pub(crate) err: String,
    pub(crate) begun: Vec<ListeningRequest>,
    pub(crate) granting: bool,
    pub(crate) now: f64,
}

pub(super) fn grant_the_floor(fake: &mut Fake, program: &str) {
    if program == "avatar-claim-floor" && fake.granting {
        fake.files
            .insert("addressee.json".to_owned(), GRANTED.to_owned());
    }
}

impl Clock for Fake {
    fn monotonic(&self) -> f64 {
        self.now
    }

    fn wall(&self) -> f64 {
        self.now
    }

    fn sleep(&mut self, seconds: f64) {
        self.now += seconds;
    }
}

impl Console for Fake {
    fn print_out(&mut self, text: &str) {
        self.out.push_str(text);
    }

    fn print_err(&mut self, text: &str) {
        self.err.push_str(text);
    }
}

impl Environment for Fake {
    fn variable(&self, name: &str) -> Option<String> {
        self.environment.get(name).cloned()
    }

    fn hostname(&self) -> String {
        "here".to_owned()
    }
}

impl Files for Fake {
    fn exists(&mut self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    fn read(&mut self, name: &str) -> Option<String> {
        self.files.get(name).cloned()
    }

    fn modified_at(&mut self, _name: &str) -> Option<f64> {
        None
    }

    fn write(&mut self, name: &str, content: &str) {
        self.files.insert(name.to_owned(), content.to_owned());
    }

    fn touch(&mut self, name: &str) {
        self.files.entry(name.to_owned()).or_default();
    }
}
