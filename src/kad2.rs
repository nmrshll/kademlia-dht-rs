use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::codec::{Decoder, FramedRead, LengthDelimitedCodec};

// use crate::key::Key;
use crate::req2::{MyCodec, MyCodecErr};
use crate::rout2::{Node, RoutingTable};
// use crate::routing::KnownNode;
// use crate::req2::DummyData;

#[derive(Clone)]
pub struct Kad2 {
    routes: Arc<Mutex<RoutingTable>>, // replace with interior mut Sync handle
    // store: Arc<Mutex<HashMap<String, String>>>,
    pub node_self: Node,
}
impl<'k> Kad2 {
    pub async fn new() -> Result<Kad2, Box<dyn Error>> {
        let addr: SocketAddr = ([0, 0, 0, 0], 8908).into(); // TODO config::port()
                                                            // let mut listener = TcpListener::bind(&addr).await?;
        let node_self: Node = Node::new_self(&addr)?;

        Ok(Kad2 {
            routes: Arc::new(Mutex::new(RoutingTable::new(&node_self))),
            // store: Arc::new(Mutex::new(Hashmap < String, String > ::new())),
            node_self,
        })
    }

    pub async fn start_echo(self) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let addr = &self.node_self.addr.clone();
        let mut listener = TcpListener::bind(addr).await?;

        // Start a server
        let task_echo = tokio::spawn(async move {
            loop {
                // Asynchronously wait for an inbound TcpStream.
                match listener.accept().await {
                    Err(e) => {
                        tracing::warn!("failed starting listener: {:?}", e);
                        break;
                    }
                    Ok((stream, _)) => {
                        tokio::spawn(async move {
                            Self::handle_echo_stream(stream).await; // TODO err handling
                        });
                    }
                }
            }
        });

        Ok(task_echo)
    }

    pub async fn handle_echo_stream(mut stream: TcpStream) {
        let mut buf = [0u8; 1024];
        // In a loop, read data from the socket and write the data back.
        loop {
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
        }
    }

    pub async fn start(
        &self,
        bootstrap: Option<Node>,
    ) -> Result<JoinHandle<Result<(), MyCodecErr>>, Box<dyn Error>> {
        let addr = &self.node_self.addr.clone();
        let mut listener = TcpListener::bind(addr).await?;

        // Start a server
        let task_kad = tokio::spawn(async move {
            loop {
                let (stream, _addr) = listener.accept().await.map_err(|e: std::io::Error| {
                    tracing::warn!("failed listener.accept(): {:?}", e);
                    return e;
                })?;
                tokio::spawn(async move {
                    Self::handle_stream(stream).await; // TODO err handling
                });
            }
            Ok::<_, MyCodecErr>(()) // TODO not MyCodecErr
        });

        Ok(task_kad)
    }

    pub async fn handle_stream(mut stream: TcpStream) {
        // create a codec per connection to parse all messages sent on that connection
        let codec = MyCodec::new();
        let mut framedCodecStream = codec.framed(stream); // no split ?

        // Spawn a task that prints all received messages to STDOUT
        tokio::spawn(async move {
            while let Some(res) = framedCodecStream.next().await {
                match res {
                    Ok(dummyData) => println!("GOT: {:?}", dummyData),
                    Err(e) => println!("ERROR: {}", e),
                }
            }
            Ok::<(), MyCodecErr>(())
        });
    }

    // pub async fn node_self(&self) -> Node {
    //     self.routes.lock().await.node_self.clone() // TODO rm mutex, give back &'k node_self
    // }
}

// TOKIO CODEC BINCODE
// FramedRead upgrades TcpStream from an AsyncRead to a Stream
type IOErrorStream = FramedRead<TcpStream, LengthDelimitedCodec>;
// stream::FromErr maps underlying IO errors into Bincode errors
// type BincodeErrStream = stream::FromErr<IOErrorStream, bincode::Error>;
// ReadBincode maps underlying bytes into Bincode-deserializable structs
// type BincodeStream = ReadBincode<BincodeErrStream, DummyData>;
