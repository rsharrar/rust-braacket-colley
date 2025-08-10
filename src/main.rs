use rust_braacket_colley::process_html_to_internal;

#[tokio::main]
async fn main() {
    process_html_to_internal().await.expect("Failed");
}
