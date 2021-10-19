#[tokio::main]
async fn main() {
    warp::serve(warp::fs::dir("debugger-ui/build"))
        .run(([0, 0, 0, 0], 3000))
        .await;
}
