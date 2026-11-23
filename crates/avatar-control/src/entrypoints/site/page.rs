const ERROR_TEMPLATE: &str = "<!DOCTYPE HTML>\n<html lang=\"en\">\n    <head>\n        <meta charset=\"utf-8\">\n        <title>Error response</title>\n    </head>\n    <body>\n        <h1>Error response</h1>\n        <p>Error code: CODE</p>\n        <p>Message: MESSAGE.</p>\n        <p>Error code explanation: CODE - EXPLAIN</p>\n    </body>\n</html>\n";

pub(super) fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(super) fn error_page(code: u16, message: &str, explanation: &str) -> String {
    ERROR_TEMPLATE
        .replace("CODE", &code.to_string())
        .replace("MESSAGE", &escape_html(message))
        .replace("EXPLAIN", &escape_html(explanation))
}
