use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Action {
    N,
    S,
    E,
    W,
    PickUp,
    Drop,
    Open,
    Wait,
}

impl Action {
    pub const fn tie_break_order() -> [Action; 8] {
        [
            Action::N,
            Action::S,
            Action::E,
            Action::W,
            Action::PickUp,
            Action::Drop,
            Action::Open,
            Action::Wait,
        ]
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Action::N => (0, -1),
            Action::S => (0, 1),
            Action::E => (1, 0),
            Action::W => (-1, 0),
            Action::PickUp | Action::Drop | Action::Open | Action::Wait => (0, 0),
        }
    }
}

impl FromStr for Action {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim() {
            "N" => Ok(Action::N),
            "S" => Ok(Action::S),
            "E" => Ok(Action::E),
            "W" => Ok(Action::W),
            "PickUp" => Ok(Action::PickUp),
            "Drop" => Ok(Action::Drop),
            "Open" => Ok(Action::Open),
            "Wait" => Ok(Action::Wait),
            other => bail!("unknown action `{other}`"),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Action::N => "N",
            Action::S => "S",
            Action::E => "E",
            Action::W => "W",
            Action::PickUp => "PickUp",
            Action::Drop => "Drop",
            Action::Open => "Open",
            Action::Wait => "Wait",
        };
        f.write_str(text)
    }
}
