use futures::{future, SinkExt, StreamExt};
use std::io;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;

// use crate::key::Key;
use crate::proto2::{CodecErr, FindValResp, ProtoErr, ProtocolCodec, Reply, Request, RequestBody};
use crate::rout2::{KnownNode, Node};
use crate::state2::{State, StateClient, StateErr};

// TODO separate Rust API (put,get) and RPC api

pub struct Kad {
    // TODO make Kad own only refs/handles and be Copy,
    // then we can use self on methods inside tasks ?
    pub node_self: Node,
    pub task:
        futures::future::Join3<JoinHandle<Result<(), CodecErr>>, JoinHandle<()>, JoinHandle<()>>,
}
impl<'k> Kad {
    pub fn start(_bootstrap: Option<Node>) -> Result<Self, KadErr> {
        let addr: SocketAddr = ([0, 0, 0, 0], 8908).into(); // TODO config::port()
        let node_self: Node = Node::new_self(&addr)?;

        // Start the state manager task
        let (tasks_state, state_client) = State::start(node_self);
        // Start a server
        let task_kad_server = tokio::spawn(async move {
            let mut listener = TcpListener::bind(addr).await?;
            loop {
                let incoming = listener.accept().await.map_err(|e: io::Error| {
                    tracing::warn!("failed listener.accept(): {:?}", e);
                    return CodecErr::from(e); // TODO not CodecErr
                })?; // returns Result<_, CodecErr>

                let state = state_client.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_stream(incoming, state).await {
                        println!("failed to handle stream: {}", e);
                    }
                });
            }
        });

        let futs = future::join3(task_kad_server, tasks_state.kv, tasks_state.router);
        Ok(Kad {
            node_self,
            task: futs,
        })
    }

    pub async fn handle_stream(
        inc: (TcpStream, SocketAddr),
        state: StateClient,
    ) -> Result<(), ReqErr> {
        let mut framed2 = Framed::new(inc.0, ProtocolCodec);

        while let Some(framed_res) = framed2.next().await {
            let req = framed_res?;

            // update router with src_node, then process request into a Result<Reply,_>
            let src_node = Node {
                key: req.header.sender_key,
                addr: inc.1,
            };
            state.router.clone().update(src_node).await?;
            let proc_res = Self::process(req, state.clone()).await;

            // transform any error into a Reply, then send it back
            let reply: Reply = proc_res.unwrap_or_else(Reply::from);
            framed2.send(reply).await?;
        }
        Ok(())
    }

    pub async fn process(req: Request, state: StateClient) -> Result<Reply, ProtoErr> {
        println!("GOT: {:?}", &req);

        match req.body {
            RequestBody::Ping => Ok(Reply::Ping),
            RequestBody::Store(k, v) => {
                let _res: () = state.kv.set(k, v).await?;
                // TODO Wait what about the hash of the key ? => key is already hashed
                Ok(Reply::Ping)
            }
            RequestBody::FindNode(id) => {
                // find closest nodes in routes
                let res: Vec<KnownNode> = state.router.closest_nodes(id).await?;
                Ok(Reply::FindNode(res))
            }
            RequestBody::FindValue(k) => {
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
}

#[derive(Error, Debug)]
pub enum ReqErr {
    #[error("CodecErr: {0}")]
    CodecErr(#[from] CodecErr),
    #[error("State err: {0}")]
    StateErr(#[from] StateErr),
    #[error("unknown error: {0}")]
    Unknown(#[from] Box<dyn std::error::Error>),
}
#[derive(Error, Debug)]
pub enum KadErr {
    #[error("IoErr: {0}")]
    IoErr(#[from] std::io::Error),
    #[error("unknown error: {0}")]
    Unknown(#[from] Box<dyn std::error::Error>),
}
