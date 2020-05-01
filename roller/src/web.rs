use std::future::Future;
use warp::Filter;

pub fn serve_frontend() -> impl Filter + Clone + 'static {
    let app = warp::get().map(|| "...hello");
    app
}
