use std::net::TcpListener;

use newsletter_api::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("http://127.0.0.1:0").unwrap();
    match run(listener) {
        Ok(_) => Ok(()),
        Err(e) => return Err(e),
    }
}
