use serde::{Deserialize, Serialize};

pub mod clock;
pub mod control;
pub mod fixture;
pub mod position;
mod utils;

use control::{ButtonCoordinate, ButtonGridLocation, ButtonState, InputEvent};
use fixture::{FixtureId, FixtureParams, FixtureState};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Input(InputEvent),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    ButtonStatesUpdated(Vec<(ButtonGridLocation, ButtonCoordinate, ButtonState)>),
    ButtonLabelsUpdated(Vec<(ButtonGridLocation, ButtonCoordinate, String)>),
    FixtureParamsUpdated(Vec<(FixtureId, FixtureParams)>),
    FixtureStatesUpdated(Vec<(FixtureId, FixtureState)>),
}
