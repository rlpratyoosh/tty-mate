use std::io;
use tokio::net::TcpListener;
use tty_mate_server::run_server;


#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    run_server(listener).await
}
