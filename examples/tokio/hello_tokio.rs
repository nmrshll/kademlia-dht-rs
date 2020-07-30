use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio_util::codec::{BytesCodec, Decoder};

async fn serveTCP() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // // And this is where much of the magic of this server happens. We
        // // crucially want all clients to make progress concurrently, rather than
        // // blocking one on completion of another. To achieve this we use the
        // // `tokio::spawn` function to execute the work in the background.
        // //
        // // Essentially here we're executing a new task to run concurrently,
        // // which will allow all of our clients to be processed concurrently.
        // tokio::spawn(async move {
        //     // We're parsing each socket with the `BytesCodec` included in `tokio::codec`.
        //     let mut framed = BytesCodec::new().framed(socket);

        //     // We loop while there are messages coming from the Stream `framed`.
        //     // The stream will return None once the client disconnects.
        //     while let Some(message) = framed.next().await {
        //         match message {
        //             Ok(bytes) => println!("bytes: {:?}", bytes),
        //             Err(err) => println!("Socket closed with error: {:?}", err),
        //         }
        //     }
        //     println!("Socket received FIN packet and closed connection");
        // });

        tokio::spawn(async move {
            let mut buf = [0_u8; 1024];

            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }
                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed writing to socket");
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { serveTCP().await });
    Ok(())
}
