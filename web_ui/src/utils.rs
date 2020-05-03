use std::rc::Rc;
use yew::prelude::*;

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format_args!($($t)*).to_string())))
}

pub fn callback_fn<T, F: Fn(T) + Sized + 'static>(f: F) -> Callback<T> {
    Callback::Callback(Rc::new(f))
}
