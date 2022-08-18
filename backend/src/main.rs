use backend::server::{Server, Args};
use clap::Parser;
use log;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    let server = Server::new(args);

    server.run().await.unwrap();
}
