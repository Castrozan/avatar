use std::fmt;

use avatar_transcript::echo::is_python_space;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Party {
    Human,
    Avatar,
}

impl fmt::Display for Party {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(match self {
            Self::Human => "human",
            Self::Avatar => "avatar",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpokenTurn {
    pub party: Party,
    pub text: String,
}

pub struct TurnAssembler {
    settle_seconds: f64,
    party: Option<Party>,
    words: String,
    last_heard_at: f64,
}

impl TurnAssembler {
    #[must_use]
    pub fn new(settle_seconds: f64) -> Self {
        Self {
            settle_seconds,
            party: None,
            words: String::new(),
            last_heard_at: 0.0,
        }
    }

    pub fn add(&mut self, party: Party, text: &str, now: f64) -> Vec<SpokenTurn> {
        let changed = self.party.filter(|current| *current != party);
        let finished = changed.map_or_else(Vec::new, |_| self.finish());
        self.party = Some(party);
        self.words.push_str(text);
        self.last_heard_at = now;
        finished
    }

    pub fn settled(&mut self, now: f64) -> Vec<SpokenTurn> {
        let paused = now - self.last_heard_at >= self.settle_seconds;
        self.party
            .filter(|_| paused)
            .map_or_else(Vec::new, |_| self.finish())
    }

    pub fn finish(&mut self) -> Vec<SpokenTurn> {
        let party = self.party.take();
        let words = std::mem::take(&mut self.words);
        let text = words.trim_matches(is_python_space).to_owned();
        let turn = party
            .filter(|_| !text.is_empty())
            .map(|party| SpokenTurn { party, text });
        turn.into_iter().collect()
    }
}
