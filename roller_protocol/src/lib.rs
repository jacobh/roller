use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    ButtonPressed(ButtonGridLocation, ButtonCoordinate),
    ButtonReleased(ButtonGridLocation, ButtonCoordinate),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    ButtonStatesUpdated(Vec<(ButtonGridLocation, ButtonCoordinate, ButtonState)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ButtonCoordinate {
    pub row_idx: usize,
    pub column_idx: usize,
}
impl fmt::Display for ButtonCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.column_idx, self.row_idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ButtonGridLocation {
    Main,
    MetaRight,
    MetaBottom,
}
impl ButtonGridLocation {
    pub fn css_name(&self) -> &'static str {
        match self {
            ButtonGridLocation::Main => "main",
            ButtonGridLocation::MetaRight => "meta-right",
            ButtonGridLocation::MetaBottom => "meta-bottom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum ButtonState {
    Active,
    Inactive,
    Deactivated,
    Unused,
}
impl ButtonState {
    pub fn css_class(&self) -> &'static str {
        match self {
            ButtonState::Active => "button--active",
            ButtonState::Inactive => "button--inactive",
            ButtonState::Deactivated => "button--deactivated",
            ButtonState::Unused => "button--unused",
        }
    }
}
impl Default for ButtonState {
    fn default() -> ButtonState {
        ButtonState::Unused
    }
}
