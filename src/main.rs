use rust_braacket_colley::make_colley_ranking;

#[tokio::main]
async fn main() {
    make_colley_ranking().await.expect("Failed");
}
