use warp::Filter;

pub fn serve_frontend() {
    let app = warp::get().map(|| "...hello");
    
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    std::thread::spawn(move || {
        rt.block_on(async {
            warp::serve(app).bind(([0, 0, 0, 0], 8888)).await;
        });
    });
}
