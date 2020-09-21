// use bytes::Bytes;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
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

    // We'll have to pass a clone of self to be consumed by the async functions
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
    Update,
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
        let router = RoutingTable::new(&node_self);
        let (tx_router, mut rx) = mpsc::channel(32);

        let task_router = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    RouterCmd::GetClosestNodes { key, resp } => {
                        let res = router.closest_nodes(key, 3); // TODO not random

                        // Ignore errors
                        let _ = resp.send(Ok(res)); // TODO error handling
                    }
                    _ => unimplemented!(), // TODO implement other varidants
                }
            }
        });
        (RouterStateMan { tx: tx_router }, task_router)
    }

    // TODO implement requests
    pub async fn closest_nodes(mut self, key: Key) -> Result<Vec<KnownNode>, StateErr> {
        let (resp, resp_rx) = oneshot::channel();
        let cmd = RouterCmd::GetClosestNodes { key, resp };
        // Send the SET request
        self.tx.send(cmd).await.unwrap();
        // Await the response
        let res: Result<Vec<KnownNode>, StateErr> = resp_rx.await.unwrap(/*RecvError*/); // TODO err handling
        println!("GOT = {:?}", res);
        Ok(res.unwrap()) // TODO err handling
    }
}

use thiserror::Error;
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StateErr {
    #[error("unknown StateManager error")]
    Unknown,
}
