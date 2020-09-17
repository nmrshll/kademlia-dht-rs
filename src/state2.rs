// use bytes::Bytes;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
//
use crate::rout2::{Node, RoutingTable};
use crate::Key;

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<Result<T, StateErr>>;
/// A command for the Key-value store task
#[derive(Debug)]
pub enum KvCmd {
    Get {
        key: String,
        resp: Responder<Option<String>>,
    },
    Set {
        key: String,
        val: String,
        resp: Responder<()>,
    },
}
/// A command for the router task
#[derive(Debug)]
pub enum RouterCmd {
    GetClosestNodes { resp: Responder<()> },
    Update,
    Remove,
    // TODO maybe more ?
}

pub struct State2 {}
impl State2 {
    pub fn start(node_self: Node) -> (StateTasks, StateClient) {
        let (kv_man, task_kv) = KvStateMan::start();
        let (router_man, task_router) = RouterStateMan::start(node_self);

        return (
            StateTasks {
                kv: task_kv,
                router: task_router,
            },
            (kv_man, router_man),
        );
    }
}
pub type StateClient = (KvStateMan, RouterStateMan);
pub struct StateTasks {
    pub kv: JoinHandle<()>,
    pub router: JoinHandle<()>,
}

#[derive(Clone)]
pub struct KvStateMan {
    tx: mpsc::Sender<KvCmd>,
}
impl KvStateMan {
    pub fn start() -> (Self, JoinHandle<()>) {
        // Create state and channel to send commands to the kv manager task
        let mut kv: HashMap<String, String> = HashMap::new();
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
    pub async fn get(mut self, key: String) -> Result<Option<String>, StateErr> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = KvCmd::Get { key, resp: resp_tx };
        // Send the get request
        self.tx.send(cmd).await.unwrap();
        // await response
        let res: Option<String> = resp_rx.await.unwrap(/*RecvError*/).unwrap(); // TODO err handling
        println!("GOT = {:?}", res);
        Ok(res)
    }
    pub async fn set(mut self, key: String, val: String) -> Result<(), StateErr> {
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
                    RouterCmd::GetClosestNodes { resp } => {
                        let res = router.closest_nodes(Key::random(), 3); // TODO not random

                        // Ignore errors
                        let _ = resp.send(Ok(())); // TODO error handling
                    }
                    _ => unimplemented!(), // TODO implement other varidants
                }
            }
        });
        (RouterStateMan { tx: tx_router }, task_router)
    }

    // TODO implement requests
}

use thiserror::Error;
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StateErr {
    #[error("unknown StateManager error")]
    Unknown,
}

/////////////////////
/// Old state manager
//////////

pub struct State {}
impl State {
    pub fn start(node_self: Node) -> (StateTasks, CmdChans) {
        // Create new state
        let mut kv: HashMap<String, String> = HashMap::new();
        let router = RoutingTable::new(&node_self);

        // A channel to send commands to the kv manager task
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

        // A channel to send commands to the router manager task
        let (tx_router, mut rx) = mpsc::channel(32);
        let task_router = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    RouterCmd::GetClosestNodes { resp } => {
                        let res = router.closest_nodes(Key::random(), 3); // TODO not random

                        // Ignore errors
                        let _ = resp.send(Ok(())); // TODO error handling
                    }
                    _ => unimplemented!(), // TODO implement other varidants
                }
            }
        });

        return (
            StateTasks {
                kv: task_kv,
                router: task_router,
            },
            (tx_kv, tx_router),
        );
    }
}

// pub struct CmdChans {
//     kv: mpsc::Sender<KvCmd>,
// }

pub type KvCmdChan = mpsc::Sender<KvCmd>;
pub type RouterCmdChan = mpsc::Sender<RouterCmd>;
pub type CmdChans = (KvCmdChan, RouterCmdChan);
