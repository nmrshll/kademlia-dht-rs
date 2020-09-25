use std::collections::HashMap;
use tokio::sync::{mpsc, mpsc::error::SendError, oneshot, oneshot::error::RecvError};
use tokio::task::JoinHandle;
//
use crate::rout2::{KnownNode, Node, RoutingTable};
use crate::Key;

pub struct State {}
impl State {
    pub fn start(node_self: Node) -> (StateTasks, StateClient) {
        let (kv_man, task_kv) = KvStateMan::start();
        let (router_man, task_router) = RouterStateMan::start(node_self);

        return (
            StateTasks {
                kv: task_kv,
                router: task_router,
            },
            StateClient {
                kv: kv_man,
                router: router_man,
            },
        );
    }
}
#[derive(Clone)]
pub struct StateClient {
    pub kv: KvStateMan,
    pub router: RouterStateMan,
}
pub struct StateTasks {
    pub kv: JoinHandle<()>,
    pub router: JoinHandle<()>,
}

////////////////////////
/// Key-value store
////////////

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<Result<T, StateErr>>;
/// A command for the Key-value store task
#[derive(Debug)]
pub enum KvCmd {
    Get {
        key: Key,
        resp: Responder<Option<String>>,
    },
    Set {
        key: Key,
        val: String,
        resp: Responder<()>,
    },
}
#[derive(Clone)]
pub struct KvStateMan {
    tx: mpsc::Sender<KvCmd>,
}
impl KvStateMan {
    pub fn start() -> (Self, JoinHandle<()>) {
        // Create state and channel to send commands to the kv manager task
        let mut kv: HashMap<Key, String> = HashMap::new();
        let (tx_kv, mut rx) = mpsc::channel(32);

        let task_kv = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    KvCmd::Get { key, resp } => {
                        let res = kv.get(&key).cloned();
                        // Ignore errors
                        let _ = resp.send(Ok(res)); // TODO error handling
                    }
                    KvCmd::Set { key, val, resp } => {
                        let _res = kv.insert(key, val);
                        // Ignore errors
                        let _ = resp.send(Ok(())); // TODO error handling
                    }
                }
            }
        });
        (KvStateMan { tx: tx_kv }, task_kv)
    }

    // pass a clone of self to be consumed by the async functions
    pub async fn get(mut self, key: Key) -> Result<Option<String>, StateErr> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = KvCmd::Get { key, resp: resp_tx };
        // Send the get request
        self.tx.send(cmd).await.unwrap();
        // await response
        let res: Option<String> = resp_rx.await.unwrap(/*RecvError*/).unwrap(); // TODO err handling
        println!("GOT = {:?}", res);
        Ok(res)
    }
    pub async fn set(mut self, key: Key, val: String) -> Result<(), StateErr> {
        let (resp, resp_rx) = oneshot::channel();
        let cmd = KvCmd::Set { key, val, resp };
        // Send the SET request
        self.tx.send(cmd).await.unwrap();
        // Await the response
        let res: Result<(), StateErr> = resp_rx.await.unwrap(/*RecvError*/); // TODO err handling
        println!("GOT = {:?}", res);
        Ok(())
    }
}

///////////////////////////
/// Router
//////////////

/// A command for the router task
#[derive(Debug)]
pub enum RouterCmd {
    GetClosestNodes {
        key: Key,
        resp: Responder<Vec<KnownNode>>,
    },
    Update {
        new_node: Node,
        resp: Responder<()>,
    },
    Remove,
    // TODO maybe more ?
}
#[derive(Clone)]
pub struct RouterStateMan {
    tx: mpsc::Sender<RouterCmd>,
}
impl RouterStateMan {
    pub fn start(node_self: Node) -> (Self, JoinHandle<()>) {
        // Create state and chan to send commands to the router manager task
        let mut router = RoutingTable::new(&node_self);
        let (tx_router, mut rx) = mpsc::channel(32);

        let task_router = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    RouterCmd::GetClosestNodes { key, resp } => {
                        let res = router.closest_nodes(key, 3);
                        if let Err(resp) = resp.send(Ok(res)) {
                            println!("failed responding with: {:?}", resp);
                        }
                    }
                    RouterCmd::Update { new_node, resp } => {
                        router.update(&new_node);
                        if let Err(resp) = resp.send(Ok(())) {
                            println!("failed responding with: {:?}", resp);
                        }
                    }
                    _ => unimplemented!(), // TODO implement other varidants
                }
            }
        });
        (RouterStateMan { tx: tx_router }, task_router)
    }
    // TODO implement requests
    // pass a clone of self to be consumed by the async functions
    pub async fn closest_nodes(mut self, key: Key) -> Result<Vec<KnownNode>, StateErr> {
        let (resp, resp_rx) = oneshot::channel();
        let cmd = RouterCmd::GetClosestNodes { key, resp };
        // Send the SET request, await the response
        self.tx.send(cmd).await.unwrap(); // SendError
        let res: Result<Vec<KnownNode>, StateErr> = resp_rx.await?; // RecvError
        println!("GOT = {:?}", res);
        res
    }
    pub async fn update(mut self, new_node: Node) -> Result<(), StateErr> {
        let (resp, resp_rx) = oneshot::channel();
        let cmd = RouterCmd::Update { new_node, resp };
        // Send the UPDATE request, await the response
        self.tx.send(cmd).await?; // SendError
        let res: Result<(), StateErr> = resp_rx.await?; // RecvError
        println!("GOT = {:?}", res);
        res
    }
}

use thiserror::Error;
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StateErr {
    #[error("router SendErr")]
    RouterSendErr,
    #[error("kv-store SendErr")]
    KvSendErr,
    #[error("channel RecvErr")]
    RecvErr,
    // #[error("actor failed responding: SendErr")]
    // RespSendErr,
    #[error("unknown StateManager error")]
    Unknown,
}
impl From<SendError<RouterCmd>> for StateErr {
    fn from(e: SendError<RouterCmd>) -> Self {
        eprintln!("SendError: {}", e);
        StateErr::RouterSendErr
    }
}
impl From<SendError<KvCmd>> for StateErr {
    fn from(e: SendError<KvCmd>) -> Self {
        eprintln!("SendError: {}", e);
        StateErr::KvSendErr
    }
}
impl From<RecvError> for StateErr {
    fn from(e: RecvError) -> Self {
        eprintln!("RecvError: {}", e);
        StateErr::RecvErr
    }
}
