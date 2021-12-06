use warp::Filter;

fn hello() -> & 'static str {
    "Hello, world!\n"
}

#[tokio::main]
async fn main() {
    warp::serve(warp::any().map(hello)).run(([0, 0, 0, 0], 3000)).await;
}
