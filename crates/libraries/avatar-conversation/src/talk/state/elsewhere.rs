use crate::talk::ports::Files;

const RECORD_FILE: &str = "conversation-elsewhere";

pub fn machine_holding_the_conversation(files: &mut impl Files) -> Option<String> {
    let recorded = files.read(RECORD_FILE)?;
    let machine = recorded.trim();
    (!machine.is_empty()).then(|| machine.to_owned())
}

pub fn remember_the_conversation_at(files: &mut impl Files, machine: &str) {
    files.write(RECORD_FILE, machine);
}

pub fn forget_the_conversation_elsewhere(files: &mut impl Files) {
    files.write(RECORD_FILE, "");
}
