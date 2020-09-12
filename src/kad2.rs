// use futures::stream::FuturesUnordered;
use futures::future;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::codec::Decoder;

// use crate::key::Key;
use crate::proto2::{CodecErr, ProtoErr, ProtocolCodec, Reply, Request};
use crate::rout2::{Node, RoutingTable};
use crate::State;
// use crate::routing::KnownNode;

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
    pub async fn start(&self, _bootstrap: Option<Node>) -> Result<KadHandle, io::Error> {
        let addr = &self.node_self.addr.clone();
        let mut listener = TcpListener::bind(addr).await?;

        // Start the state manager task
        let (task_state, cmd_chan) = State::new().start().split();

        // Start a server
        let task_kad_server = tokio::spawn(async move {
            loop {
                let (stream, _addr) = listener.accept().await.map_err(|e: io::Error| {
                    tracing::warn!("failed listener.accept(): {:?}", e);
                    return CodecErr::from(e); // TODO not CodecErr
                })?; // returns Result<_, CodecErr>
                tokio::spawn(async move {
                    Self::handle_stream(stream).await; // TODO err handling
                });
            }
        });

        Ok(future::join(task_state, task_kad_server))
    }

    pub async fn handle_stream(stream: TcpStream) {
        // create a codec per connection to parse all messages sent on that connection
        let codec = ProtocolCodec::new();
        let mut framed_codec_stream = codec.framed(stream); // no split ?

        // TODO is this spawn needed ?
        while let Some(res) = framed_codec_stream.next().await {
            match res {
                Ok(req) => {
                    let resp = Self::process(req);
                    //  MONDAY TODO respond on the stream
                }
                Err(e) => println!("ERROR: {}", e),
            }
        }
    }

    pub fn process(req: Request) -> Reply {
        println!("GOT: {:?}", &req);
        // TODO update routes without lock
        match req {
            Request::Ping => Reply::Ping,
            Request::Store(_k, _v) => {
                // TODO store value
                Reply::Ping
            }
            Request::FindNode(_id) => {
                // TODO find colsest nodes in routes
                Reply::FindNode(vec![])
            }
            Request::FindValue(_k) => {
                // TODO hash key
                // TODO lookup hash in store
                // TODO return value if found, FindValueResult::Nodes with closest nodes if not found
                Reply::Ping
            }
            // TODO for error management return a reply with an error if error
            _ => Reply::Err(ProtoErr::Unknown), // TODO above in match arms
        }
    }

    // pub async fn node_self(&self) -> Node {
    //     self.routes.lock().await.node_self.clone() // TODO rm mutex, give back &'k node_self
    // }
}

type KadHandle = futures::future::Join<JoinHandle<()>, JoinHandle<Result<(), CodecErr>>>;

// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum KadErr {
//     #[error("unknown Request")]
//     UnknownRequest,
//     #[error("unknown ProtocolCodec error")]
//     Unknown,
// }

struct Echoer {
    pub node_self: Node,
}
impl Echoer {
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
}
