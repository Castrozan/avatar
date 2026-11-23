use tokio_tungstenite::tungstenite::handshake::server::{
    Callback, ErrorResponse, Request, Response,
};
use tokio_tungstenite::tungstenite::http::StatusCode;
use tokio_tungstenite::tungstenite::http::header::ORIGIN;

pub(super) struct OriginCheck {
    allowed_origins: Vec<String>,
}

impl OriginCheck {
    pub(super) fn new(allowed_origins: Vec<String>) -> Self {
        Self { allowed_origins }
    }

    fn permits(&self, request: &Request) -> bool {
        request
            .headers()
            .get(ORIGIN)
            .and_then(|origin| origin.to_str().ok())
            .is_some_and(|origin| self.allowed_origins.iter().any(|allowed| allowed == origin))
    }
}

impl Callback for OriginCheck {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        self.permits(request)
            .then_some(response)
            .ok_or_else(unauthorized)
    }
}

fn unauthorized() -> ErrorResponse {
    let mut response = ErrorResponse::new(None);
    *response.status_mut() = StatusCode::UNAUTHORIZED;
    response
}
