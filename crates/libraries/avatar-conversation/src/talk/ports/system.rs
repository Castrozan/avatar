pub trait Clock {
    fn monotonic(&self) -> f64;
    fn wall(&self) -> f64;
    fn sleep(&mut self, seconds: f64);
}

pub trait Console {
    fn print_out(&mut self, text: &str);
    fn print_err(&mut self, text: &str);
}

pub trait Environment {
    fn variable(&self, name: &str) -> Option<String>;
    fn hostname(&self) -> String;
}
