pub mod error;
pub mod fallback;
pub mod hex;
pub mod http;
pub mod scribe;
pub mod segment;
pub mod spoken_languages;

pub use error::TranscriptionError;
pub use fallback::transcribe_with_local_fallback;
pub use hex::{HexClient, hex_discovery_file};
pub use http::{HttpMethod, HttpRequest, HttpResponse, HttpSender, ReqwestSender};
pub use scribe::ScribeClient;
