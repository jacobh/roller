use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    ButtonPressed(ButtonGridLocation, ButtonCoordinate),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ButtonCoordinate {
    pub row_idx: usize,
    pub column_idx: usize,
}
impl fmt::Display for ButtonCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.column_idx, self.row_idx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
