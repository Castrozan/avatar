use std::io;

use serde::Serialize;
use serde_json::Serializer;
use serde_json::ser::Formatter;

struct PythonSeparators;

fn separate<Target: ?Sized + io::Write>(target: &mut Target, first: bool) -> io::Result<()> {
    if first {
        return Ok(());
    }
    target.write_all(b", ")
}

impl Formatter for PythonSeparators {
    fn begin_array_value<Target: ?Sized + io::Write>(
        &mut self,
        target: &mut Target,
        first: bool,
    ) -> io::Result<()> {
        separate(target, first)
    }

    fn begin_object_key<Target: ?Sized + io::Write>(
        &mut self,
        target: &mut Target,
        first: bool,
    ) -> io::Result<()> {
        separate(target, first)
    }

    fn begin_object_value<Target: ?Sized + io::Write>(
        &mut self,
        target: &mut Target,
    ) -> io::Result<()> {
        target.write_all(b": ")
    }
}

#[must_use]
pub fn python_json(value: &impl Serialize) -> String {
    let mut bytes = Vec::new();
    let mut serializer = Serializer::with_formatter(&mut bytes, PythonSeparators);
    if value.serialize(&mut serializer).is_err() {
        return String::new();
    }
    String::from_utf8(bytes).unwrap_or_default()
}
