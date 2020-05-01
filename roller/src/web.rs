use warp::Filter;

use midi::Note;
use roller_protocol::{ButtonCoordinate, ButtonGridLocation};

use crate::ControlEvent;

// Specific for akai apc mini. really the internal button location should be specific with coords
fn coordinate_to_note(loc: &ButtonGridLocation, coord: &ButtonCoordinate) -> Note {
    match loc {
        ButtonGridLocation::Main => Note::new((8 * coord.row_idx + coord.column_idx) as u8),
        ButtonGridLocation::MetaRight => Note::new((89 - coord.row_idx) as u8),
        ButtonGridLocation::MetaBottom => Note::new((64 + coord.column_idx) as u8),
    }
}

pub fn serve_frontend(_event_sender: async_std::sync::Sender<ControlEvent>) {
    let index = warp::get().and(warp::fs::file("web_ui/index.html"));
    let assets = warp::get().and(warp::fs::dir("web_ui"));

    let app = warp::path::end().and(index).or(assets);

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    std::thread::spawn(move || {
        rt.block_on(async {
            warp::serve(app).bind(([0, 0, 0, 0], 8888)).await;
        });
    });
}
