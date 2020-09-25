use futures::{future, SinkExt, StreamExt};
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;

// use crate::key::Key;
use crate::proto2::{CodecErr, FindValResp, ProtoErr, ProtocolCodec, Reply, Request};
use crate::rout2::{KnownNode, Node, RoutingTable};
use crate::state2::{State, StateClient, StateErr};

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
                    if let Err(e) = Self::handle_stream(stream, state).await {
                        println!("failed to handle stream: {}", e);
                    }
                });
            }
        });

        let fut = future::join3(task_kad_server, tasks_state.kv, tasks_state.router);
        Ok(fut)
    }

    pub async fn handle_stream(stream: TcpStream, state: StateClient) -> Result<(), ReqHandleErr> {
        let mut framed2 = Framed::new(stream, ProtocolCodec);

        while let Some(framed_res) = framed2.next().await {
            let req = framed_res?;
            let proc_res = Self::process(req, state.clone()).await;
            // transform any error into a Reply
            let reply: Reply = proc_res.unwrap_or_else(Reply::from);

            framed2.send(reply).await?;
        }
        Ok(())
    }

    pub async fn process(req: Request, state: StateClient) -> Result<Reply, ProtoErr> {
        // TODO request needs to contain src Node
        // TODO update routes with src Node
        println!("GOT: {:?}", &req);
        match req {
            Request::Ping => Ok(Reply::Ping),
            Request::Store(k, v) => {
                let _res: () = state.kv.set(k, v).await?;
                // TODO Wait what about the hash of the key ?
                Ok(Reply::Ping) // TODO ping ? really ? find a way to communicate error
            }
            Request::FindNode(id) => {
                // find closest nodes in routes
                let res: Vec<KnownNode> = state.router.closest_nodes(id).await?;
                Ok(Reply::FindNode(res))
            }
            Request::FindValue(k) => {
                // let hash = k.hash(); // The key is the hash already
                // lookup Key in store
                let res: Result<Option<String>, StateErr> = state.kv.get(k).await;
                let fvr: FindValResp = match res {
                    // return value if found
                    Ok(Some(val_str)) => FindValResp::Value(val_str),
                    // if not found, return closest nodes
                    _ => {
                        let closest_nodes: Vec<KnownNode> = state.router.closest_nodes(k).await?;
                        FindValResp::Nodes(closest_nodes)
                    }
                };
                Ok(Reply::FindVal(fvr))
            }
        }
    }

    // pub async fn node_self(&self) -> Node {
    //     self.routes.lock().await.node_self.clone() // TODO rm mutex, give back &'k node_self
    // }
}

type KadHandle =
    futures::future::Join3<JoinHandle<Result<(), CodecErr>>, JoinHandle<()>, JoinHandle<()>>;

#[derive(Error, Debug)]
pub enum ReqHandleErr {
    #[error("CodecErr: {0}")]
    CodecErr(#[from] CodecErr),
    #[error("State err: {0}")]
    StateErr(#[from] StateErr),
    #[error("unknown error: {0}")]
    Unknown(#[from] Box<dyn std::error::Error>),
}
