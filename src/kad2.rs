// use futures::stream::FuturesUnordered;
use futures::future;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::codec::Decoder;

// use crate::key::Key;
use crate::proto2::{CodecErr, ProtoErr, ProtocolCodec, Reply, Request};
use crate::rout2::{Node, RoutingTable};
use crate::state2::{State, StateClient, StateErr};
// use crate::routing::KnownNode;

// TODO separate Rust API (put,get) and RPC api

#[derive(Clone)]
pub struct Kad2 {
    // TODO make Kad own only refs/handles and be Copy,
    // then we can use self on methods inside tasks ?
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
    pub async fn start(self, _bootstrap: Option<Node>) -> Result<KadHandle, io::Error> {
        let addr = &self.node_self.addr.clone();
        let mut listener = TcpListener::bind(addr).await?;

        // Start the state manager task
        let (tasks_state, state_client) = State::start(self.node_self);

        // Start a server
        let task_kad_server = tokio::spawn(async move {
            loop {
                let (stream, _addr) = listener.accept().await.map_err(|e: io::Error| {
                    tracing::warn!("failed listener.accept(): {:?}", e);
                    return CodecErr::from(e); // TODO not CodecErr
                })?; // returns Result<_, CodecErr>

                let state = state_client.clone();
                tokio::spawn(async move {
                    Self::handle_stream(stream, state).await; // TODO err handling
                });
            }
        });

        let fut = future::join3(task_kad_server, tasks_state.kv, tasks_state.router);
        Ok(fut)
    }

    pub async fn handle_stream(stream: TcpStream, state: StateClient) {
        // create a codec per connection to parse all messages sent on that connection
        let codec = ProtocolCodec::new();
        let mut framed_codec_stream = codec.framed(stream); // no split ?

        // TODO is this spawn needed ?
        while let Some(res) = framed_codec_stream.next().await {
            match res {
                Ok(req) => {
                    let _resp = Self::process(req, state.clone());
                    //  MONDAY TODO respond on the stream
                }
                Err(e) => println!("ERROR: {}", e),
            }
        }
    }

    pub async fn process(req: Request, state: StateClient) -> Reply {
        // TODO request needs to contain src Node
        // TODO update routes with src Node
        println!("GOT: {:?}", &req);
        match req {
            Request::Ping => Reply::Ping,
            Request::Store(k, v) => {
                let _res: Result<(), StateErr> = state.kv.set(k, v).await;
                // TODO Wait what about the hash of the key ?
                Reply::Ping // TODO ping ? really ?
            }
            Request::FindNode(_id) => {
                // TODO find closest nodes in routes
                let _res: Result<(), StateErr> = state.router.closest_nodes().await;
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

type KadHandle =
    futures::future::Join3<JoinHandle<Result<(), CodecErr>>, JoinHandle<()>, JoinHandle<()>>;

// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum KadErr {
//     #[error("unknown Request")]
//     UnknownRequest,
//     #[error("unknown ProtocolCodec error")]
//     Unknown,
// }
