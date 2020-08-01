use kademlia::net2::RpcClient;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed setting global subscriber");

    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());
    let mut listener = TcpListener::bind(&addr).await?;
    tracing::info!("server running on {}", addr);

    // Start a server
    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (mut stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        // let state = Arc::clone(&state);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                if let Err(e) = stream.read(&mut buf).await {
                    tracing::info!("an error occurred; error = {:?}", e);
                }

                match stream.read(&mut buf).await {
                    Err(e) => tracing::info!("an error occurred; error = {:?}", e),
                    // no data back
                    Ok(0) => return,
                    Ok(n) => {
                        stream
                            .write_all(&buf[0..n])
                            .await
                            .expect("failed to write data to socket");
                    }
                }

                // if n == 0 {
                //     return;
                // }

                // socket
                //     .write_all(&buf[0..n])
                //     .await
                //     .expect("failed to write data to socket");
            }
        });

        // tokio::spawn(async move {
        //     tracing::debug!("accepted connection");
        //     if let Err(e) = process(state, stream, addr).await {
        //         tracing::info!("an error occurred; error = {:?}", e);
        //     }
        // });
    }

    //  Send request. won't run here
    // let rpcc = RpcClient::new();
    // rpcc.send().await?;

    Ok(())
}
