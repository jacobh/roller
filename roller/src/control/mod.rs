pub mod button;
pub mod control_mapping;
mod default_control_mapping;
pub mod fader;
pub mod midi;

pub use default_control_mapping::default_control_mapping;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteState {
    On,
    Off,
}
