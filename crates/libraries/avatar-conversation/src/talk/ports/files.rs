pub trait Files {
    fn exists(&mut self, name: &str) -> bool;
    fn read(&mut self, name: &str) -> Option<String>;
    fn modified_at(&mut self, name: &str) -> Option<f64>;
    fn write(&mut self, name: &str, content: &str);
    fn touch(&mut self, name: &str);
}
