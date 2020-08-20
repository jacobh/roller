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

pub fn get_query_params() -> Option<web_sys::UrlSearchParams> {
    let location = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.location())?;

    let url_str = location.href().ok()?;
    let location_url = web_sys::Url::new(&url_str).ok()?;

    Some(location_url.search_params())
}
