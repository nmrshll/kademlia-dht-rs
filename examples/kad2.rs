use kademlia::kad2::Kad;
use std::error::Error;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed setting global subscriber");

    let kad = Kad::start(None)?;
    let addr = kad.node_self.addr;
    tracing::info!("server running on {}", addr);
    let _ = tokio::join!(kad.task);

    Ok(())
}
