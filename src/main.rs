#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    newsletter_api::run().await
}
