// Quiz transitions follow tetris-fumen 1.1.3 (MIT; see LICENSES/tetris-fumen.txt).
use crate::Block;

#[derive(Clone)]
pub(crate) enum Quiz {
    Sequence {
        hold: Option<char>,
        current: Option<char>,
        remaining: String,
    },
    Text(String),
}

fn is_piece(piece: char) -> bool {
    matches!(
        piece,
        'T' | 'I' | 'O' | 'S' | 'Z' | 'J' | 'L' | 't' | 'i' | 'o' | 's' | 'z' | 'j' | 'l'
    )
}

fn slot(text: &str) -> Option<Option<char>> {
    let mut characters = text.chars();
    match (characters.next(), characters.next()) {
        (None, None) => Some(None),
        (Some(piece), None) if is_piece(piece) => Some(Some(piece)),
        _ => None,
    }
}

fn after_next(remaining: &str) -> &str {
    match remaining.chars().next() {
        Some(';') | None => remaining,
        Some(character) => &remaining[character.len_utf8()..],
    }
}

impl Quiz {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        if !text.starts_with("#Q=") {
            return None;
        }
        Self::from_text(text)
    }

    fn from_text(text: &str) -> Option<Self> {
        if !text.starts_with("#Q=") {
            return Some(Self::Text(text.to_owned()));
        }
        let text: String = text
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        let (hold, rest) = text.strip_prefix("#Q=[")?.split_once("](")?;
        let (current, remaining) = rest.split_once(')')?;
        Self::sequence(slot(hold)?, slot(current)?, remaining.to_owned())
    }

    fn sequence(hold: Option<char>, current: Option<char>, remaining: String) -> Option<Self> {
        if hold.is_some_and(|piece| !is_piece(piece))
            || current.is_some_and(|piece| !is_piece(piece))
        {
            return None;
        }
        Some(Self::Sequence {
            hold,
            current,
            remaining,
        })
    }

    fn text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Sequence {
                hold,
                current,
                remaining,
            } => format!(
                "#Q=[{}]({}){remaining}",
                hold.map(|piece| piece.to_string()).unwrap_or_default(),
                current.map(|piece| piece.to_string()).unwrap_or_default(),
            ),
        }
    }

    fn next_if_end(&self) -> Option<Self> {
        if let Self::Sequence {
            hold: None,
            current: None,
            remaining,
        } = self
        {
            if let Some(next) = remaining.strip_prefix(';') {
                return Self::from_text(next);
            }
        }
        Some(self.clone())
    }

    fn formatted(&self) -> Option<Self> {
        let quiz = self.next_if_end()?;
        match quiz {
            Self::Sequence {
                current: None,
                hold: Some(hold),
                remaining,
            } => Self::sequence(None, Some(hold), remaining),
            Self::Sequence {
                current: None,
                hold: None,
                remaining,
            } => match remaining.chars().next() {
                None => Some(Self::Text(String::new())),
                Some(';') => Self::from_text(&remaining[1..]),
                Some(piece) => Self::sequence(None, Some(piece), after_next(&remaining).to_owned()),
            },
            _ => Some(quiz),
        }
    }

    pub(crate) fn comment(&self) -> String {
        self.formatted()
            .map(|quiz| quiz.text())
            .unwrap_or_else(|| self.text())
    }

    pub(crate) fn advance(&mut self, block: Block) {
        let Some(next) = self.next_if_end() else {
            return;
        };
        let Self::Sequence {
            hold,
            current,
            remaining,
        } = next
        else {
            return;
        };
        if hold.is_none() && current.is_none() && remaining.is_empty() {
            return;
        }
        let used = match block {
            Block::I => Some('I'),
            Block::L => Some('L'),
            Block::O => Some('O'),
            Block::Z => Some('Z'),
            Block::T => Some('T'),
            Block::J => Some('J'),
            Block::S => Some('S'),
            Block::Empty | Block::Garbage => None,
        };
        let following = remaining.chars().next().filter(|&piece| piece != ';');
        let rest = after_next(&remaining);
        let advanced = if used.is_some()
            && (used == current || (hold.is_some() && current.is_none() && used == following))
        {
            if current.is_some() {
                Self::sequence(hold, following, rest.to_owned())
            } else {
                rest.chars().next().and_then(|piece| {
                    Self::sequence(hold, Some(piece), after_next(rest).to_owned())
                })
            }
        } else if used.is_some() && used == hold {
            Self::sequence(current, following, rest.to_owned())
        } else if used.is_some() && hold.is_none() && used == following {
            Self::sequence(current, rest.chars().next(), after_next(rest).to_owned())
        } else {
            None
        };
        if let Some(quiz) = advanced.or_else(|| self.formatted()) {
            *self = quiz;
        }
    }
}
