use crate::talk::ports::AgentPane;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSpec {
    host: String,
    pane: String,
}

pub(super) fn parse_agent(text: &str) -> Result<AgentSpec, String> {
    let (host, pane) = text.split_once(':').unwrap_or((text, ""));
    let valid = valid_host(host) && !pane.is_empty();
    valid
        .then(|| AgentSpec {
            host: host.to_owned(),
            pane: pane.to_owned(),
        })
        .ok_or_else(|| format!("{text:?} is not HOST:PANE"))
}

impl AgentSpec {
    #[must_use]
    pub fn resolve(&self, this_machine: Option<&str>) -> AgentPane {
        AgentPane {
            host: (this_machine != Some(self.host.as_str())).then(|| self.host.clone()),
            pane: Some(self.pane.clone()),
        }
    }
}

fn valid_host(host: &str) -> bool {
    let mut characters = host.chars();
    let first = characters
        .next()
        .is_some_and(|start| start.is_ascii_alphanumeric());
    first && characters.all(|rest| rest.is_ascii_alphanumeric() || rest == '.' || rest == '-')
}
