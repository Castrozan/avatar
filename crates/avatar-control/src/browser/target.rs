use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PageTarget {
    pub id: String,
    #[serde(rename = "type")]
    pub target_type: String,
    pub url: Option<String>,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub web_socket_debugger_url: Option<String>,
}

impl PageTarget {
    fn is_page_containing(&self, url_fragment: &str) -> bool {
        self.target_type == "page"
            && self
                .url
                .as_deref()
                .is_some_and(|url| url.contains(url_fragment))
    }
}

#[must_use]
pub fn select_page_target<'targets>(
    targets: &'targets [PageTarget],
    url_fragment: &str,
) -> Option<&'targets PageTarget> {
    targets
        .iter()
        .find(|target| target.is_page_containing(url_fragment))
}
