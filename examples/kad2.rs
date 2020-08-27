// use kademlia::net2::RpcClient;
use kademlia::kad2::Kad2;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed setting global subscriber");

    let kad = Kad2::new().await?;
    let addr = kad.node_self.addr;
    let task_kad = kad.start_echo().await?;
    tracing::info!("server running on {}", addr);
    tokio::join!(task_kad);

    Ok(())
}
