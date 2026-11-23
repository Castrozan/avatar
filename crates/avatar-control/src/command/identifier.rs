use std::fs::File;
use std::io::{self, Read};

const RANDOM_SOURCE: &str = "/dev/urandom";

#[must_use]
pub fn format_version_four(mut bytes: [u8; 16]) -> String {
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = hexadecimal(&bytes);
    let (first, rest) = hex.split_at(8);
    let (second, rest) = rest.split_at(4);
    let (third, rest) = rest.split_at(4);
    let (fourth, last) = rest.split_at(4);
    format!("{first}-{second}-{third}-{fourth}-{last}")
}

fn hexadecimal(bytes: &[u8]) -> String {
    bytes
        .iter()
        .flat_map(|byte| [byte >> 4, byte & 0x0f])
        .filter_map(|nibble| char::from_digit(u32::from(nibble), 16))
        .collect()
}

pub fn random_identifier() -> io::Result<String> {
    let mut bytes = [0_u8; 16];
    File::open(RANDOM_SOURCE)?.read_exact(&mut bytes)?;
    Ok(format_version_four(bytes))
}
