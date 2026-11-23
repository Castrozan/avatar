use std::io::{self, Write};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256, Sha512};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Integrity {
    algorithm: Algorithm,
    expected: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Algorithm {
    Sha256,
    Sha512,
}

impl Algorithm {
    fn named(label: &str) -> Option<Self> {
        match label {
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            _ => None,
        }
    }
}

impl Integrity {
    pub(crate) fn parse(text: &str) -> io::Result<Self> {
        let (label, encoded) = text.split_once('-').ok_or_else(|| malformed(text))?;
        let algorithm = Algorithm::named(label).ok_or_else(|| malformed(text))?;
        let expected = STANDARD.decode(encoded).map_err(|_| malformed(text))?;
        Ok(Self {
            algorithm,
            expected,
        })
    }

    pub(crate) fn verifier<W: Write>(&self, sink: W) -> Verifier<W> {
        let hasher = match self.algorithm {
            Algorithm::Sha256 => Hasher::Sha256(Sha256::new()),
            Algorithm::Sha512 => Hasher::Sha512(Sha512::new()),
        };
        Verifier { sink, hasher }
    }

    pub(crate) fn matches(&self, digest: &[u8]) -> bool {
        self.expected == digest
    }

    pub(crate) fn describe(digest: &[u8]) -> String {
        STANDARD.encode(digest)
    }
}

fn malformed(text: &str) -> io::Error {
    io::Error::other(format!("malformed integrity value {text}"))
}

enum Hasher {
    Sha256(Sha256),
    Sha512(Sha512),
}

pub(crate) struct Verifier<W> {
    sink: W,
    hasher: Hasher,
}

impl<W> Verifier<W> {
    pub(crate) fn finish(self) -> Vec<u8> {
        match self.hasher {
            Hasher::Sha256(hasher) => hasher.finalize().to_vec(),
            Hasher::Sha512(hasher) => hasher.finalize().to_vec(),
        }
    }

    fn absorb(&mut self, bytes: &[u8]) {
        match &mut self.hasher {
            Hasher::Sha256(hasher) => hasher.update(bytes),
            Hasher::Sha512(hasher) => hasher.update(bytes),
        }
    }
}

impl<W: Write> Write for Verifier<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.sink.write(buffer)?;
        self.absorb(&buffer[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.sink.flush()
    }
}

#[cfg(test)]
mod tests;
