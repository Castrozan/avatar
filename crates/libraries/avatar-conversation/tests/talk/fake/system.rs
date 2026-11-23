use avatar_conversation::talk::ports::{Clock, Console, Environment};

use super::World;

impl Clock for World {
    fn monotonic(&self) -> f64 {
        self.now
    }

    fn wall(&self) -> f64 {
        self.wall_time()
    }

    fn sleep(&mut self, seconds: f64) {
        self.now += seconds;
    }
}

impl Console for World {
    fn print_out(&mut self, text: &str) {
        self.out.push_str(text);
    }

    fn print_err(&mut self, text: &str) {
        self.err.push_str(text);
    }
}

impl Environment for World {
    fn variable(&self, name: &str) -> Option<String> {
        self.environment_map()
            .get(name)?
            .as_str()
            .map(str::to_owned)
    }

    fn hostname(&self) -> String {
        let configured = self.scenario_field("hostname").as_str();
        configured.unwrap_or("chise.local").to_owned()
    }
}
